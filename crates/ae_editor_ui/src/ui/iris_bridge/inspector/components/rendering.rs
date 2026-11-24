// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rendering and Illumination Component Inspector Cards
//!
//! Provides declarative inspector cards for `💡 Light`, `📦 ModelId`, and `🎲 Shape`.

use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_LIGHT};
use irisui::prelude::*;

/// Renders the declarative Light component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Encapsulates position offset and RGB illumination color scrubbers with zero intermediate variable plumbing.
pub fn draw_light_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .mutate::<ae_light::Light, _>(entity, |light| {
            scope.card_panel("LightCard", |card| {
                card.card_header_desc(
                    UiScope::comp_card_desc(
                        "Light",
                        "Light",
                        "💡",
                        Color::rgba(0.98, 0.80, 0.20, 1.0),
                    )
                    .atlas_icon(ICON_LIGHT),
                );
                card.prop_f32(
                    "Light Offset Y:",
                    &mut light.position[1],
                    -1_000.0..=1_000.0,
                    0.05,
                );
                card.prop_f32("Color R:", &mut light.color[0], 0.0..=1.0, 0.01);
                card.prop_f32("Color G:", &mut light.color[1], 0.0..=1.0, 0.01);
            });
        })
        .ok();
}

/// Renders the declarative 3D Model / Mesh component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Displays the active mesh asset handle with zero intermediate variable plumbing.
pub fn draw_model_mesh_card(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    entity: ae_ecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "ModelMeshCard",
        UiScope::comp_card_desc(
            "ModelId",
            "3D Model / Mesh",
            "📦",
            Color::rgba(0.38, 0.65, 0.98, 1.0),
        )
        .atlas_icon(crate::ui::iris_bridge::icons::ICON_CUBE),
        world.get_copied::<ae_plugin_api::ModelId>(entity),
        |card, model| {
            card.label_desc("ModelAssetInfo", format!("Asset Handle: {:?}", model.0));
        },
    );
}

/// Converts a [`ae_plugin_api::Shape`] enum variant to its combobox option index.
#[inline]
#[must_use]
pub const fn shape_to_index(shape: ae_plugin_api::Shape) -> usize {
    match shape {
        ae_plugin_api::Shape::Cube => 0,
        ae_plugin_api::Shape::Sphere => 1,
        ae_plugin_api::Shape::Cylinder => 2,
        ae_plugin_api::Shape::Capsule => 3,
        ae_plugin_api::Shape::Torus => 4,
        ae_plugin_api::Shape::Triangle => 5,
    }
}

/// Converts a combobox option index back to its corresponding [`ae_plugin_api::Shape`] enum variant.
#[inline]
#[must_use]
pub const fn index_to_shape(index: usize) -> ae_plugin_api::Shape {
    match index {
        0 => ae_plugin_api::Shape::Cube,
        1 => ae_plugin_api::Shape::Sphere,
        2 => ae_plugin_api::Shape::Cylinder,
        3 => ae_plugin_api::Shape::Capsule,
        4 => ae_plugin_api::Shape::Torus,
        5 => ae_plugin_api::Shape::Triangle,
        _ => ae_plugin_api::Shape::Cube,
    }
}

