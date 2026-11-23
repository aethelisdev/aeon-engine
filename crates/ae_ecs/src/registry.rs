// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic ECS component registry, reflection, and versioned binary serialization.
//!
//! Provides the core `ComponentHandler`, `TypedComponentHandler<T>`, and `ComponentRegistry`
//! abstractions over `World` and `Entity` so component snapshots, cloning, JSON debugging,
//! and schema-versioned migrations stay isolated inside `ae_ecs`.

use crate::{Component, Entity, World};
use std::any::{TypeId, type_name};
use std::collections::{HashMap, HashSet};

/// Deterministic FNV-1a 64-bit offset basis constant.
const FNV1A_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;
/// Deterministic FNV-1a 64-bit prime multiplier constant.
const FNV1A_PRIME_64: u64 = 0x0100_0000_01b3;

/// Computes a deterministic 64-bit FNV-1a type hash from a canonical component type name.
///
/// Used as the stable binary discriminator in length-prefixed component streams so
/// serialized records remain independent of runtime Rust `TypeId` layouts across builds
/// and target platforms (Windows, Linux, Android).
#[must_use]
pub const fn component_type_hash(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash = FNV1A_OFFSET_BASIS_64;
    let mut idx = 0;
    while idx < bytes.len() {
        hash ^= bytes[idx] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME_64);
        idx += 1;
    }
    hash
}

/// Interface for type-erased component operations, serialization, and lifecycle hooks.
pub trait ComponentHandler: Send + Sync {
    /// Human-readable type identifier (e.g. "Position", "Rotation", "Name").
    fn type_name(&self) -> &'static str;

    /// Deterministic 64-bit FNV-1a discriminator matching [`component_type_hash`].
    fn type_hash(&self) -> u64;

    /// Active binary schema revision emitted when capturing this component (`>= 1`).
    fn schema_version(&self) -> u16;

    /// Runtime `TypeId` of the underlying component.
    fn type_id(&self) -> TypeId;

    /// Captures component state from `world` on `entity` as binary `postcard` bytes.
    fn capture(&self, world: &World, entity: Entity) -> Option<Vec<u8>>;

    /// Captures component state as a JSON string strictly for live RPC debugging/inspection.
    fn capture_debug_json(&self, world: &World, entity: Entity) -> Option<String>;

    /// Converts an offline JSON object representation into binary `postcard` bytes for migration tools.
    fn transcode_json_to_binary(&self, json: &str) -> Result<Vec<u8>, String>;

    /// Restores serialized binary `postcard` component data encoded at `schema_ver` onto `entity`,
    /// running registered schema upcasters when `schema_ver` predates [`Self::schema_version`].
    fn apply_versioned(
        &self,
        world: &mut World,
        entity: Entity,
        schema_ver: u16,
        data: &[u8],
    ) -> Result<(), String>;

    /// Restores serialized binary `postcard` component data onto `entity` in `world` using the
    /// handler's current schema version (with automatic fallback to legacy upcasters).
    fn apply(&self, world: &mut World, entity: Entity, data: &[u8]) -> Result<(), String>;

    /// Clones the component directly from `source` entity to `target` entity without serialization overhead.
    fn clone_component(
        &self,
        world: &World,
        source: Entity,
        target: Entity,
        target_world: &mut World,
    );

    /// Checks whether the component exists on `entity` in `world`.
    fn has_component(&self, world: &World, entity: Entity) -> bool;

    /// Removes the component from `entity` in `world`.
    fn remove_component(&self, world: &mut World, entity: Entity) -> bool;

    /// Instantiates and attaches a default instance of this component onto `entity` in `world`.
    fn add_default(&self, world: &mut World, entity: Entity) -> Result<(), String>;
}

type LegacyComponentDecoder<T> = Box<dyn Fn(&[u8]) -> Result<T, String> + Send + Sync>;

