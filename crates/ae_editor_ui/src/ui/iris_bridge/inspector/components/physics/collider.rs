// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collider Component Inspector Card
//!
//! Provides the declarative `Collider` inspector card implementing declarative UI rendering, shape combobox
//! selection (`Capsule`, `Box`, `Sphere`, `Trimesh`, `Convex Hull`), dynamic dimension
//! scalar drag bindings, sensor toggle, and default component attachment.

use irisui::prelude::*;

/// Maps a [`ae_core::ecs::ColliderShape`] to its corresponding zero-based combobox option index.
#[inline]
#[must_use]
pub const fn shape_to_index(shape: ae_core::ecs::ColliderShape) -> usize {
    match shape {
        ae_core::ecs::ColliderShape::Capsule { .. } => 0,
        ae_core::ecs::ColliderShape::Box { .. } => 1,
        ae_core::ecs::ColliderShape::Sphere { .. } => 2,
        ae_core::ecs::ColliderShape::Trimesh => 3,
        ae_core::ecs::ColliderShape::ConvexHull => 4,
    }
}

/// Maps a zero-based combobox option index to its corresponding canonical default [`ae_core::ecs::ColliderShape`].
#[inline]
#[must_use]
pub const fn index_to_shape(idx: usize) -> ae_core::ecs::ColliderShape {
    match idx {
        0 => ae_core::ecs::ColliderShape::Capsule {
            half_height: 0.50,
            radius: 0.40,
            center_y: 0.0,
        },
        1 => ae_core::ecs::ColliderShape::Box {
            half_extents: [0.50, 0.50, 0.50],
        },
        2 => ae_core::ecs::ColliderShape::Sphere { radius: 0.50 },
        3 => ae_core::ecs::ColliderShape::Trimesh,
        _ => ae_core::ecs::ColliderShape::ConvexHull,
    }
}

/// Renders numeric property scrubbers for a box collider's 3D half-extents.
#[inline]
fn draw_box_collider_fields(card: &mut UiScope<'_>, half_extents: &mut [f32; 3]) {
    card.prop_f32(
        "Half Extent X:",
        &mut half_extents[0],
        0.001..=10_000.0,
        0.05,
    );
    card.prop_f32(
        "Half Extent Y:",
        &mut half_extents[1],
        0.001..=10_000.0,
        0.05,
    );
    card.prop_f32(
        "Half Extent Z:",
        &mut half_extents[2],
        0.001..=10_000.0,
        0.05,
    );
}

/// Renders numeric property scrubber for a sphere collider's radius.
#[inline]
fn draw_sphere_collider_fields(card: &mut UiScope<'_>, radius: &mut f32) {
    card.prop_f32("Radius:", radius, 0.001..=10_000.0, 0.05);
}

/// Renders numeric property scrubbers for a capsule collider's height, radius, and vertical offset.
#[inline]
fn draw_capsule_collider_fields(
    card: &mut UiScope<'_>,
    half_height: &mut f32,
    radius: &mut f32,
    center_y: &mut f32,
) {
    card.prop_f32("Half Height:", half_height, 0.001..=10_000.0, 0.05);
    card.prop_f32("Radius:", radius, 0.001..=10_000.0, 0.05);
    card.prop_f32("Center Y:", center_y, -10_000.0..=10_000.0, 0.05);
}

