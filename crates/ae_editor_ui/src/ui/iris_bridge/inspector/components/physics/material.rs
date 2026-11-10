// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Material Component Inspector Card
//!
//! Provides the declarative `PhysicsMaterial` inspector card implementing declarative UI rendering, surface type
//! combobox with preset action button, and friction / restitution numeric controls with two-way data-binding.

use irisui::prelude::*;

/// Maps a [`ae_core::ecs::SurfaceType`] variant to its zero-based dropdown option index.
#[inline]
#[must_use]
pub const fn surface_type_to_index(surface: ae_core::ecs::SurfaceType) -> usize {
    match surface {
        ae_core::ecs::SurfaceType::Default => 0,
        ae_core::ecs::SurfaceType::Metal => 1,
        ae_core::ecs::SurfaceType::Wood => 2,
        ae_core::ecs::SurfaceType::Stone => 3,
        ae_core::ecs::SurfaceType::Flesh => 4,
        ae_core::ecs::SurfaceType::Dirt => 5,
        ae_core::ecs::SurfaceType::Glass => 6,
        ae_core::ecs::SurfaceType::Rubber => 7,
    }
}

/// Maps a zero-based dropdown option index to its corresponding [`ae_core::ecs::SurfaceType`] variant.
#[inline]
#[must_use]
pub const fn index_to_surface_type(idx: usize) -> ae_core::ecs::SurfaceType {
    match idx {
        0 => ae_core::ecs::SurfaceType::Default,
        1 => ae_core::ecs::SurfaceType::Metal,
        2 => ae_core::ecs::SurfaceType::Wood,
        3 => ae_core::ecs::SurfaceType::Stone,
        4 => ae_core::ecs::SurfaceType::Flesh,
        5 => ae_core::ecs::SurfaceType::Dirt,
        6 => ae_core::ecs::SurfaceType::Glass,
        _ => ae_core::ecs::SurfaceType::Rubber,
    }
}

/// Renders the declarative Physics Material component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates the entire component card hierarchy, header descriptor, two-way bound surface type
/// dropdown, in-place preset reset action, and friction/restitution numeric scrubbers.
pub fn draw_physics_material_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "PhysicsMaterialCard",
        CardHeaderDescriptor::new("Physics Material")
            .icon("🧱")
            .color(Color::rgba(0.96, 0.62, 0.15, 1.0))
            .action("×", UiScope::action_tag("PhysicsMaterial")),
        world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity),
        |card, mut m| {
            card.prop_combobox_mapped_with_action(
                "Surface Type:",
                surface_type_to_index(m.surface_type),
                &[
                    "Default", "Metal", "Wood", "Stone", "Flesh", "Dirt", "Glass", "Rubber",
                ],
                "↺ Preset",
                &mut m.surface_type,
                index_to_surface_type,
            )
            .on_action(|| {
                *m = ae_core::ecs::PhysicsMaterial::from_preset(m.surface_type);
            });
            card.prop_f32("Friction:", &mut m.friction, 0.0..=100.0, 0.05);
            card.prop_f32("Restitution:", &mut m.restitution, 0.0..=1.0, 0.05);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_material_declarative_scope_renders_structure() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PhysicsMaterial {
            friction: 0.75,
            restitution: 0.25,
            surface_type: ae_core::ecs::SurfaceType::Wood,
        },));

        let mut scope = UiScope::new(&mut tree, root);
        draw_physics_material_card(&mut scope, &world, entity);

        let has_del = tree
            .iter()
            .any(|(_, n)| n.tag == UiScope::action_tag("PhysicsMaterial"));
        assert!(has_del, "Delete button must be tagged");

        let has_preset = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("PresetBtn") && n.role == WidgetRole::Button);
        assert!(has_preset, "Preset reset button must exist");

        let has_combo = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
        assert!(has_combo, "Surface Type combobox pill must exist");

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "Must emit 2 numeric drag pills (Friction & Restitution)"
        );
    }

    #[test]
    fn test_physics_material_preset_button_in_place_mutation_and_undo() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PhysicsMaterial {
            friction: 0.99,
            restitution: 0.99,
            surface_type: ae_core::ecs::SurfaceType::Rubber,
        },));

        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_physics_material_card(&mut scope, &world, entity);
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 600.0));
        }

        let preset_rect = tree.assert_node_rect("PresetBtn");
        let click_pt = Point::new(preset_rect.x + 10.0, preset_rect.y + 10.0);

        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_physics_material_card(&mut scope, &world, entity);
        }

        let expected =
            ae_core::ecs::PhysicsMaterial::from_preset(ae_core::ecs::SurfaceType::Rubber);
        let mat = world.get::<&ae_core::ecs::PhysicsMaterial>(entity).unwrap();
        assert!((mat.friction - expected.friction).abs() < f32::EPSILON);
        assert!((mat.restitution - expected.restitution).abs() < f32::EPSILON);
        assert_eq!(mat.surface_type, ae_core::ecs::SurfaceType::Rubber);
    }

    #[test]
    fn test_physics_material_surface_type_mapping_roundtrip() {
        let types = [
            ae_core::ecs::SurfaceType::Default,
            ae_core::ecs::SurfaceType::Metal,
            ae_core::ecs::SurfaceType::Wood,
            ae_core::ecs::SurfaceType::Stone,
            ae_core::ecs::SurfaceType::Flesh,
            ae_core::ecs::SurfaceType::Dirt,
            ae_core::ecs::SurfaceType::Glass,
            ae_core::ecs::SurfaceType::Rubber,
        ];
        for (i, &st) in types.iter().enumerate() {
            assert_eq!(surface_type_to_index(st), i);
            assert_eq!(index_to_surface_type(i), st);
        }
    }

    #[test]
    fn test_draw_physics_material_card_all_presets_render() {
        let presets = [
            ae_core::ecs::SurfaceType::Default,
            ae_core::ecs::SurfaceType::Metal,
            ae_core::ecs::SurfaceType::Wood,
            ae_core::ecs::SurfaceType::Stone,
            ae_core::ecs::SurfaceType::Flesh,
            ae_core::ecs::SurfaceType::Dirt,
            ae_core::ecs::SurfaceType::Glass,
            ae_core::ecs::SurfaceType::Rubber,
        ];

        for surface in presets {
            let mut tree = UiTree::new();
            let root = tree.create_root().expect("Root node must exist");
            let mut world = hecs::World::new();
            let entity = world.spawn((ae_core::ecs::PhysicsMaterial::from_preset(surface),));

            let mut scope = UiScope::new(&mut tree, root);
            draw_physics_material_card(&mut scope, &world, entity);

            let has_card = tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("PhysicsMaterialCard"));
            assert!(
                has_card,
                "PhysicsMaterialCard must render for surface {:?}",
                surface
            );
        }
    }

    #[test]
    fn test_draw_physics_material_card_missing_component_safely_ignored() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        let mut scope = UiScope::new(&mut tree, root);
        draw_physics_material_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("PhysicsMaterialCard")),
            "PhysicsMaterialCard must not be rendered when entity lacks PhysicsMaterial component"
        );
    }
}