// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec2;
use rapier2d::prelude::ColliderHandle;

/// Geometric collision shape representation for 2D rigid bodies.
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderShape2D {
    /// Axis-aligned box with half-extents `[half_width, half_height]`.
    Cuboid { half_extents: Vec2 },
    /// Circle with a given radius.
    Ball { radius: f32 },
    /// Capsule aligned with Y axis with half height and circular cap radius.
    Capsule { half_height: f32, radius: f32 },
}

/// 2D collision geometry and surface friction/bounciness component.
#[derive(Debug, Clone)]
pub struct Collider2D {
    /// Collision primitive shape definition.
    pub shape: ColliderShape2D,
    /// Coulomb friction coefficient (`>= 0.0`).
    pub friction: f32,
    /// Restitution / bounciness coefficient clamped to `[0.0, 1.0]`.
    pub restitution: f32,
    /// Trigger flag: when true, collider generates overlap events without exerting contact forces.
    pub is_sensor: bool,
    /// Internal handle to the Rapier2D collider instance.
    pub handle: Option<ColliderHandle>,
}

impl Default for Collider2D {
    fn default() -> Self {
        Self {
            shape: ColliderShape2D::Cuboid {
                half_extents: Vec2::new(0.5, 0.5),
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
            handle: None,
        }
    }
}

impl Collider2D {
    /// Constructs a box collider with given half-extents.
    pub fn cuboid(half_width: f32, half_height: f32) -> Self {
        Self {
            shape: ColliderShape2D::Cuboid {
                half_extents: Vec2::new(half_width.max(0.001), half_height.max(0.001)),
            },
            ..Default::default()
        }
    }

    /// Constructs a circular ball collider with given radius.
    pub fn ball(radius: f32) -> Self {
        Self {
            shape: ColliderShape2D::Ball {
                radius: radius.max(0.001),
            },
            ..Default::default()
        }
    }

    /// Sets whether this collider behaves as an overlap sensor/trigger.
    pub fn with_sensor(mut self, sensor: bool) -> Self {
        self.is_sensor = sensor;
        self
    }

    /// Sets friction coefficient for surface interaction.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0);
        self
    }

    /// Sets restitution (bounciness) coefficient for collisions.
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }
}