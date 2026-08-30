// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character and Player Component Inspector Cards
//!
//! Provides handlers for `🚶 Kinematic Character Controller` and `🎮 PlayerTag`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::{CompactNumericRowParams, ComponentCategory, InspectorNumberInputId};
use super::physics::{render_component_header, render_numeric_row_compact};
use irisui::prelude::*;

/// Inspector handler for `🚶 Kinematic Character Controller`.
pub struct CharacterControllerHandler;

impl ComponentInspectorHandler for CharacterControllerHandler {
    fn component_name(&self) -> &'static str {
        "CharacterController"
    }

    fn display_title(&self) -> &'static str {
        "Kinematic Character Controller"
    }

    fn icon(&self) -> &'static str {
        "🚶"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.30, 0.65, 1.0) // Vibrant Pink / Magenta (#ec4899)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 6.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("CharacterControllerCard");
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

        let char_data = ctx
            .world
            .get::<&ae_core::ecs::CharacterController>(ctx.entity)
            .map(|c| {
                (
                    c.height,
                    c.radius,
                    c.center_y,
                    c.max_slope_climb_angle,
                    c.step_height,
                    c.is_grounded,
                )
            })
            .unwrap_or((1.80, 0.40, 0.0, 45.0, 0.30, false));

        let label_w = 110.0;
        let box_w = 44.0;

        // Height
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Height:",
                input_id: InspectorNumberInputId::CharacterHeight,
                val: char_data.0,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Radius
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Radius:",
                input_id: InspectorNumberInputId::CharacterRadius,
                val: char_data.1,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Center Y
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Center Y:",
                input_id: InspectorNumberInputId::CharacterCenterY,
                val: char_data.2,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Max Slope Angle (e.g. 45°)
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Max Slope Angle:",
                input_id: InspectorNumberInputId::CharacterMaxSlope,
                val: char_data.3,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Step Height
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Step Height:",
                input_id: InspectorNumberInputId::CharacterStepHeight,
                val: char_data.4,
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Grounded Status Text (e.g. 🛡 In Air)
        let pill_rect = Rect::new(ctx.base_x + padding, cur_y + 2.0, 90.0, 18.0);
        let pill_id = tree.create_node();
        if let Some(node) = tree.get_mut(pill_id) {
            node.set_name("GroundedStatusPill");
            node.computed_rect = pill_rect;
            let (text_str, text_col) = if char_data.5 {
                ("🛡 Grounded", Color::rgba(0.20, 0.85, 0.35, 1.0))
            } else {
                ("🛡 In Air", Color::rgba(0.92, 0.70, 0.05, 1.0))
            };
            node.set_text(text_str);
            node.font_size = 10.5;
            node.line_height = 18.0;
            node.text_color = text_col;
        }
        let _ = tree.add_child(card_id, pill_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let controller = ae_core::ecs::CharacterController {
            height: 1.80,
            radius: 0.40,
            center_y: 0.0,
            max_slope_climb_angle: 45.0,
            step_height: 0.30,
            is_grounded: true,
        };
        let _ = world.insert_one(entity, controller);
    }
}

/// Inspector handler for `🎮 PlayerTag`.
pub struct PlayerTagHandler;

impl ComponentInspectorHandler for PlayerTagHandler {
    fn component_name(&self) -> &'static str {
        "PlayerTag"
    }

    fn display_title(&self) -> &'static str {
        "PlayerTag"
    }

    fn icon(&self) -> &'static str {
        "🎮"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.62, 0.15, 1.0) // Amber / Orange (#f59e0b)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::PlayerTag>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let card_h = 24.0 + 32.0 + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("PlayerTagCard");
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

        let desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(desc_id) {
            node.set_name("PlayerTagDesc");
            node.set_text("Designates this entity as the active Player target for gameplay logic and camera tracking.");
            node.font_size = 10.5;
            node.line_height = 14.0;
            node.text_color = Color::rgba(0.54, 0.56, 0.60, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                ctx.base_y + padding + 22.0,
                ctx.card_w - padding * 2.0,
                28.0,
            );
        }
        let _ = tree.add_child(card_id, desc_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::PlayerTag);
    }
}