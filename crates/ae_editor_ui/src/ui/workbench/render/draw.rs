// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame rendering execution for the editor workbench.

use super::iris_pass::IrisPassParams;
use super::types::EditorUiRenderParams;
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::EngineUi;
use irisui::prelude::Size;

impl EngineUi {
    /// Orchestrates the drawing of all editor panels, toolbar menus, preference views,
    /// hierarchy snapshots, interactive HUD nodes, and overlay dialogs for the frame.
    pub fn render(
        &mut self,
        params: EditorUiRenderParams<'_>,
    ) -> ae_renderer::render::ViewportRect {
        params.ui_actions.append(&mut self.pending_actions);
        self.dispatch_action_bus(params.ui_actions);

        // Track live engine frames per second (FPS) directly from timing parameters
        self.fps = params.fps;

        let win_size = params.window.inner_size();
        let zoom = self.scale_factor();
        let viewport_rect = self.workbench.prepare_viewport_layout(Size::new(
            win_size.width as f32 / zoom,
            win_size.height as f32 / zoom,
        ));

        // Synchronize engine asset loading state with autonomous DialogHost progress modal
        if self.is_loading_assets {
            if !self.workbench.dialog_host.has_progress() {
                self.workbench.dialog_host.open_progress(
                    irisui::workbench::ProgressModalDescriptor::new("Loading Assets")
                        .with_icon("🚀")
                        .with_progress(0.65)
                        .with_cancellable(false)
                        .with_size(360.0, 140.0)
                        .with_status_message("Processing geometry, materials & textures"),
                );
            }
        } else if self.workbench.dialog_host.has_progress() {
            let _ = self.workbench.dialog_host.close_progress();
        }

        self.selected_entity = self
            .selected_entity
            .or_else(|| params.editor_state.selected_entities.first().copied());

        // 2. Process Iris UI Overlay & Inspector Actions
        let mut cur_gs = *params.graphics_settings;
        let mut cur_snap = *params.snapping;
        let mut cur_cfg = params.editor_state.config;
        let mut cur_live = params.editor_state.enable_live_editor_updates;

        params
            .ui_actions
            .append(&mut self.iris_overlay.pending_actions);

        if crate::assets::scanner::rescan_assets_if_needed(
            &mut self.asset_browser,
            params.models,
            params.textures,
            params.shaders,
        ) {
            self.workbench
                .notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Assets);
        }
        self.process_timeline_actions(params.world, params.ui_actions);
        self.process_ui_designer_actions(params.world, params.ui_actions);

        // 3. Iris UI GPU SDF Native Render Pass
        self.execute_iris_pass(IrisPassParams {
            device: params.device,
            queue: params.queue,
            encoder: params.encoder,
            window: params.window,
            window_surface_view: params.window_surface_view,
            viewport_texture_view: params.viewport_texture_view,
            viewport_rect,
            is_editing: *params.mode == ae_core::modules::EngineMode::Edit,
            undo_stack: params.undo_stack,
            redo_stack: params.redo_stack,
            history_revision: params.editor_state.history_revision,
            graphics_settings: &mut cur_gs,
            snapping_settings: &mut cur_snap,
            editor_config: &mut cur_cfg,
            enable_live_updates: &mut cur_live,
            enabled_modules: params.enabled_modules,
            camera: params.camera,
            world: params.world,
            textures: params.textures,
            models: params.models,
            is_2d_mode: params.is_2d_mode,
        });

        // 3a. Drain action bus events triggered during the Iris overlay render pass
        self.dispatch_action_bus(params.ui_actions);

        // 3b. Immediate post-pass action drain for declarative overlay actions produced during render
        params
            .ui_actions
            .append(&mut self.iris_overlay.pending_actions);

