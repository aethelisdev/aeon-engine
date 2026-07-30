// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// AE Core - ECS Manager and Hierarchical Transform System.
/// Manages the `hecs::World` database and hierarchical transform updates.
use hecs::World;
use rayon::prelude::*;

pub use ae_plugin_api::{
    AssetHandle, BoundingBox, BoundingRadius, CharacterController, Children, Collider,
    ColliderShape, Color, GlobalTransform, Light, ModelId, Name, Parent, PlayerTag, Position,
    RaycastHit, RigidBody, RigidBodyType, Rotation, Scale, Shape, SpriteId, TransformDirty,
    Velocity,
};

/// Central ECS manager.
/// The `update()` method runs parallel velocity integration using Rayon's `par_bridge()`.
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

    /// Runs parallel velocity integration for moving entities not simulated by the physics solver.
    pub fn update(&mut self, delta_time: f32) {
        let total_entities = self.world.len();

        let query = self.world.query_mut::<(
            &mut Position,
            &Velocity,
            Option<&RigidBody>,
            Option<&Collider>,
        )>();

        if total_entities > 512 {
            query
                .into_iter()
                .par_bridge()
                .for_each(|(pos, vel, rb, col)| {
                    if rb.is_none() && col.is_none() {
                        if vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0 {
                            pos.x += vel.x * delta_time;
                            pos.y += vel.y * delta_time;
                            pos.z += vel.z * delta_time;
                        }
                    }
                });
        } else {
            for (pos, vel, rb, col) in query.into_iter() {
                if rb.is_none() && col.is_none() {
                    if vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0 {
                        pos.x += vel.x * delta_time;
                        pos.y += vel.y * delta_time;
                        pos.z += vel.z * delta_time;
                    }
                }
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

        // 3. Recursive DFS helper to compute and insert/update GlobalTransform components for hierarchies
        fn propagate(
            world: &mut hecs::World,
            entity: hecs::Entity,
            parent_transform: &cgmath::Matrix4<f32>,
            parent_to_children: &std::collections::HashMap<hecs::Entity, Vec<hecs::Entity>>,
            visited: &mut std::collections::HashSet<hecs::Entity>,
        ) {
            if !visited.insert(entity) {
                return; // Cycle detected — stop recursion to prevent stack overflow
            }

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
                .map(|s| cgmath::Vector3::new(s.x, s.y, s.z))
                .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

            let local_matrix = cgmath::Matrix4::from_translation(pos)
                * cgmath::Matrix4::from(rot)
                * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

            let global_matrix = parent_transform * local_matrix;

            if let Ok(mut gt) = world.get::<&mut GlobalTransform>(entity) {
                gt.0 = global_matrix;
            } else {
                let _ = world.insert_one(entity, GlobalTransform(global_matrix));
            }

            if let Some(children) = parent_to_children.get(&entity) {
                for &child in children {
                    propagate(world, child, &global_matrix, parent_to_children, visited);
                }
            }
        }

        let identity = cgmath::Matrix4::identity();
        for root in root_entities {
            propagate(world, root, &identity, &parent_to_children, &mut visited);
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
                .map(|s| cgmath::Vector3::new(s.x, s.y, s.z))
                .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

            gt.0 = cgmath::Matrix4::from_translation(pos)
                * cgmath::Matrix4::from(rot)
                * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        }
    }
}

/// Level of Detail (LOD) model selection component based on camera distance.
#[derive(Clone, Debug)]
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
}