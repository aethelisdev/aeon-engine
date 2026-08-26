// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Visual rendering, colors, shapes, bounding volumes, and lighting ECS components.
//!

use serde::{Deserialize, Serialize};

slotmap::new_key_type! {
    /// Generational handle for assets stored in `AssetStorage`.
    pub struct AssetHandle;
}

/// Asset handle reference to a loaded 3D model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelId(pub AssetHandle);

/// Asset handle reference to a loaded 2D texture sprite.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteId(pub AssetHandle);

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

impl Default for Color {
    fn default() -> Self {
        Self::white()
    }
}

/// Point light component with position and RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
        }
    }
}

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

/// Bounding sphere radius for broad-phase frustum culling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingRadius(pub f32);

impl Default for BoundingRadius {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Axis-Aligned Bounding Box for spatial queries and selection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: [-0.5, -0.5, -0.5],
            max: [0.5, 0.5, 0.5],
        }
    }
}