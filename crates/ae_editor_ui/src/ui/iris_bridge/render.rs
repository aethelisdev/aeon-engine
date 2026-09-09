// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rendering, command buffer generation, and text extraction subsystem for Iris UI editor overlays.

use super::types::IrisEditorOverlay;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSection};

/// Context parameters for recursive text section extraction with strict multi-layer Z-hierarchy.
struct TextCollectionContext<'a> {
    clip_rect: Option<Rect>,
    active_dropdown_rects: &'a [Rect],
    active_modal_rects: &'a [Rect],
    floating_window_rects: &'a [Rect],
    is_inside_dropdown: bool,
    is_inside_modal: bool,
    is_inside_floating: bool,
}

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
    pub fn collect_text_sections_from_tree<'a>(
        tree: &'a UiTree,
        active_dropdown_rects: &[Rect],
        active_modal_rects: &[Rect],
        floating_window_rects: &[Rect],
    ) -> Vec<TextSection<'a>> {
        let mut sections = Vec::new();
        if let Some(root) = tree.root() {
            let ctx = TextCollectionContext {
                clip_rect: None,
                active_dropdown_rects,
                active_modal_rects,
                floating_window_rects,
                is_inside_dropdown: false,
                is_inside_modal: false,
                is_inside_floating: false,
            };
            Self::collect_node_text_from_tree(tree, root, &ctx, &mut sections);
        }
        sections
    }

    /// Recursive helper extracting text sections from a node subtree.
    fn collect_node_text_from_tree<'a>(
        tree: &'a UiTree,
        current: WidgetId,
        ctx: &TextCollectionContext<'_>,
        sections: &mut Vec<TextSection<'a>>,
    ) {
        let Some(node) = tree.get(current) else {
            return;
        };
        if !node.visible {
            return;
        }

        let is_dropdown_element = matches!(
            node.role,
            WidgetRole::DropdownPopup
                | WidgetRole::DropdownItem
                | WidgetRole::DropdownIcon
                | WidgetRole::DropdownShortcut
                | WidgetRole::DropdownLabel
        );
        let child_is_inside_dropdown = ctx.is_inside_dropdown || is_dropdown_element;

        let is_modal_element = node.role == WidgetRole::ModalWindow;
        let child_is_inside_modal = ctx.is_inside_modal || is_modal_element;

        let is_floating_element = node.role == WidgetRole::FloatingWindow;
        let child_is_inside_floating = ctx.is_inside_floating || is_floating_element;

        let child_clip = if node.style.clip_children {
            match ctx.clip_rect {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            ctx.clip_rect
        };

        if let Some(text) = &node.text
            && !text.is_empty()
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            let mut effective_clip = ctx.clip_rect;

            // Estimate visual horizontal footprint of the text inside computed_rect
            let text_char_count = text.chars().count() as f32;
            let approx_text_width = text_char_count * (node.font_size * 0.62);
            let (text_min_x, text_max_x) = match node.text_align {
                TextAlign::Left => (
                    node.computed_rect.x,
                    (node.computed_rect.x + approx_text_width).min(node.computed_rect.right()),
                ),
                TextAlign::Right => (
                    (node.computed_rect.right() - approx_text_width).max(node.computed_rect.x),
                    node.computed_rect.right(),
                ),
                TextAlign::Center => {
                    let cx = node.computed_rect.x + node.computed_rect.width * 0.5;
                    (
                        (cx - approx_text_width * 0.5).max(node.computed_rect.x),
                        (cx + approx_text_width * 0.5).min(node.computed_rect.right()),
                    )
                }
            };

            let text_center_y = node.computed_rect.y + node.computed_rect.height * 0.5;
            let mut is_fully_occluded = false;

            // 1. Dropdown menus and popups are top-most. Everything EXCEPT elements inside
            // this specific dropdown popup hierarchy must be occluded by active dropdowns.
            if !child_is_inside_dropdown {
                for popup in ctx.active_dropdown_rects {
                    let vert_overlap = node.computed_rect.bottom() > popup.y
                        && node.computed_rect.y < popup.bottom();
                    if !vert_overlap {
                        continue;
                    }
                    let horiz_overlap = text_max_x > popup.x && text_min_x < popup.right();
                    if !horiz_overlap {
                        continue;
                    }

                    // If text is 100% covered horizontally and vertically by popup, suppress it completely
                    if text_min_x >= popup.x
                        && text_max_x <= popup.right()
                        && text_center_y >= popup.y
                        && text_center_y <= popup.bottom()
                    {
                        is_fully_occluded = true;
                        break;
                    }

                    // Scissor clip if partially overlapping horizontally
                    if text_min_x < popup.x && text_max_x > popup.x {
                        let clip_sub = Rect::new(0.0, 0.0, popup.x, 100_000.0);
                        effective_clip = match effective_clip {
                            Some(c) => Some(c.intersect(clip_sub)),
                            None => Some(clip_sub),
                        };
                    } else if text_min_x < popup.right() && text_max_x > popup.right() {
                        let clip_sub = Rect::new(popup.right(), 0.0, 100_000.0, 100_000.0);
                        effective_clip = match effective_clip {
                            Some(c) => Some(c.intersect(clip_sub)),
                            None => Some(clip_sub),
                        };
                    }
                }
            }

            // 2. Modal dialogs occlude background docked panels, but do NOT occlude dropdowns or their own content
            if !child_is_inside_dropdown && !child_is_inside_modal && !is_fully_occluded {
                for modal in ctx.active_modal_rects {
                    let vert_overlap = node.computed_rect.bottom() > modal.y
                        && node.computed_rect.y < modal.bottom();
                    if !vert_overlap {
                        continue;
                    }
                    let horiz_overlap = text_max_x > modal.x && text_min_x < modal.right();
                    if !horiz_overlap {
                        continue;
                    }

                    if text_min_x >= modal.x
                        && text_max_x <= modal.right()
                        && text_center_y >= modal.y
                        && text_center_y <= modal.bottom()
                    {
                        is_fully_occluded = true;
                        break;
                    }

                    if text_min_x < modal.x && text_max_x > modal.x {
                        let clip_sub = Rect::new(0.0, 0.0, modal.x, 100_000.0);
                        effective_clip = match effective_clip {
                            Some(c) => Some(c.intersect(clip_sub)),
                            None => Some(clip_sub),
                        };
                    } else if text_min_x < modal.right() && text_max_x > modal.right() {
                        let clip_sub = Rect::new(modal.right(), 0.0, 100_000.0, 100_000.0);
                        effective_clip = match effective_clip {
                            Some(c) => Some(c.intersect(clip_sub)),
                            None => Some(clip_sub),
                        };
                    }
                }
            }

            // 3. Floating windows occlude background docked panels, but do NOT occlude modals or dropdowns
            if !child_is_inside_dropdown
                && !child_is_inside_modal
                && !child_is_inside_floating
                && !is_fully_occluded
            {
                for floating in ctx.floating_window_rects {
                    let vert_overlap = node.computed_rect.bottom() > floating.y
                        && node.computed_rect.y < floating.bottom();
                    if !vert_overlap {
                        continue;
                    }
                    let horiz_overlap = text_max_x > floating.x && text_min_x < floating.right();
                    if !horiz_overlap {
                        continue;
                    }

                    if text_min_x >= floating.x
                        && text_max_x <= floating.right()
                        && text_center_y >= floating.y
                        && text_center_y <= floating.bottom()
                    {
                        is_fully_occluded = true;
                        break;
                    }

                    if text_min_x < floating.x && text_max_x > floating.x {
                        let clip_sub = Rect::new(0.0, 0.0, floating.x, 100_000.0);
                        effective_clip = match effective_clip {
                            Some(c) => Some(c.intersect(clip_sub)),
                            None => Some(clip_sub),
                        };
                    } else if text_min_x < floating.right() && text_max_x > floating.right() {
                        let clip_sub = Rect::new(floating.right(), 0.0, 100_000.0, 100_000.0);
                        effective_clip = match effective_clip {
                            Some(c) => Some(c.intersect(clip_sub)),
                            None => Some(clip_sub),
                        };
                    }
                }
            }

            let is_visible_in_clip = match effective_clip {
                Some(clip) => {
                    clip.width > 0.0
                        && clip.height > 0.0
                        && node.computed_rect.right() > clip.x
                        && node.computed_rect.x < clip.right()
                        && node.computed_rect.bottom() > clip.y
                        && node.computed_rect.y < clip.bottom()
                }
                None => true,
            };

            if is_visible_in_clip && !is_fully_occluded {
                sections.push(
                    TextSection::new(text.clone(), node.computed_rect)
                        .with_font_size(node.font_size, node.line_height)
                        .with_color(node.text_color)
                        .with_align(node.text_align)
                        .with_clip(effective_clip),
                );
            }
        }

        let child_ctx = TextCollectionContext {
            clip_rect: child_clip,
            active_dropdown_rects: ctx.active_dropdown_rects,
            active_modal_rects: ctx.active_modal_rects,
            floating_window_rects: ctx.floating_window_rects,
            is_inside_dropdown: child_is_inside_dropdown,
            is_inside_modal: child_is_inside_modal,
            is_inside_floating: child_is_inside_floating,
        };

        for &child in &node.children {
            Self::collect_node_text_from_tree(tree, child, &child_ctx, sections);
        }
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

        let mut active_dropdown_rects: Vec<Rect> = Vec::new();
        let mut active_modal_rects: Vec<Rect> = Vec::new();

        if let Some(r) = self.dropdown_rect {
            active_dropdown_rects.push(r);
        }
        if let Some(ref t) = self.preferences_targets {
            active_modal_rects.push(t.card_rect);
            if let Some(r) = t.active_dropdown_popup_rect {
                active_dropdown_rects.push(r);
            }
        }
        if let Some(ref t) = self.about_targets {
            active_modal_rects.push(t.dialog_rect);
        }
        if let Some(ref t) = self.delete_targets {
            active_modal_rects.push(t.dialog_rect);
        }
        if let Some(ref t) = self.new_folder_targets {
            active_modal_rects.push(t.dialog_rect);
        }
        if let Some(ref t) = self.rename_targets {
            active_modal_rects.push(t.dialog_rect);
        }
        if let Some(ref t) = self.loading_targets {
            active_modal_rects.push(t.card_rect);
        }
        if let Some(ref hud) = self.viewport_hud_targets
            && let Some(r) = hud.active_dropdown_popup_rect
        {
            active_dropdown_rects.push(r);
        }
        if let Some(ref hier) = self.hierarchy_targets {
            if let Some(r) = hier.active_add_menu_rect {
                active_dropdown_rects.push(r);
            }
            if let Some(r) = hier.active_submenu_rect {
                active_dropdown_rects.push(r);
            }
            if let Some(r) = hier.active_sub_submenu_rect {
                active_dropdown_rects.push(r);
            }
            if let Some((_, r, _, _)) = hier.active_context_menu {
                active_dropdown_rects.push(r);
            }
        }
        if let Some(ref insp) = self.inspector_targets {
            if let Some(r) = insp.active_add_menu_rect {
                active_dropdown_rects.push(r);
            }
            if let Some(r) = insp.active_submenu_rect {
                active_dropdown_rects.push(r);
            }
            if let Some(r) = insp.active_dropdown_popup_rect {
                active_dropdown_rects.push(r);
            }
            if let Some(r) = insp.color_picker_popup_rect {
                active_dropdown_rects.push(r);
            }
        }
        if let Some(ref assets) = self.assets_targets {
            if let Some(ref ctx_menu) = assets.context_menu {
                active_dropdown_rects.push(ctx_menu.card_rect);
            }
            if let Some(ref modal) = assets.preview_modal {
                active_modal_rects.push(modal.dialog_rect);
            }
        }

        let sections = Self::collect_text_sections_from_tree(
            &self.tree,
            &active_dropdown_rects,
            &active_modal_rects,
            &self.floating_window_rects,
        );
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
            if self.next_thumbnail_layer >= 256 {
                break;
            }
            if self.thumbnail_layers.contains_key(&item.path) {
                continue;
            }

            if let Some(rgba) = crate::ui::panels::assets::thumbnails::generate_thumbnail_rgba_64(
                &item.path,
                item.category,
            ) {
                let layer = self.next_thumbnail_layer;
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

                self.thumbnail_layers.insert(item.path.clone(), layer);
                self.next_thumbnail_layer += 1;
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