// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Rendering Subsystem
//!
//! Coordinates frame rendering across the egui docking host, Iris UI overlays,
//! and action dispatchers for settings, hierarchy, and inspector mutations.

pub mod egui_pass;
pub mod inspector_actions;
pub mod iris_pass;
pub mod overlay_actions;
pub mod types;

pub use types::EditorUiRenderParams;

use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;
use iris_pass::IrisPassParams;
use overlay_actions::PreferencesActionContext;

impl EngineUi {
    /// Orchestrates the drawing of all editor panels, toolbar menus, preference views,
    /// hierarchy snapshots, interactive HUD nodes, and overlay dialogs for the frame.
    pub fn render(&mut self, mut params: EditorUiRenderParams<'_>) -> egui::Rect {
        params.ui_actions.append(&mut self.pending_actions);

        // Update smoothed and displayed FPS counter
        let alpha = 0.08f32;
        self.smoothed_fps = alpha * params.fps + (1.0 - alpha) * self.smoothed_fps;

        let now = std::time::Instant::now();
        if now.duration_since(self.last_fps_update).as_secs_f32() >= 0.10 {
            self.displayed_fps = self.smoothed_fps;
            self.last_fps_update = now;
        }

        // 1. Primary Egui Pass (Central Docking, Menubar/Statusbar Spacers, Quick Asset Modal)
        let egui_out = self.execute_egui_pass(&mut params);

        // 2. Process Iris UI Overlay & Inspector Actions
        let mut cur_gs = (*params.graphics_settings).clone();
        let mut cur_snap = *params.snapping;
        let mut cur_cfg = params.editor_state.config.clone();
        let mut cur_live = params.editor_state.enable_live_editor_updates;
        let mut gs_changed = false;
        let mut snap_changed = false;
        let mut cfg_changed = false;
        let mut live_changed = false;

        self.process_preferences_actions(PreferencesActionContext {
            graphics_settings: &mut cur_gs,
            snapping: &mut cur_snap,
            editor_config: &mut cur_cfg,
            enable_live_updates: &mut cur_live,
            gs_changed: &mut gs_changed,
            snap_changed: &mut snap_changed,
            cfg_changed: &mut cfg_changed,
            live_changed: &mut live_changed,
            ui_actions: params.ui_actions,
        });

        self.process_viewport_hud_actions(&mut cur_snap, &mut snap_changed, params.ui_actions);

        self.process_stats_actions();

        self.process_hierarchy_actions(params.ui_actions);

        self.process_inspector_actions(params.world, params.ui_actions);

        crate::ui::panels::assets::scanner::rescan_assets_if_needed(
            &mut self.asset_browser,
            params.models,
            params.textures,
            params.shaders,
        );
        self.process_assets_actions(params.ui_actions);

        if gs_changed {
            params
                .ui_actions
                .push(EngineUiAction::UpdateGraphicsSettings(cur_gs.clone()));
        }
        if snap_changed {
            params
                .ui_actions
                .push(EngineUiAction::UpdateSnapSettings(cur_snap));
        }
        if cfg_changed {
            params
                .ui_actions
                .push(EngineUiAction::UpdateEditorConfig(cur_cfg.clone()));
        }
        if live_changed {
            params
                .ui_actions
                .push(EngineUiAction::SetLiveEditorUpdates(cur_live));
        }

        // 3. Secondary Iris UI GPU SDF Overlay Render Pass
        self.execute_iris_pass(IrisPassParams {
            device: params.device,
            queue: params.queue,
            encoder: params.encoder,
            window: params.window,
            window_surface_view: params.window_surface_view,
            is_hovering_interactive: egui_out.is_hovering_interactive,
            is_editing: *params.mode == ae_core::modules::EngineMode::Edit,
            undo_stack: params.undo_stack,
            redo_stack: params.redo_stack,
            graphics_settings: &cur_gs,
            snapping_settings: &cur_snap,
            editor_config: &cur_cfg,
            enable_live_updates: cur_live,
            enabled_modules: params.enabled_modules,
            camera: params.camera,
            world: params.world,
            stats_panel_rect: egui_out.stats_rect,
            hierarchy_panel_rect: egui_out.hierarchy_rect,
            inspector_panel_rect: egui_out.inspector_rect,
            console_panel_rect: egui_out.console_rect,
            assets_panel_rect: egui_out.assets_rect,
        });

        // 4. Viewport Texture Re-registration Check
        let mut new_rect = egui_out.viewport_rect;
        if !new_rect.is_positive()
            || !new_rect.min.x.is_finite()
            || !new_rect.min.y.is_finite()
            || !new_rect.max.x.is_finite()
            || !new_rect.max.y.is_finite()
        {
            new_rect = egui::Rect::ZERO;
        }

        if new_rect.width() != self.viewport_rect_width
            || new_rect.height() != self.viewport_rect_height
        {
            if let Some(old_id) = self.viewport_texture_id.take() {
                self.renderer.free_texture(&old_id);
            }
            self.viewport_rect_width = new_rect.width();
            self.viewport_rect_height = new_rect.height();
        }

        self.last_viewport_rect = new_rect;

        new_rect
    }
}