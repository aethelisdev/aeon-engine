// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Action Dispatcher
//!
//! Handles interaction and state update events originating from the 2D UI Designer canvas,
//! element tree, transform manipulators, and inspector property editors.
//!

use crate::ui::iris_bridge;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

impl EngineUi {
    /// Dispatches all pending 2D Visual UI Designer panel actions.
    ///
    /// Translates declarative [`UiDesignerAction`](crate::ui::iris_bridge::UiDesignerAction)
    /// events into ECS entity mutations and editor commands.
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