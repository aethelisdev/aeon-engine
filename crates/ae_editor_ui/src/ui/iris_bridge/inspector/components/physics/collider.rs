// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collider Component Inspector Card
//!
//! Provides the `ColliderHandler` implementing UI rendering, shape combobox
//! selection (`Capsule`, `Box`, `Sphere`, `Trimesh`, `Convex Hull`), dynamic
//! dimension rows, and default component attachment.

use super::helpers::{
    render_checkbox_row, render_combobox_row, render_component_header, render_numeric_row_compact,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{
    ComboboxRowParams, CompactNumericRowParams, ComponentCategory, ComponentCheckboxId,
    InspectorDropdownId, InspectorNumberInputId,
};
use irisui::prelude::*;

/// Inspector handler for `🛡️ Collider`.
pub struct ColliderHandler;

impl ComponentInspectorHandler for ColliderHandler {
    fn component_name(&self) -> &'static str {
        "Collider"
    }

    fn display_title(&self) -> &'static str {
        "Collider"
    }

    fn icon(&self) -> &'static str {
        "🛡"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.88, 0.45, 1.0) // Emerald Green (#2ecc71)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Collider>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (shape, friction, restitution, is_sensor) = ctx
            .world
            .get::<&ae_core::ecs::Collider>(ctx.entity)
            .map(|c| (c.shape, c.friction, c.restitution, c.is_sensor))
            .unwrap_or((
                ae_core::ecs::ColliderShape::Capsule {
                    half_height: 0.50,
                    radius: 0.40,
                    center_y: 0.0,
                },
                0.70,
                0.0,
                false,
            ));

        let (shape_str, dim_rows) = match shape {
            ae_core::ecs::ColliderShape::Box { .. } => ("Box", 3),
            ae_core::ecs::ColliderShape::Sphere { .. } => ("Sphere", 1),
            ae_core::ecs::ColliderShape::Capsule { .. } => ("Capsule", 3),
            ae_core::ecs::ColliderShape::Trimesh => ("Trimesh", 0),
            ae_core::ecs::ColliderShape::ConvexHull => ("Convex Hull", 0),
        };

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + ((4 + dim_rows) as f32) * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ColliderCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        // Header + Trash Button
        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let mut cur_y = ctx.base_y + padding + 22.0;
        let label_w = 85.0;
        let box_w = 44.0;

        // Shape Dropdown Combo
        render_combobox_row(
            tree,
            card_id,
            ctx,
            ComboboxRowParams {
                label: "Shape:",
                selected_text: shape_str,
                dropdown_id: InspectorDropdownId::ColliderShape,
                label_w,
                row_y: cur_y,
            },
        );
        cur_y += row_h + 3.0;

        // Shape-specific dimension rows
        match shape {
            ae_core::ecs::ColliderShape::Box { half_extents } => {
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Half Extent X:",
                        input_id: InspectorNumberInputId::ColliderBoxX,
                        val: half_extents[0],
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Half Extent Y:",
                        input_id: InspectorNumberInputId::ColliderBoxY,
                        val: half_extents[1],
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Half Extent Z:",
                        input_id: InspectorNumberInputId::ColliderBoxZ,
                        val: half_extents[2],
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
            }
            ae_core::ecs::ColliderShape::Sphere { radius } => {
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Radius:",
                        input_id: InspectorNumberInputId::ColliderRadius,
                        val: radius,
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
            }
            ae_core::ecs::ColliderShape::Capsule {
                half_height,
                radius,
                center_y,
            } => {
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Half Height:",
                        input_id: InspectorNumberInputId::ColliderHalfHeight,
                        val: half_height,
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Radius:",
                        input_id: InspectorNumberInputId::ColliderRadius,
                        val: radius,
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
                render_numeric_row_compact(
                    tree,
                    card_id,
                    ctx,
                    CompactNumericRowParams {
                        label: "Center Y:",
                        input_id: InspectorNumberInputId::ColliderCenterY,
                        val: center_y,
                        row_y: cur_y,
                        label_w,
                        box_w,
                        unit: None,
                    },
                );
                cur_y += row_h + 3.0;
            }
            ae_core::ecs::ColliderShape::Trimesh | ae_core::ecs::ColliderShape::ConvexHull => {}
        }

        // Friction
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Friction:",
                input_id: InspectorNumberInputId::ColliderFriction,
                val: friction,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Restitution
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Restitution:",
                input_id: InspectorNumberInputId::ColliderRestitution,
                val: restitution,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Is Sensor (Trigger) Checkbox
        render_checkbox_row(
            tree,
            card_id,
            ctx,
            "Is Sensor (Trigger)",
            ComponentCheckboxId::ColliderIsSensor,
            is_sensor,
            cur_y,
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let collider = ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Capsule {
                half_height: 0.50,
                radius: 0.40,
                center_y: 0.0,
            },
            friction: 0.70,
            restitution: 0.0,
            is_sensor: false,
        };
        let _ = world.insert_one(entity, collider);
        if world.get::<&ae_core::ecs::RigidBody>(entity).is_err() {
            let _ = world.insert_one(entity, ae_core::ecs::RigidBody::default());
        }
    }
}