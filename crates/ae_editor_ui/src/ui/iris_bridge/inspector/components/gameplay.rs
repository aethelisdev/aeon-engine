// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Gameplay and Motion Component Inspector Cards
//!
//! Provides handlers for `💨 CharacterAction`, `📎 Velocity`, and `🔄 Rotator`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::{CompactNumericRowParams, ComponentCategory, InspectorNumberInputId};
use super::physics::{render_component_header, render_numeric_row_compact};
use irisui::prelude::*;

/// Inspector handler for `💨 CharacterAction`.
pub struct CharacterActionHandler;

impl ComponentInspectorHandler for CharacterActionHandler {
    fn component_name(&self) -> &'static str {
        "CharacterAction"
    }

    fn display_title(&self) -> &'static str {
        "CharacterAction"
    }

    fn icon(&self) -> &'static str {
        "💨"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.68, 0.40, 0.96, 1.0) // Purple / Violet (#a855f7)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::CharacterAction>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 2.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("CharacterActionCard");
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

        let act_data = ctx
            .world
            .get::<&ae_core::ecs::CharacterAction>(ctx.entity)
            .map(|a| (a.speed, a.cooldown))
            .unwrap_or((50.0, 0.20));

        let label_w = 95.0;
        let box_w = 44.0;

        // Speed / Range [ 50 ] m/s
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Speed / Range:",
                input_id: InspectorNumberInputId::ActionSpeedRange,
                val: act_data.0,
                row_y: cur_y,
                label_w,
                box_w,
                unit: Some("m/s"),
            },
        );
        cur_y += row_h + 3.0;

        // Cooldown [ 0.20 ] s
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Cooldown:",
                input_id: InspectorNumberInputId::ActionCooldown,
                val: act_data.1,
                row_y: cur_y,
                label_w,
                box_w,
                unit: Some("s"),
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::CharacterAction::new());
    }
}

/// Inspector handler for `📎 Velocity`.
pub struct VelocityHandler;

impl ComponentInspectorHandler for VelocityHandler {
    fn component_name(&self) -> &'static str {
        "Velocity"
    }

    fn display_title(&self) -> &'static str {
        "Velocity"
    }

    fn icon(&self) -> &'static str {
        "📎"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.22, 0.74, 0.98, 1.0) // Sky Blue / Cyan (#38bdf8)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Velocity>(entity).is_ok()
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
            node.set_name("VelocityCard");
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

        let vel = ctx
            .world
            .get::<&ae_core::ecs::Velocity>(ctx.entity)
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0.0, 0.0, 0.0]);

        let label_w = 20.0;
        let box_w = 44.0;

        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "X:",
                input_id: InspectorNumberInputId::VelocityX,
                val: vel[0],
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
                label: "Y:",
                input_id: InspectorNumberInputId::VelocityY,
                val: vel[1],
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
                label: "Z:",
                input_id: InspectorNumberInputId::VelocityZ,
                val: vel[2],
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(
            entity,
            ae_core::ecs::Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
    }
}

/// Inspector handler for `🔄 Rotator`.
pub struct RotatorHandler;

impl ComponentInspectorHandler for RotatorHandler {
    fn component_name(&self) -> &'static str {
        "Rotator"
    }

    fn display_title(&self) -> &'static str {
        "Rotator (Continuous Spin)"
    }

    fn icon(&self) -> &'static str {
        "🔄"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.35, 0.85, 0.90, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Rotator>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 4.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("RotatorCard");
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

        let (speed, axis) = ctx
            .world
            .get::<&ae_core::ecs::Rotator>(ctx.entity)
            .map(|r| (r.speed, r.axis))
            .unwrap_or((1.5, [0.0, 1.0, 0.0]));

        let label_w = 95.0;
        let box_w = 44.0;

        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Speed (rad/s):",
                input_id: InspectorNumberInputId::VelocityX,
                val: speed,
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
                label: "Axis X:",
                input_id: InspectorNumberInputId::VelocityX,
                val: axis[0],
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
                label: "Axis Y:",
                input_id: InspectorNumberInputId::VelocityY,
                val: axis[1],
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
                label: "Axis Z:",
                input_id: InspectorNumberInputId::VelocityZ,
                val: axis[2],
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(
            entity,
            ae_core::ecs::Rotator {
                speed: 1.5,
                axis: [0.0, 1.0, 0.0],
            },
        );
    }
}