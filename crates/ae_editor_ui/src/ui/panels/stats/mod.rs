// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Performance Stats & Profiler UI Panel Module.
//!
//! Exposes real-time framerate, CPU frame breakdown, RAM/VRAM resource consumption,
//! and viewport rendering mode toggles (Wireframe / Grid).
//!

pub mod breakdown;
pub mod graph;
pub mod metrics;
pub mod profiler;

pub use profiler::{StatsPanelContext, draw_stats_card};