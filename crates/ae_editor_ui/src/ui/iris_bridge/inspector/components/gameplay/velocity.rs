// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Velocity Component Inspector Card
//!
//! Provides a declarative inspector card for 3D linear velocity components (`X`, `Y`, `Z`).

use irisui::prelude::*;

/// Renders the declarative Velocity component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates 3D linear velocity coordinate vector scrubbers (X, Y, Z) with zero intermediate variable plumbing.
pub fn draw_velocity_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok_mut(
        "VelocityCard",
        UiScope::component_card_desc("Velocity", "📎", Color::rgba(0.22, 0.74, 0.98, 1.0)),
        world.get::<&mut ae_core::ecs::Velocity>(entity),
        |card, v| {
            card.prop_vec3_xyz(
                "Linear",
                (&mut v.x, &mut v.y, &mut v.z),
                0.05,
                Some([0.0, 0.0, 0.0]),
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut scope = UiScope::new(&mut tree, root);
        draw_velocity_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("Velocity");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Velocity card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| {
                n.role == irisui::core::WidgetRole::NumericInput
                    && n.name
                        .as_deref()
                        .unwrap_or("")
                        .starts_with("NumBox_Linear_")
            })
            .count();
        assert_eq!(
            numeric_pills, 3,
            "Velocity card must create exactly 3 numeric inputs for X, Y, Z"
        );

        // Test in-place mutation
        {
            let mut v = world.get::<&mut ae_core::ecs::Velocity>(entity).unwrap();
            v.x = 5.5;
            v.y = -4.2;
            v.z = 10.0;
        }

        {
            let v = world.get::<&ae_core::ecs::Velocity>(entity).unwrap();
            assert_eq!(v.x, 5.5);
            assert_eq!(v.y, -4.2);
            assert_eq!(v.z, 10.0);
        }
    }

    #[test]
    fn test_draw_velocity_card_direct_structure() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_velocity_card(&mut scope, &world, entity);

        assert!(
            tree.has_node_named("VelocityCard"),
            "VelocityCard container must be rendered"
        );
        assert!(
            tree.has_node_named("NumBox_Linear_X"),
            "NumBox_Linear_X must be present"
        );
        assert!(
            tree.has_node_named("NumBox_Linear_Y"),
            "NumBox_Linear_Y must be present"
        );
        assert!(
            tree.has_node_named("NumBox_Linear_Z"),
            "NumBox_Linear_Z must be present"
        );
    }

    #[test]
    fn test_draw_velocity_card_missing_component_safely_ignored() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_velocity_card(&mut scope, &world, entity);

        assert!(
            !tree.has_node_named("VelocityCard"),
            "VelocityCard must not be rendered when entity lacks Velocity"
        );
    }
}