// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Physics, rigid bodies, colliders, kinematic controllers, and material properties.

use serde::{Deserialize, Serialize};

use super::rendering::{AssetHandle, BoundingBox};

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

fn valid_box_extents(bounds: &BoundingBox) -> Option<([f32; 3], f32)> {
    let finite = bounds
        .min
        .iter()
        .chain(bounds.max.iter())
        .all(|v| v.is_finite());
    if !finite {
        return None;
    }
    let dx = (bounds.max[0] - bounds.min[0]).abs();
    let dy = (bounds.max[1] - bounds.min[1]).abs();
    let dz = (bounds.max[2] - bounds.min[2]).abs();
    if dx.max(dy).max(dz) <= 1e-4 {
        return None;
    }
    let center_y = (bounds.min[1] + bounds.max[1]) * 0.5;
    Some(([dx, dy, dz], center_y))
}

impl ColliderShape {
    /// Fits a box collider shape's half-extents cleanly to `bounds`, falling back to `0.5` unit cube
    /// extents if `bounds` is degenerate or contains non-finite coordinates.
    #[must_use]
    pub fn fit_box_from_bounds(bounds: &BoundingBox) -> Self {
        let Some(([dx, dy, dz], _)) = valid_box_extents(bounds) else {
            return Self::Box {
                half_extents: [0.5, 0.5, 0.5],
            };
        };
        Self::Box {
            half_extents: [
                (dx * 0.5).max(0.01),
                (dy * 0.5).max(0.01),
                (dz * 0.5).max(0.01),
            ],
        }
    }

    /// Fits a sphere collider radius from the largest extent of `bounds`, falling back safely to `0.5`
    /// on degenerate or non-finite inputs.
    #[must_use]
    pub fn fit_sphere_from_bounds(bounds: &BoundingBox) -> Self {
        let Some(([dx, dy, dz], _)) = valid_box_extents(bounds) else {
            return Self::Sphere { radius: 0.5 };
        };
        Self::Sphere {
            radius: (dx.max(dy).max(dz) * 0.5).max(0.05),
        }
    }

    /// Fits an upright capsule collider (`half_height`, `radius`, `center_y`) onto `bounds`, using
    /// the narrowest horizontal axis (`min(dx, dz)`) so outstretched arms do not inflate radius.
    #[must_use]
    pub fn fit_capsule_from_bounds(bounds: &BoundingBox) -> Self {
        let Some(([dx, dy, dz], center_y)) = valid_box_extents(bounds) else {
            return Self::Capsule {
                half_height: 0.5,
                radius: 0.4,
                center_y: 0.0,
            };
        };
        let total_height = dy.max(0.2);
        let radius = (dx.min(dz) * 0.5).clamp(0.05, (total_height * 0.45).max(0.05));
        let half_height = (total_height * 0.5 - radius).max(0.05);
        Self::Capsule {
            half_height,
            radius,
            center_y,
        }
    }
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

    /// Constructs a character controller centered and sized around `bounds`, using torso depth
    /// (`min(dx, dz)`) for radius and `(min.y + max.y) * 0.5` for `center_y` so feet align cleanly.
    #[must_use]
    pub fn from_bounds(bounds: &BoundingBox) -> Self {
        let Some(([dx, dy, dz], center_y)) = valid_box_extents(bounds) else {
            return Self::default();
        };
        let height = dy.max(0.2);
        let radius = (dx.min(dz) * 0.5).clamp(0.15, (height * 0.45).max(0.15));
        Self {
            height,
            radius,
            center_y,
            ..Self::default()
        }
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
    pub entity: ae_ecs::Entity,
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
    Rubber,
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
                restitution: 0.15,
                density: 7800.0,
            },
            Self::Wood => SurfaceProperties {
                friction: 0.6,
                restitution: 0.15,
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
                friction: 0.2,
                restitution: 0.15,
                density: 2500.0,
            },
            Self::Rubber => SurfaceProperties {
                friction: 0.8,
                restitution: 0.80,
                density: 1100.0,
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

    /// Returns the human-readable display name string for this surface type.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Metal => "Metal",
            Self::Wood => "Wood",
            Self::Stone => "Stone",
            Self::Flesh => "Flesh",
            Self::Dirt => "Dirt",
            Self::Glass => "Glass",
            Self::Rubber => "Rubber",
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_character_and_collider_shapes_from_feet_origin_bounds() {
        let humanoid_bounds = BoundingBox {
            min: [-0.8, 0.0, -0.35],
            max: [0.8, 1.8, 0.35],
        };
        let ctrl = CharacterController::from_bounds(&humanoid_bounds);
        assert!((ctrl.height - 1.8).abs() < 1e-6);
        assert!((ctrl.center_y - 0.9).abs() < 1e-6);
        assert!((ctrl.radius - 0.35).abs() < 1e-6);

        let ColliderShape::Capsule {
            half_height,
            radius,
            center_y,
        } = ColliderShape::fit_capsule_from_bounds(&humanoid_bounds)
        else {
            panic!("Expected Capsule");
        };
        assert!((center_y - 0.9).abs() < 1e-6);
        assert!((radius - 0.35).abs() < 1e-6);
        assert!((half_height - 0.55).abs() < 1e-6);

        assert_eq!(
            ColliderShape::fit_box_from_bounds(&humanoid_bounds),
            ColliderShape::Box {
                half_extents: [0.8, 0.9, 0.35]
            }
        );
    }

    #[test]
    fn test_fit_shapes_reject_nan_and_zero_bounds_safely() {
        let nan_bounds = BoundingBox {
            min: [f32::NAN, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        };
        assert_eq!(
            CharacterController::from_bounds(&nan_bounds),
            CharacterController::default()
        );
        assert_eq!(
            ColliderShape::fit_box_from_bounds(&nan_bounds),
            ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5]
            }
        );

        let zero_bounds = BoundingBox {
            min: [1.0, 1.0, 1.0],
            max: [1.0, 1.0, 1.0],
        };
        assert_eq!(
            CharacterController::from_bounds(&zero_bounds),
            CharacterController::default()
        );
        assert_eq!(
            ColliderShape::fit_sphere_from_bounds(&zero_bounds),
            ColliderShape::Sphere { radius: 0.5 }
        );
    }
}