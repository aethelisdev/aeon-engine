// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport & Auxiliary Tool Overlay Action Dispatchers
//!
//! Handles action events for Viewport HUD, Performance Stats, Scene Hierarchy,
//! Animation Timeline, Material Studio, and UI Designer overlays.

use crate::ui::iris_bridge;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

impl EngineUi {
    /// Dispatches all pending Viewport HUD toolbar actions.
    pub fn process_viewport_hud_actions(
        &mut self,
        snapping: &mut ae_editor::snapping::SnapSettings,
        snap_changed: &mut bool,
        camera: &ae_renderer::camera::Camera,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        for action in self.iris_overlay.take_viewport_hud_actions() {
            match action {
                iris_bridge::ViewportHudAction::SelectDropdownItem(dd_id, idx) => {
                    if let Some(resolved) =
                        iris_bridge::viewport_hud::popup::resolve_viewport_hud_dropdown_action(
                            dd_id, idx, camera,
                        )
                    {
                        match resolved {
                            iris_bridge::ViewportHudAction::SetCameraMode(cmode) => {
                                ui_actions.push(EngineUiAction::SetCameraMode(cmode));
                            }
                            iris_bridge::ViewportHudAction::SetCameraTransform {
                                pitch,
                                yaw,
                                position,
                                mode,
                            } => {
                                ui_actions.push(EngineUiAction::SetCameraTransform {
                                    pitch,
                                    yaw,
                                    position,
                                    mode,
                                });
                            }
                            iris_bridge::ViewportHudAction::ToggleWireframe => {
                                self.wireframe_enabled = !self.wireframe_enabled;
                            }
                            _ => {}
                        }
                    }
                }
                iris_bridge::ViewportHudAction::SetCameraMode(cmode) => {
                    ui_actions.push(EngineUiAction::SetCameraMode(cmode));
                }
                iris_bridge::ViewportHudAction::ToggleCameraProjection => {
                    ui_actions.push(EngineUiAction::ToggleCameraProjection);
                }
                iris_bridge::ViewportHudAction::SetCameraTransform {
                    pitch,
                    yaw,
                    position,
                    mode,
                } => {
                    ui_actions.push(EngineUiAction::SetCameraTransform {
                        pitch,
                        yaw,
                        position,
                        mode,
                    });
                }
                iris_bridge::ViewportHudAction::ToggleWireframe => {
                    self.wireframe_enabled = !self.wireframe_enabled;
                }
                iris_bridge::ViewportHudAction::SetGizmoMode(gmode) => {
                    self.gizmo_mode = gmode;
                }
                iris_bridge::ViewportHudAction::ToggleGizmoSpace => {
                    self.gizmo_space = self.gizmo_space.toggle();
                }
                iris_bridge::ViewportHudAction::ToggleSnapping => {
                    snapping.mode = match snapping.mode {
                        ae_editor::snapping::SnapMode::Off => ae_editor::snapping::SnapMode::Toggle,
                        _ => ae_editor::snapping::SnapMode::Off,
                    };
                    *snap_changed = true;
                }
                iris_bridge::ViewportHudAction::ToggleDropdown(dd) => {
                    self.iris_overlay.viewport_hud.dropdown = dd;
                }
                iris_bridge::ViewportHudAction::ResumeGame => {
                    ui_actions.push(EngineUiAction::ResumeGame);
                }
                iris_bridge::ViewportHudAction::ExitToEditor => {
                    ui_actions.push(EngineUiAction::ChangeMode(
                        ae_core::modules::EngineMode::Edit,
                    ));
                }
            }
        }
    }

    /// Dispatches all pending Stats panel toggle actions.
    pub fn process_stats_actions(&mut self) {
        for action in self.iris_overlay.take_stats_actions() {
            match action {
                iris_bridge::StatsPanelAction::ToggleWireframe => {
                    self.wireframe_enabled = !self.wireframe_enabled;
                }
                iris_bridge::StatsPanelAction::ToggleGrid => {
                    self.grid_enabled = !self.grid_enabled;
                }
                iris_bridge::StatsPanelAction::Scroll(_) => {}
            }
        }
    }

