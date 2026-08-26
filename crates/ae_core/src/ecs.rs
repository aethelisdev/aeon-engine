// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// AE Core - ECS Manager and Hierarchical Transform System.
/// Manages the `hecs::World` database and hierarchical transform updates.
use hecs::World;
use serde::{Deserialize, Serialize};

pub use ae_plugin_api::{
    AssetHandle, BoundingBox, BoundingRadius, CharacterAction, CharacterController, Children,
    Collider, ColliderShape, Color, DestructibleTarget, EphemeralProjectile, GlobalTransform,
    Hidden, Light, ModelId, MovingPlatform, Name, Parent, PhysicsMaterial, PlayerTag, Position,
    RaycastHit, RigidBody, RigidBodyType, Rotation, Rotator, Scale, Shape, SpriteId, SurfaceType,
    TransformDirty, TriggerZone, Velocity,
};

pub use crate::registry::{ComponentHandler, ComponentRegistry, TypedComponentHandler};

/// Central ECS manager.
/// The `update()` method runs linear velocity integration over non-physics entities.
pub struct EcsManager {
    pub world: World,
}

impl EcsManager {
    /// Creates an empty EcsManager.
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    /// Runs velocity integration for moving entities not simulated by the physics solver.
    /// Iterates sequentially over contiguous archetype storage in `hecs::World` to ensure 100%
    /// thread-safe, deterministic execution across all `hecs` library releases.
    pub fn update(&mut self, delta_time: f32) {
        for (pos, vel, rb, col) in self.world.query_mut::<(
            &mut Position,
            &Velocity,
            Option<&RigidBody>,
            Option<&Collider>,
        )>() {
            if rb.is_none() && col.is_none() && (vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0) {
                pos.x += vel.x * delta_time;
                pos.y += vel.y * delta_time;
                pos.z += vel.z * delta_time;
            }
        }
    }
}

