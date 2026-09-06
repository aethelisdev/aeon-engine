// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Visual UI Designer & Canvas Editor Docking Panel.
//!
//! Bridges the standalone `ae_uidesign` (AUD) crate into the `ae_editor_ui` docking system.
//!

use crate::ui::types::EngineUiAction;
pub use ae_uidesign::{CanvasAspectRatio, UiDesignerState, UiDragState, UiElementType};

/// Context parameters passed into the UI Designer panel renderer.
pub struct UiDesignerContext<'a> {
    pub world: &'a hecs::World,
    pub selected_entity: Option<hecs::Entity>,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub state: &'a mut UiDesignerState,
}

/// Renders the 2D UI Designer panel frame, canvas, anchor lines, and widget overlays.
pub fn draw_ui_designer_panel(ui: &mut egui::Ui, ctx: &mut UiDesignerContext<'_>) {
    let mut actions = Vec::new();
    let mut aud_ctx = ae_uidesign::UiDesignerContext {
        world: ctx.world,
        selected_entity: ctx.selected_entity,
        actions: &mut actions,
        state: ctx.state,
    };

    ae_uidesign::draw_ui_designer_panel(ui, &mut aud_ctx);

    for action in actions {
        match action {
            ae_uidesign::UiDesignerAction::SpawnElement(elem_type) => {
                ctx.ui_actions
                    .push(EngineUiAction::SpawnUiElement(elem_type));
            }
            ae_uidesign::UiDesignerAction::SelectEntity(opt_ent) => {
                ctx.ui_actions.push(EngineUiAction::SelectEntity(opt_ent));
            }
            ae_uidesign::UiDesignerAction::UpdateElementOffset { entity, offset } => {
                if let Ok(elem) = ctx.world.get::<&ae_core::ecs::UiElement>(entity) {
                    let mut updated = *elem;
                    updated.offset = offset;
                    if let Ok(serialized) = serde_json::to_vec(&updated) {
                        ctx.ui_actions.push(EngineUiAction::ModifyComponent(
                            entity,
                            "UiElement",
                            serialized,
                        ));
                    }
                }
            }
            ae_uidesign::UiDesignerAction::SetAspectRatio(ratio) => {
                ctx.state.aspect_ratio = ratio;
            }
            ae_uidesign::UiDesignerAction::SetZoom(zoom) => {
                ctx.state.zoom = zoom;
            }
            ae_uidesign::UiDesignerAction::ToggleGrid => {
                ctx.state.show_grid = !ctx.state.show_grid;
            }
            ae_uidesign::UiDesignerAction::ToggleAnchorGuides => {
                ctx.state.show_anchor_guides = !ctx.state.show_anchor_guides;
            }
            ae_uidesign::UiDesignerAction::CycleGridSnap => {
                ctx.state.snap_grid = match ctx.state.snap_grid {
                    None => Some(8.0),
                    Some(8.0) => Some(16.0),
                    Some(16.0) => Some(32.0),
                    _ => None,
                };
            }
            ae_uidesign::UiDesignerAction::ResetView => {
                ctx.state.zoom = 1.0;
                ctx.state.pan_offset = [0.0, 0.0];
            }
            ae_uidesign::UiDesignerAction::PanCanvas(delta) => {
                ctx.state.pan_offset[0] += delta[0];
                ctx.state.pan_offset[1] += delta[1];
            }
        }
    }
}