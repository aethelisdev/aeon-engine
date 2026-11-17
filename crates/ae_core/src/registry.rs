// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic ECS Component Registry & Reflection subsystem.
//!
//! Provides an industry-standard component registry and dynamic serialization
//! pipeline for `hecs::World`. Enables automated snapshot capturing, state
//! restoration, entity cloning, and serialization without hardcoded component structs.
#![deny(clippy::allow_attributes)]
#![deny(clippy::unwrap_used)]

use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Interface for type-erased component operations, serialization, and lifecycle hooks.
pub trait ComponentHandler: Send + Sync {
    /// Human-readable type identifier (e.g. "Position", "Rotation", "Name").
    fn type_name(&self) -> &'static str;

    /// Deterministic 64-bit FNV-1a discriminator matching `ae_plugin_api::component_type_hash`.
    fn type_hash(&self) -> u64;

    /// Active binary schema revision emitted when capturing this component (`>= 1`).
    fn schema_version(&self) -> u16;

    /// TypeId of the underlying component.
    fn type_id(&self) -> TypeId;

    /// Captures component state from `world` on `entity` as binary `postcard` bytes.
    fn capture(&self, world: &hecs::World, entity: hecs::Entity) -> Option<Vec<u8>>;

    /// Captures component state as a JSON string strictly for live RPC debugging/inspection (`ae_mcp`).
    fn capture_debug_json(&self, world: &hecs::World, entity: hecs::Entity) -> Option<String>;

    /// Converts an offline JSON object representation into binary `postcard` bytes for migration tools.
    fn transcode_json_to_binary(&self, json: &str) -> Result<Vec<u8>, String>;

    /// Restores serialized binary `postcard` component data encoded at `schema_ver` onto `entity`,
    /// running registered schema upcasters when `schema_ver` predates [`Self::schema_version`].
    fn apply_versioned(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        schema_ver: u16,
        data: &[u8],
    ) -> Result<(), String>;

    /// Restores serialized binary `postcard` component data onto `entity` in `world` using the
    /// handler's current schema version (with automatic fallback to legacy upcasters).
    fn apply(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        data: &[u8],
    ) -> Result<(), String>;

    /// Clones the component directly from `source` entity to `target` entity without serialization overhead.
    fn clone_component(
        &self,
        world: &hecs::World,
        source: hecs::Entity,
        target: hecs::Entity,
        target_world: &mut hecs::World,
    );

    /// Checks if the component exists on the given entity in `world`.
    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool;

    /// Removes the component from the given entity in `world`.
    fn remove_component(&self, world: &mut hecs::World, entity: hecs::Entity) -> bool;

    /// Instantiates and attaches a default instance of this component onto `entity` in `world`.
    fn add_default(&self, world: &mut hecs::World, entity: hecs::Entity) -> Result<(), String>;
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

impl<T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone> ComponentHandler
    for TypedComponentHandler<T>
{
    fn type_name(&self) -> &'static str {
        self.name
    }

    fn type_hash(&self) -> u64 {
        ae_plugin_api::component_type_hash(self.name)
    }

