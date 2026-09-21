// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Iris docking chrome construction and computed panel geometry extraction.
//!
//! Renders native GPU SDF split containers, tab bars with compact snug widths,
//! active cyan indicator underlines, divider lines, 5-way compass navigation drop zones,
//! proportional tab shrinking, and overflow chevrons with interactive dropdown menus.
//!

pub mod builder;
pub mod overflow;
pub mod overlays;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use builder::build_native_dock;
pub use overflow::build_native_dock_overflow_menu;
pub use overlays::build_native_dock_drag_overlays;
pub use types::*;