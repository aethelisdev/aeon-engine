// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Preferences Subsystem
//!
//! Provides the complete declarative GPU SDF Preferences modal dialog implementation,
//! including modular tabs for General, Graphics, Editor, Navigation, Keymap, System,
//! Add-ons, Input, Modules, and Experimental configurations.

pub mod builder;
pub mod components;
pub mod tabs;
pub mod types;

#[cfg(test)]
mod tabs_tests;
#[cfg(test)]
mod tests;

pub use builder::{
    PREF_CARD_HEIGHT, PREF_CARD_WIDTH, SIDEBAR_TABS, SIDEBAR_WIDTH, TITLEBAR_HEIGHT,
    build_preferences_dialog,
};
pub use components::{pref_dropdown_row, pref_heading, pref_section_card, pref_toggle_row};
pub use types::{
    PHYSICS_HZ_PRESETS, PREF_SECTIONS, PREF_TAG_CARD, PREF_TAG_CLOSE, PREF_TAG_CONTENT_VIEW,
    PREF_TAG_DROPDOWN_BASE, PREF_TAG_DROPDOWN_ITEM_BASE, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PREF_TAG_SECTION_BASE, PREF_TAG_TAB_BASE, PREF_TAG_TITLEBAR,
    PREF_TAG_TOGGLE_BASE, PREFERENCES_TAG_DOMAIN, PreferencesAction, PreferencesDialogState,
    PreferencesDropdownId, PreferencesParams, PreferencesToggleId, encode_dropdown_item_tag,
    encode_dropdown_tag, encode_section_tag, encode_tab_tag, encode_toggle_tag, is_preferences_tag,
    parse_dropdown_item_tag, parse_dropdown_tag, parse_section_tag, parse_tab_tag,
    parse_toggle_tag,
};