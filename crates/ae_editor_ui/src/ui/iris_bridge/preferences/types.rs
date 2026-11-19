// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Preferences Types & Semantic Tags
//!
//! Defines strongly typed parameters, semantic viewport tags, and state structures
//! for the dockable hardware-accelerated Preferences panel.

use ae_core::modules::EngineModule;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::widgets::scroll_area::ScrollAreaStyle;
use std::collections::HashSet;

/// Semantic tag for the scrollable content view container in the Preferences panel.
pub const PREF_TAG_CONTENT_VIEW: u64 = UiTag::from_str("pref.content_view").raw();
/// Semantic tag for the inner preferences scroll column container.
pub const PREF_TAG_SCROLL_CONTAINER: u64 = UiTag::from_str("pref.scroll_container").raw();
/// Semantic tag for the vertical scrollbar track in the Preferences panel.
pub const PREF_TAG_SCROLLBAR_TRACK: u64 = UiTag::from_str("pref.scrollbar_track").raw();
/// Semantic tag for the vertical scrollbar thumb in the Preferences panel.
pub const PREF_TAG_SCROLLBAR_THUMB: u64 = UiTag::from_str("pref.scrollbar_thumb").raw();

/// Standard discrete physics simulation frequency presets in Hz.
pub const PHYSICS_HZ_PRESETS: [f32; 7] = [30.0, 60.0, 90.0, 120.0, 144.0, 180.0, 240.0];

/// Parameters passed to construct the Preferences panel UI subtree.
pub struct PreferencesParams<'a> {
    /// Mutable reference to the currently active sidebar tab index (`0..=9`).
    pub active_tab: &'a mut u8,
    /// Current display/UI zoom factor (`0.75..=1.50`).
    pub zoom_factor: f32,
    /// Mutable reference to graphics settings for declarative two-way data binding.
    pub graphics_settings: &'a mut GraphicsSettings,
    /// Mutable reference to snapping settings for declarative two-way data binding.
    pub snapping_settings: &'a mut SnapSettings,
    /// Mutable reference to editor configuration for declarative two-way data binding.
    pub editor_config: &'a mut EditorConfig,
    /// Mutable reference to live hot-reload editor updates active state.
    pub enable_live_updates: &'a mut bool,
    /// Set of currently enabled engine core modules.
    pub enabled_modules: &'a HashSet<EngineModule>,
    /// Mutable reference to active workspace theme preset.
    pub current_theme: &'a mut crate::ui::iris_bridge::theme::ThemePreset,
}

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