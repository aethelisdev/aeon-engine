// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Profiler Iris UI Module
//!
//! Orchestrates the retained-mode frame pacing oscillograph, CPU/GPU pass breakdowns,
//! draw call distribution, granular VRAM memory cards, and viewport overlay toggles.
//!

pub mod cpu_breakdown;
pub mod gpu_breakdown;
pub mod graph;
pub mod metrics;
pub mod overlays;
pub mod panel;
pub mod sync;
#[cfg(test)]
mod tests;
pub mod types;

pub use graph::append_oscilloscope_quads;
pub use panel::{build_stats_panel, update_stats_panel_text_values, update_stats_panel_values};
pub use sync::sync_stats_panel;
pub use types::{
    StatsPanelAction, StatsPanelNodes, StatsPanelParams, StatsPanelRetainedState, StatsPanelTargets,
};