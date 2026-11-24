// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Uniform spatial grid partitioning and axis-aligned bounding boxes driven by ECS transforms.
#![deny(clippy::unwrap_used)]

use ae_ecs::{Entity, World};
use ae_plugin_api::{
    CharacterController, GlobalTransform, Position, RigidBody, RigidBodyType, TransformDirty,
    Velocity,
};
use std::collections::HashMap;

/// Axis-Aligned Bounding Box (AABB) for raycasting and frustum culling.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    /// Minimum XYZ corner.
    pub min: [f32; 3],
    /// Maximum XYZ corner.
    pub max: [f32; 3],
}

impl AABB {
    /// Creates a new AABB from minimum and maximum corners.
    #[must_use]
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Checks whether this AABB overlaps or touches `other`.
    #[must_use]
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }
}

/// Uniform grid for spatial partitioning with O(1) incremental updates for moving entities.
pub struct SpatialGrid {
    /// Uniform grid cell edge length in world units.
    pub cell_size: f32,
    /// Maps 3D grid cell coordinates to a list of entities.
    pub cells: HashMap<(i32, i32, i32), Vec<Entity>>,
    /// Maps entities to their current cell for fast delta updates.
    pub entity_to_cell: HashMap<Entity, (i32, i32, i32)>,
    /// Tracks the entity count from the last rebuild phase.
    pub last_entity_count: usize,
    /// Explicit dirty flag to trigger a full rebuild on the next `sync()` call.
    pub needs_rebuild: bool,
    /// Reusable scratch buffer for dynamic entities to eliminate per-frame allocations.
    pub scratch_dynamic_entities: Vec<(Entity, [f32; 3])>,
}

impl SpatialGrid {
    /// Creates a new `SpatialGrid` with the specified positive cell size.
    ///
    /// Non-finite or non-positive sizes fall back safely to `1.0`.
    #[must_use]
    pub fn new(cell_size: f32) -> Self {
        let safe_cell_size = if cell_size.is_finite() && cell_size > 0.0 {
            cell_size
        } else {
            1.0
        };
        Self {
            cell_size: safe_cell_size,
            cells: HashMap::new(),
            entity_to_cell: HashMap::new(),
            last_entity_count: usize::MAX,
            needs_rebuild: true,
            scratch_dynamic_entities: Vec::with_capacity(128),
        }
    }

    /// Marks the `SpatialGrid` as needing a full rebuild on the next `sync()` call.
    pub fn mark_dirty(&mut self) {
        self.needs_rebuild = true;
    }

    /// Converts world coordinates to grid cell coordinates, clamping non-finite inputs to origin.
    #[must_use]
    pub fn world_to_grid(&self, pos: [f32; 3]) -> (i32, i32, i32) {
        let inv = if self.cell_size.is_finite() && self.cell_size > 0.0 {
            1.0 / self.cell_size
        } else {
            1.0
        };
        let to_cell = |v: f32| {
            if v.is_finite() {
                (v * inv).floor() as i32
            } else {
                0
            }
        };
        (to_cell(pos[0]), to_cell(pos[1]), to_cell(pos[2]))
    }

    /// Inserts an entity into the grid at world-space `pos`.
    pub fn insert(&mut self, entity: Entity, pos: [f32; 3]) {
        let cell = self.world_to_grid(pos);
        self.cells.entry(cell).or_default().push(entity);
        self.entity_to_cell.insert(entity, cell);
    }

    /// Removes an entity from its current cell in the grid.
    pub fn remove(&mut self, entity: Entity) {
        if let Some(old_cell) = self.entity_to_cell.remove(&entity)
            && let Some(vec) = self.cells.get_mut(&old_cell)
            && let Some(pos) = vec.iter().position(|&e| e == entity)
        {
            vec.swap_remove(pos);
        }
    }

