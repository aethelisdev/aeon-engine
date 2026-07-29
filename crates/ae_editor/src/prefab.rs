// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::undo_redo::EntitySnapshot;
use ae_core::ecs::Position;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Reusable prefab template containing an entity snapshot.
/// Can be saved to `.aeprefab` (JSON) files and instantiated into any ECS world
/// at a target 3D position with new entity IDs and full component restoration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prefab {
    /// Human-readable prefab name.
    pub name: String,
    /// Root entity component snapshot.
    pub root_snapshot: EntitySnapshot,
}

impl Prefab {
    /// Creates a new `Prefab` from an entity in the ECS world.
    pub fn create_from_entity(world: &hecs::World, entity: hecs::Entity) -> Self {
        let snapshot = EntitySnapshot::capture(world, entity);
        let name = snapshot
            .name
            .as_ref()
            .map(|n| n.0.clone())
            .unwrap_or_else(|| "PrefabEntity".to_string());
        Self {
            name,
            root_snapshot: snapshot,
        }
    }

    /// Instantiates the prefab into the target ECS world, optionally overriding its 3D position.
    /// Returns the new entity handle. Automatically marks transform dirty for physics sync.
    pub fn instantiate(
        &self,
        world: &mut hecs::World,
        target_pos: Option<Position>,
    ) -> hecs::Entity {
        let entity = self.root_snapshot.spawn(world);

        if let Some(pos) = target_pos {
            let _ = world.insert_one(entity, pos);
        }

        let _ = world.insert_one(entity, ae_core::ecs::TransformDirty);
        let _ = world.remove_one::<ae_core::ecs::GlobalTransform>(entity);

        entity
    }

    /// Serializes the prefab to a JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Deserializes a prefab from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Saves the prefab to a file on disk (typically `.aeprefab`).
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Loads a prefab from a file on disk.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::{Color, Name, Rotation, Scale, Shape};

    #[test]
    fn test_prefab_create_save_load_instantiate() {
        let mut world = hecs::World::new();

        let original_entity = world.spawn((
            Name("TestBarrel".to_string()),
            Shape::Cylinder,
            Position {
                x: 10.0,
                y: 2.0,
                z: -5.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 2.0,
                z: 1.0,
            },
            Color::soft_blue(),
        ));

        // Create prefab from entity
        let prefab = Prefab::create_from_entity(&world, original_entity);
        assert_eq!(prefab.name, "TestBarrel");

        // Serialize & deserialize
        let json = prefab.to_json().expect("Failed to serialize prefab");
        let loaded_prefab = Prefab::from_json(&json).expect("Failed to deserialize prefab");
        assert_eq!(loaded_prefab.name, "TestBarrel");

        // Instantiate into world at new position
        let new_pos = Position {
            x: 50.0,
            y: 0.0,
            z: 50.0,
        };
        let new_entity = loaded_prefab.instantiate(&mut world, Some(new_pos));

        // Verify component values on instantiated entity
        assert_ne!(original_entity, new_entity);
        assert_eq!(world.get::<&Name>(new_entity).unwrap().0, "TestBarrel");
        let pos = world.get::<&Position>(new_entity).unwrap();
        assert_eq!(pos.x, 50.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 50.0);
    }
}