// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Moving Platform Component Inspector Card
//!
//! Provides a declarative inspector card for waypoint-driven moving platforms.

use irisui::prelude::*;

/// Renders the declarative Moving Platform component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Encapsulates speed scrubber and target waypoint position 3D vector with zero intermediate variable plumbing.
pub fn draw_moving_platform_card(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    entity: ae_ecs::Entity,
) {
    world
        .mutate::<ae_plugin_api::MovingPlatform, _>(entity, |plat| {
            scope.card_panel("MovingPlatformCard", |card| {
                card.card_header_desc(UiScope::comp_card_desc(
                    "MovingPlatform",
                    "Moving Platform",
                    "🚡",
                    Color::rgba(0.70, 0.50, 1.0, 1.0),
                ));
                card.prop_f32("Speed (m/s):", &mut plat.speed, 0.0..=100.0, 0.1);
                card.prop_vec3("Target Pos:", &mut plat.target_position, 0.1, None);
            });
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_platform_render_tag() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::MovingPlatform::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut scope = UiScope::new(&mut tree, root);
        draw_moving_platform_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("MovingPlatform");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let has_speed = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NumBox_Target Pos:_X"));
        assert!(has_speed, "MovingPlatform must render target pos X box");
    }

    #[test]
    fn test_draw_moving_platform_card_direct_structure() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::MovingPlatform::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_moving_platform_card(&mut scope, &world, entity);

        assert!(
            tree.has_node_named("MovingPlatformCard"),
            "MovingPlatformCard container must be rendered"
        );
        assert!(
            tree.has_node_named("NumBox_Target Pos:_X"),
            "NumBox_Target Pos:_X must be present"
        );
    }

    #[test]
    fn test_draw_moving_platform_card_missing_component_safely_ignored() {
        let mut world = ae_ecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_moving_platform_card(&mut scope, &world, entity);

        assert_eq!(
            tree.len(),
            1,
            "MovingPlatform card must safely do nothing when component missing"
        );
    }
}