/// Renders the declarative Collider component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates the entire component card hierarchy, header descriptor, two-way bound shape dropdown,
/// dynamic shape dimension scrubbers, friction/restitution controls, and trigger sensor toggle.
pub fn draw_collider_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "ColliderCard",
        CardHeaderDescriptor::new("Collider")
            .icon("🛡")
            .atlas_icon(crate::ui::iris_bridge::icons::ICON_WIREFRAME)
            .color(Color::rgba(0.20, 0.88, 0.45, 1.0))
            .action("×", UiScope::action_tag("Collider")),
        world.get::<&mut ae_core::ecs::Collider>(entity),
        |card, mut c| {
            card.prop_combobox_mapped(
                "Shape:",
                shape_to_index(c.shape),
                &["Capsule", "Box", "Sphere", "Trimesh", "Convex Hull"],
                &mut c.shape,
                index_to_shape,
            );

            match &mut c.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => {
                    draw_box_collider_fields(card, half_extents);
                }
                ae_core::ecs::ColliderShape::Sphere { radius } => {
                    draw_sphere_collider_fields(card, radius);
                }
                ae_core::ecs::ColliderShape::Capsule {
                    half_height,
                    radius,
                    center_y,
                } => {
                    draw_capsule_collider_fields(card, half_height, radius, center_y);
                }
                ae_core::ecs::ColliderShape::Trimesh => {
                    card.label("Trimesh (Auto from Mesh)");
                }
                ae_core::ecs::ColliderShape::ConvexHull => {
                    card.label("Convex Hull (Auto from Mesh)");
                }
            }

            card.prop_f32("Friction:", &mut c.friction, 0.0..=2.0, 0.02);
            card.prop_f32("Restitution:", &mut c.restitution, 0.0..=1.0, 0.02);
            card.prop_checkbox("Is Sensor (Trigger)", &mut c.is_sensor);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::icons::ICON_WIREFRAME;

    #[test]
    fn test_collider_declarative_scope_renders_wireframe_and_chevron() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [1.0, 2.0, 3.0],
            },
            friction: 0.5,
            restitution: 0.2,
            is_sensor: true,
        },));

        let mut scope = UiScope::new(&mut tree, root);
        draw_collider_card(&mut scope, &world, entity);

        let has_wireframe_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_WIREFRAME)
        });
        assert!(
            has_wireframe_icon,
            "Collider card header must render ICON_WIREFRAME"
        );

        let has_chevron_down = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboChevron"));
        assert!(
            has_chevron_down,
            "Collider Shape combobox must render chevron down when closed"
        );

        let del_tag = UiScope::action_tag("Collider");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Collider delete button must be tagged"
        );
    }

    #[test]
    fn test_collider_declarative_in_place_mutation_and_undo() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [1.0, 2.0, 3.0],
            },
            friction: 0.5,
            restitution: 0.2,
            is_sensor: false,
        },));

        let mut scope = UiScope::new(&mut tree, root);
        draw_collider_card(&mut scope, &world, entity);

        // Verify initial state
        {
            let c = world.get::<&ae_core::ecs::Collider>(entity).unwrap();
            assert_eq!(
                c.shape,
                ae_core::ecs::ColliderShape::Box {
                    half_extents: [1.0, 2.0, 3.0]
                }
            );
            assert_eq!(c.friction, 0.5);
            assert_eq!(c.restitution, 0.2);
            assert!(!c.is_sensor);
        }

        // Test in-place mutation of fields
        {
            let mut c = world.get::<&mut ae_core::ecs::Collider>(entity).unwrap();
            if let ae_core::ecs::ColliderShape::Box {
                ref mut half_extents,
            } = c.shape
            {
                half_extents[0] = 4.5;
            }
            c.friction = 0.85;
            c.is_sensor = true;
        }

        // Verify mutation reflected in ECS world
        {
            let c = world.get::<&ae_core::ecs::Collider>(entity).unwrap();
            assert_eq!(
                c.shape,
                ae_core::ecs::ColliderShape::Box {
                    half_extents: [4.5, 2.0, 3.0]
                }
            );
            assert_eq!(c.friction, 0.85);
            assert!(c.is_sensor);
        }
    }

    #[test]
    fn test_collider_shape_mapping_roundtrip() {
        assert_eq!(
            shape_to_index(ae_core::ecs::ColliderShape::Capsule {
                half_height: 1.0,
                radius: 0.5,
                center_y: 0.0,
            }),
            0
        );
        assert_eq!(
            shape_to_index(ae_core::ecs::ColliderShape::Box {
                half_extents: [1.0, 1.0, 1.0],
            }),
            1
        );
        assert_eq!(
            shape_to_index(ae_core::ecs::ColliderShape::Sphere { radius: 1.0 }),
            2
        );
        assert_eq!(shape_to_index(ae_core::ecs::ColliderShape::Trimesh), 3);
        assert_eq!(shape_to_index(ae_core::ecs::ColliderShape::ConvexHull), 4);

        match index_to_shape(0) {
            ae_core::ecs::ColliderShape::Capsule { .. } => {}
            _ => panic!("Expected Capsule"),
        }
        match index_to_shape(1) {
            ae_core::ecs::ColliderShape::Box { .. } => {}
            _ => panic!("Expected Box"),
        }
        match index_to_shape(2) {
            ae_core::ecs::ColliderShape::Sphere { .. } => {}
            _ => panic!("Expected Sphere"),
        }
        assert_eq!(index_to_shape(3), ae_core::ecs::ColliderShape::Trimesh);
        assert_eq!(index_to_shape(4), ae_core::ecs::ColliderShape::ConvexHull);
    }

    #[test]
    fn test_draw_collider_card_all_shapes_render() {
        let shapes = [
            ae_core::ecs::ColliderShape::Capsule {
                half_height: 0.5,
                radius: 0.4,
                center_y: 0.0,
            },
            ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            ae_core::ecs::ColliderShape::Sphere { radius: 0.5 },
            ae_core::ecs::ColliderShape::Trimesh,
            ae_core::ecs::ColliderShape::ConvexHull,
        ];

        for shape in shapes {
            let mut tree = UiTree::new();
            let root = tree.create_root().expect("Root node must exist");
            let mut world = hecs::World::new();
            let entity = world.spawn((ae_core::ecs::Collider {
                shape,
                friction: 0.5,
                restitution: 0.0,
                is_sensor: false,
            },));

            let mut scope = UiScope::new(&mut tree, root);
            draw_collider_card(&mut scope, &world, entity);

            let has_card = tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ColliderCard"));
            assert!(has_card, "ColliderCard must render for shape {:?}", shape);
        }
    }

    #[test]
    fn test_draw_collider_card_sensor_checkbox_click_toggle() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [1.0, 1.0, 1.0],
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let tag = {
            let mut scope = UiScope::new(&mut tree, root);
            draw_collider_card(&mut scope, &world, entity);
            scope.tag_for("Is Sensor (Trigger)")
        };
        assert!(
            !world
                .get::<&ae_core::ecs::Collider>(entity)
                .unwrap()
                .is_sensor
        );

        // Click toggle
        tree.event_state.frame_clicked_tag = Some(tag);
        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_collider_card(&mut scope, &world, entity);
        }
        assert!(
            world
                .get::<&ae_core::ecs::Collider>(entity)
                .unwrap()
                .is_sensor,
            "Click must set is_sensor=true in ECS"
        );

        // Next frame unclicked: must stay true
        tree.event_state.frame_clicked_tag = None;
        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_collider_card(&mut scope, &world, entity);
        }
        assert!(
            world
                .get::<&ae_core::ecs::Collider>(entity)
                .unwrap()
                .is_sensor,
            "is_sensor must persist as true"
        );
    }

    #[test]
    fn test_draw_collider_card_missing_component_safely_ignored() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        let mut scope = UiScope::new(&mut tree, root);
        draw_collider_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ColliderCard")),
            "ColliderCard must not be rendered when entity lacks Collider component"
        );
    }
}