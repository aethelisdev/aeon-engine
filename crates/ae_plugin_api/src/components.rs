// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standard ECS component definitions shared across the engine, plugins, and gameplay scripts.
//!

use serde::{Deserialize, Serialize};

/// 3D world-space position component for ECS entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    /// Creates a new `Position` with the given coordinates.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns the origin position `(0.0, 0.0, 0.0)`.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Quaternion-based rotation component for orientation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Scale {
    /// Creates a new `Scale` with the given x, y, z scale factors.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns unit scale `(1.0, 1.0, 1.0)`.
    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

/// Marker component designating that an entity is currently hidden (not rendered).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hidden;

/// RGBA color component for entity material tinting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Creates a new `Color` with the given RGBA components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns a dark gray color `(0.2, 0.2, 0.2, 1.0)`.
    pub fn dark_gray() -> Self {
        Self {
            r: 0.2,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        }
    }

    /// Returns a soft blue color `(0.4, 0.6, 0.8, 1.0)`.
    pub fn soft_blue() -> Self {
        Self {
            r: 0.4,
            g: 0.6,
            b: 0.8,
            a: 1.0,
        }
    }

    /// Returns pure white `(1.0, 1.0, 1.0, 1.0)`.
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Returns bright red `(1.0, 0.2, 0.2, 1.0)`.
    pub fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        }
    }

    /// Returns bright green `(0.2, 1.0, 0.3, 1.0)`.
    pub fn green() -> Self {
        Self {
            r: 0.2,
            g: 1.0,
            b: 0.3,
            a: 1.0,
        }
    }

    /// Returns bright yellow `(1.0, 0.85, 0.1, 1.0)`.
    pub fn yellow() -> Self {
        Self {
            r: 1.0,
            g: 0.85,
            b: 0.1,
            a: 1.0,
        }
    }
}

/// Point light component with position and RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

slotmap::new_key_type! {
    /// Generational handle for assets stored in `AssetStorage`.
    pub struct AssetHandle;
}

/// Asset handle reference to a loaded 3D model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelId(pub AssetHandle);

/// Linear velocity component for physics-driven entity movement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Velocity {
    /// Returns zero velocity `(0.0, 0.0, 0.0)`.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Zero-cost marker tag identifying the player-controlled entity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTag;

/// Human-readable display name for an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

/// Built-in geometric shape type for primitive entity rendering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Triangle,
    #[default]
    Cube,
    Sphere,
    Cylinder,
    Capsule,
    Torus,
}

/// Asset handle reference to a loaded 2D texture sprite.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteId(pub AssetHandle);

/// Reference to a physics material asset for friction and restitution properties.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsMaterialHandle(pub AssetHandle);

/// Bounding sphere radius for broad-phase frustum culling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingRadius(pub f32);

/// Axis-Aligned Bounding Box for spatial queries and selection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Marker component flagging entities whose transform changed this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformDirty;

/// Physics body simulation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    Static,
    Dynamic,
    Kinematic,
}

/// Physics collision shape for rigid bodies and static colliders.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    Box {
        half_extents: [f32; 3],
    },
    Sphere {
        radius: f32,
    },
    Capsule {
        half_height: f32,
        radius: f32,
        #[serde(default)]
        center_y: f32,
    },
    Trimesh,
    ConvexHull,
}

/// Rigid body physics component attached to simulated entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub gravity_scale: f32,
}

/// Collider physics component attached to geometric entities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    #[serde(default)]
    pub is_sensor: bool,
}

/// Kinematic character controller physics component for 3D character locomotion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharacterController {
    pub height: f32,
    pub radius: f32,
    #[serde(default)]
    pub center_y: f32,
    pub max_slope_climb_angle: f32,
    pub step_height: f32,
    pub is_grounded: bool,
}

impl CharacterController {
    /// Returns the computed capsule half-height ensuring a safe positive minimum bound `0.05`.
    pub fn capsule_half_height(&self) -> f32 {
        (self.height * 0.5 - self.radius).max(0.05)
    }
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            height: 1.8,
            radius: 0.4,
            center_y: 0.0,
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

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::identity()
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::one()
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::white()
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
        }
    }
}

impl Default for Name {
    fn default() -> Self {
        Self("Entity".to_string())
    }
}

impl Default for BoundingRadius {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: [-0.5, -0.5, -0.5],
            max: [0.5, 0.5, 0.5],
        }
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
        }
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        }
    }
}

/// Parent entity reference for hierarchical transforms and scene graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parent(pub hecs::Entity);

impl Default for Parent {
    fn default() -> Self {
        Self(hecs::Entity::DANGLING)
    }
}

/// Child entity list for hierarchical transforms and scene graphs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Children(pub Vec<hecs::Entity>);

/// Cached world-space transform matrix for hierarchical transforms, picking, and rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub cgmath::Matrix4<f32>);

// =========================================================================
// =========================================================================
// MODULAR GAMEPLAY SCRIPTING & ECS BEHAVIOR COMPONENTS
// =========================================================================

