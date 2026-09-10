// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec2;
use rapier2d::prelude::RigidBodyHandle;

/// Classification of body dynamics governing mass, inertia, and force integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BodyType2D {
    /// Responds to external forces, impulses, and contacts.
    #[default]
    Dynamic,
    /// Static obstacle with infinite mass unaffected by forces or velocities.
    Fixed,
    /// Driven by explicit velocity commands, ignoring forces and impacts.
    KinematicVelocityBased,
    /// Driven by direct position updates, teleports without velocity integration.
    KinematicPositionBased,
}

/// 2D rigid body component representing physics presence and state in the simulation.
#[derive(Debug, Clone)]
pub struct RigidBody2D {
    /// Movement dynamics classification.
    pub body_type: BodyType2D,
    /// Linear velocity vector in world space (units/sec).
    pub linear_velocity: Vec2,
    /// Angular velocity in radians per second.
    pub angular_velocity: f32,
    /// Linear damping coefficient dissipating kinetic energy over time (`>= 0.0`).
    pub linear_damping: f32,
    /// Angular damping coefficient dissipating rotational energy (`>= 0.0`).
    pub angular_damping: f32,
    /// Whether rotation along the 2D Z-axis is locked (preventing tilt/tipping).
    pub lock_rotations: bool,
    /// Gravity scaling multiplier (`1.0` = normal gravity, `0.0` = zero-gravity).
    pub gravity_scale: f32,
    /// Internal handle to the Rapier2D rigid body instance.
    pub handle: Option<RigidBodyHandle>,
}

impl Default for RigidBody2D {
    fn default() -> Self {
        Self {
            body_type: BodyType2D::Dynamic,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            lock_rotations: false,
            gravity_scale: 1.0,
            handle: None,
        }
    }
}

impl RigidBody2D {
    /// Constructs a new dynamic rigid body with default gravity and zero velocity.
    pub fn new(body_type: BodyType2D) -> Self {
        Self {
            body_type,
            ..Default::default()
        }
    }

    /// Convenience builder setting rotation lock on the rigid body.
    pub fn with_lock_rotations(mut self, locked: bool) -> Self {
        self.lock_rotations = locked;
        self
    }

    /// Convenience builder setting gravity scale factor.
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }
}