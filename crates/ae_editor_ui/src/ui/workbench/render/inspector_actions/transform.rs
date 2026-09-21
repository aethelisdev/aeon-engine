// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Transform & Object Color Action Handlers
//!
//! Provides handlers for resetting position, rotation, and scale components,
//! as well as applying color palette mutations to selected scene entities.

use crate::ui::iris_bridge::inspector::TransformAxisType;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;

impl EngineUi {
    /// Resets Position, Rotation, or Scale transform components to canonical defaults.
    ///
    /// # Arguments
    /// * `world` - Hecs ECS world reference.
    /// * `ui_actions` - Queue of dispatchable engine UI undo/redo actions.
    /// * `entity` - Target entity undergoing transform reset.
    /// * `axis` - Which transform channel (Position, Rotation, Scale) to reset.
    pub(crate) fn handle_reset_transform(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
        entity: hecs::Entity,
        axis: TransformAxisType,
    ) {
        match axis {
            TransformAxisType::Position => {
                let old_pos = world
                    .get::<&ae_core::ecs::Position>(entity)
                    .map(|p| *p)
                    .unwrap_or(ae_core::ecs::Position {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    });
                let new_pos = ae_core::ecs::Position {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                };
                ui_actions.push(EngineUiAction::ModifyPosition(entity, old_pos, new_pos));
            }
            TransformAxisType::Rotation => {
                let old_rot = world
                    .get::<&ae_core::ecs::Rotation>(entity)
                    .map(|r| *r)
                    .unwrap_or_else(|_| ae_core::ecs::Rotation::identity());
                let new_rot = ae_core::ecs::Rotation::identity();
                self.inspector_euler = [0.0, 0.0, 0.0];
                ui_actions.push(EngineUiAction::ModifyRotation(entity, old_rot, new_rot));
            }
            TransformAxisType::Scale => {
                let old_scale = world
                    .get::<&ae_core::ecs::Scale>(entity)
                    .map(|s| *s)
                    .unwrap_or(ae_core::ecs::Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    });
                let new_scale = ae_core::ecs::Scale {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                };
                ui_actions.push(EngineUiAction::ModifyScale(entity, old_scale, new_scale));
            }
        }
    }

    /// Updates object color and synchronizes UI hex string and HSV pickers.
    ///
    /// # Arguments
    /// * `world` - Hecs ECS world reference.
    /// * `ui_actions` - Queue of dispatchable engine UI undo/redo actions.
    /// * `entity` - Target entity receiving color change.
    /// * `col` - New RGBA color value.
    pub(crate) fn handle_set_object_color(
        &mut self,
        world: &hecs::World,
        ui_actions: &mut Vec<EngineUiAction>,
        entity: hecs::Entity,
        col: irisui::prelude::Color,
    ) {
        let old_col = world
            .get::<&ae_core::ecs::Color>(entity)
            .map(|c| *c)
            .unwrap_or(ae_core::ecs::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            });
        let new_col = ae_core::ecs::Color {
            r: col.r,
            g: col.g,
            b: col.b,
            a: col.a,
        };
        let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
        self.inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        let (h, s, v) = irisui::prelude::rgb_to_hsv(col.r, col.g, col.b);
        self.iris_overlay.inspector.hsv = [h, s, v];
        ui_actions.push(EngineUiAction::ModifyColor(entity, old_col, new_col));
    }
}