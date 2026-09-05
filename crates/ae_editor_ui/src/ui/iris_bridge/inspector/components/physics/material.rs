// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Material Component Inspector Card
//!
//! Provides the `PhysicsMaterialHandler` implementing UI rendering, surface type
//! combobox with preset action button, and friction / restitution numeric controls.

use super::helpers::{
    render_combobox_row_with_btn, render_component_header, render_numeric_row_compact,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{
    ComboboxWithButtonParams, CompactNumericRowParams, ComponentCategory, InspectorDropdownId,
    InspectorNumberInputId,
};
use irisui::prelude::*;

/// Inspector handler for Physics Material.
pub struct PhysicsMaterialHandler;

impl ComponentInspectorHandler for PhysicsMaterialHandler {
    fn component_name(&self) -> &'static str {
        "PhysicsMaterial"
    }

    fn display_title(&self) -> &'static str {
        "Physics Material"
    }

    fn icon(&self) -> &'static str {
        "🧱"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.62, 0.15, 1.0) // Amber / Orange (#f59e0b)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::PhysicsMaterial>(entity).is_ok()
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
            node.set_name("PhysicsMaterialCard");
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

        let mat_data = ctx
            .world
            .get::<&ae_core::ecs::PhysicsMaterial>(ctx.entity)
            .map(|m| (m.friction, m.restitution, m.surface_type.display_name()))
            .unwrap_or((0.70, 0.0, "Default"));

        // Surface Type Dropdown + Preset Reset Button
        render_combobox_row_with_btn(
            tree,
            card_id,
            ctx,
            ComboboxWithButtonParams {
                label: "Surface Type:",
                selected_text: mat_data.2,
                dropdown_id: InspectorDropdownId::SurfaceType,
                btn_label: "↺ Preset",
                row_y: cur_y,
            },
        );
        cur_y += row_h + 3.0;

        // Friction
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Friction:",
                input_id: InspectorNumberInputId::PhysMatFriction,
                val: mat_data.0,
                row_y: cur_y,
                label_w: 85.0,
                box_w: 44.0,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Restitution (Bounciness)
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Restitution (Bounciness):",
                input_id: InspectorNumberInputId::PhysMatRestitution,
                val: mat_data.1,
                row_y: cur_y,
                label_w: 155.0,
                box_w: 44.0,
                unit: None,
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let mat = ae_core::ecs::PhysicsMaterial {
            friction: 0.70,
            restitution: 0.0,
            surface_type: ae_core::ecs::SurfaceType::Default,
        };
        let _ = world.insert_one(entity, mat);
    }
}