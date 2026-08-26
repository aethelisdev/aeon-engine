// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Physics, rigid bodies, colliders, kinematic controllers, and material properties.
//!

use serde::{Deserialize, Serialize};

use super::rendering::AssetHandle;

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

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

/// Reference to a physics material asset for friction and restitution properties.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsMaterialHandle(pub AssetHandle);

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

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
        }
    }
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

/// Physical surface classification for collision audio and impact effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SurfaceType {
    #[default]
    Default,
    Metal,
    Wood,
    Stone,
    Flesh,
    Dirt,
    Glass,
}

/// Physical properties profile defining standard friction, restitution, and density parameters for a surface type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceProperties {
    /// Surface friction coefficient.
    pub friction: f32,
    /// Bounciness / restitution coefficient.
    pub restitution: f32,
    /// Material density in kg/m^3.
    pub density: f32,
}

impl SurfaceType {
    /// Returns the canonical physics properties preset for this surface type.
    pub const fn properties(&self) -> SurfaceProperties {
        match self {
            Self::Default => SurfaceProperties {
                friction: 0.7,
                restitution: 0.0,
                density: 1000.0,
            },
            Self::Metal => SurfaceProperties {
                friction: 0.4,
                restitution: 0.85,
                density: 7800.0,
            },
            Self::Wood => SurfaceProperties {
                friction: 0.6,
                restitution: 0.25,
                density: 700.0,
            },
            Self::Stone => SurfaceProperties {
                friction: 0.9,
                restitution: 0.05,
                density: 2500.0,
            },
            Self::Flesh => SurfaceProperties {
                friction: 0.8,
                restitution: 0.0,
                density: 1000.0,
            },
            Self::Dirt => SurfaceProperties {
                friction: 0.8,
                restitution: 0.0,
                density: 1500.0,
            },
            Self::Glass => SurfaceProperties {
                friction: 0.08,
                restitution: 0.35,
                density: 2500.0,
            },
        }
    }

    /// Returns the default friction coefficient for this surface type.
    pub const fn default_friction(&self) -> f32 {
        self.properties().friction
    }

    /// Returns the default restitution (bounciness) coefficient for this surface type.
    pub const fn default_restitution(&self) -> f32 {
        self.properties().restitution
    }
}

/// Physics material properties defining friction, bounciness (restitution), and surface type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    /// Surface material type for collision audio and VFX cues.
    pub surface_type: SurfaceType,
    /// Friction coefficient between surfaces (0.0 to 1.0+).
    pub friction: f32,
    /// Bounciness / restitution coefficient (0.0 = no bounce, 1.0 = full elasticity).
    pub restitution: f32,
}

impl PhysicsMaterial {
    /// Creates a new `PhysicsMaterial` with the given surface type, friction, and restitution.
    pub fn new(surface_type: SurfaceType, friction: f32, restitution: f32) -> Self {
        Self {
            surface_type,
            friction,
            restitution,
        }
    }

    /// Creates a `PhysicsMaterial` populated with standard industry preset values for the specified `SurfaceType`.
    pub fn from_preset(surface_type: SurfaceType) -> Self {
        let props = surface_type.properties();
        Self {
            surface_type,
            friction: props.friction,
            restitution: props.restitution,
        }
    }

    /// Resets this material's friction and restitution to match the preset defaults of its current `surface_type`.
    pub fn apply_preset(&mut self, surface_type: SurfaceType) {
        let props = surface_type.properties();
        self.surface_type = surface_type;
        self.friction = props.friction;
        self.restitution = props.restitution;
    }
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self::from_preset(SurfaceType::Default)
    }
}