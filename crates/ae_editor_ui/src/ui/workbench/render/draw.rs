// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame rendering execution for the editor workbench.
//!

use super::iris_pass::IrisPassParams;
use super::overlay_actions::PreferencesActionContext;
use super::types::EditorUiRenderParams;
use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;
use irisui::prelude::Rect;

impl EngineUi {
    /// Orchestrates the drawing of all editor panels, toolbar menus, preference views,
    /// hierarchy snapshots, interactive HUD nodes, and overlay dialogs for the frame.
    pub fn render(
        &mut self,
        params: EditorUiRenderParams<'_>,
    ) -> ae_renderer::render::ViewportRect {
        params.ui_actions.append(&mut self.pending_actions);

        // Track live engine frames per second (FPS) directly from timing parameters
        self.fps = params.fps;

        let win_size = params.window.inner_size();
        let zoom = self.scale_factor();
        let screen_w = win_size.width as f32 / zoom;
        let screen_h = win_size.height as f32 / zoom;

        self.layout_state.clamp_floating_windows(
            screen_w,
            screen_h,
            IrisEditorOverlay::MENUBAR_HEIGHT,
            IrisEditorOverlay::STATUS_BAR_HEIGHT,
        );

        let workspace_rect = Rect::new(
            0.0,
            IrisEditorOverlay::MENUBAR_HEIGHT,
            screen_w,
            (screen_h - IrisEditorOverlay::MENUBAR_HEIGHT - IrisEditorOverlay::STATUS_BAR_HEIGHT)
                .max(0.0),
        );

        // 1. Compute Native Docking Layout
        let computed = irisui::dock::compute_dock_layout_with_viewer(
            &self.layout_state.dock_state.tree,
            workspace_rect,
            crate::ui::iris_bridge::native_dock::SPLITTER_THICKNESS,
            crate::ui::iris_bridge::native_dock::NATIVE_DOCK_TAB_HEIGHT,
            &crate::ui::panel_layout::PanelTabViewer::default(),
        );

        let is_floating = |panel: PanelId| {
            crate::ui::iris_bridge::floating_layer::is_panel_in_floating_window(
                &self.layout_state,
                panel,
            )
        };

        let get_panel_rect = |panel: PanelId| -> Option<Rect> {
            if is_floating(panel) {
                crate::ui::iris_bridge::floating_layer::active_panel_content_rect(
                    &self.layout_state,
                    panel,
                )
            } else {
                computed
                    .leaves
                    .iter()
                    .find(|leaf| leaf.tabs.get(leaf.active_tab) == Some(&panel))
                    .map(|leaf| leaf.content_rect)
            }
        };

        let viewport_rect = if is_floating(PanelId::Viewport) {
            crate::ui::iris_bridge::floating_layer::resolve_floating_viewport_rect(
                &self.layout_state,
            )
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
        } else {
            get_panel_rect(PanelId::Viewport).unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
        };

        // Cache all active UI bounds for point occlusion testing (exclude 3D viewport canvas)
        self.ui_rects.clear();
        for leaf in &computed.leaves {
            if leaf.tabs.is_empty() {
                continue;
            }
            let is_viewport = leaf.tabs.get(leaf.active_tab) == Some(&PanelId::Viewport);
            if is_viewport {
                // For docked viewport leaf, only the top tab strip is UI; canvas belongs to 3D scene
                self.ui_rects.push(leaf.tab_bar_rect);
            } else {
                self.ui_rects.push(leaf.rect);
            }
        }
        for win in &self.layout_state.dock_state.floating_windows {
            let is_viewport = win.tree.iter().any(|(_, node)| match node {
                irisui::dock::DockNode::Leaf { tabs, active_tab } => {
                    tabs.get(*active_tab) == Some(&PanelId::Viewport)
                }
                _ => false,
            });
            if is_viewport {
                // For detached viewport window, only the top 26px title/tab strip is UI
                const TAB_BAR_H: f32 = 26.0;
                self.ui_rects
                    .push(Rect::new(win.rect.x, win.rect.y, win.rect.width, TAB_BAR_H));
            } else {
                self.ui_rects.push(Rect::new(
                    win.rect.x,
                    win.rect.y,
                    win.rect.width,
                    win.rect.height,
                ));
            }
        }

        // Synchronize inspector Euler angles on entity selection change
        let selected_ent = self
            .selected_entity
            .or_else(|| params.editor_state.selected_entities.first().copied());
        self.selected_entity = selected_ent;
        if selected_ent != self.last_selected_entity {
            self.last_selected_entity = selected_ent;
            if let Some(ent) = selected_ent
                && let Ok(rot) = params.world.get::<&ae_core::ecs::Rotation>(ent)
            {
                self.inspector_euler =
                    crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&rot);
            } else {
                self.inspector_euler = [0.0; 3];
            }
        } else if let Some(ent) = selected_ent
            && let Ok(world_rot) = params.world.get::<&ae_core::ecs::Rotation>(ent)
        {
            let is_editing_rotation = self
                .iris_overlay
                .inspector
                .drag_number
                .as_ref()
                .is_some_and(|d| {
                    matches!(
                        d.id,
                        crate::ui::iris_bridge::inspector::InspectorNumberInputId::RotX
                            | crate::ui::iris_bridge::inspector::InspectorNumberInputId::RotY
                            | crate::ui::iris_bridge::inspector::InspectorNumberInputId::RotZ
                    )
                })
                || self
                    .iris_overlay
                    .inspector
                    .active_number_input
                    .as_ref()
                    .is_some_and(|s| {
                        matches!(
                            s.id,
                            crate::ui::iris_bridge::inspector::InspectorNumberInputId::RotX
                                | crate::ui::iris_bridge::inspector::InspectorNumberInputId::RotY
                                | crate::ui::iris_bridge::inspector::InspectorNumberInputId::RotZ
                        )
                    });

            if !is_editing_rotation {
                let cached_quat = crate::ui::iris_bridge::inspector::euler_deg_to_quaternion(
                    self.inspector_euler[0],
                    self.inspector_euler[1],
                    self.inspector_euler[2],
                );
                let dot = (cached_quat.x * world_rot.x
                    + cached_quat.y * world_rot.y
                    + cached_quat.z * world_rot.z
                    + cached_quat.w * world_rot.w)
                    .abs();
                if dot < 0.999 {
                    self.inspector_euler =
                        crate::ui::iris_bridge::inspector::quaternion_to_euler_deg(&world_rot);
                }
            }
        }

