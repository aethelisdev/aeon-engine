// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport 3D Billboard Module Root
//!
//! Subsystem connection point and public exports for 3D camera-facing
//! viewport billboard badge rendering.

pub mod pipeline;
pub mod system;
#[cfg(test)]
mod tests;
pub mod types;

pub use pipeline::BillboardPipeline;
pub use system::{BillboardPrepareParams, BillboardSystem};
pub use types::{BillboardIconType, BillboardUniform, BillboardVertex};