    fn schema_version(&self) -> u16 {
        self.schema_ver
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn capture(&self, world: &hecs::World, entity: hecs::Entity) -> Option<Vec<u8>> {
        world
            .get::<&T>(entity)
            .ok()
            .and_then(|comp| postcard::to_stdvec(&*comp).ok())
    }

    fn capture_debug_json(&self, world: &hecs::World, entity: hecs::Entity) -> Option<String> {
        world
            .get::<&T>(entity)
            .ok()
            .and_then(|comp| serde_json::to_string(&*comp).ok())
    }

    fn transcode_json_to_binary(&self, json: &str) -> Result<Vec<u8>, String> {
        let comp: T = serde_json::from_str(json).map_err(|e| e.to_string())?;
        postcard::to_stdvec(&comp).map_err(|e| e.to_string())
    }

    fn apply_versioned(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        schema_ver: u16,
        data: &[u8],
    ) -> Result<(), String> {
        let comp = self.decode_payload(schema_ver, data)?;
        world
            .insert_one(entity, comp)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    fn apply(
        &self,
        world: &mut hecs::World,
        entity: hecs::Entity,
        data: &[u8],
    ) -> Result<(), String> {
        self.apply_versioned(world, entity, self.schema_ver, data)
    }

    fn clone_component(
        &self,
        world: &hecs::World,
        source: hecs::Entity,
        target: hecs::Entity,
        target_world: &mut hecs::World,
    ) {
        if let Ok(comp) = world.get::<&T>(source) {
            let cloned = (*comp).clone();
            let _ = target_world.insert_one(target, cloned);
        }
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&T>(entity).is_ok()
    }

    fn remove_component(&self, world: &mut hecs::World, entity: hecs::Entity) -> bool {
        world.remove_one::<T>(entity).is_ok()
    }

    fn add_default(&self, world: &mut hecs::World, entity: hecs::Entity) -> Result<(), String> {
        if let Some(factory) = self.default_fn {
            let instance = factory();
            world
                .insert_one(entity, instance)
                .map_err(|e| format!("{:?}", e))?;
            Ok(())
        } else {
            Err(format!(
                "Component '{}' has no default constructor registered",
                self.name
            ))
        }
    }
}

/// Central registry storing all registered component handlers for dynamic ECS operations.
pub struct ComponentRegistry {
    handlers: Vec<Box<dyn ComponentHandler>>,
    type_id_map: HashMap<TypeId, usize>,
    type_hash_map: HashMap<u64, usize>,
    name_map: HashMap<String, usize>,
}

impl ComponentRegistry {
    /// Creates an empty component registry.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            type_id_map: HashMap::new(),
            type_hash_map: HashMap::new(),
            name_map: HashMap::new(),
        }
    }

    /// Registers a component type with its default type name.
    pub fn register<
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone + Default,
    >(
        &mut self,
    ) {
        let full_name = type_name::<T>();
        let short_name = full_name.rsplit("::").next().unwrap_or(full_name);
        self.register_with_default::<T>(short_name, T::default);
    }

