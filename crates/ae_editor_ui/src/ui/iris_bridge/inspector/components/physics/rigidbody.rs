// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # RigidBody Component Inspector Card
//!
//! Provides the `RigidBodyHandler` implementing UI rendering, body type combobox
//! (`Dynamic`, `Kinematic`, `Static`), mass and gravity scale numeric inputs,
//! and paired component attachment with `Collider`.

use super::helpers::{render_component_header, render_numeric_row_compact};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{
    CompactNumericRowParams, ComponentCategory, InspectorDropdownId, InspectorNumberInputId,
};
use irisui::prelude::*;

/// Inspector handler for `⚙ RigidBody`.
pub struct RigidBodyHandler;

impl ComponentInspectorHandler for RigidBodyHandler {
    fn component_name(&self) -> &'static str {
        "RigidBody"
    }

    fn display_title(&self) -> &'static str {
        "RigidBody"
    }

    fn icon(&self) -> &'static str {
        "⚙"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.22, 0.74, 0.98, 1.0) // Sky Blue / Cyan (#38bdf8)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::RigidBody>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 3.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("RigidBodyCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

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

        let rb_data = ctx
            .world
            .get::<&ae_core::ecs::RigidBody>(ctx.entity)
            .map(|rb| (rb.body_type, rb.mass, rb.gravity_scale))
            .unwrap_or((ae_core::ecs::RigidBodyType::Kinematic, 1.0, 1.0));

        let body_type_str = match rb_data.0 {
            ae_core::ecs::RigidBodyType::Dynamic => "Dynamic",
            ae_core::ecs::RigidBodyType::Kinematic => "Kinematic",
            ae_core::ecs::RigidBodyType::Static => "Static",
        };

        // Row 1: Body Type Dropdown (Snug left-aligned)
        let combo_w = 90.0;
        let combo_rect = Rect::new(ctx.base_x + padding, cur_y, combo_w, row_h);
        let is_open = ctx.params.active_dropdown == Some(InspectorDropdownId::RigidBodyType);
        let is_hovered = combo_rect.contains_point(ctx.params.cursor_pos);

        let combo_node_id = tree.create_node();
        if let Some(node) = tree.get_mut(combo_node_id) {
            node.set_name("RigidBodyTypeComboPill");
            node.computed_rect = combo_rect;
            let (bg, border) = if is_open {
                (
                    Color::rgba(0.118, 0.125, 0.145, 1.0),
                    Color::rgba(0.353, 0.376, 0.439, 0.95),
                )
            } else if is_hovered {
                (
                    Color::rgba(0.200, 0.208, 0.235, 1.0),
                    Color::rgba(0.271, 0.282, 0.329, 0.95),
                )
            } else {
                (
                    Color::rgba(0.157, 0.165, 0.188, 0.98),
                    Color::rgba(0.212, 0.220, 0.259, 0.85),
                )
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, border)
                .border_radius(5.0);
        }
        let _ = tree.add_child(card_id, combo_node_id);

        let txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(txt_id) {
            node.set_name("RigidBodyTypeComboTxt");
            let arrow = if is_open { "▲" } else { "▼" };
            node.set_text(format!("{}  {}", body_type_str, arrow));
            node.font_size = 10.5;
            node.line_height = row_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_open {
                Color::WHITE
            } else {
                Color::rgba(0.886, 0.894, 0.918, 1.0)
            };
            node.computed_rect = combo_rect;
        }
        let _ = tree.add_child(combo_node_id, txt_id);

        ctx.targets
            .dropdowns
            .push((InspectorDropdownId::RigidBodyType, combo_rect, 0));
        cur_y += row_h + 3.0;

        // Row 2: Mass [ 1.0 ]
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Mass:",
                input_id: InspectorNumberInputId::RigidBodyMass,
                val: rb_data.1,
                row_y: cur_y,
                label_w: 55.0,
                box_w: 44.0,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Row 3: Gravity [ 1.00 ]
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Gravity:",
                input_id: InspectorNumberInputId::RigidBodyGravity,
                val: rb_data.2,
                row_y: cur_y,
                label_w: 55.0,
                box_w: 44.0,
                unit: None,
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::RigidBody::default());
        if world.get::<&ae_core::ecs::Collider>(entity).is_err() {
            let _ = world.insert_one(entity, ae_core::ecs::Collider::default());
        }
    }
}