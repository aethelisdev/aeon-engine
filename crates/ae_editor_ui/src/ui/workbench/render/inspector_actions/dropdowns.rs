// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Dropdown Selection Handlers
//!
//! Maps dropdown selection events to ECS physics, collider, shape, and UI components.

use crate::ui::iris_bridge::inspector::InspectorDropdownId;

/// Mutates ECS components from Inspector dropdown selection.
///
/// # Arguments
/// * `world` - Hecs ECS world reference.
/// * `entity` - Target entity being modified.
/// * `dd_id` - Identifier of the modified dropdown component.
/// * `opt_idx` - Zero-based index of the chosen option item.
pub(crate) fn handle_select_dropdown(
    world: &hecs::World,
    entity: hecs::Entity,
    dd_id: InspectorDropdownId,
    opt_idx: usize,
) {
    match dd_id {
        InspectorDropdownId::RigidBodyType => {
            let bt = match opt_idx {
                0 => ae_core::ecs::RigidBodyType::Dynamic,
                1 => ae_core::ecs::RigidBodyType::Kinematic,
                2 => ae_core::ecs::RigidBodyType::Static,
                _ => ae_core::ecs::RigidBodyType::Kinematic,
            };
            if let Ok(mut rb) = world.get::<&mut ae_core::ecs::RigidBody>(entity) {
                rb.body_type = bt;
            }
        }
        InspectorDropdownId::ColliderShape => {
            if let Ok(mut c) = world.get::<&mut ae_core::ecs::Collider>(entity) {
                c.shape = match opt_idx {
                    0 => ae_core::ecs::ColliderShape::Capsule {
                        half_height: 0.5,
                        radius: 0.4,
                        center_y: 0.0,
                    },
                    1 => ae_core::ecs::ColliderShape::Box {
                        half_extents: [0.5, 0.5, 0.5],
                    },
                    2 => ae_core::ecs::ColliderShape::Sphere { radius: 0.5 },
                    3 => ae_core::ecs::ColliderShape::Trimesh,
                    4 => ae_core::ecs::ColliderShape::ConvexHull,
                    _ => c.shape,
                };
            }
        }
        InspectorDropdownId::SurfaceType => {
            let surf = match opt_idx {
                0 => ae_core::ecs::SurfaceType::Default,
                1 => ae_core::ecs::SurfaceType::Metal,
                2 => ae_core::ecs::SurfaceType::Wood,
                3 => ae_core::ecs::SurfaceType::Stone,
                4 => ae_core::ecs::SurfaceType::Flesh,
                5 => ae_core::ecs::SurfaceType::Dirt,
                6 => ae_core::ecs::SurfaceType::Glass,
                7 => ae_core::ecs::SurfaceType::Rubber,
                _ => ae_core::ecs::SurfaceType::Default,
            };
            if let Ok(mut m) = world.get::<&mut ae_core::ecs::PhysicsMaterial>(entity) {
                *m = ae_core::ecs::PhysicsMaterial::from_preset(surf);
            }
        }
        InspectorDropdownId::ShapeType => {
            let shp = match opt_idx {
                0 => ae_core::ecs::Shape::Cube,
                1 => ae_core::ecs::Shape::Sphere,
                2 => ae_core::ecs::Shape::Cylinder,
                3 => ae_core::ecs::Shape::Capsule,
                4 => ae_core::ecs::Shape::Torus,
                5 => ae_core::ecs::Shape::Triangle,
                _ => ae_core::ecs::Shape::Cube,
            };
            if let Ok(mut s) = world.get::<&mut ae_core::ecs::Shape>(entity) {
                *s = shp;
            }
        }
        InspectorDropdownId::UiAnchor => {
            let anchor = match opt_idx {
                0 => ae_core::ecs::UiAnchor::TopLeft,
                1 => ae_core::ecs::UiAnchor::TopCenter,
                2 => ae_core::ecs::UiAnchor::TopRight,
                3 => ae_core::ecs::UiAnchor::CenterLeft,
                4 => ae_core::ecs::UiAnchor::Center,
                5 => ae_core::ecs::UiAnchor::CenterRight,
                6 => ae_core::ecs::UiAnchor::BottomLeft,
                7 => ae_core::ecs::UiAnchor::BottomCenter,
                8 => ae_core::ecs::UiAnchor::BottomRight,
                _ => ae_core::ecs::UiAnchor::Center,
            };
            if let Ok(mut u) = world.get::<&mut ae_core::ecs::UiElement>(entity) {
                u.anchor = anchor;
            }
        }
        InspectorDropdownId::UiTextAlignment => {
            let align = match opt_idx {
                0 => ae_core::ui::UiTextAlignment::Left,
                1 => ae_core::ui::UiTextAlignment::Center,
                2 => ae_core::ui::UiTextAlignment::Right,
                _ => ae_core::ui::UiTextAlignment::Left,
            };
            if let Ok(mut t) = world.get::<&mut ae_core::ecs::UiText>(entity) {
                t.alignment = align;
            }
        }
        _ => {}
    }
}