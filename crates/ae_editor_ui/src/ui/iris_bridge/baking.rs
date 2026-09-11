// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Two-pass hardware geometry baking and GPU batching engine for Iris UI.
//!
//! Organizes UI primitives into decoupled layers to enforce exactly 2 draw calls
//! for the entire editor interface: Layer 0 (SDF quads) and Layer 1 (2D Texture Array quads).

pub mod engine;
pub mod geometry;
#[cfg(test)]
pub mod tests;

pub use engine::*;
pub use geometry::*;