/// Generic handler implementation for any component implementing `Serialize + DeserializeOwned + Clone`.
///
/// Supports explicit `schema_ver` tagging and historical binary schema upcasters via
/// [`Self::with_schema_version`] and [`Self::with_migration`].
pub struct TypedComponentHandler<T> {
    name: &'static str,
    schema_ver: u16,
    default_fn: Option<fn() -> T>,
    legacy_decoders: Vec<(u16, LegacyComponentDecoder<T>)>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> TypedComponentHandler<T> {
    /// Creates a new typed component handler with the given human-readable name at `schema_ver = 1`.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            schema_ver: 1,
            default_fn: None,
            legacy_decoders: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new typed component handler with a default value constructor at `schema_ver = 1`.
    pub fn with_default(name: &'static str, default_fn: fn() -> T) -> Self {
        Self {
            name,
            schema_ver: 1,
            default_fn: Some(default_fn),
            legacy_decoders: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the active binary schema revision emitted when capturing this component (`>= 1`).
    #[must_use]
    pub fn with_schema_version(mut self, schema_ver: u16) -> Self {
        self.schema_ver = schema_ver.max(1);
        self
    }

    /// Registers a strongly typed upcaster converting legacy `postcard` payloads encoded at
    /// `from_schema_ver` (`OldT`) into the current component layout `T`.
    ///
    /// Automatically advances `schema_ver` to at least `from_schema_ver + 1`.
    #[must_use]
    pub fn with_migration<OldT: serde::de::DeserializeOwned + 'static>(
        mut self,
        from_schema_ver: u16,
        upcast_fn: fn(OldT) -> T,
    ) -> Self {
        self.schema_ver = self.schema_ver.max(from_schema_ver.saturating_add(1));
        self.legacy_decoders.push((
            from_schema_ver,
            Box::new(move |bytes| {
                let old: OldT = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
                Ok(upcast_fn(old))
            }),
        ));
        self
    }
}

impl<T: serde::de::DeserializeOwned> TypedComponentHandler<T> {
    fn decode_payload(&self, schema_ver: u16, data: &[u8]) -> Result<T, String> {
        if schema_ver == self.schema_ver
            && let Ok(comp) = postcard::from_bytes::<T>(data)
        {
            return Ok(comp);
        }

        if schema_ver != self.schema_ver {
            for (ver, decoder) in &self.legacy_decoders {
                if *ver == schema_ver {
                    return decoder(data);
                }
            }
        }

        if let Ok(comp) = postcard::from_bytes::<T>(data) {
            return Ok(comp);
        }

        for (_, decoder) in self.legacy_decoders.iter().rev() {
            if let Ok(comp) = decoder(data) {
                return Ok(comp);
            }
        }

        postcard::from_bytes::<T>(data).map_err(|e| {
            format!(
                "Failed to decode component '{}' (record schema_ver={}, current={}): {e}",
                self.name, schema_ver, self.schema_ver
            )
        })
    }
}

impl<T: Component + serde::Serialize + serde::de::DeserializeOwned + Clone> ComponentHandler
    for TypedComponentHandler<T>
{
    fn type_name(&self) -> &'static str {
        self.name
    }

    fn type_hash(&self) -> u64 {
        component_type_hash(self.name)
    }

    fn schema_version(&self) -> u16 {
        self.schema_ver
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn capture(&self, world: &World, entity: Entity) -> Option<Vec<u8>> {
        world
            .inspect::<T, _>(entity, |comp| postcard::to_stdvec(comp).ok())
            .ok()
            .flatten()
    }

    fn capture_debug_json(&self, world: &World, entity: Entity) -> Option<String> {
        world
            .inspect::<T, _>(entity, |comp| serde_json::to_string(comp).ok())
            .ok()
            .flatten()
    }

    fn transcode_json_to_binary(&self, json: &str) -> Result<Vec<u8>, String> {
        let comp: T = serde_json::from_str(json).map_err(|e| e.to_string())?;
        postcard::to_stdvec(&comp).map_err(|e| e.to_string())
    }

    fn apply_versioned(
        &self,
        world: &mut World,
        entity: Entity,
        schema_ver: u16,
        data: &[u8],
    ) -> Result<(), String> {
        let comp = self.decode_payload(schema_ver, data)?;
        world
            .insert_one(entity, comp)
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }

    fn apply(&self, world: &mut World, entity: Entity, data: &[u8]) -> Result<(), String> {
        self.apply_versioned(world, entity, self.schema_ver, data)
    }

    fn clone_component(
        &self,
        world: &World,
        source: Entity,
        target: Entity,
        target_world: &mut World,
    ) {
        if let Ok(cloned) = world.get_cloned::<T>(source) {
            let _ = target_world.insert_one(target, cloned);
        }
    }

    fn has_component(&self, world: &World, entity: Entity) -> bool {
        world.has::<T>(entity)
    }

    fn remove_component(&self, world: &mut World, entity: Entity) -> bool {
        world.remove_one::<T>(entity).is_ok()
    }

    fn add_default(&self, world: &mut World, entity: Entity) -> Result<(), String> {
        if let Some(factory) = self.default_fn {
            let instance = factory();
            world
                .insert_one(entity, instance)
                .map_err(|e| format!("{e:?}"))?;
            Ok(())
        } else {
            Err(format!(
                "Component '{}' has no default constructor registered",
                self.name
            ))
        }
    }
}

/// Central registry storing registered component handlers for dynamic ECS operations.
///
/// Engine layers register concrete components once into an instance (or a static singleton)
/// while the reflection, binary serialization, cloning, and upcasting mechanics stay inside
/// `ae_ecs`.
#[derive(Default)]
pub struct ComponentRegistry {
    handlers: Vec<Box<dyn ComponentHandler>>,
    type_id_map: HashMap<TypeId, usize>,
    type_hash_map: HashMap<u64, usize>,
    name_map: HashMap<String, usize>,
}

impl ComponentRegistry {
    /// Creates an empty component registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a component type with its short Rust type name and `Default` constructor.
    pub fn register<
        T: Component + serde::Serialize + serde::de::DeserializeOwned + Clone + Default,
    >(
        &mut self,
    ) {
        let full_name = type_name::<T>();
        let short_name = full_name.rsplit("::").next().unwrap_or(full_name);
        self.register_with_default::<T>(short_name, T::default);
    }

    /// Registers a component type with a custom identifier name and no default constructor.
    pub fn register_with_name<
        T: Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        name: &'static str,
    ) {
        self.register_handler(TypedComponentHandler::<T>::new(name));
    }

    /// Registers a component type with a custom identifier name and a default constructor.
    pub fn register_with_default<
        T: Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        name: &'static str,
        default_fn: fn() -> T,
    ) {
        self.register_handler(TypedComponentHandler::<T>::with_default(name, default_fn));
    }