impl Default for EcsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Updates the world-space GlobalTransform matrices of all entities in parent-child relationships,
/// and synchronizes standalone entities possessing GlobalTransform components with their local transform.
/// Uses an event/dirty-propagation model (`TransformDirty`): if a parent or child is dirty,
/// updates matrices and cascades down the subtree; unchanged subtrees skip matrix multiplications.
pub fn update_hierarchy_transforms(world: &mut hecs::World) {
    use cgmath::SquareMatrix;

    let mut all_entities_with_parent = Vec::new();
    let mut parent_to_children =
        std::collections::HashMap::<hecs::Entity, Vec<hecs::Entity>>::new();

    // 1. Query Parent components to collect active hierarchy relationships.
    for (entity, parent_ref) in world.query::<(hecs::Entity, &Parent)>().iter() {
        if world.contains(parent_ref.0) {
            all_entities_with_parent.push((entity, parent_ref.0));
            parent_to_children
                .entry(parent_ref.0)
                .or_default()
                .push(entity);
        }
    }

    let mut visited = std::collections::HashSet::new();

    if !all_entities_with_parent.is_empty() {
        // 2. Identify root parent entities
        let mut root_entities = std::collections::HashSet::new();
        for &parent in parent_to_children.keys() {
            if world.get::<&Parent>(parent).is_err() {
                root_entities.insert(parent);
            }
        }

        // 3. Recursive DFS helper with dirty-propagation optimization
        fn propagate(
            world: &mut hecs::World,
            entity: hecs::Entity,
            parent_transform: &cgmath::Matrix4<f32>,
            parent_to_children: &std::collections::HashMap<hecs::Entity, Vec<hecs::Entity>>,
            visited: &mut std::collections::HashSet<hecs::Entity>,
            parent_is_dirty: bool,
        ) {
            if !visited.insert(entity) {
                return; // Cycle detected — stop recursion to prevent stack overflow
            }

            let self_is_dirty = world.get::<&TransformDirty>(entity).is_ok();
            let has_gt = world.get::<&GlobalTransform>(entity).is_ok();
            let is_dirty = parent_is_dirty || self_is_dirty || !has_gt;

            let global_matrix = if is_dirty {
                let pos = world
                    .get::<&Position>(entity)
                    .ok()
                    .map(|p| cgmath::Vector3::new(p.x, p.y, p.z))
                    .unwrap_or_else(|| cgmath::Vector3::new(0.0, 0.0, 0.0));
                let rot = world
                    .get::<&Rotation>(entity)
                    .ok()
                    .map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
                    .unwrap_or_else(|| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
                let scale = world
                    .get::<&Scale>(entity)
                    .ok()
                    .map(|s| {
                        let min = 1e-4;
                        let sx = if s.x.abs() < min {
                            f32::copysign(min, s.x)
                        } else {
                            s.x
                        };
                        let sy = if s.y.abs() < min {
                            f32::copysign(min, s.y)
                        } else {
                            s.y
                        };
                        let sz = if s.z.abs() < min {
                            f32::copysign(min, s.z)
                        } else {
                            s.z
                        };
                        cgmath::Vector3::new(sx, sy, sz)
                    })
                    .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

                let local_matrix = cgmath::Matrix4::from_translation(pos)
                    * cgmath::Matrix4::from(rot)
                    * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

                let g_mat = parent_transform * local_matrix;

                if let Ok(mut gt) = world.get::<&mut GlobalTransform>(entity) {
                    gt.0 = g_mat;
                } else {
                    let _ = world.insert_one(entity, GlobalTransform(g_mat));
                }

                g_mat
            } else {
                world
                    .get::<&GlobalTransform>(entity)
                    .map(|gt| gt.0)
                    .unwrap_or(*parent_transform)
            };

            if let Some(children) = parent_to_children.get(&entity) {
                for &child in children {
                    propagate(
                        world,
                        child,
                        &global_matrix,
                        parent_to_children,
                        visited,
                        is_dirty,
                    );
                }
            }
        }

        let identity = cgmath::Matrix4::identity();
        for root in root_entities {
            let root_is_dirty = world.get::<&TransformDirty>(root).is_ok();
            propagate(
                world,
                root,
                &identity,
                &parent_to_children,
                &mut visited,
                root_is_dirty,
            );
        }
    }

    // 4. Synchronize standalone entities holding GlobalTransform components not updated by hierarchy traversal
    for (entity, (gt, pos_opt, rot_opt, scale_opt)) in world
        .query_mut::<(
            hecs::Entity,
            (
                &mut GlobalTransform,
                Option<&Position>,
                Option<&Rotation>,
                Option<&Scale>,
            ),
        )>()
        .into_iter()
    {
        if !visited.contains(&entity) {
            let pos = pos_opt
                .map(|p| cgmath::Vector3::new(p.x, p.y, p.z))
                .unwrap_or_else(|| cgmath::Vector3::new(0.0, 0.0, 0.0));
            let rot = rot_opt
                .map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
                .unwrap_or_else(|| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
            let scale = scale_opt
                .map(|s| {
                    let min = 1e-4;
                    let sx = if s.x.abs() < min {
                        f32::copysign(min, s.x)
                    } else {
                        s.x
                    };
                    let sy = if s.y.abs() < min {
                        f32::copysign(min, s.y)
                    } else {
                        s.y
                    };
                    let sz = if s.z.abs() < min {
                        f32::copysign(min, s.z)
                    } else {
                        s.z
                    };
                    cgmath::Vector3::new(sx, sy, sz)
                })
                .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

            gt.0 = cgmath::Matrix4::from_translation(pos)
                * cgmath::Matrix4::from(rot)
                * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        }
    }
}

/// Level of Detail (LOD) model selection component based on camera distance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LodGroup {
    /// LOD 0 model handle (High detail)
    pub lod_0: AssetHandle,
    /// LOD 1 model handle (Medium detail, optional)
    pub lod_1: Option<AssetHandle>,
    /// LOD 2 model handle (Low detail, optional)
    pub lod_2: Option<AssetHandle>,
    /// Distance threshold between LOD 0 and LOD 1
    pub threshold_1: f32,
    /// Distance threshold between LOD 1 and LOD 2
    pub threshold_2: f32,
}

impl Default for LodGroup {
    fn default() -> Self {
        Self {
            lod_0: AssetHandle::default(),
            lod_1: None,
            lod_2: None,
            threshold_1: 15.0,
            threshold_2: 35.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::SquareMatrix;

    /// Tests that update_hierarchy_transforms synchronizes GlobalTransform for standalone entities.
    #[test]
    fn test_update_hierarchy_transforms_standalone_sync() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Position {
                x: 10.0,
                y: 5.0,
                z: -2.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            GlobalTransform(cgmath::Matrix4::identity()),
        ));

        update_hierarchy_transforms(&mut world);

        let gt = world.get::<&GlobalTransform>(entity).unwrap();
        assert_eq!(gt.0.w.x, 10.0);
        assert_eq!(gt.0.w.y, 5.0);
        assert_eq!(gt.0.w.z, -2.0);
    }

    /// Tests that update_hierarchy_transforms propagates dirty parent transforms down to children.
    #[test]
    fn test_update_hierarchy_transforms_dirty_propagation() {
        let mut world = hecs::World::new();
        let parent = world.spawn((
            Position {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            TransformDirty,
        ));
        let child = world.spawn((
            Parent(parent),
            Position {
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        ));

        update_hierarchy_transforms(&mut world);

        let child_gt = world.get::<&GlobalTransform>(child).unwrap();
        assert_eq!(child_gt.0.w.x, 7.0);
    }
}