    /// Dispatches all pending Scene Hierarchy panel action events.
    pub fn process_hierarchy_actions(&mut self, ui_actions: &mut Vec<EngineUiAction>) {
        for action in self.iris_overlay.take_hierarchy_actions() {
            match action {
                iris_bridge::HierarchyAction::SelectEntity(ent) => {
                    ui_actions.push(EngineUiAction::SelectEntity(ent));
                }
                iris_bridge::HierarchyAction::ToggleVisibility(ent) => {
                    ui_actions.push(EngineUiAction::ToggleVisibility(ent));
                }
                iris_bridge::HierarchyAction::DeleteSelected => {
                    ui_actions.push(EngineUiAction::DeleteSelected);
                }
                iris_bridge::HierarchyAction::SpawnShape(shape) => {
                    ui_actions.push(EngineUiAction::SpawnShape(shape));
                }
                iris_bridge::HierarchyAction::SpawnDefaultSprite => {
                    ui_actions.push(EngineUiAction::SpawnDefaultSprite);
                }
                iris_bridge::HierarchyAction::SpawnPlayerSprite => {
                    ui_actions.push(EngineUiAction::SpawnPlayerSprite);
                }
                iris_bridge::HierarchyAction::SpawnEmpty2D => {
                    ui_actions.push(EngineUiAction::SpawnEmpty2D);
                }
                iris_bridge::HierarchyAction::SpawnUiElement(elem) => {
                    ui_actions.push(EngineUiAction::SpawnUiElement(elem));
                }
                iris_bridge::HierarchyAction::OpenModelDialog => {
                    ui_actions.push(EngineUiAction::OpenModelDialog);
                }
                iris_bridge::HierarchyAction::OpenLoadPrefabDialog => {
                    ui_actions.push(EngineUiAction::OpenLoadPrefabDialog);
                }
                iris_bridge::HierarchyAction::InstantiatePrefab(path) => {
                    ui_actions.push(EngineUiAction::InstantiatePrefab(path));
                }
                iris_bridge::HierarchyAction::SpawnPhase1TestSandbox => {
                    ui_actions.push(EngineUiAction::SpawnPhase1TestSandbox);
                }
                iris_bridge::HierarchyAction::StressTest(n) => {
                    ui_actions.push(EngineUiAction::StressTest(n));
                }
                iris_bridge::HierarchyAction::AaaOpenWorldTest => {
                    ui_actions.push(EngineUiAction::AaaOpenWorldTest);
                }
                iris_bridge::HierarchyAction::Explode => {
                    ui_actions.push(EngineUiAction::Explode);
                }
                _ => {}
            }
        }
    }

    /// Drains and processes all queued actions from the Animation Timeline Studio panel.
    pub fn process_timeline_actions(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        let Some(entity) = self.selected_entity else {
            self.iris_overlay.timeline.actions.clear();
            return;
        };

        for action in self.iris_overlay.take_timeline_actions() {
            match action {
                iris_bridge::TimelineAction::TogglePlayPause => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.state = if updated.state == ae_animation::AnimationState::Playing {
                            ae_animation::AnimationState::Paused
                        } else {
                            ae_animation::AnimationState::Playing
                        };
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::Stop => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.state = ae_animation::AnimationState::Stopped;
                        updated.current_time = 0.0;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::StepFrame(delta_frames) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        let duration = updated
                            .current_clip
                            .as_ref()
                            .map_or(1.0, |c| c.duration.max(0.1));
                        let frame_step = 1.0 / 30.0;
                        updated.current_time = (updated.current_time
                            + delta_frames as f32 * frame_step)
                            .clamp(0.0, duration);
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::ToggleLoop => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.looping = !updated.looping;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::SetSpeed(speed) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        updated.speed = speed;
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::ScrubTo(time) => {
                    if let Ok(player) = world.get::<&ae_animation::AnimationPlayer>(entity) {
                        let mut updated = (*player).clone();
                        let duration = updated
                            .current_clip
                            .as_ref()
                            .map_or(1.0, |c| c.duration.max(0.1));
                        updated.current_time = time.clamp(0.0, duration);
                        ui_actions.push(EngineUiAction::modify_component(
                            entity,
                            "AnimationPlayer",
                            &updated,
                        ));
                    }
                }
                iris_bridge::TimelineAction::AddAnimationPlayer(ent) => {
                    ui_actions.push(EngineUiAction::AddComponent(ent, "AnimationPlayer"));
                }
            }
        }
    }

