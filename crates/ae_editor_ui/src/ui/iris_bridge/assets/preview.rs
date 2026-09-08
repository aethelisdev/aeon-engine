// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Interactive Quick Asset Preview Modal Subsystem
//!
//! Renders a floating, hardware-accelerated GPU SDF modal window providing rich,
//! interactive preview inspections: 3D mesh orbit wireframe viewports with mouse drag
//! rotation and zoom, texture specifications, WGSL shader diagnostics, scene summaries,
//! and direct spawn/load operations.
//!

pub(crate) mod details;
pub(crate) mod modal;
pub(crate) mod model;

pub use modal::{PREVIEW_MODAL_HEIGHT, PREVIEW_MODAL_WIDTH, build_asset_preview_modal};