    /// Registers a component type with a custom identifier name.
    pub fn register_with_name<
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        name: &'static str,
    ) {
        let type_id = TypeId::of::<T>();
        let hash = ae_plugin_api::component_type_hash(name);
        let idx = self.handlers.len();
        self.handlers
            .push(Box::new(TypedComponentHandler::<T>::new(name)));
        self.type_id_map.insert(type_id, idx);
        self.type_hash_map.insert(hash, idx);
        self.name_map.insert(name.to_string(), idx);
    }

    /// Registers a component type with a custom identifier name and a default constructor.
    pub fn register_with_default<
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
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
        T: hecs::Component + serde::Serialize + serde::de::DeserializeOwned + Clone,
    >(
        &mut self,
        handler: TypedComponentHandler<T>,
    ) {
        let name = handler.name;
        let type_id = TypeId::of::<T>();
        let hash = ae_plugin_api::component_type_hash(name);
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

    /// Retrieves a component handler by `TypeId`.
    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<&dyn ComponentHandler> {
        self.type_id_map
            .get(&type_id)
            .map(|&idx| &*self.handlers[idx])
    }

    /// Retrieves a component handler by deterministic 64-bit `type_hash`.
    pub fn get_by_type_hash(&self, type_hash: u64) -> Option<&dyn ComponentHandler> {
        self.type_hash_map
            .get(&type_hash)
            .map(|&idx| &*self.handlers[idx])
    }

    /// Returns the set of all registered 64-bit component type hashes.
    pub fn known_type_hashes(&self) -> std::collections::HashSet<u64> {
        self.type_hash_map.keys().copied().collect()
    }

    /// Retrieves a component handler by component name string.
    pub fn get_by_name(&self, name: &str) -> Option<&dyn ComponentHandler> {
        self.name_map.get(name).map(|&idx| &*self.handlers[idx])
    }

    /// Returns a slice of all registered component handlers.
    pub fn handlers(&self) -> &[Box<dyn ComponentHandler>] {
        &self.handlers
    }

    /// Returns the total number of registered component handlers.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Returns true if no component handlers are registered.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Clones all registered components from `source` entity to `target` entity.
    pub fn clone_entity(
        &self,
        world: &hecs::World,
        source: hecs::Entity,
        target: hecs::Entity,
        target_world: &mut hecs::World,
    ) {
        for handler in &self.handlers {
            handler.clone_component(world, source, target, target_world);
        }
    }

    /// Builds a default component registry pre-populated with all core engine components.
    pub fn default_engine_registry() -> Self {
        use ae_plugin_api::*;
        let mut registry = Self::new();
        registry.register_with_default::<EntityGuid>("EntityGuid", EntityGuid::default);
        registry.register_with_default::<Position>("Position", Position::default);
        registry.register_with_default::<Rotation>("Rotation", Rotation::default);
        registry.register_with_default::<Scale>("Scale", Scale::default);
        registry.register_with_default::<Velocity>("Velocity", Velocity::default);
        registry.register_with_default::<Name>("Name", Name::default);
        registry.register_with_default::<Shape>("Shape", Shape::default);
        registry.register_with_default::<Color>("Color", Color::default);
        registry.register_with_default::<Light>("Light", Light::default);
        registry.register_with_default::<SpriteId>("SpriteId", SpriteId::default);
        registry.register_with_default::<ModelId>("ModelId", ModelId::default);
        registry.register_with_default::<RigidBody>("RigidBody", RigidBody::default);
        registry.register_with_default::<Collider>("Collider", Collider::default);
        registry.register_with_default::<Rotator>("Rotator", Rotator::default);
        registry.register_with_default::<MovingPlatform>("MovingPlatform", MovingPlatform::default);
        registry.register_with_default::<TriggerZone>("TriggerZone", TriggerZone::default);
        registry.register_with_default::<DestructibleTarget>(
            "DestructibleTarget",
            DestructibleTarget::default,
        );
        registry
            .register_with_default::<CharacterAction>("CharacterAction", CharacterAction::default);
        registry.register_with_default::<EphemeralProjectile>(
            "EphemeralProjectile",
            EphemeralProjectile::default,
        );
        registry.register_with_default::<CharacterController>(
            "CharacterController",
            CharacterController::default,
        );
        registry.register_with_default::<PlayerTag>("PlayerTag", PlayerTag::default);
        registry.register_with_default::<BoundingRadius>("BoundingRadius", BoundingRadius::default);
        registry.register_with_default::<BoundingBox>("BoundingBox", BoundingBox::default);
        registry.register_with_default::<Hidden>("Hidden", Hidden::default);
        registry.register_with_default::<Parent>("Parent", Parent::default);
        registry.register_with_default::<Children>("Children", Children::default);
        registry.register_with_default::<crate::ecs::LodGroup>(
            "LodGroup",
            crate::ecs::LodGroup::default,
        );
        registry.register_with_default::<PhysicsMaterialHandle>(
            "PhysicsMaterialHandle",
            PhysicsMaterialHandle::default,
        );
        registry
            .register_with_default::<PhysicsMaterial>("PhysicsMaterial", PhysicsMaterial::default);
        registry.register_with_default::<TransformDirty>("TransformDirty", TransformDirty::default);
        registry.register_with_default::<UiElement>("UiElement", UiElement::default);
        registry.register_with_default::<UiPanel>("UiPanel", UiPanel::default);
        registry.register_with_default::<UiText>("UiText", UiText::default);
        registry.register_with_default::<UiProgressBar>("UiProgressBar", UiProgressBar::default);
        registry.register_with_default::<UiButton>("UiButton", UiButton::default);
        registry.register_with_default::<UiImage>("UiImage", UiImage::default);
        registry.register_with_default::<UiSlider>("UiSlider", UiSlider::default);
        registry.register_with_default::<UiCheckbox>("UiCheckbox", UiCheckbox::default);
        registry.register_with_default::<UiTextInput>("UiTextInput", UiTextInput::default);
        registry.register_with_default::<UiLayoutGroup>("UiLayoutGroup", UiLayoutGroup::default);
        registry.register_with_default::<PlayerHealthBarTag>("PlayerHealthBarTag", || {
            PlayerHealthBarTag
        });
        registry.register_with_default::<ScoreDisplayTag>("ScoreDisplayTag", || ScoreDisplayTag);
        registry.register_with_default::<ReticleTag>("ReticleTag", || ReticleTag);
        registry.register_with_default::<SpriteRenderer>("SpriteRenderer", SpriteRenderer::default);

        registry
    }

    /// Returns a reference to the global engine component registry singleton.
    pub fn global() -> &'static ComponentRegistry {
        static REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::default_engine_registry)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::default_engine_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    #[test]
    fn test_component_registry_add_default_and_remove() {
        let registry = ComponentRegistry::global();
        let mut world = World::new();
        let entity = world.spawn(());

        let rotator_handler = registry
            .get_by_name("Rotator")
            .expect("Rotator must be registered");
        assert!(!rotator_handler.has_component(&world, entity));

        rotator_handler
            .add_default(&mut world, entity)
            .expect("add_default must succeed");
        assert!(rotator_handler.has_component(&world, entity));

        let removed = rotator_handler.remove_component(&mut world, entity);
        assert!(removed);
        assert!(!rotator_handler.has_component(&world, entity));
    }

    #[test]
    fn test_component_registry_all_registered_components_support_add_default() {
        let registry = ComponentRegistry::global();
        for handler in registry.handlers() {
            let mut world = World::new();
            let entity = world.spawn(());
            let res = handler.add_default(&mut world, entity);
            assert!(
                res.is_ok(),
                "Component '{}' must support add_default: {:?}",
                handler.type_name(),
                res.err()
            );
            assert!(handler.has_component(&world, entity));
        }
    }

    #[test]
    fn test_component_registry_sprite_renderer_roundtrip() {
        let registry = ComponentRegistry::global();
        let mut world = World::new();
        let entity = world.spawn(());

        let handler = registry
            .get_by_name("SpriteRenderer")
            .expect("SpriteRenderer must be registered in ComponentRegistry");

        let sprite = ae_plugin_api::SpriteRenderer {
            tint: [0.2, 0.4, 0.8, 1.0],
            sorting_layer: 10,
            order_in_layer: 5,
            flip_x: true,
            flip_y: false,
            uv_rect: [0.1, 0.2, 0.8, 0.9],
            ..Default::default()
        };
        world
            .insert_one(entity, sprite.clone())
            .expect("insert_one sprite must succeed in test");

        assert!(handler.has_component(&world, entity));

        let data = handler
            .capture(&world, entity)
            .expect("capture must succeed for SpriteRenderer");

        let mut target_world = World::new();
        let target_entity = target_world.spawn(());
        handler
            .apply(&mut target_world, target_entity, &data)
            .expect("apply must succeed for SpriteRenderer");

        let restored = target_world
            .get::<&ae_plugin_api::SpriteRenderer>(target_entity)
            .expect("restored SpriteRenderer must exist");

        assert_eq!(*restored, sprite);
    }

    #[test]
    fn test_component_registry_type_hashes_are_collision_free() {
        let registry = ComponentRegistry::global();
        let mut seen = std::collections::HashSet::with_capacity(registry.len());
        for handler in registry.handlers() {
            let hash = ae_plugin_api::component_type_hash(handler.type_name());
            assert!(
                seen.insert(hash),
                "Duplicate component_type_hash detected for '{}'",
                handler.type_name()
            );
        }
        assert!(registry.get_by_name("EntityGuid").is_some());
    }

    #[test]
    fn test_binary_dependencies_guid_postcard_lz4_crc_roundtrip() {
        let entity_guid =
            ae_plugin_api::EntityGuid::from_raw(0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00);
        let asset_guid =
            ae_plugin_api::AssetGuid::from_raw(0x00ff_eedd_ccbb_aa99_8877_6655_4433_2211);

        let raw = postcard::to_stdvec(&(entity_guid, asset_guid))
            .expect("postcard serialization must succeed");
        let compressed = lz4_flex::compress_prepend_size(&raw);
        let checksum = crc32fast::hash(&compressed);

        assert_ne!(checksum, 0);
        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .expect("lz4_flex decompression must succeed");
        let (decoded_entity, decoded_asset): (ae_plugin_api::EntityGuid, ae_plugin_api::AssetGuid) =
            postcard::from_bytes(&decompressed).expect("postcard deserialization must succeed");

        assert_eq!(decoded_entity, entity_guid);
        assert_eq!(decoded_asset, asset_guid);
    }

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
            *world.get::<&ShieldV3>(e1).expect("e1 shield"),
            ShieldV3 {
                capacity: 120.0,
                recharge_rate: 5.0,
                overcharge: false,
            }
        );
        assert_eq!(
            *world.get::<&ShieldV3>(e2).expect("e2 shield"),
            ShieldV3 {
                capacity: 250.0,
                recharge_rate: 12.5,
                overcharge: true,
            }
        );
        assert_eq!(*world.get::<&ShieldV3>(e3).expect("e3 shield"), v3_val);
    }
}