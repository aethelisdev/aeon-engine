// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character and Player Component Inspector Cards
//!
//! Provides handlers for `🚶 Kinematic Character Controller` and `🎮 PlayerTag`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use irisui::prelude::*;

#[inline]
const fn grounded_label(is_grounded: bool) -> &'static str {
    if is_grounded {
        "🛡 Grounded"
    } else {
        "🛡 In Air"
    }
}

#[inline]
fn grounded_color(theme: Theme, is_grounded: bool) -> Color {
    if is_grounded {
        theme.success
    } else {
        theme.warning
    }
}

/// Renders the declarative Kinematic Character Controller component card with numeric
/// dimension scrubbers (Height, Radius, Center Y, Max Slope Angle, Step Height) and grounded status indicator.
pub fn draw_character_controller_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    c: &mut ae_core::ecs::CharacterController,
) {
    scope.card_panel("CharacterControllerCard", |card| {
        CharacterControllerHandler.render_header(card);

        card.prop_f32("Height:", &mut c.height, 0.05..=1_000.0, 0.05)
            .on_commit(|_| {
                ctx.begin_edit("CharacterController");
                ctx.commit_edit("CharacterController");
            });
        card.prop_f32("Radius:", &mut c.radius, 0.01..=500.0, 0.02)
            .on_commit(|_| {
                ctx.begin_edit("CharacterController");
                ctx.commit_edit("CharacterController");
            });
        card.prop_f32("Center Y:", &mut c.center_y, -1_000.0..=1_000.0, 0.05)
            .on_commit(|_| {
                ctx.begin_edit("CharacterController");
                ctx.commit_edit("CharacterController");
            });
        card.prop_f32(
            "Max Slope Angle:",
            &mut c.max_slope_climb_angle,
            0.0..=89.9,
            0.5,
        )
        .on_commit(|_| {
            ctx.begin_edit("CharacterController");
            ctx.commit_edit("CharacterController");
        });
        card.prop_f32("Step Height:", &mut c.step_height, 0.0..=100.0, 0.02)
            .on_commit(|_| {
                ctx.begin_edit("CharacterController");
                ctx.commit_edit("CharacterController");
            });

        card.status_pill(
            "GroundedStatusPill",
            "GroundedStatusText",
            grounded_label(c.is_grounded),
            grounded_color(card.theme(), c.is_grounded),
        );
    });
}

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
        scope.when_ok(
            ctx.world
                .get::<&mut ae_core::ecs::CharacterController>(ctx.entity),
            |scope, mut c| {
                draw_character_controller_card(scope, ctx, &mut c);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(
            entity,
            ae_core::ecs::CharacterController {
                height: 1.80,
                radius: 0.40,
                center_y: 0.0,
                max_slope_climb_angle: 45.0,
                step_height: 0.30,
                is_grounded: true,
            },
        ));
    }
}

/// Renders the declarative PlayerTag component card indicating active player designation.
pub fn draw_player_tag_card(scope: &mut UiScope<'_>) {
    scope.card_panel("PlayerTagCard", |card| {
        PlayerTagHandler.render_header(card);

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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when(
            self.has_component(ctx.world, ctx.entity),
            draw_player_tag_card,
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::PlayerTag));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
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

        let del_tag = UiScope::action_tag("PlayerTag");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "PlayerTag card must tag its delete button with semantic delete tag"
        );
    }

    #[test]
    fn test_draw_player_tag_card_direct_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_player_tag_card(&mut scope);

        assert!(
            tree.has_node_named("PlayerTagCard"),
            "PlayerTagCard container must be rendered"
        );
        assert!(
            tree.has_node_named("PlayerTagDesc"),
            "PlayerTagDesc label must be present"
        );
        assert!(
            tree.has_node_named("PlayerTagDescSub"),
            "PlayerTagDescSub label must be present"
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

        let del_tag = UiScope::action_tag("CharacterController");
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

    #[test]
    fn test_draw_character_controller_card_in_air_state() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::CharacterController {
            height: 1.80,
            radius: 0.40,
            center_y: 0.0,
            max_slope_climb_angle: 45.0,
            step_height: 0.30,
            is_grounded: false,
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
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
        let mut scope = UiScope::new(&mut tree, root);
        let mut c = world
            .get::<&mut ae_core::ecs::CharacterController>(entity)
            .unwrap();

        draw_character_controller_card(&mut scope, &mut ctx, &mut c);

        let has_in_air = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("🛡 In Air"));
        assert!(
            has_in_air,
            "draw_character_controller_card must render '🛡 In Air' when is_grounded is false"
        );

        let has_card = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("CharacterControllerCard"));
        assert!(has_card, "CharacterControllerCard must be rendered");
    }
}