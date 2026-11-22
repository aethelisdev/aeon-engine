// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Preferences Subsystem
//!
//! Provides the declarative GPU SDF Preferences panel implementation across modular
//! configuration tabs (`General`, `Graphics`, `Editor`, `Navigation`, `Keymap`,
//! `System`, `Add-ons`, `Input`, `Modules`, and `Experimental`).

pub mod panel;
pub mod tabs;
pub mod types;

#[cfg(test)]
mod tabs_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_scroll;

pub use panel::{SIDEBAR_TABS, SIDEBAR_WIDTH, build_preferences_view};
pub use types::{
    PHYSICS_HZ_PRESETS, PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLLBAR_THUMB, PREF_TAG_SCROLLBAR_TRACK,
    PreferencesDialogState, preferences_scroll_style,
};