        // 2. Process Iris UI Overlay & Inspector Actions
        let mut cur_gs = (*params.graphics_settings).clone();
        let mut cur_snap = *params.snapping;
        let mut cur_cfg = params.editor_state.config.clone();
        let mut cur_live = params.editor_state.enable_live_editor_updates;
        let mut gs_changed = false;
        let mut snap_changed = false;
        let mut cfg_changed = false;
        let mut live_changed = false;

        self.pending_preferences_actions
            .append(&mut self.iris_overlay.preferences.interactions.actions);

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

        for action in self.iris_overlay.take_menubar_actions() {
            match action {
                crate::ui::iris_bridge::DropdownAction::UiAction(act) => {
                    params.ui_actions.push(act);
                }
                crate::ui::iris_bridge::DropdownAction::TogglePanel(pid) => {
                    self.layout_state.activate_or_open(pid);
                }
                crate::ui::iris_bridge::DropdownAction::ResetLayout => {
                    self.layout_state.reset_to_default();
                }
                crate::ui::iris_bridge::DropdownAction::OpenPreferences => {
                    self.show_preferences = true;
                }
                crate::ui::iris_bridge::DropdownAction::OpenAbout => {
                    self.show_about = true;
                }
            }
        }

        self.process_viewport_hud_actions(
            &mut cur_snap,
            &mut snap_changed,
            params.camera,
            params.ui_actions,
        );
        self.process_stats_actions();
        self.process_hierarchy_actions(params.ui_actions);
        self.process_inspector_actions(params.world, params.ui_actions);

        crate::assets::scanner::rescan_assets_if_needed(
            &mut self.asset_browser,
            params.models,
            params.textures,
            params.shaders,
        );
        self.process_assets_actions(params.ui_actions, params.camera, params.is_2d_mode);
        self.process_timeline_actions(params.world, params.ui_actions);
        self.process_material_actions(params.ui_actions);
        self.process_ui_designer_actions(params.world, params.ui_actions);

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
            graphics_settings: &mut cur_gs,
            snapping_settings: &mut cur_snap,
            editor_config: &mut cur_cfg,
            enable_live_updates: cur_live,
            enabled_modules: params.enabled_modules,
            camera: params.camera,
            world: params.world,
            textures: params.textures,
            models: params.models,
            is_2d_mode: params.is_2d_mode,
        });

        // 3a. Immediate post-pass action drain for declarative overlay actions produced during render
        self.pending_preferences_actions
            .append(&mut self.iris_overlay.preferences.interactions.actions);
        if !self.pending_preferences_actions.is_empty() {
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
        }
        self.process_assets_actions(params.ui_actions, params.camera, params.is_2d_mode);
        self.process_inspector_actions(params.world, params.ui_actions);
        if self.iris_overlay.has_pending_actions() {
            self.iris_overlay.notifier.tag_all();
            self.iris_overlay.chrome.needs_layout_rebuild = true;
        }

        // 3b. Dispatch any settings modifications mutated in-place by declarative two-way bindings
        if !gs_changed && cur_gs != *params.graphics_settings {
            params
                .ui_actions
                .push(EngineUiAction::UpdateGraphicsSettings(cur_gs.clone()));
        }
        if !snap_changed && cur_snap != *params.snapping {
            params
                .ui_actions
                .push(EngineUiAction::UpdateSnapSettings(cur_snap));
        }
        if !cfg_changed && cur_cfg != params.editor_state.config {
            params
                .ui_actions
                .push(EngineUiAction::UpdateEditorConfig(cur_cfg.clone()));
        }

        // 4. Viewport Texture Re-registration Check
        let mut new_rect = viewport_rect;
        if new_rect.width <= 0.0
            || new_rect.height <= 0.0
            || !new_rect.x.is_finite()
            || !new_rect.y.is_finite()
            || !new_rect.width.is_finite()
            || !new_rect.height.is_finite()
        {
            new_rect = Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        if new_rect.width != self.viewport_rect_width
            || new_rect.height != self.viewport_rect_height
        {
            self.viewport_rect_width = new_rect.width;
            self.viewport_rect_height = new_rect.height;
        }

        self.last_viewport_rect = new_rect;

        ae_renderer::render::ViewportRect {
            min_x: new_rect.x,
            min_y: new_rect.y,
            max_x: new_rect.right(),
            max_y: new_rect.bottom(),
        }
    }
}