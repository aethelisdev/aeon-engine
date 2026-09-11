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

/// 2D Sprite component for rendering textured or tinted planar quads.
/// Attaches to an entity possessing `Position`, `Rotation`, and `Scale` in the ECS world.
/// Supports texture atlasing via UV sub-rectangles, custom color tinting, horizontal/vertical
/// flipping, custom pivot offsets, and explicit two-tier sorting order (`sorting_layer` and `order_in_layer`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteRenderer {
    /// Optional asset handle referencing a texture in asset storage.
    /// When `None`, the sprite renders as an untextured flat-colored quad using `tint`.
    pub texture: Option<AssetHandle>,

    /// UV coordinates rectangle defined as `[u_min, v_min, u_max, v_max]`.
    /// Default is `[0.0, 0.0, 1.0, 1.0]` (full texture bounds).
    pub uv_rect: [f32; 4],

    /// Linear RGBA color tint applied multiplicatively to the texture or flat quad.
    /// Default is opaque white `[1.0, 1.0, 1.0, 1.0]`.
    pub tint: [f32; 4],

    /// Horizontally flips the rendered sprite across its horizontal pivot axis.
    pub flip_x: bool,

    /// Vertically flips the rendered sprite across its vertical pivot axis.
    pub flip_y: bool,

    /// High-level sorting layer identifier (e.g., Background = -100, Default = 0, Foreground = 100).
    /// Sprites in lower layers are drawn prior to sprites in higher layers.
    pub sorting_layer: i32,

    /// Fine-grained draw order within the same `sorting_layer`.
    /// Sprites with lower order values are rendered before higher order values.
    pub order_in_layer: i32,

    /// Normalized origin pivot offset `[x, y]` where `[0.5, 0.5]` corresponds to the quad center,
    /// and `[0.0, 0.0]` corresponds to the bottom-left corner.
    pub pivot: [f32; 2],
}

impl Default for SpriteRenderer {
    fn default() -> Self {
        Self {
            texture: None,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            sorting_layer: 0,
            order_in_layer: 0,
            pivot: [0.5, 0.5],
        }
    }
}

impl SpriteRenderer {
    /// Creates a new sprite renderer with default visual settings and an optional texture handle.
    pub fn new(texture: Option<AssetHandle>) -> Self {
        Self {
            texture,
            ..Default::default()
        }
    }

    /// Creates a flat colored sprite without a texture.
    pub fn with_color(tint: [f32; 4]) -> Self {
        Self {
            texture: None,
            tint,
            ..Default::default()
        }
    }

    /// Sets the texture UV sub-rectangle for sprite sheet / texture atlas extraction.
    pub fn with_uv_rect(mut self, u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        self.uv_rect = [u_min, v_min, u_max, v_max];
        self
    }

    /// Sets the sorting layer and fine order in layer.
    pub fn with_sorting(mut self, sorting_layer: i32, order_in_layer: i32) -> Self {
        self.sorting_layer = sorting_layer;
        self.order_in_layer = order_in_layer;
        self
    }

    /// Computes the effective UV coordinates taking `flip_x` and `flip_y` into account.
    pub fn effective_uvs(&self) -> [f32; 4] {
        let (u0, u1) = if self.flip_x {
            (self.uv_rect[2], self.uv_rect[0])
        } else {
            (self.uv_rect[0], self.uv_rect[2])
        };

        let (v0, v1) = if self.flip_y {
            (self.uv_rect[3], self.uv_rect[1])
        } else {
            (self.uv_rect[1], self.uv_rect[3])
        };

        [u0, v0, u1, v1]
    }
}