// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// AE Core - Spatial Partitioning (Spatial Grid) Module.
use hecs::Entity;
use std::collections::HashMap;

/// Axis-Aligned Bounding Box (AABB) for raycasting and frustum culling.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AABB {
    /// Creates a new AABB.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Checks for intersection with another AABB.
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }
}

/// Uniform Grid for spatial partitioning with high-performance delta updates.
pub struct SpatialGrid {
    pub cell_size: f32,
    /// Maps 3D grid cell coordinates to a list of entities.
    pub cells: HashMap<(i32, i32, i32), Vec<Entity>>,
    /// Maps entities to their current cell for fast O(1) delta updates.
    pub entity_to_cell: HashMap<Entity, (i32, i32, i32)>,
    /// Tracks the entity count from the last rebuild phase.
    pub last_entity_count: usize,
    /// Explicit dirty flag to trigger a full rebuild on the next `sync()` call.
    pub needs_rebuild: bool,
}

impl SpatialGrid {
    /// Creates a new SpatialGrid with the specified cell size.
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_to_cell: HashMap::new(),
            last_entity_count: usize::MAX, // Force rebuild on the first frame
            needs_rebuild: true,
        }
    }

    /// Marks the SpatialGrid as needing a full rebuild on the next `sync()` call.
    pub fn mark_dirty(&mut self) {
        self.needs_rebuild = true;
    }

    /// Converts world coordinates to grid cell coordinates.
    pub fn world_to_grid(&self, pos: [f32; 3]) -> (i32, i32, i32) {
        (
            (pos[0] / self.cell_size).floor() as i32,
            (pos[1] / self.cell_size).floor() as i32,
            (pos[2] / self.cell_size).floor() as i32,
        )
    }

    /// Inserts an entity into the grid.
    pub fn insert(&mut self, entity: Entity, pos: [f32; 3]) {
        let cell = self.world_to_grid(pos);
        self.cells.entry(cell).or_default().push(entity);
        self.entity_to_cell.insert(entity, cell);
    }

    /// Removes an entity from its current cell in the grid.
    pub fn remove(&mut self, entity: Entity) {
        if let Some(old_cell) = self.entity_to_cell.remove(&entity) {
            if let Some(vec) = self.cells.get_mut(&old_cell) {
                if let Some(pos) = vec.iter().position(|&e| e == entity) {
                    vec.swap_remove(pos);
                }
            }
        }
    }

    /// Clears the grid completely.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_to_cell.clear();
        self.last_entity_count = usize::MAX;
        self.needs_rebuild = true;
    }

    /// Synchronizes spatial grid with world entities.
    /// Rebuilds full grid on entity count changes, and performs fast delta updates for moving dynamic entities.
    pub fn sync(&mut self, world: &hecs::World) {
        let current_count = world.len() as usize;

        // High-performance topology check:
        // 1. Explicit rebuild requested via mark_dirty()
        // 2. Total ECS entity count changed (spawn/despawn)
        // 3. SpatialGrid is empty but world has entities
        // 4. Any registered entity in entity_to_cell is no longer alive in world
        let needs_rebuild = self.needs_rebuild
            || current_count != self.last_entity_count
            || (self.entity_to_cell.is_empty() && current_count > 0)
            || self.entity_to_cell.keys().any(|&e| !world.contains(e));

        if needs_rebuild {
            self.needs_rebuild = false;
            // Entity topology changed or new untracked entities present: perform full grid rebuild
            self.cells.clear();
            self.entity_to_cell.clear();
            self.last_entity_count = current_count;

            let mut query = world.query::<(
                hecs::Entity,
                &crate::ecs::Position,
                Option<&crate::ecs::GlobalTransform>,
            )>();

            for (entity, pos, global_transform) in query.iter() {
                let world_pos = if let Some(gt) = global_transform {
                    [gt.0.w.x, gt.0.w.y, gt.0.w.z]
                } else {
                    [pos.x, pos.y, pos.z]
                };
                self.insert(entity, world_pos);
            }
            log::info!(
                "SpatialGrid full rebuild with {} entities. Active cells: {}",
                current_count,
                self.cells.len()
            );
        } else {
            // Entity topology static: perform fast delta updates for dynamic/dirty entities
            let mut dynamic_entities = Vec::new();

            // 1. Collect entities marked dirty, moving, KCC, or dynamic physics
            for (entity, (pos, gt, _dirty, _vel, _rb, _kcc)) in world
                .query::<(
                    hecs::Entity,
                    (
                        &crate::ecs::Position,
                        Option<&crate::ecs::GlobalTransform>,
                        Option<&crate::ecs::TransformDirty>,
                        Option<&crate::ecs::Velocity>,
                        Option<&crate::ecs::RigidBody>,
                        Option<&crate::ecs::CharacterController>,
                    ),
                )>()
                .iter()
            {
                let is_dynamic = _dirty.is_some()
                    || _kcc.is_some()
                    || _vel
                        .map(|v| v.x != 0.0 || v.y != 0.0 || v.z != 0.0)
                        .unwrap_or(false)
                    || matches!(
                        _rb.map(|r| r.body_type),
                        Some(crate::ecs::RigidBodyType::Dynamic)
                    );

                if is_dynamic {
                    let world_pos = if let Some(gt_val) = gt {
                        [gt_val.0.w.x, gt_val.0.w.y, gt_val.0.w.z]
                    } else {
                        [pos.x, pos.y, pos.z]
                    };
                    dynamic_entities.push((entity, world_pos));
                }
            }

            // 2. Update cell positions for dynamic entities whose grid cell shifted
            for (entity, world_pos) in dynamic_entities {
                let new_cell = self.world_to_grid(world_pos);
                let current_cell = self.entity_to_cell.get(&entity).copied();

                if current_cell != Some(new_cell) {
                    if let Some(old_cell) = current_cell {
                        if let Some(vec) = self.cells.get_mut(&old_cell) {
                            if let Some(idx) = vec.iter().position(|&e| e == entity) {
                                vec.swap_remove(idx);
                            }
                        }
                    }
                    self.cells.entry(new_cell).or_default().push(entity);
                    self.entity_to_cell.insert(entity, new_cell);
                }
            }
        }
    }

    /// Returns all entities in the cell containing the specified coordinate.
    pub fn query_cell(&self, pos: [f32; 3]) -> Option<&Vec<Entity>> {
        let cell = self.world_to_grid(pos);
        self.cells.get(&cell)
    }

    /// Returns active grid cells within view_distance of the camera position.
    pub fn query_cells_near_camera(
        &self,
        cam_pos: cgmath::Vector3<f32>,
        view_distance: f32,
    ) -> Vec<(i32, i32, i32, &[Entity])> {
        let min_cell = self.world_to_grid([
            cam_pos.x - view_distance,
            cam_pos.y - view_distance,
            cam_pos.z - view_distance,
        ]);
        let max_cell = self.world_to_grid([
            cam_pos.x + view_distance,
            cam_pos.y + view_distance,
            cam_pos.z + view_distance,
        ]);

        let mut matching_cells = Vec::with_capacity(256);
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                for cz in min_cell.2..=max_cell.2 {
                    if let Some(entities) = self.cells.get(&(cx, cy, cz)) {
                        matching_cells.push((cx, cy, cz, entities.as_slice()));
                    }
                }
            }
        }
        matching_cells
    }
}