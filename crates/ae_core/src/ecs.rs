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
        self.world
            .query_mut::<(
                &mut Position,
                &Velocity,
                Option<&RigidBody>,
                Option<&Collider>,
            )>()
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
    }
}

impl Default for EcsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Updates the world-space GlobalTransform matrices of all entities in parent-child relationships.
pub fn update_hierarchy_transforms(world: &mut hecs::World) {
    use cgmath::SquareMatrix;

    let mut all_entities_with_parent = Vec::new();
    let mut parent_to_children =
        std::collections::HashMap::<hecs::Entity, Vec<hecs::Entity>>::new();

    // 1. Query only Parent components to collect active hierarchy relationships.
    for (entity, parent_ref) in world.query::<(hecs::Entity, &Parent)>().iter() {
        if world.contains(parent_ref.0) {
            all_entities_with_parent.push((entity, parent_ref.0));
            parent_to_children
                .entry(parent_ref.0)
                .or_default()
                .push(entity);
        }
    }

    // Early exit if there are no hierarchical relations in the scene (O(1) fast path)
    if all_entities_with_parent.is_empty() {
        return;
    }

    // 2. Identify root parent entities
    let mut root_entities = std::collections::HashSet::new();
    for &parent in parent_to_children.keys() {
        if world.get::<&Parent>(parent).is_err() {
            root_entities.insert(parent);
        }
    }

    // 3. Keep track of visited entities to abort early if cycles are detected
    let mut visited = std::collections::HashSet::new();

    // 4. Recursive DFS helper to compute and insert/update GlobalTransform components
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