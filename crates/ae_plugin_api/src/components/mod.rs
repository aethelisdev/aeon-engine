// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standard ECS component definitions shared across the engine, plugins, and gameplay scripts.
//!

pub mod gameplay;
pub mod physics;
pub mod rendering;
pub mod transform;

// Re-export all component types to preserve a flat, unified public API surface
pub use gameplay::*;
pub use physics::*;
pub use rendering::*;
pub use transform::*;