    /// Clears the grid completely and marks it dirty for rebuild.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_to_cell.clear();
        self.last_entity_count = usize::MAX;
        self.needs_rebuild = true;
    }

    /// Synchronizes spatial grid with world entities via full rebuilds on topology changes and
    /// incremental cell moves for moving or dirty entities.
    pub fn sync(&mut self, world: &World) {
        let current_count = world.len() as usize;
        let needs_rebuild = self.needs_rebuild
            || current_count != self.last_entity_count
            || (self.entity_to_cell.is_empty() && current_count > 0)
            || self.entity_to_cell.keys().any(|&e| !world.contains(e));

        if needs_rebuild {
            self.needs_rebuild = false;
            self.cells.clear();
            self.entity_to_cell.clear();
            self.last_entity_count = current_count;

            let mut query = world.query::<(Entity, &Position, Option<&GlobalTransform>)>();
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
            return;
        }

        self.scratch_dynamic_entities.clear();

        for (entity, (pos, gt, dirty, vel, rb, kcc)) in world
            .query::<(
                Entity,
                (
                    &Position,
                    Option<&GlobalTransform>,
                    Option<&TransformDirty>,
                    Option<&Velocity>,
                    Option<&RigidBody>,
                    Option<&CharacterController>,
                ),
            )>()
            .iter()
        {
            let is_dynamic = dirty.is_some()
                || kcc.is_some()
                || vel
                    .map(|v| v.x != 0.0 || v.y != 0.0 || v.z != 0.0)
                    .unwrap_or(false)
                || matches!(rb.map(|r| r.body_type), Some(RigidBodyType::Dynamic));

            if is_dynamic {
                let world_pos = if let Some(gt_val) = gt {
                    [gt_val.0.w.x, gt_val.0.w.y, gt_val.0.w.z]
                } else {
                    [pos.x, pos.y, pos.z]
                };
                self.scratch_dynamic_entities.push((entity, world_pos));
            }
        }

        for i in 0..self.scratch_dynamic_entities.len() {
            let (entity, world_pos) = self.scratch_dynamic_entities[i];
            let new_cell = self.world_to_grid(world_pos);
            let current_cell = self.entity_to_cell.get(&entity).copied();

            if current_cell != Some(new_cell) {
                if let Some(old_cell) = current_cell
                    && let Some(vec) = self.cells.get_mut(&old_cell)
                    && let Some(idx) = vec.iter().position(|&e| e == entity)
                {
                    vec.swap_remove(idx);
                }
                self.cells.entry(new_cell).or_default().push(entity);
                self.entity_to_cell.insert(entity, new_cell);
            }
        }
    }

    /// Returns all entities in the cell containing `pos`.
    #[must_use]
    pub fn query_cell(&self, pos: [f32; 3]) -> Option<&Vec<Entity>> {
        let cell = self.world_to_grid(pos);
        self.cells.get(&cell)
    }

    /// Returns active grid cells overlapping `[cam_pos - view_distance, cam_pos + view_distance]`.
    #[must_use]
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

        let volume_x = (max_cell.0 - min_cell.0 + 1).max(0) as usize;
        let volume_y = (max_cell.1 - min_cell.1 + 1).max(0) as usize;
        let volume_z = (max_cell.2 - min_cell.2 + 1).max(0) as usize;
        let total_volume = volume_x.saturating_mul(volume_y).saturating_mul(volume_z);

        let mut matching_cells = Vec::with_capacity(self.cells.len().min(total_volume).min(256));

        if total_volume > self.cells.len() {
            for (&(cx, cy, cz), entities) in &self.cells {
                if cx >= min_cell.0
                    && cx <= max_cell.0
                    && cy >= min_cell.1
                    && cy <= max_cell.1
                    && cz >= min_cell.2
                    && cz <= max_cell.2
                {
                    matching_cells.push((cx, cy, cz, entities.as_slice()));
                }
            }
        } else {
            for cx in min_cell.0..=max_cell.0 {
                for cy in min_cell.1..=max_cell.1 {
                    for cz in min_cell.2..=max_cell.2 {
                        if let Some(entities) = self.cells.get(&(cx, cy, cz)) {
                            matching_cells.push((cx, cy, cz, entities.as_slice()));
                        }
                    }
                }
            }
        }
        matching_cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_grid_sync_delta_move_and_non_finite_guards() {
        let grid = SpatialGrid::new(-10.0);
        assert_eq!(grid.cell_size, 1.0);
        assert_eq!(
            grid.world_to_grid([f32::NAN, f32::INFINITY, -2.5]),
            (0, 0, -3)
        );

        let mut world = World::new();
        let ent = world.spawn((Position::new(5.0, 0.0, 0.0), TransformDirty));
        let mut grid = SpatialGrid::new(10.0);
        grid.sync(&world);
        assert_eq!(grid.entity_to_cell.get(&ent), Some(&(0, 0, 0)));

        if let Ok(pos) = world.get_mut::<Position>(ent) {
            pos.x = 25.0;
        }
        grid.sync(&world);
        assert_eq!(grid.entity_to_cell.get(&ent), Some(&(2, 0, 0)));
    }
}