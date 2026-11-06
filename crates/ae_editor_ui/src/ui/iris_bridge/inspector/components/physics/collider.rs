// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collider Component Inspector Card
//!
//! Provides the `ColliderHandler` implementing declarative UI rendering, shape combobox
//! selection (`Capsule`, `Box`, `Sphere`, `Trimesh`, `Convex Hull`), dynamic dimension
//! scalar drag bindings, sensor toggle, and default component attachment.

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext, PropComboMapConfig, PropComboRowLayout,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for Collider.
pub struct ColliderHandler;

impl ColliderHandler {
    #[inline]
    fn shape_to_index(shape: ae_core::ecs::ColliderShape) -> usize {
        match shape {
            ae_core::ecs::ColliderShape::Capsule { .. } => 0,
            ae_core::ecs::ColliderShape::Box { .. } => 1,
            ae_core::ecs::ColliderShape::Sphere { .. } => 2,
            ae_core::ecs::ColliderShape::Trimesh => 3,
            ae_core::ecs::ColliderShape::ConvexHull => 4,
        }
    }

    #[inline]
    fn index_to_shape(idx: usize) -> ae_core::ecs::ColliderShape {
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
}

impl ComponentInspectorHandler for ColliderHandler {
    fn component_name(&self) -> &'static str {
        "Collider"
    }

    fn display_title(&self) -> &'static str {
        "Collider"
    }

    fn icon(&self) -> &'static str {
        "🛡"
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(crate::ui::iris_bridge::icons::ICON_WIREFRAME)
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.88, 0.45, 1.0) // Emerald Green (#2ecc71)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Collider>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::Collider>(ctx.entity) else {
            return;
        };

        scope.card_panel("ColliderCard", |card| {
            self.render_header(card);

            ctx.prop_combobox_mapped(
                card,
                self.component_name(),
                PropComboMapConfig::new(
                    "Shape:",
                    Self::shape_to_index(c.shape),
                    &["Capsule", "Box", "Sphere", "Trimesh", "Convex Hull"],
                    PropComboRowLayout::default(),
                ),
                &mut c.shape,
                Self::index_to_shape,
            );

            match &mut c.shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => {
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Half Extent X:",
                        &mut half_extents[0],
                        0.001..=10_000.0,
                        0.05,
                    );
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Half Extent Y:",
                        &mut half_extents[1],
                        0.001..=10_000.0,
                        0.05,
                    );
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Half Extent Z:",
                        &mut half_extents[2],
                        0.001..=10_000.0,
                        0.05,
                    );
                }
                ae_core::ecs::ColliderShape::Sphere { radius } => {
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Radius:",
                        radius,
                        0.001..=10_000.0,
                        0.05,
                    );
                }
                ae_core::ecs::ColliderShape::Capsule {
                    half_height,
                    radius,
                    center_y,
                } => {
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Half Height:",
                        half_height,
                        0.001..=10_000.0,
                        0.05,
                    );
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Radius:",
                        radius,
                        0.001..=10_000.0,
                        0.05,
                    );
                    ctx.prop_f32(
                        card,
                        self.component_name(),
                        "Center Y:",
                        center_y,
                        -10_000.0..=10_000.0,
                        0.05,
                    );
                }
                ae_core::ecs::ColliderShape::Trimesh => {
                    card.label("Trimesh (Auto from Mesh)");
                }
                ae_core::ecs::ColliderShape::ConvexHull => {
                    card.label("Convex Hull (Auto from Mesh)");
                }
            }

            ctx.prop_f32(
                card,
                self.component_name(),
                "Friction:",
                &mut c.friction,
                0.0..=2.0,
                0.02,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Restitution:",
                &mut c.restitution,
                0.0..=1.0,
                0.02,
            );
            ctx.prop_checkbox(
                card,
                self.component_name(),
                "Is Sensor (Trigger)",
                &mut c.is_sensor,
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let collider = ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Capsule {
                half_height: 0.50,
                radius: 0.40,
                center_y: 0.0,
            },
            friction: 0.70,
            restitution: 0.0,
            is_sensor: false,
        };
        let _ = world.insert_one(entity, collider);
        if world.get::<&ae_core::ecs::RigidBody>(entity).is_err() {
            let _ = world.insert_one(entity, ae_core::ecs::RigidBody::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::icons::ICON_WIREFRAME;
    use crate::ui::iris_bridge::inspector::InspectorAction;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_collider_declarative_scope_renders_wireframe_and_chevron() {
        let handler = ColliderHandler;
        assert_eq!(handler.atlas_icon(), Some(ICON_WIREFRAME));

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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 50.0, 280.0);

        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

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

        let del_tag = encode_component_delete_tag("Collider");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Collider delete button must be tagged"
        );
    }

    #[test]
    fn test_collider_declarative_in_place_mutation_and_undo() {
        let handler = ColliderHandler;
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
            50.0,
            280.0,
        );

        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

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

        // Test in-place mutation of fields with undo snapshotting
        {
            let mut c = world.get::<&mut ae_core::ecs::Collider>(entity).unwrap();
            ctx.begin_edit(handler.component_name());
            if let ae_core::ecs::ColliderShape::Box {
                ref mut half_extents,
            } = c.shape
            {
                half_extents[0] = 4.5;
            }
            c.friction = 0.85;
            c.is_sensor = true;
            ctx.commit_edit(handler.component_name());
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

        // Verify undo actions were captured
        assert!(actions.contains(&InspectorAction::StartComponentEdit(
            entity,
            handler.component_name(),
            None,
        )));
        assert!(actions.contains(&InspectorAction::CommitComponentEdit(
            entity,
            handler.component_name()
        )));
    }
}