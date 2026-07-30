// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// AE Plugin API — Shared Types for the Plugin System
/// This crate defines the types and traits shared across the FFI boundary
/// between the engine core and dynamically loaded plugins.
/// Both the engine and all plugins depend on this crate to ensure ABI compatibility.
/// # Safety Philosophy
/// This crate contains ZERO `unsafe` blocks. The PluginContext uses safe
/// mutable references with explicit lifetimes. The FFI struct uses a reference
/// so plugins never need to dereference raw pointers.
/// The compiled ABI version hash of the Aeon Engine.
/// This hash is automatically constructed at compile time using the package version
/// and active build profile (debug vs release). It is passed across the FFI boundary
/// and verified on dynamic library load to prevent severe memory corruption and
/// segmentation faults caused by Rust ABI mismatch.
pub const ENGINE_ABI_HASH: &str = if cfg!(debug_assertions) {
    "ae-abi-v0.7.0-debug"
} else {
    "ae-abi-v0.7.0-release"
};

/// Returns the platform-specific dynamic library extension.
/// - Windows → `dll`
/// - Linux → `so`
/// - macOS → `dylib`
pub fn platform_lib_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

/// Type-safe, dynamic resource container (dependency injection map).
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut::<T>())
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|any| any.downcast::<T>().ok().map(|boxed| *boxed))
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

/// Core engine modules that can be dynamically enabled/disabled at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineModule {
    Physics,
    Audio,
    Render,
}

/// The mode in which the engine runs: Edit (level editor active) or Play (game loop active).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EngineMode {
    Edit,
    Play,
}

/// Empty trait representing any event within the engine.
pub trait Event: Any + Send + Sync {}

/// Type-agnostic dynamic event bus.
pub struct DynamicEventBus {
    queues: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    pub enabled_modules: std::collections::HashSet<EngineModule>,
}

impl DynamicEventBus {
    pub fn new() -> Self {
        let mut enabled_modules = std::collections::HashSet::new();
        enabled_modules.insert(EngineModule::Physics);
        enabled_modules.insert(EngineModule::Audio);
        enabled_modules.insert(EngineModule::Render);
        Self {
            queues: HashMap::new(),
            enabled_modules,
        }
    }

    pub fn send<E: Event>(&mut self, event: E) {
        let queue = self
            .queues
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(VecDeque::<E>::new()));
        if let Some(q) = queue.downcast_mut::<VecDeque<E>>() {
            q.push_back(event);
        }
    }

    pub fn receive<E: Event>(&mut self) -> Option<VecDeque<E>> {
        let queue = self.queues.get_mut(&TypeId::of::<E>())?;
        if let Some(q) = queue.downcast_mut::<VecDeque<E>>() {
            Some(std::mem::take(q))
        } else {
            None
        }
    }

    pub fn has_events<E: Event>(&self) -> bool {
        self.queues
            .get(&TypeId::of::<E>())
            .and_then(|any| any.downcast_ref::<VecDeque<E>>())
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    pub fn clear(&mut self) {
        self.queues.clear();
    }

    pub fn is_module_enabled(&self, module: EngineModule) -> bool {
        self.enabled_modules.contains(&module)
    }

    pub fn set_module_enabled(&mut self, module: EngineModule, enabled: bool) {
        if enabled {
            self.enabled_modules.insert(module);
        } else {
            self.enabled_modules.remove(&module);
        }
    }
}

