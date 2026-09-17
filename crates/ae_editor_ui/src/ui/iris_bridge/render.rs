// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rendering, command buffer generation, and text extraction subsystem for Iris UI editor overlays.

use super::types::IrisEditorOverlay;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSection};

impl IrisEditorOverlay {
    /// Recursively converts computed node bounds and styles into `DrawCommandList` instances.
    pub(crate) fn populate_draw_commands(
        &mut self,
        current: WidgetId,
        clip_rect: Option<Rect>,
        frame_pacing: Option<&ae_core::telemetry::FrameRingBuffer>,
    ) {
        let (child_count, quad, tex_quad, ext_quad, is_oscilloscope, canvas_rect, next_clip) = {
            let Some(node) = self.tree.get(current) else {
                return;
            };
            if !node.visible {
                return;
            }

            let child_clip = if node.style.clip_children {
                match clip_rect {
                    Some(existing) => Some(existing.intersect(node.computed_rect)),
                    None => Some(node.computed_rect),
                }
            } else {
                clip_rect
            };

            let has_border = (node.style.border.width.top > 0.0
                || node.style.border.width.bottom > 0.0
                || node.style.border.width.left > 0.0
                || node.style.border.width.right > 0.0)
                && node.style.border.color.a > 0.0;

            let quad = if node.computed_rect.width > 0.0
                && node.computed_rect.height > 0.0
                && (node.style.background_color.a > 0.0
                    || has_border
                    || node.style.box_shadow.is_some())
            {
                Some(QuadInstance::from_style(
                    node.computed_rect,
                    &node.style,
                    clip_rect,
                ))
            } else {
                None
            };

            let tex_quad = if let Some(uv) = node.texture_uv {
                if node.computed_rect.width > 0.0 && node.computed_rect.height > 0.0 {
                    let tint = node.texture_tint.unwrap_or(Color::WHITE);
                    let clip_arr = match clip_rect {
                        Some(c) => [c.x, c.y, c.x + c.width, c.y + c.height],
                        None => [0.0, 0.0, 0.0, 0.0],
                    };
                    Some(TextureQuadInstance {
                        rect: [
                            node.computed_rect.x,
                            node.computed_rect.y,
                            node.computed_rect.width,
                            node.computed_rect.height,
                        ],
                        uv_rect: uv,
                        tint: [tint.r, tint.g, tint.b, tint.a],
                        clip_rect: clip_arr,
                    })
                } else {
                    None
                }
            } else {
                None
            };

            let ext_quad = if let Some(id) = node.external_texture {
                if node.computed_rect.width > 0.0 && node.computed_rect.height > 0.0 {
                    let tint = node.texture_tint.unwrap_or(Color::WHITE);
                    let uv = node.texture_uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
                    Some((
                        id,
                        ExternalTextureQuadInstance::with_uv(
                            node.computed_rect,
                            uv,
                            tint,
                            clip_rect,
                        ),
                    ))
                } else {
                    None
                }
            } else {
                None
            };

            let is_oscilloscope = node.role == WidgetRole::OscilloscopeCanvas;
            let canvas_rect = node.computed_rect;

            (
                node.children.len(),
                quad,
                tex_quad,
                ext_quad,
                is_oscilloscope,
                canvas_rect,
                child_clip,
            )
        };

        if let Some(q) = quad {
            self.command_list.push_quad(q);
        }
        if let Some(tq) = tex_quad {
            self.command_list.push_texture_quad(tq);
        }
        if let Some((id, eq)) = ext_quad {
            self.command_list.push_external_texture_quad(id, eq);
        }

        // Render oscilloscope telemetry trace at exact canvas Z-order
        if is_oscilloscope && let Some(ring) = frame_pacing {
            super::stats::append_oscilloscope_quads(&mut self.command_list, canvas_rect, ring);
        }

        for i in 0..child_count {
            let child_id = {
                let Some(node) = self.tree.get(current) else {
                    break;
                };
                if i < node.children.len() {
                    node.children[i]
                } else {
                    break;
                }
            };
            self.populate_draw_commands(child_id, next_clip, frame_pacing);
        }
    }

