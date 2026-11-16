// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dual-Layout Synchronization Pipeline
//!
//! Autonomous coordination pipeline eliminating visual coordinate divergence and jitter
//! between coarse initial shell passes and granular Taffy CSS flexbox boundary solvers.

pub mod dual_layout;
pub mod engines;
pub mod leaf_pipeline;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_single_leaf;
#[cfg(test)]
mod tests_stress;

pub use dual_layout::{DualLayoutEliminator, LayoutSyncMetrics};
pub use engines::SubtreeEngines;
pub use leaf_pipeline::{synchronize_dock_frame, synchronize_dock_leaves};