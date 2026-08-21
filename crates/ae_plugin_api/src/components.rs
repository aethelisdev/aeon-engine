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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTag;

/// Human-readable display name for an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

/// Built-in geometric shape type for primitive entity rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Triangle,
    Cube,
    Sphere,
    Cylinder,
    Capsule,
    Torus,
}

/// Asset handle reference to a loaded 2D texture sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteId(pub AssetHandle);

/// Reference to a physics material asset for friction and restitution properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Parent entity reference for hierarchical transforms and scene graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parent(pub hecs::Entity);

/// Child entity list for hierarchical transforms and scene graphs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Children(pub Vec<hecs::Entity>);

/// Cached world-space transform matrix for hierarchical transforms, picking, and rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub cgmath::Matrix4<f32>);

// =========================================================================
// PHASE 1: GAMEPLAY SCRIPTING & BEHAVIOR SYSTEM
// =========================================================================

/// Predefined gameplay behavior category for data-driven entity execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorType {
    /// Smoothly rotates the entity around a defined axis.
    Rotator,
    /// Proximity volume that toggles state when entities enter or exit its boundary.
    TriggerZone,
    /// Hit-reactive target entity with health, damage response, and despawn handling.
    DestructibleTarget,
    /// Entity that translates back and forth between waypoints.
    MovingPlatform,
    /// Character actions including raycast shooting and world interaction.
    CharacterAction,
    /// Custom user-defined or plugin-driven script behavior.
    Custom,
}

/// Generic gameplay behavior component storing runtime state and configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorComponent {
    /// Active behavior archetype category.
    pub behavior_type: BehaviorType,
    /// General speed multiplier for rotation, movement, or cooldowns.
    pub speed: f32,
    /// Primary 3D axis vector (e.g. `[0.0, 1.0, 0.0]` for Y-axis rotation).
    pub axis: [f32; 3],
    /// Current health pool for destructible targets.
    pub health: f32,
    /// Maximum health pool for destructible targets.
    pub max_health: f32,
    /// Flag indicating whether the trigger zone is currently activated.
    pub is_triggered: bool,
    /// Target world-space position for moving platforms or sliding elements.
    pub target_position: [f32; 3],
    /// Original rest position for returning or ping-pong movements.
    pub original_position: [f32; 3],
    /// Direction flag for ping-pong waypoint interpolation (`true` moving to target, `false` returning).
    pub ping_pong_forward: bool,
    /// Accumulator timer used for cooldowns, periodic oscillation, or hit-flash decay.
    pub timer: f32,
    /// Hit flash timer indicating remaining duration of the visual damage tint.
    pub hit_flash_timer: f32,
}

impl BehaviorComponent {
    /// Creates a new `Rotator` behavior component with the specified rotation speed and axis.
    pub fn rotator(speed: f32, axis: [f32; 3]) -> Self {
        Self {
            behavior_type: BehaviorType::Rotator,
            speed,
            axis,
            health: 100.0,
            max_health: 100.0,
            is_triggered: false,
            target_position: [0.0, 0.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
        }
    }

    /// Creates a new `TriggerZone` behavior component for proximity detection.
    pub fn trigger_zone() -> Self {
        Self {
            behavior_type: BehaviorType::TriggerZone,
            speed: 3.0,
            axis: [0.0, 1.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            is_triggered: false,
            target_position: [0.0, 0.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
        }
    }

    /// Creates a new `DestructibleTarget` behavior component with the given health pool.
    pub fn destructible_target(max_health: f32) -> Self {
        Self {
            behavior_type: BehaviorType::DestructibleTarget,
            speed: 1.0,
            axis: [0.0, 1.0, 0.0],
            health: max_health,
            max_health,
            is_triggered: false,
            target_position: [0.0, 0.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
        }
    }

    /// Creates a new `MovingPlatform` behavior component between two positions.
    pub fn moving_platform(
        speed: f32,
        original_position: [f32; 3],
        target_position: [f32; 3],
    ) -> Self {
        Self {
            behavior_type: BehaviorType::MovingPlatform,
            speed,
            axis: [1.0, 0.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            is_triggered: false,
            target_position,
            original_position,
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
        }
    }

    /// Creates a new `CharacterAction` behavior component for player weapon shooting and actions.
    pub fn character_action() -> Self {
        Self {
            behavior_type: BehaviorType::CharacterAction,
            speed: 10.0, // Raycast max range (e.g. 50.0m) or fire rate
            axis: [0.0, 0.0, -1.0],
            health: 100.0,
            max_health: 100.0,
            is_triggered: false,
            target_position: [0.0, 0.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            ping_pong_forward: true,
            timer: 0.0,
            hit_flash_timer: 0.0,
        }
    }
}

impl Default for BehaviorComponent {
    fn default() -> Self {
        Self::rotator(1.5, [0.0, 1.0, 0.0])
    }
}