/// Continuous rotational behavior component for rotating entities around a 3D axis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotator {
    /// Angular rotation speed in radians per second.
    pub speed: f32,
    /// 3D rotational unit axis (e.g. `[0.0, 1.0, 0.0]` for Y-axis rotation).
    pub axis: [f32; 3],
}

impl Rotator {
    /// Creates a new `Rotator` with the specified rotation speed and axis.
    pub fn new(speed: f32, axis: [f32; 3]) -> Self {
        Self { speed, axis }
    }
}

impl Default for Rotator {
    fn default() -> Self {
        Self {
            speed: 1.5,
            axis: [0.0, 1.0, 0.0],
        }
    }
}

/// Waypoint interpolation behavior component for moving entities back and forth between two points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MovingPlatform {
    /// Linear movement speed in units per second.
    pub speed: f32,
    /// Initial starting or rest position.
    pub original_position: [f32; 3],
    /// Target waypoint position for the entity to move toward.
    pub target_position: [f32; 3],
    /// Direction flag for ping-pong movement (`true` moving to target, `false` returning).
    pub ping_pong_forward: bool,
}

impl MovingPlatform {
    /// Creates a new `MovingPlatform` between two 3D positions.
    pub fn new(speed: f32, original_position: [f32; 3], target_position: [f32; 3]) -> Self {
        Self {
            speed,
            original_position,
            target_position,
            ping_pong_forward: true,
        }
    }
}

impl Default for MovingPlatform {
    fn default() -> Self {
        Self {
            speed: 2.5,
            original_position: [0.0, 0.0, 0.0],
            target_position: [0.0, 5.0, 0.0],
            ping_pong_forward: true,
        }
    }
}

/// Proximity sensor and mechanism behavior component for reactive trigger areas, doors, and elevators.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TriggerZone {
    /// Flag indicating whether the trigger zone is currently activated by an overlapping actor.
    pub is_triggered: bool,
    /// Movement speed for attached mechanism transitions (e.g. door opening/closing).
    pub speed: f32,
    /// Primary motion axis vector for the activated mechanism.
    pub axis: [f32; 3],
    /// Target position when fully triggered.
    pub target_position: [f32; 3],
    /// Original rest position when un-triggered.
    pub original_position: [f32; 3],
    /// Direction flag for mechanism movement.
    pub ping_pong_forward: bool,
}

impl TriggerZone {
    /// Creates a default `TriggerZone` for proximity detection and mechanism activation.
    pub fn new() -> Self {
        Self {
            is_triggered: false,
            speed: 3.0,
            axis: [0.0, 1.0, 0.0],
            target_position: [0.0, 4.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            ping_pong_forward: true,
        }
    }
}

impl Default for TriggerZone {
    fn default() -> Self {
        Self::new()
    }
}

/// Destructible combat target behavior component managing health, damage reaction, and hit flash effects.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DestructibleTarget {
    /// Current health points of the target entity.
    pub health: f32,
    /// Maximum health points capacity of the target entity.
    pub max_health: f32,
    /// Remaining duration in seconds for the visual damage hit-flash tint.
    pub hit_flash_timer: f32,
    /// Original RGBA color of the target entity restored after hit-flash decay.
    pub original_color: [f32; 4],
}

impl DestructibleTarget {
    /// Creates a new `DestructibleTarget` with the specified maximum health pool.
    pub fn new(max_health: f32) -> Self {
        Self {
            health: max_health,
            max_health,
            hit_flash_timer: 0.0,
            original_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl Default for DestructibleTarget {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// Character weapon and world interaction behavior component for shooting raycasts and spawning projectiles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharacterAction {
    /// Raycast range or projectile ejection speed.
    pub speed: f32,
    /// Fire direction axis relative to camera or forward transform.
    pub axis: [f32; 3],
    /// Cooldown period between consecutive weapon actions in seconds.
    pub cooldown: f32,
    /// Current timer accumulator for weapon cooldown gating.
    pub timer: f32,
}

impl CharacterAction {
    /// Creates a new `CharacterAction` component with default weapon parameters.
    pub fn new() -> Self {
        Self {
            speed: 50.0,
            axis: [0.0, 0.0, -1.0],
            cooldown: 0.2,
            timer: 0.0,
        }
    }
}

impl Default for CharacterAction {
    fn default() -> Self {
        Self::new()
    }
}

/// Ephemeral projectile marker component that despawns an entity after its lifetime expires.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EphemeralProjectile {
    /// Remaining lifetime in seconds before automatic entity destruction.
    pub lifetime_remaining: f32,
}

impl EphemeralProjectile {
    /// Creates a new `EphemeralProjectile` with the specified lifetime in seconds.
    pub fn new(lifetime_remaining: f32) -> Self {
        Self { lifetime_remaining }
    }
}

impl Default for EphemeralProjectile {
    fn default() -> Self {
        Self::new(0.7)
    }
}