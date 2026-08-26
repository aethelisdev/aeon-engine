// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Gameplay mechanics, combat actors, interactive triggers, and entity tags.
//!

use serde::{Deserialize, Serialize};

/// Zero-cost marker tag identifying the player-controlled entity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTag;

/// Human-readable display name for an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

impl Default for Name {
    fn default() -> Self {
        Self("Entity".to_string())
    }
}

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