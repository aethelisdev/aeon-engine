// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Preferences Subsystem
//!
//! Provides the complete retained GPU SDF Preferences modal dialog implementation,
//! including modular tabs for General, Graphics, Editor, Navigation, Keymap, System,
//! Add-ons, Input, Modules, and Experimental configurations.

pub mod builder;
pub mod tabs;
pub mod types;

pub use builder::{
    PREF_CARD_HEIGHT, PREF_CARD_WIDTH, SIDEBAR_TABS, SIDEBAR_WIDTH, TITLEBAR_HEIGHT,
    build_preferences_dialog,
};
pub use types::{
    PHYSICS_HZ_PRESETS, PreferencesAction, PreferencesDropdownId, PreferencesParams,
    PreferencesSliderId, PreferencesTargets, PreferencesToggleId,
};