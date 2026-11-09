// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character and Player Component Inspector Cards
//!
//! Provides handlers for `🚶 Kinematic Character Controller` and `🎮 PlayerTag`.

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

/// Renders the declarative Kinematic Character Controller component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates numeric dimension scrubbers (Height, Radius, Center Y, Max Slope Angle, Step Height)
/// and grounded status indicator with zero intermediate variable plumbing.
pub fn draw_character_controller_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "CharacterControllerCard",
        UiScope::comp_card_desc(
            "CharacterController",
            "Kinematic Character Controller",
            "🚶",
            Color::rgba(0.96, 0.30, 0.65, 1.0),
        ),
        world.get::<&mut ae_core::ecs::CharacterController>(entity),
        |card, mut c| {
            card.prop_f32("Height:", &mut c.height, 0.05..=1_000.0, 0.05);
            card.prop_f32("Radius:", &mut c.radius, 0.01..=500.0, 0.02);
            card.prop_f32("Center Y:", &mut c.center_y, -1_000.0..=1_000.0, 0.05);
            card.prop_f32(
                "Max Slope Angle:",
                &mut c.max_slope_climb_angle,
                0.0..=89.9,
                0.5,
            );
            card.prop_f32("Step Height:", &mut c.step_height, 0.0..=100.0, 0.02);
            card.status_pill(
                "GroundedStatusPill",
                "GroundedStatusText",
                grounded_label(c.is_grounded),
                grounded_color(card.theme(), c.is_grounded),
            );
        },
    );
}

/// Renders the declarative PlayerTag component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Indicates active player designation for gameplay logic and camera tracking.
pub fn draw_player_tag_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "PlayerTagCard",
        UiScope::component_card_desc("PlayerTag", "🎮", Color::rgba(0.96, 0.62, 0.15, 1.0)),
        world.get::<&ae_core::ecs::PlayerTag>(entity),
        |card, _tag| {
            card.label_desc(
                "PlayerTagDesc",
                "Designates this entity as the active Player target",
            );
            card.label_desc(
                "PlayerTagDescSub",
                "for gameplay logic and camera tracking.",
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_tag_declarative_render() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PlayerTag,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_player_tag_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("PlayerTag");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "PlayerTag card must tag its delete button with semantic delete tag"
        );
    }

    #[test]
    fn test_draw_player_tag_card_direct_structure() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PlayerTag,));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_player_tag_card(&mut scope, &world, entity);

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
    fn test_draw_player_tag_card_missing_component_safely_ignored() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_player_tag_card(&mut scope, &world, entity);

        assert!(
            !tree.has_node_named("PlayerTagCard"),
            "PlayerTagCard must not be rendered when entity lacks PlayerTag"
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

        let mut scope = UiScope::new(&mut tree, root);
        draw_character_controller_card(&mut scope, &world, entity);

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

        // Test in-place mutation of fields
        {
            let mut c = world
                .get::<&mut ae_core::ecs::CharacterController>(entity)
                .unwrap();
            c.height = 1.75;
            c.radius = 0.35;
            c.center_y = -0.05;
            c.max_slope_climb_angle = 50.0;
            c.step_height = 0.40;
            c.is_grounded = false;
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
        let mut scope = UiScope::new(&mut tree, root);

        draw_character_controller_card(&mut scope, &world, entity);

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

    #[test]
    fn test_draw_character_controller_card_missing_component_safely_ignored() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_character_controller_card(&mut scope, &world, entity);

        assert!(
            !tree.has_node_named("CharacterControllerCard"),
            "CharacterControllerCard must not be rendered when entity lacks component"
        );
    }
}