impl Default for DynamicEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// --- Standard Core Event Definitions ---
#[derive(Debug, Clone, Copy)]
pub struct CollisionEnter {
    pub entity_a: hecs::Entity,
    pub entity_b: hecs::Entity,
}
impl Event for CollisionEnter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionExit {
    pub entity_a: hecs::Entity,
    pub entity_b: hecs::Entity,
}
impl Event for CollisionExit {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerEnter {
    pub entity_a: hecs::Entity,
    pub entity_b: hecs::Entity,
}
impl Event for TriggerEnter {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerExit {
    pub entity_a: hecs::Entity,
    pub entity_b: hecs::Entity,
}
impl Event for TriggerExit {}

#[derive(Debug, Clone, Copy)]
pub struct EntitySpawned(pub hecs::Entity);
impl Event for EntitySpawned {}

#[derive(Debug, Clone, Copy)]
pub struct EntityDestroyed(pub hecs::Entity);
impl Event for EntityDestroyed {}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsTick;
impl Event for PhysicsTick {}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsUpdate;
impl Event for PhysicsUpdate {}

#[derive(Debug, Clone, Copy)]
pub struct PlaySound(pub &'static str);
impl Event for PlaySound {}

#[derive(Debug, Clone, Copy)]
pub struct StopSound(pub &'static str);
impl Event for StopSound {}

#[derive(Debug, Clone, Copy)]
pub struct AudioMute;
impl Event for AudioMute {}

#[derive(Debug, Clone, Copy)]
pub struct RenderFrame;
impl Event for RenderFrame {}

#[derive(Debug, Clone, Copy)]
pub struct ViewportResize;
impl Event for ViewportResize {}

#[derive(Debug, Clone, Copy)]
pub struct MaterialChanged;
impl Event for MaterialChanged {}

/// A resource containing the list of entity IDs that were visible in the previous frame.
#[derive(Debug, Clone)]
pub struct VisibleEntities {
    /// Inner list of visible entity generational IDs.
    pub entities: Vec<hecs::Entity>,
}

/// Context passed from the engine to plugins on each update call.
/// Contains mutable references to the ECS World, Resources, and DynamicEventBus.
/// The lifetime `'a` guarantees these references outlive the context itself.
pub struct PluginContext<'a> {
    /// Mutable reference to the `hecs::World`.
    pub world: &'a mut hecs::World,
    /// Mutable reference to the `Resources` map.
    pub resources: &'a mut Resources,
    /// Mutable reference to the `DynamicEventBus`.
    pub event_bus: &'a mut DynamicEventBus,
    /// Time elapsed since the last frame, in seconds.
    pub delta_time: f32,
}

impl<'a> PluginContext<'a> {
    /// Creates a new PluginContext.
    pub fn new(
        world: &'a mut hecs::World,
        resources: &'a mut Resources,
        event_bus: &'a mut DynamicEventBus,
        delta_time: f32,
    ) -> Self {
        Self {
            world,
            resources,
            event_bus,
            delta_time,
        }
    }

    /// Returns a mutable reference to the ECS World.
    pub fn world_mut(&mut self) -> &mut hecs::World {
        self.world
    }

    /// Returns an immutable reference to the ECS World.
    pub fn world(&self) -> &hecs::World {
        self.world
    }
}

/// FFI-safe context struct that crosses the `extern "C"` boundary.
/// Uses FFI-safe references with lifetimes to ensure complete memory safety
/// without raw pointers.
#[repr(C)]
pub struct PluginContextFFI<'a> {
    /// Reference to the ECS World.
    pub world: Option<&'a mut hecs::World>,
    /// Reference to the Resources map.
    pub resources: Option<&'a mut Resources>,
    /// Reference to the DynamicEventBus.
    pub event_bus: Option<&'a mut DynamicEventBus>,
    /// Delta time since last frame.
    pub delta_time: f32,
}

impl<'a> PluginContextFFI<'a> {
    /// Provides safe access to the World for plugin code.
    pub fn get_world(&mut self) -> Option<&mut hecs::World> {
        match &mut self.world {
            Some(w) => Some(*w),
            None => None,
        }
    }

    /// Provides safe access to the Resources map for plugin code.
    pub fn get_resources(&mut self) -> Option<&mut Resources> {
        match &mut self.resources {
            Some(r) => Some(*r),
            None => None,
        }
    }

