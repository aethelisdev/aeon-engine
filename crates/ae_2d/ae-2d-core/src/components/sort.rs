// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use serde::{Deserialize, Serialize};

/// Mode determining how depth and rendering order are resolved in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortMode {
    /// Sorts purely based on `sorting_layer` followed by `order_in_layer`.
    #[default]
    LayerAndOrder,

    /// Sorts based on `sorting_layer`, then world-space Y position (lower Y drawn in front, standard for top-down games),
    /// and lastly `order_in_layer`.
    YSort,
}

/// Compact 64-bit sort key for zero-allocation, high-performance sprite draw sorting.
/// Encodes layer, Y-depth or order, and texture index to minimize pipeline state swaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpriteSortKey(pub u64);

impl SpriteSortKey {
    /// Constructs a 64-bit sort key from layer, order, and texture asset index.
    /// Bit layout:
    /// - `[63..48]` (16 bits): Signed sorting layer mapped to unsigned (`layer as i16 as u16 ^ 0x8000`).
    /// - `[47..32]` (16 bits): Signed order in layer mapped to unsigned.
    /// - `[31..0]`  (32 bits): Texture handle index or raw asset identifier.
    pub fn from_layer_order_texture(layer: i32, order: i32, texture_index: u32) -> Self {
        let unsigned_layer =
            ((layer.clamp(i16::MIN as i32, i16::MAX as i32) as i16) as u16) ^ 0x8000;
        let unsigned_order =
            ((order.clamp(i16::MIN as i32, i16::MAX as i32) as i16) as u16) ^ 0x8000;

        let key = ((unsigned_layer as u64) << 48)
            | ((unsigned_order as u64) << 32)
            | (texture_index as u64);
        Self(key)
    }

    /// Constructs a sort key incorporating continuous Y-position for isometric/top-down rendering.
    /// Inverts Y so that entities positioned further down the screen (lower Y) receive a higher sort key,
    /// drawing them in front of entities further up.
    pub fn from_y_sort(layer: i32, y_pos: f32, texture_index: u32) -> Self {
        let unsigned_layer =
            ((layer.clamp(i16::MIN as i32, i16::MAX as i32) as i16) as u16) ^ 0x8000;
        // Invert Y coordinate so smaller Y has larger key (renders on top)
        let inverted_y = (-y_pos).clamp(-32768.0, 32767.0) as i16;
        let unsigned_y = (inverted_y as u16) ^ 0x8000;

        let key =
            ((unsigned_layer as u64) << 48) | ((unsigned_y as u64) << 32) | (texture_index as u64);
        Self(key)
    }
}