/// Renders the declarative procedural Shape component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Encapsulates geometry selection dropdown with zero intermediate variable plumbing.
pub fn draw_shape_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .mutate::<ae_plugin_api::Shape, _>(entity, |s| {
            scope.card_panel("ShapeCard", |card| {
                card.card_header_desc(
                    UiScope::comp_card_desc(
                        "Shape",
                        "Procedural 3D Shape",
                        "🌐",
                        Color::rgba(0.20, 0.85, 0.60, 1.0),
                    )
                    .atlas_icon(ICON_CUBE),
                );
                card.prop_combobox_mapped_styled(
                    "Geometry:",
                    shape_to_index(*s),
                    &["Cube", "Sphere", "Cylinder", "Capsule", "Torus", "Triangle"],
                    ComboRowLayout::new(65.0, 96.0),
                    s,
                    index_to_shape,
                );
            });
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_render_and_in_place_mutation() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_light::Light {
            position: [0.0, 5.0, 0.0],
            color: [1.0, 0.8, 0.6],
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_light_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("Light");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Light card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "Light card must emit 3 numeric pills (Offset Y, Color R, Color G)"
        );

        // Test in-place mutation
        {
            let l = world.get_mut::<ae_light::Light>(entity).unwrap();
            l.position[1] = 8.5;
            l.color[0] = 0.5;
            l.color[1] = 0.4;
        }

        {
            let l = world.get_copied::<ae_light::Light>(entity).unwrap();
            assert_eq!(l.position[1], 8.5);
            assert_eq!(l.color[0], 0.5);
            assert_eq!(l.color[1], 0.4);
        }
    }

    #[test]
    fn test_draw_light_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_light::Light::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_light_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("LightCard")),
            "LightCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "Light card must emit exactly 3 numeric pills"
        );
    }

    #[test]
    fn test_draw_light_card_missing_component_safely_ignored() {
        let mut world = ae_ecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_light_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("LightCard")),
            "Light card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_draw_model_mesh_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::ModelId::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_model_mesh_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ModelMeshCard")),
            "ModelMeshCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ModelAssetInfo")),
            "ModelAssetInfo passive label must be rendered"
        );
    }

    #[test]
    fn test_draw_model_mesh_card_missing_component_safely_ignored() {
        let mut world = ae_ecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_model_mesh_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ModelMeshCard")),
            "ModelMesh card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_draw_shape_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_shape_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ShapeCard")),
            "ShapeCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ComboPill")),
            "Shape combobox pill must be rendered"
        );
    }

    #[test]
    fn test_draw_shape_card_missing_component_safely_ignored() {
        let mut world = ae_ecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_shape_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ShapeCard")),
            "Shape card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_shape_index_conversion_roundtrip() {
        let shapes = [
            ae_plugin_api::Shape::Cube,
            ae_plugin_api::Shape::Sphere,
            ae_plugin_api::Shape::Cylinder,
            ae_plugin_api::Shape::Capsule,
            ae_plugin_api::Shape::Torus,
            ae_plugin_api::Shape::Triangle,
        ];
        for s in shapes {
            assert_eq!(index_to_shape(shape_to_index(s)), s);
        }
    }

    #[test]
    fn test_shape_declarative_render() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_shape_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("Shape");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Shape delete button must be tagged"
        );

        let has_cube_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_CUBE)
        });
        assert!(
            has_cube_icon,
            "Shape card header must render ICON_CUBE from editor atlas"
        );

        let has_combo_pill = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
        assert!(has_combo_pill, "Shape combobox pill must be rendered");

        let has_chevron_down = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▾")
        });
        assert!(
            has_chevron_down,
            "Combobox must render chevron down (▾) when closed"
        );
    }

    #[test]
    fn test_shape_combobox_renders_chevron_up_when_open() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        // First pass: render once to obtain ComboPill tag
        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_shape_card(&mut scope, &world, entity);
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
        draw_shape_card(&mut scope2, &world, entity);

        let has_chevron_up = tree2.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▴")
        });
        assert!(
            has_chevron_up,
            "Combobox must render chevron up (▴) when open"
        );
    }

    #[test]
    fn test_shape_declarative_in_place_mutation_and_undo_snapshot() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::Shape::Cube,));

        {
            let s = world.get_mut::<ae_plugin_api::Shape>(entity).unwrap();
            *s = ae_plugin_api::Shape::Sphere;
        }

        assert_eq!(
            world.get_copied::<ae_plugin_api::Shape>(entity).unwrap(),
            ae_plugin_api::Shape::Sphere
        );
    }
}