    /// Registers a custom-configured [`TypedComponentHandler`] (including schema version and
    /// legacy migration upcasters), replacing any prior handler with the same type hash.
    pub fn register_handler<
        T: Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        handler: TypedComponentHandler<T>,
    ) {
        let name = handler.name;
        let type_id = TypeId::of::<T>();
        let hash = component_type_hash(name);
        if let Some(&existing_idx) = self.type_hash_map.get(&hash) {
            self.handlers[existing_idx] = Box::new(handler);
            self.type_id_map.insert(type_id, existing_idx);
            self.name_map.insert(name.to_string(), existing_idx);
            return;
        }
        let idx = self.handlers.len();
        self.handlers.push(Box::new(handler));
        self.type_id_map.insert(type_id, idx);
        self.type_hash_map.insert(hash, idx);
        self.name_map.insert(name.to_string(), idx);
    }

    /// Retrieves a component handler by runtime `TypeId`.
    #[must_use]
    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<&dyn ComponentHandler> {
        self.type_id_map
            .get(&type_id)
            .map(|&idx| &*self.handlers[idx])
    }

    /// Retrieves a component handler by deterministic 64-bit `type_hash`.
    #[must_use]
    pub fn get_by_type_hash(&self, type_hash: u64) -> Option<&dyn ComponentHandler> {
        self.type_hash_map
            .get(&type_hash)
            .map(|&idx| &*self.handlers[idx])
    }

    /// Returns the set of all registered 64-bit component type hashes.
    #[must_use]
    pub fn known_type_hashes(&self) -> HashSet<u64> {
        self.type_hash_map.keys().copied().collect()
    }

    /// Retrieves a component handler by component name string.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&dyn ComponentHandler> {
        self.name_map.get(name).map(|&idx| &*self.handlers[idx])
    }

    /// Returns a slice of all registered component handlers.
    #[must_use]
    pub fn handlers(&self) -> &[Box<dyn ComponentHandler>] {
        &self.handlers
    }

    /// Returns the total number of registered component handlers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Returns true if no component handlers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Clones all registered components from `source` entity to `target` entity.
    pub fn clone_entity(
        &self,
        world: &World,
        source: Entity,
        target: Entity,
        target_world: &mut World,
    ) {
        for handler in &self.handlers {
            handler.clone_component(world, source, target, target_world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_schema_version_upcast_migration_v1_to_v2_and_v3() {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct ShieldV1 {
            capacity: f32,
        }

        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct ShieldV2 {
            capacity: f32,
            recharge_rate: f32,
        }

        #[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
        struct ShieldV3 {
            capacity: f32,
            recharge_rate: f32,
            overcharge: bool,
        }

        let mut registry = ComponentRegistry::new();
        registry.register_handler(
            TypedComponentHandler::<ShieldV3>::with_default("EnergyShield", ShieldV3::default)
                .with_schema_version(3)
                .with_migration::<ShieldV1>(1, |v1| ShieldV3 {
                    capacity: v1.capacity,
                    recharge_rate: 5.0,
                    overcharge: false,
                })
                .with_migration::<ShieldV2>(2, |v2| ShieldV3 {
                    capacity: v2.capacity,
                    recharge_rate: v2.recharge_rate,
                    overcharge: true,
                }),
        );

        let handler = registry
            .get_by_name("EnergyShield")
            .expect("registered EnergyShield handler");
        assert_eq!(handler.schema_version(), 3);

        let v1_bytes = postcard::to_stdvec(&ShieldV1 { capacity: 120.0 }).expect("encode v1");
        let v2_bytes = postcard::to_stdvec(&ShieldV2 {
            capacity: 250.0,
            recharge_rate: 12.5,
        })
        .expect("encode v2");
        let v3_val = ShieldV3 {
            capacity: 500.0,
            recharge_rate: 25.0,
            overcharge: true,
        };
        let v3_bytes = postcard::to_stdvec(&v3_val).expect("encode v3");

        let mut world = World::new();
        let e1 = world.spawn(());
        let e2 = world.spawn(());
        let e3 = world.spawn(());

        handler
            .apply_versioned(&mut world, e1, 1, &v1_bytes)
            .expect("migrate v1 -> v3");
        handler
            .apply_versioned(&mut world, e2, 2, &v2_bytes)
            .expect("migrate v2 -> v3");
        handler
            .apply_versioned(&mut world, e3, 3, &v3_bytes)
            .expect("apply current v3");

        assert_eq!(
            world.get_cloned::<ShieldV3>(e1).expect("e1 shield"),
            ShieldV3 {
                capacity: 120.0,
                recharge_rate: 5.0,
                overcharge: false,
            }
        );
        assert_eq!(
            world.get_cloned::<ShieldV3>(e2).expect("e2 shield"),
            ShieldV3 {
                capacity: 250.0,
                recharge_rate: 12.5,
                overcharge: true,
            }
        );
        assert_eq!(world.get_cloned::<ShieldV3>(e3).expect("e3 shield"), v3_val);

        // Negative checks: corrupted binary payload, dangling entity, and missing default factory
        assert!(handler.apply_versioned(&mut world, e3, 99, &[]).is_err());
        assert!(
            handler
                .apply_versioned(&mut world, Entity::DANGLING, 3, &v3_bytes)
                .is_err()
        );
        registry.register_with_name::<ShieldV3>("NoDefaultShield");
        let no_def = registry
            .get_by_name("NoDefaultShield")
            .expect("NoDefaultShield registered");
        assert!(no_def.add_default(&mut world, e1).is_err());
        assert!(registry.get_by_name("NonExistent").is_none());
    }
}