    /// Collects text rendering sections from all visible layout nodes in the tree.
    /// Delegates directly to Iris UI's native layer-aware typography collection engine.
    pub fn collect_text_sections_from_tree<'a>(
        tree: &'a UiTree,
        _active_dropdown_rects: &[Rect],
        _active_modal_rects: &[Rect],
        _floating_window_rects: &[Rect],
    ) -> Vec<TextSection<'a>> {
        collect_text_sections(tree)
    }

    /// Renders the Iris UI overlay into the target surface framebuffer.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
    ) {
        self.ensure_tools_texture(device, queue);

        if !self.is_visible
            || (self.command_list.quads.is_empty()
                && self.command_list.texture_quads.is_empty()
                && self.tree.root().is_none())
        {
            return;
        }

        let zoom = if zoom_factor.is_finite() && zoom_factor > 0.1 {
            zoom_factor
        } else {
            1.0
        };

        let logical_screen_size = (
            (physical_screen_size.0 as f32 / zoom).round().max(1.0) as u32,
            (physical_screen_size.1 as f32 / zoom).round().max(1.0) as u32,
        );

        if self.text_renderer.is_none() {
            self.text_renderer = Some(TextRenderer::new(device, queue, self.target_format));
        }

        let sections = collect_text_sections(&self.tree);
        if let Some(txt_renderer) = &mut self.text_renderer {
            txt_renderer.prepare(
                device,
                queue,
                &mut self.text_system,
                physical_screen_size,
                zoom,
                &sections,
            );
        }

        ae_renderer::render::iris_render_pass(ae_renderer::render::IrisRenderPassParams {
            device,
            queue,
            encoder,
            target_view,
            renderer: &mut self.renderer,
            command_list: &self.command_list,
            text_renderer: self.text_renderer.as_ref(),
            screen_size: logical_screen_size,
        });
    }

    /// Uploads dynamic 64x64 thumbnail previews for active asset browser items into the 2D Texture Array.
    pub fn ensure_asset_thumbnails(
        &mut self,
        queue: &wgpu::Queue,
        items: &[crate::ui::panels::assets::types::AssetItem],
    ) {
        let Some((ref texture, _, _)) = self.tools_texture else {
            return;
        };

        for item in items {
            if self.assets.next_thumbnail_layer >= 256 {
                break;
            }
            if self.assets.thumbnail_layers.contains_key(&item.path) {
                continue;
            }

            if let Some(rgba) = crate::ui::panels::assets::thumbnails::generate_thumbnail_rgba_64(
                &item.path,
                item.category,
            ) {
                let layer = self.assets.next_thumbnail_layer;
                let mips = ae_texture::generate_mipmap_chain(64, 64, &rgba);

                for (mip_level, level_data) in mips.iter().enumerate() {
                    let mip_size = wgpu::Extent3d {
                        width: level_data.width,
                        height: level_data.height,
                        depth_or_array_layers: 1,
                    };
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture,
                            mip_level: mip_level as u32,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &level_data.bytes,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * level_data.width),
                            rows_per_image: Some(level_data.height),
                        },
                        mip_size,
                    );
                }

                self.assets
                    .thumbnail_layers
                    .insert(item.path.clone(), layer);
                self.assets.next_thumbnail_layer += 1;
            }
        }
    }

    /// Ensures that the editor tools 2D texture array (`editor_atlas.png`) is loaded into GPU memory.
    /// The master atlas is loaded and sliced into 16 isolated 64x64 pixel layers. Each layer receives
    /// its own independent mipmap chain, physically eliminating texture atlas seam bleeding and
    /// filtering artifacts while maintaining full icon resolution and crispness.
    pub fn ensure_tools_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.tools_texture.is_some() {
            return;
        }

        const TOOLS_ICON_BYTES: &[u8] =
            include_bytes!("../../../../../assets/icons/editor_atlas.png");

        let Ok(img) = image::load_from_memory(TOOLS_ICON_BYTES) else {
            log::warn!("Failed to decode editor_atlas.png texture atlas");
            return;
        };
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let raw_rgba = rgba.as_raw();

        let tile_size = 64u32;
        let cols = width / tile_size;
        let rows = height / tile_size;
        let layer_count = 256u32;

        // 64x64 tile has 7 mip levels (64, 32, 16, 8, 4, 2, 1)
        let mip_level_count = (tile_size as f32).log2().floor() as u32 + 1;

        let size = wgpu::Extent3d {
            width: tile_size,
            height: tile_size,
            depth_or_array_layers: layer_count,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Iris UI Editor Tools Texture Array"),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for r in 0..rows {
            for c in 0..cols {
                let layer = r * cols + c;
                let mut tile_bytes = Vec::with_capacity((tile_size * tile_size * 4) as usize);
                for y in 0..tile_size {
                    let src_y = r * tile_size + y;
                    let src_x = c * tile_size;
                    let start = ((src_y * width + src_x) * 4) as usize;
                    let end = start + (tile_size as usize * 4);
                    tile_bytes.extend_from_slice(&raw_rgba[start..end]);
                }

                let mips = ae_texture::generate_mipmap_chain(tile_size, tile_size, &tile_bytes);

                for (mip_level, level_data) in mips.iter().enumerate() {
                    let mip_size = wgpu::Extent3d {
                        width: level_data.width,
                        height: level_data.height,
                        depth_or_array_layers: 1,
                    };
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: mip_level as u32,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &level_data.bytes,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * level_data.width),
                            rows_per_image: Some(level_data.height),
                        },
                        mip_size,
                    );
                }
            }
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Iris UI Editor Tools Texture Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let bind_group = self
            .renderer
            .texture_pipeline
            .create_texture_bind_group(device, &view);

        self.renderer
            .set_texture_bind_group(Some(bind_group.clone()));
        self.tools_texture = Some((texture, view, bind_group));
    }
}