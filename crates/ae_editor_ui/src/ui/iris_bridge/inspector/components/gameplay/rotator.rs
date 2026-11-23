// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rotator Component Inspector Card
//!
//! Provides a declarative inspector card for continuous rotation components around an arbitrary 3D axis.

use irisui::prelude::*;

/// Renders the declarative Rotator component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Encapsulates continuous rotation speed and 3D rotational axis vector scrubbers with zero intermediate variable plumbing.
pub fn draw_rotator_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .mutate::<ae_plugin_api::Rotator, _>(entity, |r| {
            scope.card_panel("RotatorCard", |card| {
                card.card_header_desc(
                    CardHeaderDescriptor::new("Rotator (Continuous Spin)")
                        .icon("🔄")
                        .color(Color::rgba(0.35, 0.85, 0.90, 1.0))
                        .action("×", UiScope::action_tag("Rotator")),
                );
                card.prop_f32("Speed (rad/s):", &mut r.speed, -100.0..=100.0, 0.05);
                card.prop_vec3("Axis:", &mut r.axis, 0.05, Some([0.0, 1.0, 0.0]));
            });
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotator_render_and_in_place_mutation() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Rotator {
            speed: 1.5,
            axis: [0.0, 1.0, 0.0],
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut scope = UiScope::new(&mut tree, root);
        draw_rotator_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("Rotator");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Rotator card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.role == irisui::core::WidgetRole::NumericInput)
            .count();
        assert_eq!(
            numeric_pills, 4,
            "Rotator card must emit 4 numeric inputs (1 Speed + 3 Axis axes)"
        );

        // Test in-place mutation
        {
            let r = world.get_mut::<ae_plugin_api::Rotator>(entity).unwrap();
            r.speed = 2.5;
            r.axis = [1.0, 0.0, 0.0];
        }

        {
            let r = world.get_copied::<ae_plugin_api::Rotator>(entity).unwrap();
            assert_eq!(r.speed, 2.5);
            assert_eq!(r.axis, [1.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn test_draw_rotator_card_direct_structure() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Rotator {
            speed: 1.5,
            axis: [0.0, 1.0, 0.0],
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_rotator_card(&mut scope, &world, entity);

        assert!(
            tree.has_node_named("RotatorCard"),
            "RotatorCard container must be rendered"
        );
        assert!(
            tree.has_node_named("NumBox_Axis:_X"),
            "NumBox_Axis:_X must be present"
        );
        assert!(
            tree.has_node_named("NumBox_Axis:_Y"),
            "NumBox_Axis:_Y must be present"
        );
        assert!(
            tree.has_node_named("NumBox_Axis:_Z"),
            "NumBox_Axis:_Z must be present"
        );
    }

    #[test]
    fn test_draw_rotator_card_missing_component_safely_ignored() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn(());
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_rotator_card(&mut scope, &world, entity);

        assert!(
            !tree.has_node_named("RotatorCard"),
            "RotatorCard must not be rendered when entity lacks Rotator"
        );
    }
}