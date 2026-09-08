// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Subsystem (`iris_bridge::timeline`)
//!
//! Provides the 100% GPU SDF hardware-accelerated Animation Timeline Studio panel
//! for Aeon Engine, utilizing decoupled Retained UI trees, responsive transport controls,
//! adaptive time rulers, and interactive scrubbing.
//!

pub mod events;
pub mod panel;
pub mod ruler;
#[cfg(test)]
mod tests;
pub mod transport;
pub mod types;

pub use events::{handle_timeline_click, handle_timeline_drag};
pub use panel::build_timeline_panel;
pub use types::{TimelineAction, TimelinePanelParams, TimelinePanelTargets};