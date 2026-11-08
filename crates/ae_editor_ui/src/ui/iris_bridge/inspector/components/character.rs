// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character and Player Component Inspector Cards
//!
//! Provides handlers for `🚶 Kinematic Character Controller` and `🎮 PlayerTag`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut c) = ctx
            .world
            .get::<&mut ae_core::ecs::CharacterController>(ctx.entity)
        else {
            return;
        };

        scope.card_panel("CharacterControllerCard", |card| {
            self.render_header(card);

            ctx.prop_f32(
                card,
                self.component_name(),
                "Height:",
                &mut c.height,
                0.05..=1_000.0,
                0.05,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Radius:",
                &mut c.radius,
                0.01..=500.0,
                0.02,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Center Y:",
                &mut c.center_y,
                -1_000.0..=1_000.0,
                0.05,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Max Slope Angle:",
                &mut c.max_slope_climb_angle,
                0.0..=89.9,
                0.5,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Step Height:",
                &mut c.step_height,
                0.0..=100.0,
                0.02,
            );

            // Grounded Status Text (e.g. 🛡 In Air / 🛡 Grounded)
            card.container_named(
                "GroundedStatusPill",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .height(18.0),
                |status_scope| {
                    if c.is_grounded {
                        status_scope.label_styled_passive(
                            "GroundedStatusText",
                            "🛡 Grounded",
                            10.5,
                            status_scope.theme().success,
                            TextAlign::Left,
                            Style::new().height(18.0),
                        );
                    } else {
                        status_scope.label_styled_passive(
                            "GroundedStatusText",
                            "🛡 In Air",
                            10.5,
                            status_scope.theme().warning,
                            TextAlign::Left,
                            Style::new().height(18.0),
                        );
                    }
                },
            );
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        scope.card_panel("PlayerTagCard", |card| {
            self.render_header(card);

            card.label_styled_passive(
                "PlayerTagDesc",
                "Designates this entity as the active Player target",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new().margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
            );
            card.label_styled_passive(
                "PlayerTagDescSub",
                "for gameplay logic and camera tracking.",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new(),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::PlayerTag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_player_tag_declarative_render() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PlayerTag,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        let handler = PlayerTagHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("PlayerTag");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "PlayerTag card must tag its delete button with semantic delete tag"
        );
    }

    #[test]
    fn test_character_controller_declarative_render_and_grounded_status() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::CharacterController {
            height: 2.0,
            radius: 0.5,
            center_y: 0.1,
            max_slope_climb_angle: 40.0,
            step_height: 0.25,
            is_grounded: true,
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        let mut ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            10.0,
            20.0,
            260.0,
        );

        let handler = CharacterControllerHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("CharacterController");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "CharacterController card must tag its delete button with semantic delete tag"
        );

        let has_grounded_text = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("🛡 Grounded"));
        assert!(
            has_grounded_text,
            "CharacterController must display '🛡 Grounded' text when is_grounded is true"
        );

        // Verify initial state
        {
            let c = world
                .get::<&ae_core::ecs::CharacterController>(entity)
                .unwrap();
            assert_eq!(c.height, 2.0);
            assert_eq!(c.radius, 0.5);
            assert_eq!(c.center_y, 0.1);
            assert_eq!(c.max_slope_climb_angle, 40.0);
            assert_eq!(c.step_height, 0.25);
            assert!(c.is_grounded);
        }

        // Test in-place mutation of fields with undo snapshotting
        {
            let mut c = world
                .get::<&mut ae_core::ecs::CharacterController>(entity)
                .unwrap();
            ctx.begin_edit(handler.component_name());
            c.height = 1.75;
            c.radius = 0.35;
            c.center_y = -0.05;
            c.max_slope_climb_angle = 50.0;
            c.step_height = 0.40;
            c.is_grounded = false;
            ctx.commit_edit(handler.component_name());
        }

        // Verify mutation reflected in ECS world
        {
            let c = world
                .get::<&ae_core::ecs::CharacterController>(entity)
                .unwrap();
            assert_eq!(c.height, 1.75);
            assert_eq!(c.radius, 0.35);
            assert_eq!(c.center_y, -0.05);
            assert_eq!(c.max_slope_climb_angle, 50.0);
            assert_eq!(c.step_height, 0.40);
            assert!(!c.is_grounded);
        }
    }
}