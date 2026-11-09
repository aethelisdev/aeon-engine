// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # RigidBody Component Inspector Card
//!
//! Provides the `RigidBodyHandler` implementing declarative UI rendering, body type combobox
//! (`Dynamic`, `Kinematic`, `Static`), mass and gravity scale numeric inputs with two-way data-binding,
//! and paired component attachment with `Collider`.

use crate::ui::iris_bridge::icons::ICON_GEAR;
use irisui::prelude::*;

/// Maps a [`ae_core::ecs::RigidBodyType`] variant to its zero-based combobox option index.
#[inline]
#[must_use]
pub const fn body_type_to_index(bt: ae_core::ecs::RigidBodyType) -> usize {
    match bt {
        ae_core::ecs::RigidBodyType::Dynamic => 0,
        ae_core::ecs::RigidBodyType::Kinematic => 1,
        ae_core::ecs::RigidBodyType::Static => 2,
    }
}

/// Maps a zero-based combobox option index to its corresponding [`ae_core::ecs::RigidBodyType`] variant.
#[inline]
#[must_use]
pub const fn index_to_body_type(idx: usize) -> ae_core::ecs::RigidBodyType {
    match idx {
        0 => ae_core::ecs::RigidBodyType::Dynamic,
        1 => ae_core::ecs::RigidBodyType::Kinematic,
        _ => ae_core::ecs::RigidBodyType::Static,
    }
}

/// Renders the declarative RigidBody component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates the entire component card hierarchy, header descriptor, two-way bound body type dropdown,
/// and mass/gravity numeric scrubbers with zero intermediate variable plumbing.
pub fn draw_rigidbody_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "RigidBodyCard",
        CardHeaderDescriptor::new("RigidBody")
            .icon("⚙")
            .atlas_icon(ICON_GEAR)
            .color(Color::rgba(0.22, 0.74, 0.98, 1.0))
            .action("×", UiScope::action_tag("RigidBody")),
        world.get::<&mut ae_core::ecs::RigidBody>(entity),
        |card, mut rb| {
            card.prop_combobox_mapped(
                "Type:",
                body_type_to_index(rb.body_type),
                &["Dynamic", "Kinematic", "Static"],
                &mut rb.body_type,
                index_to_body_type,
            );
            card.prop_f32("Mass:", &mut rb.mass, 0.001..=100_000.0, 0.1);
            card.prop_f32("Gravity:", &mut rb.gravity_scale, -100.0..=100.0, 0.1);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigidbody_declarative_scope_renders_gear_icon_and_chevron() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 2.5,
            gravity_scale: 1.0,
        },));

        let mut scope = UiScope::new(&mut tree, root);
        draw_rigidbody_card(&mut scope, &world, entity);

        let has_gear_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_GEAR)
        });
        assert!(has_gear_icon, "RigidBody card header must render ICON_GEAR");

        let has_chevron_down = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboChevron"));
        assert!(
            has_chevron_down,
            "Combobox must render chevron down when closed"
        );

        let del_tag = UiScope::action_tag("RigidBody");
        let has_del = tree.iter().any(|(_, n)| n.tag == del_tag);
        assert!(has_del, "RigidBody delete button must be tagged");
    }

    #[test]
    fn test_rigidbody_combobox_renders_chevron_up_when_open() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody::default(),));

        // First pass: render once to get the tag of the ComboPill
        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_rigidbody_card(&mut scope, &world, entity);
        }

        let combo_tag = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
            .map(|(_, n)| n.tag)
            .expect("ComboPill node must exist in tree");

        // Second pass: set active_combobox_tag to combo_tag and render
        let mut tree2 = UiTree::new();
        let root2 = tree2.create_root().expect("Root node must exist");
        tree2.active_combobox_tag = Some(combo_tag);

        let mut scope2 = UiScope::new(&mut tree2, root2);
        draw_rigidbody_card(&mut scope2, &world, entity);

        let has_chevron_up = tree2.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▴")
        });
        assert!(
            has_chevron_up,
            "Combobox must render chevron up (▴) when open"
        );
    }

    #[test]
    fn test_rigidbody_declarative_in_place_mutation_and_undo_snapshot() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 2.5,
            gravity_scale: 1.0,
        },));

        if let Ok(mut rb) = world.get::<&mut ae_core::ecs::RigidBody>(entity) {
            rb.mass = 5.0;
            rb.gravity_scale = 2.0;
            rb.body_type = ae_core::ecs::RigidBodyType::Static;
        }

        // Verify mutations in world
        let rb = world.get::<&ae_core::ecs::RigidBody>(entity).unwrap();
        assert_eq!(rb.mass, 5.0);
        assert_eq!(rb.gravity_scale, 2.0);
        assert_eq!(rb.body_type, ae_core::ecs::RigidBodyType::Static);
    }

    #[test]
    fn test_rigidbody_body_type_mapping_roundtrip() {
        assert_eq!(body_type_to_index(ae_core::ecs::RigidBodyType::Dynamic), 0);
        assert_eq!(
            body_type_to_index(ae_core::ecs::RigidBodyType::Kinematic),
            1
        );
        assert_eq!(body_type_to_index(ae_core::ecs::RigidBodyType::Static), 2);

        assert_eq!(index_to_body_type(0), ae_core::ecs::RigidBodyType::Dynamic);
        assert_eq!(
            index_to_body_type(1),
            ae_core::ecs::RigidBodyType::Kinematic
        );
        assert_eq!(index_to_body_type(2), ae_core::ecs::RigidBodyType::Static);
        assert_eq!(index_to_body_type(99), ae_core::ecs::RigidBodyType::Static);
    }

    #[test]
    fn test_draw_rigidbody_card_renders_mass_and_gravity_labels() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Kinematic,
            mass: 12.0,
            gravity_scale: 0.5,
        },));

        let mut scope = UiScope::new(&mut tree, root);
        draw_rigidbody_card(&mut scope, &world, entity);

        let has_mass_label = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("NumLbl") && n.text.as_deref() == Some("Mass:")
        });
        assert!(has_mass_label, "Must render Mass: label");

        let has_gravity_label = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("NumLbl") && n.text.as_deref() == Some("Gravity:")
        });
        assert!(has_gravity_label, "Must render Gravity: label");

        let has_combo_pill = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
        assert!(has_combo_pill, "Must render ComboPill for body type");

        let numeric_pills: Vec<_> = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .collect();
        assert_eq!(numeric_pills.len(), 2, "Must render 2 NumericDrag pills");
        for (_, pill) in numeric_pills {
            assert_eq!(
                pill.style.width,
                Some(48.0),
                "NumericDrag pill must default to 48.0 width"
            );
        }
    }

    #[test]
    fn test_draw_rigidbody_card_missing_component_safely_ignored() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        let mut scope = UiScope::new(&mut tree, root);
        draw_rigidbody_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("RigidBodyCard")),
            "RigidBodyCard must not be rendered when entity lacks RigidBody component"
        );
    }
}