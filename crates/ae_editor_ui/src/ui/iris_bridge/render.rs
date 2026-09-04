// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rendering, command buffer generation, and text extraction subsystem for Iris UI editor overlays.

use super::types::IrisEditorOverlay;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSection};

impl IrisEditorOverlay {
    /// Recursively converts computed node bounds and styles into `DrawCommandList` instances.
    pub(crate) fn populate_draw_commands(&mut self, current: WidgetId, clip_rect: Option<Rect>) {
        let (child_count, quad, tex_quad, next_clip) = {
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

            (node.children.len(), quad, tex_quad, child_clip)
        };

        if let Some(q) = quad {
            self.command_list.push_quad(q);
        }
        if let Some(tq) = tex_quad {
            self.command_list.push_texture_quad(tq);
        }

        for i in 0..child_count {
            if let Some(child) = self
                .tree
                .get(current)
                .and_then(|n| n.children.get(i).copied())
            {
                self.populate_draw_commands(child, next_clip);
            }
        }
    }

    /// Collects text rendering sections from all visible layout nodes in the tree.
    pub fn collect_text_sections_from_tree<'a>(
        tree: &'a UiTree,
        active_popup_rects: &[Rect],
    ) -> Vec<TextSection<'a>> {
        let mut sections = Vec::new();
        if let Some(root) = tree.root() {
            Self::collect_node_text_from_tree(
                tree,
                root,
                None,
                active_popup_rects,
                false,
                &mut sections,
            );
        }
        sections
    }

    /// Recursive helper extracting text sections from a node subtree.
    fn collect_node_text_from_tree<'a>(
        tree: &'a UiTree,
        current: WidgetId,
        clip_rect: Option<Rect>,
        active_popup_rects: &[Rect],
        is_inside_popup: bool,
        sections: &mut Vec<TextSection<'a>>,
    ) {
        let Some(node) = tree.get(current) else {
            return;
        };
        if !node.visible {
            return;
        }

        let child_is_inside_popup = is_inside_popup
            || node
                .name
                .as_deref()
                .map(|n| {
                    (n.contains("Popup")
                        || n.contains("ColorPicker")
                        || n.contains("Picker")
                        || n.contains("AddMenu")
                        || n.contains("Submenu")
                        || n.contains("SubItem")
                        || n.contains("ContextMenu")
                        || n.starts_with("DropdownMenu")
                        || n.starts_with("DropdownItem")
                        || n.starts_with("DropdownIcon")
                        || n.starts_with("DropdownShortcut")
                        || n.contains("Modal")
                        || n.contains("About"))
                        && !n.contains("Combo")
                })
                .unwrap_or(false);

        let child_clip = if node.style.clip_children {
            match clip_rect {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            clip_rect
        };

        if let Some(text) = &node.text
            && !text.is_empty()
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            let is_visible_in_clip = match clip_rect {
                Some(clip) => {
                    node.computed_rect.right() > clip.x
                        && node.computed_rect.x < clip.right()
                        && node.computed_rect.bottom() > clip.y
                        && node.computed_rect.y < clip.bottom()
                }
                None => true,
            };

            let is_occluded_by_popup = if !child_is_inside_popup {
                active_popup_rects.iter().any(|popup| {
                    node.computed_rect.right() > popup.x
                        && node.computed_rect.x < popup.right()
                        && node.computed_rect.bottom() > popup.y
                        && node.computed_rect.y < popup.bottom()
                })
            } else {
                false
            };

            if is_visible_in_clip && !is_occluded_by_popup {
                sections.push(
                    TextSection::new(text.clone(), node.computed_rect)
                        .with_font_size(node.font_size, node.line_height)
                        .with_color(node.text_color)
                        .with_align(node.text_align)
                        .with_clip(clip_rect),
                );
            }
        }

        for &child in &node.children {
            Self::collect_node_text_from_tree(
                tree,
                child,
                child_clip,
                active_popup_rects,
                child_is_inside_popup,
                sections,
            );
        }
    }

    /// Renders the Iris UI overlay into the target surface framebuffer.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        screen_size: (u32, u32),
    ) {
        self.ensure_tools_texture(device, queue);

        if !self.is_visible
            || (self.command_list.quads.is_empty()
                && self.command_list.texture_quads.is_empty()
                && self.tree.root().is_none())
        {
            return;
        }

        if self.text_renderer.is_none() {
            self.text_renderer = Some(TextRenderer::new(device, queue, self.target_format));
        }

        let mut active_popups: Vec<Rect> = Vec::new();
        if let Some(r) = self.dropdown_rect {
            active_popups.push(r);
        }
        if let Some(ref t) = self.preferences_targets
            && let Some(r) = t.active_dropdown_popup_rect
        {
            active_popups.push(r);
        }
        if let Some(ref hud) = self.viewport_hud_targets
            && let Some(r) = hud.active_dropdown_popup_rect
        {
            active_popups.push(r);
        }
        if let Some(ref hier) = self.hierarchy_targets {
            if let Some(r) = hier.active_add_menu_rect {
                active_popups.push(r);
            }
            if let Some(r) = hier.active_submenu_rect {
                active_popups.push(r);
            }
            if let Some((_, r, _, _)) = hier.active_context_menu {
                active_popups.push(r);
            }
        }
        if let Some(ref insp) = self.inspector_targets {
            if let Some(r) = insp.active_add_menu_rect {
                active_popups.push(r);
            }
            if let Some(r) = insp.active_submenu_rect {
                active_popups.push(r);
            }
            if let Some(r) = insp.active_dropdown_popup_rect {
                active_popups.push(r);
            }
            if let Some(r) = insp.color_picker_popup_rect {
                active_popups.push(r);
            }
        }

        let sections = Self::collect_text_sections_from_tree(&self.tree, &active_popups);
        if let Some(txt_renderer) = &mut self.text_renderer {
            txt_renderer.prepare(
                device,
                queue,
                &mut self.text_system,
                [screen_size.0 as f32, screen_size.1 as f32],
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
            screen_size,
        });
    }

    /// Ensures that the editor tools texture atlas (`editor_tools.png`) is loaded into GPU memory.
    pub fn ensure_tools_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.tools_texture.is_some() {
            return;
        }

        const TOOLS_ICON_BYTES: &[u8] =
            include_bytes!("../../../../../assets/icons/editor_tools.png");

        let Ok(img) = image::load_from_memory(TOOLS_ICON_BYTES) else {
            log::warn!("Failed to decode editor_tools.png texture atlas");
            return;
        };
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Iris UI Editor Tools Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self
            .renderer
            .texture_pipeline
            .create_texture_bind_group(device, &view);

        self.renderer
            .set_texture_bind_group(Some(bind_group.clone()));
        self.tools_texture = Some((texture, view, bind_group));
    }
}