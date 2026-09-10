// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use ae_texture::AssetHandle;
use serde::{Deserialize, Serialize};

/// 2D Sprite component for rendering textured or tinted planar quads.
/// Attaches to an entity possessing `Position`, `Rotation`, and `Scale` in the ECS world.
/// Supports texture atlasing via UV sub-rectangles, custom color tinting, horizontal/vertical
/// flipping, custom pivot offsets, and explicit two-tier sorting order (`sorting_layer` and `order_in_layer`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteRenderer {
    /// Optional asset handle referencing a texture in `ae_texture::TextureStorage`.
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
    /// Values are clamped and checked to ensure `u_min <= u_max` and `v_min <= v_max`.
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
    /// Returns `[u_min, v_min, u_max, v_max]` with swapped min/max coordinates when flipped.
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