    /// Provides safe access to the DynamicEventBus for plugin code.
    pub fn get_event_bus(&mut self) -> Option<&mut DynamicEventBus> {
        match &mut self.event_bus {
            Some(e) => Some(*e),
            None => None,
        }
    }
}

/// Type signature for the plugin update function exported via `extern "C"`.
/// Plugins must export a function matching this signature with `#[no_mangle]`.
pub type PluginUpdateFn = unsafe extern "C" fn(&mut PluginContextFFI<'_>);

/// Trait abstraction for scripting/plugin backends.
/// This trait enables future scripting systems (Lua, WASM, etc.)
/// to plug into the same architecture alongside native Rust plugins.
/// # Implementors
/// - `NativePluginBackend` — Loads `.dll`/`.so`/`.dylib` via libloading
/// - Future: `LuaBackend`, `WasmBackend`, etc.
pub trait ScriptingBackend {
    /// Returns the human-readable name of this backend (e.g., "Native", "Lua").
    fn name(&self) -> &str;

    /// Loads a plugin/script from the given path.
    fn load(&mut self, path: &Path) -> Result<(), String>;

    /// Unloads the currently loaded plugin/script.
    fn unload(&mut self) -> Result<(), String>;

    /// Calls the plugin's update function with the given context.
    fn call_update(&self, ctx: &mut PluginContext) -> Result<(), String>;

    /// Checks if the source file has changed and a reload is needed.
    fn needs_reload(&self) -> bool;

    /// Performs a hot reload cycle: unload old → load new.
    fn reload(&mut self) -> Result<(), String>;
}

use serde::{Deserialize, Serialize};

/// 3D world-space position component for ECS entities.
/// Stored as individual `f32` fields rather than a vector type for
/// direct serialization compatibility and ECS query efficiency.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Quaternion-based rotation component for gimbal-lock-free 3D orientation.
/// Fields `(x, y, z, w)` represent the quaternion components where `w` is scalar.
/// Use `identity()` for no rotation. Quaternion math avoids gimbal lock and
/// supports smooth interpolation (SLERP) for animation systems.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Rotation {
    /// Returns the identity quaternion (no rotation: `w=1, x=y=z=0`).
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

/// Non-uniform scale component for entity transform.
/// Each axis (x, y, z) can be scaled independently. A scale of `(1, 1, 1)`
/// represents the original size. Minimum clamped to `0.001` during gizmo
/// interaction to prevent degenerate zero-volume transforms.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Scale {
    /// Returns unit scale `(1.0, 1.0, 1.0)` — the entity's original size.
    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

/// RGBA color component for entity material tinting.
/// All channels are in the `[0.0, 1.0]` linear range. Alpha `a` is used
/// for transparency sorting when the render pipeline supports it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Returns a dark gray color `(0.2, 0.2, 0.2, 1.0)` for default surfaces.
    pub fn dark_gray() -> Self {
        Self {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        }
    }
    /// Returns a soft blue color `(0.4, 0.6, 0.8, 1.0)` used as the default cube color.
    pub fn soft_blue() -> Self {
        Self {
            r: 0.4,
            g: 0.6,
            b: 0.8,
            a: 1.0,
        }
    }
}

/// Point light component with position and RGB color.
/// Position is in world-space `[f32; 3]`. Color channels are in `[0.0, 1.0]` range.
/// Used by the PBR lighting pipeline for per-fragment illumination calculation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

slotmap::new_key_type! {
    /// Generational handle for assets stored in `AssetStorage`.
    /// Backed by `slotmap::KeyData` — provides O(1) lookup with generation
    /// tracking to prevent use-after-free on removed assets.
    /// Automatically derives `Serialize`/`Deserialize` via slotmap's serde feature.
    pub struct AssetHandle;
}

/// Asset handle reference to a loaded 3D model (GLTF/GLB/FBX).
/// The inner `AssetHandle` indexes into `AssetManager::models`.
/// Entities with this component are rendered using the model's vertex/index buffers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelId(pub AssetHandle);

/// Linear velocity component for physics-driven entity movement.
/// Applied per-frame by `EcsManager::update()` as `position += velocity * dt`.
/// Velocity integration runs in parallel via Rayon for cache-friendly throughput.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Zero-cost marker tag identifying the player-controlled entity.
/// Empty struct (ZST) — occupies no memory. Used as a query filter
/// in `fixed_update_play_mode()` to isolate player input handling.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerTag;

/// Human-readable display name for an entity (shown in Hierarchy panel).
/// Stored as a heap-allocated `String` to support arbitrary-length names
/// and runtime renaming via the inspector UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

/// Built-in geometric shape type for primitive entity rendering.
/// Entities with a `Shape` component are rendered using engine-generated
/// vertex data (no external model required). Currently supports Triangle,
/// Cube, Sphere, Cylinder, Capsule, and Torus primitives.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    /// A flat triangle primitive (3 vertices).
    Triangle,
    /// A unit cube primitive (24 vertices, 36 indices).
    Cube,
    /// A parametric sphere primitive.
    Sphere,
    /// A parametric cylinder primitive.
    Cylinder,
    /// A parametric capsule primitive.
    Capsule,
    /// A parametric torus primitive.
    Torus,
}

/// Asset handle reference to a loaded 2D texture used as a billboard sprite.
/// The inner `AssetHandle` indexes into `AssetManager::textures`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpriteId(pub AssetHandle);

/// Reference to a physics material asset for friction and bounciness properties.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialHandle(pub AssetHandle);

/// Bounding sphere radius for broad-phase frustum culling.
/// Entities outside the camera frustum (by this radius) are excluded
/// from the render instance list before GPU submission.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingRadius(pub f32);

/// Axis-Aligned Bounding Box for precise raycasting and entity selection.
/// Defines the local-space extents of an entity's geometry. Used by the
/// picking system to test ray-AABB intersection in local coordinates.
/// For imported models, `min`/`max` are computed from mesh vertex data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Marker component flagging entities whose transform changed this frame.
/// Inserted by gizmo drag and inspector edits. Consumed by the render
/// pipeline to rebuild only the model matrices of dirty entities,
/// avoiding a full-scene matrix recomputation every frame.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransformDirty;

/// Physics body type. Static: does not move, Dynamic: subject to gravity + force, Kinematic: controlled via code.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RigidBodyType {
    Static,
    Dynamic,
    Kinematic,
}

