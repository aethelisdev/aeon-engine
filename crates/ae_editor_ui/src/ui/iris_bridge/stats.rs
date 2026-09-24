// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Profiler Iris UI Module
//!
//! Orchestrates the declarative frame pacing oscillograph, CPU/GPU pass breakdowns,
//! draw call distribution, granular VRAM memory cards, and viewport overlay toggles.
//!

pub mod cpu_breakdown;
pub mod gpu_breakdown;
pub mod graph;
pub mod metrics;
pub mod overlays;
pub mod panel;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use graph::append_oscilloscope_quads;
pub use panel::build_stats_panel;
pub use types::{
    STATS_TAG_CANVAS, STATS_TAG_PANEL_ROOT, STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME,
    StatsPanelAction, StatsPanelParams, StatsPanelState, is_stats_tag,
};