    /// Dispatches all pending Material & Surface Studio panel actions.
    pub fn process_material_actions(&mut self, ui_actions: &mut Vec<EngineUiAction>) {
        for action in self.iris_overlay.take_material_actions() {
            match action {
                iris_bridge::MaterialAction::AssignTextureToEntity(ent, path) => {
                    ui_actions.push(EngineUiAction::AssignTextureToEntity(ent, path));
                }
                iris_bridge::MaterialAction::RemoveTextureFromEntity(ent) => {
                    ui_actions.push(EngineUiAction::RemoveTextureFromEntity(ent));
                }
                iris_bridge::MaterialAction::SetModelSubmeshAlphaMode(model_id, idx, mode) => {
                    ui_actions.push(EngineUiAction::SetModelSubmeshAlphaMode(
                        model_id, idx, mode,
                    ));
                }
                iris_bridge::MaterialAction::SetModelSubmeshTexture(model_id, idx, path) => {
                    ui_actions.push(EngineUiAction::SetModelSubmeshTexture(model_id, idx, path));
                }
                iris_bridge::MaterialAction::PickAndAssignEntityTexture(ent) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                        .pick_file()
                    {
                        ui_actions.push(EngineUiAction::AssignTextureToEntity(
                            ent,
                            path.to_string_lossy().to_string(),
                        ));
                    }
                }
                iris_bridge::MaterialAction::PickAndSetSubmeshTexture(model_id, idx) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                        .pick_file()
                    {
                        ui_actions.push(EngineUiAction::SetModelSubmeshTexture(
                            model_id,
                            idx,
                            path.to_string_lossy().to_string(),
                        ));
                    }
                }
                iris_bridge::MaterialAction::AddColorComponent(ent) => {
                    ui_actions.push(EngineUiAction::AddComponent(ent, "Color"));
                }
                iris_bridge::MaterialAction::Scroll(_) => {}
            }
        }
    }

    /// Dispatches all pending 2D Visual UI Designer panel actions.
    pub fn process_ui_designer_actions(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
    ) {
        for action in self.iris_overlay.take_ui_designer_actions() {
            match action {
                iris_bridge::UiDesignerAction::SpawnElement(elem_type) => {
                    ui_actions.push(EngineUiAction::SpawnUiElement(elem_type));
                }
                iris_bridge::UiDesignerAction::SelectEntity(opt_ent) => {
                    self.selected_entity = opt_ent;
                    ui_actions.push(EngineUiAction::SelectEntity(opt_ent));
                }
                iris_bridge::UiDesignerAction::UpdateElementOffset { entity, offset } => {
                    if let Ok(elem) = world.get::<&ae_core::ecs::UiElement>(entity) {
                        let mut updated = *elem;
                        updated.offset = offset;
                        if let Ok(serialized) = serde_json::to_vec(&updated) {
                            ui_actions.push(EngineUiAction::ModifyComponent(
                                entity,
                                "UiElement",
                                serialized,
                            ));
                        }
                    }
                }
                iris_bridge::UiDesignerAction::SetAspectRatio(ratio) => {
                    self.ui_designer_state.aspect_ratio = ratio;
                }
                iris_bridge::UiDesignerAction::SetZoom(zoom) => {
                    self.ui_designer_state.zoom = zoom;
                }
                iris_bridge::UiDesignerAction::ToggleGrid => {
                    self.ui_designer_state.show_grid = !self.ui_designer_state.show_grid;
                }
                iris_bridge::UiDesignerAction::ToggleAnchorGuides => {
                    self.ui_designer_state.show_anchor_guides =
                        !self.ui_designer_state.show_anchor_guides;
                }
                iris_bridge::UiDesignerAction::CycleGridSnap => {
                    self.ui_designer_state.snap_grid = match self.ui_designer_state.snap_grid {
                        None => Some(8.0),
                        Some(8.0) => Some(16.0),
                        Some(16.0) => Some(32.0),
                        _ => None,
                    };
                }
                iris_bridge::UiDesignerAction::ResetView => {
                    self.ui_designer_state.zoom = 1.0;
                    self.ui_designer_state.pan_offset = [0.0, 0.0];
                }
                iris_bridge::UiDesignerAction::PanCanvas(delta) => {
                    self.ui_designer_state.pan_offset[0] += delta[0];
                    self.ui_designer_state.pan_offset[1] += delta[1];
                }
                iris_bridge::UiDesignerAction::ToggleAspectDropdown
                | iris_bridge::UiDesignerAction::ToggleAddMenu
                | iris_bridge::UiDesignerAction::ClosePopups => {}
            }
        }
    }
}