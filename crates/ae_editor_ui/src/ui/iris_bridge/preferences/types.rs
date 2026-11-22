// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Preferences Types & Semantic Tags
//!
//! Defines strongly typed parameters, semantic viewport tags, and state structures
//! for the dockable hardware-accelerated Preferences panel.

use irisui::prelude::*;
use irisui::widgets::scroll_area::ScrollAreaStyle;

/// Semantic tag for the scrollable content view container in the Preferences panel.
pub const PREF_TAG_CONTENT_VIEW: u64 = UiTag::from_str("pref.content_view").raw();
/// Semantic tag for the vertical scrollbar track in the Preferences panel.
pub const PREF_TAG_SCROLLBAR_TRACK: u64 = UiTag::from_str("pref.scrollbar_track").raw();
/// Semantic tag for the vertical scrollbar thumb in the Preferences panel.
pub const PREF_TAG_SCROLLBAR_THUMB: u64 = UiTag::from_str("pref.scrollbar_thumb").raw();

/// Standard discrete physics simulation frequency presets in Hz.
pub const PHYSICS_HZ_PRESETS: [f32; 7] = [30.0, 60.0, 90.0, 120.0, 144.0, 180.0, 240.0];

/// Standard scrollbar styling for the Preferences panel.
#[must_use]
pub const fn preferences_scroll_style() -> ScrollAreaStyle {
    ScrollAreaStyle {
        thickness: 6.0,
        inset: 3.0,
        ..ScrollAreaStyle::dark_default()
    }
}

/// Persistent interactive state for the Preferences configuration panel.
///
/// Collapsible card open/closed states are owned autonomously by [`irisui::prelude::UiTree`].
#[derive(Debug, Default, Clone)]
pub struct PreferencesDialogState {
    /// Currently selected tab index in the Preferences panel (`0..=9`).
    pub tab: u8,
    /// Currently active workspace theme preset (Obsidian Deep or Cold Slate).
    pub theme_preset: crate::ui::iris_bridge::theme::ThemePreset,
}