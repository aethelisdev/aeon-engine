// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae-2d-core` - Core Foundation, ECS Components, and Math Types for Aeon 2D Subsystem.
//!
//! Provides fundamental data structures for 2D graphics and game development, including:
//! - [`Camera2D`]: 2D orthographic tracking camera with smoothing, deadzones, and aspect ratio adaptation.
//! - [`SpriteRenderer`]: ECS component encapsulating texture references, UV mapping, sorting layers, and tinting.
//! - [`SortMode`] and [`SpriteSortKey`]: Bit-packed 64-bit sorting keys for zero-allocation sprite ordering.
//! - [`ActiveDimensionMode`]: Architectural flag governing mutual exclusion between 2D and 3D pipelines.
//!
//! This crate is intentionally decoupled from hardware graphics APIs (e.g., `wgpu`), ensuring that headless
//! game servers and simulation workers can utilize 2D components without GPU drivers or windowing baggage.
//!

pub mod camera;
pub mod components;
pub mod mode;

#[cfg(test)]
mod tests;

pub use camera::Camera2D;
pub use components::{SortMode, SpriteRenderer, SpriteSortKey};
pub use mode::ActiveDimensionMode;