        // 3c. Dispatch any settings modifications mutated in-place by declarative two-way bindings
        if cur_gs != *params.graphics_settings {
            params
                .ui_actions
                .push(EngineUiAction::UpdateGraphicsSettings(cur_gs));
            self.workbench
                .notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }
        if cur_snap != *params.snapping {
            params
                .ui_actions
                .push(EngineUiAction::UpdateSnapSettings(cur_snap));
            self.workbench
                .notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }
        if cur_cfg != params.editor_state.config {
            params
                .ui_actions
                .push(EngineUiAction::UpdateEditorConfig(cur_cfg));
            self.workbench
                .notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }
        if cur_live != params.editor_state.enable_live_editor_updates {
            params
                .ui_actions
                .push(EngineUiAction::SetLiveEditorUpdates(cur_live));
            self.workbench
                .notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }

        // 4. Synchronize cached viewport screen rectangle
        self.last_viewport_rect = viewport_rect;

        ae_renderer::render::ViewportRect {
            min_x: viewport_rect.x,
            min_y: viewport_rect.y,
            max_x: viewport_rect.right(),
            max_y: viewport_rect.bottom(),
        }
    }

    /// Drains all queued actions from [`Self::action_bus`], routes dock/layout mutations internally,
    /// and passes remaining actions to `ui_actions`.
    fn dispatch_action_bus(&mut self, ui_actions: &mut Vec<EngineUiAction>) {
        if self.workbench.action_bus.is_empty() {
            return;
        }
        let drained_actions: Vec<EngineUiAction> = self.workbench.action_bus.drain().collect();
        for action in drained_actions {
            match action {
                EngineUiAction::TogglePanel(pid) => {
                    self.workbench.open_or_focus_panel(pid);
                }
                EngineUiAction::ResetLayout => {
                    self.workbench
                        .reset_dock_state(crate::ui::panel_layout::create_default_dock_state());
                }
                EngineUiAction::OpenPreferences => {
                    self.workbench.open_or_focus_panel(PanelId::Preferences);
                }
                EngineUiAction::SetWireframe(enabled) => {
                    self.wireframe_enabled = enabled;
                    self.workbench.notifier.tag_redraw(PanelId::Viewport);
                    self.workbench.notifier.tag_redraw(PanelId::Stats);
                }
                EngineUiAction::SetGridEnabled(enabled) => {
                    self.grid_enabled = enabled;
                    self.workbench.notifier.tag_redraw(PanelId::Viewport);
                    self.workbench.notifier.tag_redraw(PanelId::Stats);
                }
                EngineUiAction::SetGizmoMode(mode) => {
                    self.gizmo_mode = mode;
                    self.workbench.notifier.tag_redraw(PanelId::Viewport);
                }
                EngineUiAction::ToggleGizmoSpace => {
                    self.gizmo_space = self.gizmo_space.toggle();
                    self.workbench.notifier.tag_redraw(PanelId::Viewport);
                }
                EngineUiAction::SetUiScale(scale) => {
                    self.ui_zoom_factor = scale.clamp(0.6, 2.0);
                    ui_actions.push(EngineUiAction::SetUiScale(scale));
                }
                EngineUiAction::SetThemePreset(preset) => {
                    self.iris_overlay.preferences.theme_preset = preset;
                    let tokens = preset.tokens();
                    self.workbench.set_theme(tokens.to_iris_theme());
                    self.workbench.set_dock_style(
                        crate::ui::iris_bridge::editor_dock_style_with_tokens(&tokens),
                    );
                    self.workbench.notifier.tag_all();
                }
                EngineUiAction::PickAndAssignEntityTexture(ent) => {
                    if let Some(path) = pick_texture_file_path() {
                        ui_actions.push(EngineUiAction::AssignTextureToEntity(ent, path));
                    }
                }
                EngineUiAction::PickAndSetSubmeshTexture(model_id, idx) => {
                    if let Some(path) = pick_texture_file_path() {
                        ui_actions
                            .push(EngineUiAction::SetModelSubmeshTexture(model_id, idx, path));
                    }
                }
                other => ui_actions.push(other),
            }
        }
    }
}

fn pick_texture_file_path() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned())
}