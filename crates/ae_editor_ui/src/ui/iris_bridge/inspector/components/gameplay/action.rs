// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character Action Component Inspector Card
//!
//! Provides a declarative inspector card for configuring action speed, range, and cooldown times.

use irisui::prelude::*;

/// Renders the declarative Character Action component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates speed/range and cooldown numeric scrubbers with zero intermediate variable plumbing.
pub fn draw_character_action_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "CharacterActionCard",
        UiScope::comp_card_desc(
            "CharacterAction",
            "CharacterAction",
            "💨",
            Color::rgba(0.68, 0.40, 0.96, 1.0),
        ),
        world.get::<&mut ae_core::ecs::CharacterAction>(entity),
        |card, mut a| {
            card.prop_f32("Speed / Range:", &mut a.speed, 0.0..=10_000.0, 0.5);
            card.prop_f32("Cooldown:", &mut a.cooldown, 0.0..=3_600.0, 0.05);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_action_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::CharacterAction::new(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut scope = UiScope::new(&mut tree, root);
        draw_character_action_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("CharacterAction");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "CharacterAction card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "CharacterAction card must emit 2 numeric pills (Speed, Cooldown)"
        );

        // Test in-place mutation
        {
            let mut a = world
                .get::<&mut ae_core::ecs::CharacterAction>(entity)
                .unwrap();
            a.speed = 120.0;
            a.cooldown = 1.5;
        }

        {
            let a = world.get::<&ae_core::ecs::CharacterAction>(entity).unwrap();
            assert_eq!(a.speed, 120.0);
            assert_eq!(a.cooldown, 1.5);
        }
    }

    #[test]
    fn test_draw_character_action_card_direct_structure() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::CharacterAction::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_character_action_card(&mut scope, &world, entity);

        assert!(
            tree.has_node_named("CharacterActionCard"),
            "CharacterActionCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "draw_character_action_card must render 2 NumericDrag widgets"
        );
    }

    #[test]
    fn test_draw_character_action_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_character_action_card(&mut scope, &world, entity);

        assert_eq!(
            tree.len(),
            1,
            "CharacterAction card must safely do nothing when component missing"
        );
    }
}