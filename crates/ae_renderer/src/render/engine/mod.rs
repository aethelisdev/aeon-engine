// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Render Engine Core Sub-module — Manages RenderState, frame rendering pipelines, and vertex skinning updates.
//!

pub mod frame;
pub mod skinning;
pub mod state;

pub use state::*;