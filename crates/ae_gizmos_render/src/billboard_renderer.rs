// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! WGPU billboard overlay rendering subsystem.

pub mod pipeline;
pub mod system;
#[cfg(test)]
pub mod tests;

pub use pipeline::{BillboardPipeline, billboard_vertex_layout};
pub use system::{BillboardPrepareParams, BillboardSystem};