// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Egui Host Pass and Central Docking System
//!
//! Executes egui frame processing, zoom scaling, texture caching, central panel docking,
//! modal drawing, primitive tessellation, and WGPU render pass execution.

use crate::ui::docking;
use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::workbench::render::types::EditorUiRenderParams;
use crate::ui::workbench::state::EngineUi;
use egui_wgpu::ScreenDescriptor;

/// Output bundle produced by the egui layout and rendering pass.
pub struct EguiPassOutput {
    pub is_hovering_interactive: bool,
    pub viewport_rect: egui::Rect,
    pub stats_rect: Option<egui::Rect>,
    pub hierarchy_rect: Option<egui::Rect>,
    pub inspector_rect: Option<egui::Rect>,
    pub material_rect: Option<egui::Rect>,
    pub console_rect: Option<egui::Rect>,
    pub assets_rect: Option<egui::Rect>,
    pub timeline_rect: Option<egui::Rect>,
    pub ui_designer_rect: Option<egui::Rect>,
}

impl EngineUi {
    /// Executes the primary egui frame, managing viewport textures, central docking layout,
    /// tessellation, and the egui WGPU command encoder pass.
    pub fn execute_egui_pass(&mut self, params: &mut EditorUiRenderParams<'_>) -> EguiPassOutput {
        // Register/Update viewport texture if provided
        if let Some(view) = params.viewport_texture_view {
            if let Some(id) = self.viewport_texture_id {
                self.renderer.update_egui_texture_from_wgpu_texture(
                    params.device,
                    view,
                    wgpu::FilterMode::Linear,
                    id,
                );
            } else {
                let id = self.renderer.register_native_texture(
                    params.device,
                    view,
                    wgpu::FilterMode::Linear,
                );
                self.viewport_texture_id = Some(id);
            }
        } else if let Some(old_id) = self.viewport_texture_id.take() {
            self.renderer.free_texture(&old_id);
        }

        // Handle Ctrl + / - / 0 UI Zoom Shortcuts and apply active UI scaling
        let zoom_delta = self.context.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    0.05f32
                } else if i.key_pressed(egui::Key::Minus) {
                    -0.05f32
                } else if i.key_pressed(egui::Key::Num0) {
                    100.0f32
                } else {
                    0.0f32
                }
            } else {
                0.0f32
            }
        });

        if zoom_delta == 100.0 {
            self.ui_zoom_factor = 1.0;
        } else if zoom_delta != 0.0 {
            self.ui_zoom_factor = (self.ui_zoom_factor + zoom_delta).clamp(0.6, 2.0);
        }

        self.context.set_zoom_factor(self.ui_zoom_factor);

        let raw_input = self.state.take_egui_input(params.window);

        let viewport_rect_cell = std::cell::Cell::new(egui::Rect::ZERO);
        let stats_rect_cell = std::cell::Cell::new(None);
        let hierarchy_rect_cell = std::cell::Cell::new(None);
        let inspector_rect_cell = std::cell::Cell::new(None);
        let material_rect_cell = std::cell::Cell::new(None);
        let console_rect_cell = std::cell::Cell::new(None);
        let assets_rect_cell = std::cell::Cell::new(None);
        let timeline_rect_cell = std::cell::Cell::new(None);
        let ui_designer_rect_cell = std::cell::Cell::new(None);
        let ui_rects_collector = std::cell::RefCell::new(Vec::new());

        let full_output = self.context.run_ui(raw_input, |ui| {
            let ctx = ui.ctx().clone();
            let is_editing = *params.mode == ae_core::modules::EngineMode::Edit;

            // 1. Top Menubar Spacer (reserving top 26px for Iris UI MenuBar)
            egui::Panel::top("top_panel_spacer")
                .exact_size(IrisEditorOverlay::MENUBAR_HEIGHT)
                .resizable(false)
                .frame(egui::Frame::NONE)
                .show(ui, |_ui| {});

            // 2. Bottom Status Bar Spacer (reserving bottom 22px for Iris UI StatusBar)
            let bottom_resp = egui::Panel::bottom("utility_bar_spacer")
                .exact_size(IrisEditorOverlay::STATUS_BAR_HEIGHT)
                .resizable(false)
                .frame(egui::Frame::NONE)
                .show(ui, |_ui| {});
            ui_rects_collector
                .borrow_mut()
                .push(bottom_resp.response.rect);

            // 3. Central Tree Docking System (iris-dock - Tree-based split tab layout)
            let mut tab_viewer = docking::EditorTabViewer {
                world: params.world,
                is_editing,
                ui_actions: params.ui_actions,
                camera: params.camera,
                asset_browser: &mut self.asset_browser,
                viewport_texture_id: self.viewport_texture_id,
                viewport_rect_out: &viewport_rect_cell,
                stats_rect_out: &stats_rect_cell,
                hierarchy_rect_out: &hierarchy_rect_cell,
                inspector_rect_out: &inspector_rect_cell,
                material_rect_out: &material_rect_cell,
                console_rect_out: &console_rect_cell,
                assets_rect_out: &assets_rect_cell,
                timeline_rect_out: &timeline_rect_cell,
                ui_designer_rect_out: &ui_designer_rect_cell,
                enabled_modules: params.enabled_modules,
            };

            egui::CentralPanel::default()
                .frame(egui::Frame::new().inner_margin(egui::Margin::ZERO))
                .show(ui, |ui| {
                    Self::draw_docking_system(ui, &mut self.layout_state, &mut tab_viewer);
                });

            // Quick Asset Inspector Modal (Only in Edit mode)
            if is_editing {
                if let Some(rect) =
                    crate::ui::panels::assets::preview_modal::draw_asset_preview_modal(
                        &ctx,
                        &mut self.asset_browser,
                        params.models,
                        params.textures,
                        params.shaders,
                        params.ui_actions,
                    )
                {
                    ui_rects_collector.borrow_mut().push(rect);
                }
            } else {
                self.asset_browser.preview_modal = None;
            }

            // Draw floating cursor tooltip when dragging an asset across the editor
            if self.asset_browser.drag_payload.is_some() {
                crate::ui::panels::assets::drag_drop::draw_drag_cursor_tooltip(
                    &ctx,
                    &self.asset_browser,
                );
            }

            // Clean up status message after duration
            if let Some((_, start_time)) = &mut self.status_message
                && start_time.elapsed().as_secs() >= 5
            {
                self.status_message = None;
            }
        });

        self.ui_rects = ui_rects_collector.into_inner();
        self.state
            .handle_platform_output(params.window, full_output.platform_output);

        // Hover test on interactive Iris modal targets
        let mut is_hovering_interactive = false;
        let p = self.iris_overlay.cursor_pos;

        if let Some(ref targets) = self.iris_overlay.about_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.bottom_close_rect.contains_point(p)
                || targets.link_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }
        if let Some(ref targets) = self.iris_overlay.delete_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.confirm_btn_rect.contains_point(p)
                || targets.cancel_btn_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }
        if let Some(ref targets) = self.iris_overlay.new_folder_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.confirm_btn_rect.contains_point(p)
                || targets.cancel_btn_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }
        if let Some(ref targets) = self.iris_overlay.rename_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.confirm_btn_rect.contains_point(p)
                || targets.cancel_btn_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }

        let clipped_primitives = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [
                params.window.inner_size().width,
                params.window.inner_size().height,
            ],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, image_deltas) in &full_output.textures_delta.set {
            for image_delta in image_deltas {
                self.renderer
                    .update_texture(params.device, params.queue, *id, image_delta);
            }
        }
        self.renderer.update_buffers(
            params.device,
            params.queue,
            params.encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let render_pass = params
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Egui Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: params.window_surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            let mut render_pass = render_pass.forget_lifetime();
            self.renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        EguiPassOutput {
            is_hovering_interactive,
            viewport_rect: viewport_rect_cell.get(),
            stats_rect: stats_rect_cell.get(),
            hierarchy_rect: hierarchy_rect_cell.get(),
            inspector_rect: inspector_rect_cell.get(),
            material_rect: material_rect_cell.get(),
            console_rect: console_rect_cell.get(),
            assets_rect: assets_rect_cell.get(),
            timeline_rect: timeline_rect_cell.get(),
            ui_designer_rect: ui_designer_rect_cell.get(),
        }
    }
}