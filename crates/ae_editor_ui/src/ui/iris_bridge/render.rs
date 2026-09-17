// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rendering, command buffer generation, and text extraction subsystem for Iris UI editor overlays.

use super::types::IrisEditorOverlay;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSection};

impl IrisEditorOverlay {
    /// Recursively converts computed node bounds and styles into layered `DrawCommandList` instances,
    /// ensuring strict back-to-front layer ordering: Background -> Content -> Floating -> Modal -> Popup -> Tooltip.
    pub(crate) fn populate_draw_commands(
        &mut self,
        current: WidgetId,
        clip_rect: Option<Rect>,
        frame_pacing: Option<&ae_core::telemetry::FrameRingBuffer>,
    ) {
        let mut layer_lists: [DrawCommandList; 6] = Default::default();
        Self::populate_tree_commands_by_layer(
            &self.tree,
            current,
            clip_rect,
            UiLayer::Background,
            frame_pacing,
            &mut layer_lists,
        );
        self.command_list.clear();
        for list in layer_lists {
            self.command_list.append(list);
        }
    }

    /// Helper that traverses the UI tree, grouping draw commands by effective `UiLayer`.
    pub(crate) fn populate_tree_commands_by_layer(
        tree: &UiTree,
        current: WidgetId,
        clip_rect: Option<Rect>,
        inherited_layer: UiLayer,
        frame_pacing: Option<&ae_core::telemetry::FrameRingBuffer>,
        layer_lists: &mut [DrawCommandList; 6],
    ) {
        let Some(node) = tree.get(current) else {
            return;
        };
        if !node.visible {
            return;
        }

        let effective_layer = if node.layer > inherited_layer {
            node.layer
        } else {
            inherited_layer
        };

        // Decouple scissor clip when transitioning into an elevated overlay layer
        // so that child popups or modal cards are not clipped by parent panel boundaries.
        let effective_clip = if effective_layer > inherited_layer {
            None
        } else {
            clip_rect
        };

        let child_clip = if node.style.clip_children {
            match effective_clip {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            effective_clip
        };

        let has_border = (node.style.border.width.top > 0.0
            || node.style.border.width.bottom > 0.0
            || node.style.border.width.left > 0.0
            || node.style.border.width.right > 0.0)
            && node.style.border.color.a > 0.0;

        let target_list = &mut layer_lists[effective_layer.index()];

        if node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
            && (node.style.background_color.a > 0.0
                || has_border
                || node.style.box_shadow.is_some())
        {
            target_list.push_quad(QuadInstance::from_style(
                node.computed_rect,
                &node.style,
                effective_clip,
            ));
        }

        if let Some(uv) = node.texture_uv
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            let tint = node.texture_tint.unwrap_or(Color::WHITE);
            let clip_arr = match effective_clip {
                Some(c) => [c.x, c.y, c.x + c.width, c.y + c.height],
                None => [0.0, 0.0, 0.0, 0.0],
            };
            target_list.push_texture_quad(TextureQuadInstance {
                rect: [
                    node.computed_rect.x,
                    node.computed_rect.y,
                    node.computed_rect.width,
                    node.computed_rect.height,
                ],
                uv_rect: uv,
                tint: [tint.r, tint.g, tint.b, tint.a],
                clip_rect: clip_arr,
            });
        }

        if let Some(id) = node.external_texture
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            let tint = node.texture_tint.unwrap_or(Color::WHITE);
            let uv = node.texture_uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
            target_list.push_external_texture_quad(
                id,
                ExternalTextureQuadInstance::with_uv(node.computed_rect, uv, tint, effective_clip),
            );
        }

        // Render oscilloscope telemetry trace at exact canvas Z-order
        if node.role == WidgetRole::OscilloscopeCanvas
            && let Some(ring) = frame_pacing
        {
            super::stats::append_oscilloscope_quads(target_list, node.computed_rect, ring);
        }

        for &child_id in &node.children {
            Self::populate_tree_commands_by_layer(
                tree,
                child_id,
                child_clip,
                effective_layer,
                frame_pacing,
                layer_lists,
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_layer_ordering_and_clipping_decoupling() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        // Content layer panel with clip_children enabled
        let panel = tree.create_node();
        if let Some(node) = tree.get_mut(panel) {
            node.computed_rect = Rect::new(0.0, 0.0, 400.0, 400.0);
            node.style = Style::new()
                .background(Color::rgba(0.1, 0.1, 0.1, 1.0))
                .clip_children(true);
            node.layer = UiLayer::Content;
        }
        let _ = tree.add_child(root, panel);

        // Popup layer dropdown inside the panel (breaks out of parent scissor clip)
        let popup = tree.create_node();
        if let Some(node) = tree.get_mut(popup) {
            node.computed_rect = Rect::new(100.0, 350.0, 200.0, 200.0);
            node.style = Style::new().background(Color::rgba(0.2, 0.2, 0.2, 1.0));
            node.layer = UiLayer::Popup;
        }
        let _ = tree.add_child(panel, popup);

        // Modal dialog (Preferences) added to root
        let modal = tree.create_node();
        if let Some(node) = tree.get_mut(modal) {
            node.computed_rect = Rect::new(50.0, 50.0, 500.0, 500.0);
            node.style = Style::new().background(Color::rgba(0.3, 0.3, 0.3, 1.0));
            node.layer = UiLayer::Modal;
        }
        let _ = tree.add_child(root, modal);

        let mut layer_lists: [DrawCommandList; 6] = Default::default();
        IrisEditorOverlay::populate_tree_commands_by_layer(
            &tree,
            root,
            None,
            UiLayer::Background,
            None,
            &mut layer_lists,
        );

        assert_eq!(layer_lists[UiLayer::Content.index()].quads.len(), 1);
        assert_eq!(layer_lists[UiLayer::Modal.index()].quads.len(), 1);
        assert_eq!(layer_lists[UiLayer::Popup.index()].quads.len(), 1);

        // Verify popup decoupled from parent panel scissor clipping
        let popup_quad = &layer_lists[UiLayer::Popup.index()].quads[0];
        assert_eq!(popup_quad.clip_rect, [0.0, 0.0, 0.0, 0.0]); // Unclipped full screen

        let mut final_list = DrawCommandList::new();
        for list in layer_lists {
            final_list.append(list);
        }

        assert_eq!(final_list.quads.len(), 3);
        // Content (Panel) at index 0 (width 400.0)
        assert_eq!(final_list.quads[0].rect[2], 400.0);
        // Modal (Preferences) at index 1 (width 500.0)
        assert_eq!(final_list.quads[1].rect[2], 500.0);
        // Popup (Dropdown) at index 2 (width 200.0 - rendered on top of Modal and Content)
        assert_eq!(final_list.quads[2].rect[2], 200.0);
    }
}