/// Physics collision shape for rigid bodies and static colliders.
/// Supports standard shapes (Box, Sphere, Capsule) as well as complex geometries
/// extracted from meshes (Trimesh, ConvexHull) using their associated ModelId.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    /// A rectangular box shape defined by half-extents along the X, Y, and Z axes.
    Box { half_extents: [f32; 3] },
    /// A sphere shape defined by its radius.
    Sphere { radius: f32 },
    /// A capsule shape aligned along the Y-axis defined by half-height and radius.
    Capsule { half_height: f32, radius: f32 },
    /// Triangle mesh shape for static environments. Resolves vertices/indices from ModelId component.
    Trimesh,
    /// Convex hull shape for dynamic/kinematic bodies. Resolves vertices from ModelId component.
    ConvexHull,
}

/// Rigid body physics component to be attached to entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub gravity_scale: f32,
}

/// Collider physics component to be attached to entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    #[serde(default)]
    pub is_sensor: bool,
}

/// Kinematic character controller physics component for 3D player movement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharacterController {
    pub height: f32,
    pub radius: f32,
    pub max_slope_climb_angle: f32,
    pub step_height: f32,
    pub is_grounded: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            height: 1.8,
            radius: 0.4,
            max_slope_climb_angle: 45.0,
            step_height: 0.3,
            is_grounded: false,
        }
    }
}

/// Structured hit output returned by 3D physics raycasting queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    pub entity: hecs::Entity,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
}

/// Parent entity reference for hierarchical transforms (future use).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Parent(pub hecs::Entity);

/// Child entity list for hierarchical transforms (future use).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Children(pub Vec<hecs::Entity>);

/// Cached world-space transform matrix for hierarchical rendering (future use).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform(pub cgmath::Matrix4<f32>);