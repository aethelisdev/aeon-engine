// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Preferences Types & Hit Targets
//!
//! Defines parameter structures, interactive hit target descriptors, and actions
//! for the hardware-accelerated Preferences modal dialog.

use ae_core::modules::EngineModule;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::widgets::scroll_area::ScrollAreaStyle;
use std::collections::HashSet;

/// Preferences subsystem 64-bit semantic tag domain.
pub const PREFERENCES_TAG_DOMAIN: u64 = 0x0080_0000_0000_0000;
/// Bitmask for filtering preferences semantic tags.
pub const PREFERENCES_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

/// Semantic tag for the entire modal card background (used to absorb clicks).
pub const PREF_TAG_CARD: u64 = PREFERENCES_TAG_DOMAIN | 0x0001;
/// Semantic tag for the draggable titlebar strip.
pub const PREF_TAG_TITLEBAR: u64 = PREFERENCES_TAG_DOMAIN | 0x0002;
/// Semantic tag for the titlebar '✕' close button.
pub const PREF_TAG_CLOSE: u64 = PREFERENCES_TAG_DOMAIN | 0x0003;
/// Semantic tag for the vertical scrollbar track.
pub const PREF_TAG_SCROLLBAR_TRACK: u64 = PREFERENCES_TAG_DOMAIN | 0x0004;
/// Semantic tag for the vertical scrollbar thumb.
pub const PREF_TAG_SCROLLBAR_THUMB: u64 = PREFERENCES_TAG_DOMAIN | 0x0005;
/// Semantic tag for the scrollable content view container.
pub const PREF_TAG_CONTENT_VIEW: u64 = PREFERENCES_TAG_DOMAIN | 0x0006;
/// Semantic tag for the inner preferences scroll column container.
pub const PREF_TAG_SCROLL_CONTAINER: u64 = PREFERENCES_TAG_DOMAIN | 0x0007;

/// Base semantic tag for sidebar navigation tabs (0..=9).
pub const PREF_TAG_TAB_BASE: u64 = PREFERENCES_TAG_DOMAIN | 0x0010;
/// Base semantic tag for collapsible card sections.
pub const PREF_TAG_SECTION_BASE: u64 = PREFERENCES_TAG_DOMAIN | 0x0100;
/// Base semantic tag for toggle switches / checkboxes.
pub const PREF_TAG_TOGGLE_BASE: u64 = PREFERENCES_TAG_DOMAIN | 0x0400;

/// Static registry of collapsible section identifiers.
pub const PREF_SECTIONS: [&str; 19] = [
    "general_scale",
    "general_viewport",
    "graphics_shadows",
    "graphics_perf",
    "graphics_aa",
    "graphics_pp",
    "graphics_env",
    "graphics_atmosphere",
    "graphics_clouds",
    "graphics_fog",
    "editor_snapping",
    "editor_physics",
    "editor_runtime",
    "editor_history",
    "modules_list",
    "nav_speeds",
    "nav_orbit",
    "input_mouse",
    "input_mapping",
];

/// Returns true if the given tag belongs to the Preferences semantic domain.
#[inline]
pub fn is_preferences_tag(tag: u64) -> bool {
    (tag & PREFERENCES_TAG_MASK) == PREFERENCES_TAG_DOMAIN
}

/// Encodes a sidebar tab index (0..=9) into a semantic tag.
#[inline]
pub fn encode_tab_tag(tab_idx: u8) -> u64 {
    PREF_TAG_TAB_BASE | (tab_idx as u64)
}

/// Decodes a sidebar tab index from a semantic tag if valid.
#[inline]
pub fn parse_tab_tag(tag: u64) -> Option<u8> {
    if (tag & 0xFFFF_FFFF_FFFF_FFF0) == PREF_TAG_TAB_BASE {
        let idx = (tag & 0xF) as u8;
        if idx <= 9 {
            return Some(idx);
        }
    }
    None
}

/// Encodes a collapsible section static identifier into a semantic tag.
#[inline]
pub fn encode_section_tag(sec_id: &'static str) -> u64 {
    let idx = PREF_SECTIONS.iter().position(|&s| s == sec_id).unwrap_or(0);
    PREF_TAG_SECTION_BASE | (idx as u64)
}

/// Decodes a collapsible section static identifier from a semantic tag if valid.
#[inline]
pub fn parse_section_tag(tag: u64) -> Option<&'static str> {
    if (tag & 0xFFFF_FFFF_FFFF_FF00) == PREF_TAG_SECTION_BASE {
        let idx = (tag & 0xFF) as usize;
        return PREF_SECTIONS.get(idx).copied();
    }
    None
}

/// Interactive checkbox / toggle identifiers in the Preferences dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesToggleId {
    /// Live hot-reload editor updates toggle.
    LiveUpdatesEnabled,
    /// Core engine system module toggle.
    Module(EngineModule),
}

/// Encodes a toggle identifier into a semantic tag.
#[inline]
pub fn encode_toggle_tag(toggle: PreferencesToggleId) -> u64 {
    let id = match toggle {
        PreferencesToggleId::LiveUpdatesEnabled => 0,
        PreferencesToggleId::Module(EngineModule::Physics) => 10,
        PreferencesToggleId::Module(EngineModule::Audio) => 11,
        PreferencesToggleId::Module(EngineModule::Render) => 12,
        PreferencesToggleId::Module(EngineModule::Render2D) => 13,
    };
    PREF_TAG_TOGGLE_BASE | id
}

/// Decodes a toggle identifier from a semantic tag if valid.
#[inline]
pub fn parse_toggle_tag(tag: u64) -> Option<PreferencesToggleId> {
    if (tag & 0xFFFF_FFFF_FFFF_FF00) != PREF_TAG_TOGGLE_BASE {
        return None;
    }
    match tag & 0xFF {
        0 => Some(PreferencesToggleId::LiveUpdatesEnabled),
        10 => Some(PreferencesToggleId::Module(EngineModule::Physics)),
        11 => Some(PreferencesToggleId::Module(EngineModule::Audio)),
        12 => Some(PreferencesToggleId::Module(EngineModule::Render)),
        13 => Some(PreferencesToggleId::Module(EngineModule::Render2D)),
        _ => None,
    }
}

/// Strongly typed semantic hit target within the Preferences configuration dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum PreferencesTagTarget {
    /// Dialog close '✖' button.
    Close,
    /// Floating dialog titlebar header.
    Titlebar,
    /// Primary content view scrollbar draggable thumb.
    ScrollbarThumb,
    /// Primary content view scrollbar track.
    ScrollbarTrack,
    /// Sidebar category tab item by index (0..=9).
    Tab(u8),
    /// Collapsible section card header by static name.
    Section(&'static str),
    /// Checkbox or engine module toggle by identifier.
    Toggle(PreferencesToggleId),
}

/// Resolves a 64-bit semantic tag into a strongly typed [`PreferencesTagTarget`].
#[inline]
pub fn resolve_preferences_tag(tag: u64) -> Option<PreferencesTagTarget> {
    if !is_preferences_tag(tag) {
        return None;
    }
    match tag {
        PREF_TAG_CLOSE => Some(PreferencesTagTarget::Close),
        PREF_TAG_TITLEBAR => Some(PreferencesTagTarget::Titlebar),
        PREF_TAG_SCROLLBAR_THUMB => Some(PreferencesTagTarget::ScrollbarThumb),
        PREF_TAG_SCROLLBAR_TRACK => Some(PreferencesTagTarget::ScrollbarTrack),
        _ => {
            if let Some(tab_idx) = parse_tab_tag(tag) {
                Some(PreferencesTagTarget::Tab(tab_idx))
            } else if let Some(sec_id) = parse_section_tag(tag) {
                Some(PreferencesTagTarget::Section(sec_id))
            } else {
                parse_toggle_tag(tag).map(PreferencesTagTarget::Toggle)
            }
        }
    }
}

/// Standard discrete physics simulation frequency presets in Hz.
pub const PHYSICS_HZ_PRESETS: [f32; 7] = [30.0, 60.0, 90.0, 120.0, 144.0, 180.0, 240.0];

/// Parameters passed to construct the Preferences dialog UI tree.
pub struct PreferencesParams<'a> {
    /// Viewport width in physical pixels.
    pub screen_width: f32,
    /// Viewport height in physical pixels.
    pub screen_height: f32,
    /// Custom floating window position (left, top), if any.
    pub window_pos: Option<Point>,
    /// Currently active sidebar tab index (0..=9).
    pub active_tab: u8,
    /// Set of currently collapsed card/section identifiers.
    pub collapsed_sections: &'a HashSet<&'static str>,
    /// Currently active inline numeric text editing state: `(target_tag, current_string_buffer, is_all_selected)`.
    pub active_number_input: Option<(u64, &'a str, bool)>,
    /// Whether the blinking caret cursor should be visible in active inputs.
    pub blink_caret: bool,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Current display/UI zoom factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
    /// Mutable reference to graphics settings for declarative two-way data binding.
    pub graphics_settings: &'a mut GraphicsSettings,
    /// Mutable reference to snapping settings for declarative two-way data binding.
    pub snapping_settings: &'a mut SnapSettings,
    /// Mutable reference to editor configuration for declarative two-way data binding.
    pub editor_config: &'a mut EditorConfig,
    /// Whether live hot-reload editor updates are active.
    pub enable_live_updates: bool,
    /// Set of currently enabled engine core modules.
    pub enabled_modules: &'a HashSet<EngineModule>,
    /// Frame interaction events mapped by 64-bit semantic tags for declarative two-way data binding.
    pub events: &'a [(u64, InteractionEvent)],
}

/// Action resulting from user interaction within the Preferences dialog.
#[derive(Debug, Clone)]
pub enum PreferencesAction {
    /// Close the Preferences dialog.
    Close,
    /// Switch active sidebar tab.
    SelectTab(u8),
    /// Toggle expansion / folding of a card section.
    ToggleSection(&'static str),
    /// Set UI scale factor (e.g. 0.75, 1.0, 1.25).
    SetUiScale(f32),
    /// Toggle a boolean setting or engine module.
    Toggle(PreferencesToggleId),
    /// Reposition the floating modal dialog window during titlebar dragging.
    Move(Point),
}

/// Standard scrollbar styling for the Preferences modal dialog.
#[must_use]
pub const fn preferences_scroll_style() -> ScrollAreaStyle {
    ScrollAreaStyle {
        thickness: 6.0,
        inset: 3.0,
        ..ScrollAreaStyle::dark_default()
    }
}

/// Persistent interactive state for the Preferences modal dialog overlay.
#[derive(Debug, Default, Clone)]
pub struct PreferencesDialogState {
    /// Common panel interaction state (scroll_y, search, actions).
    pub interactions: crate::ui::iris_bridge::types::PanelInteractionState<(), PreferencesAction>,
    /// Bounding rectangle of the preferences floating card for click absorption and bounds clamping.
    pub card_rect: Option<Rect>,
    /// Virtual maximum scrollable distance (total_content_height - content_height).
    pub max_scroll_y: f32,
    /// Bounding rectangle of the scrollable content view.
    pub content_rect: Option<Rect>,
    /// Custom floating position coordinates for the Preferences panel.
    pub pos: Option<Point>,
    /// Active drag offset from window top-left when dragging the title bar.
    pub drag_offset: Option<Point>,
    /// Currently selected tab index in the Preferences dialog (0..=9).
    pub tab: u8,
    /// Previously rendered tab index in Preferences to trigger reactive invalidation on tab switches.
    pub last_tab: u8,
    /// Currently active continuous slider or float drag semantic tag.
    pub active_drag_tag: Option<u64>,
    /// Autonomous scroll state managing scroll_y and pointer capture thumb dragging.
    pub scroll_state: irisui::prelude::ScrollAreaState,
    /// Currently active inline numeric text editing state: `(target_tag, current_string_buffer, is_all_selected)`.
    pub active_number_input: Option<(u64, String, bool)>,
    /// Whether the text caret is currently visible during inline editing.
    pub blink_caret: bool,
    /// Persistent frame interaction event queue for declarative two-way bound property widgets.
    ///
    /// Preserved across frames and cleared with [`Vec::clear`] to guarantee zero heap allocations (Rule 9.4).
    pub pending_interaction_events: Vec<(u64, InteractionEvent)>,
    /// Set of currently collapsed card/section identifiers in the Preferences dialog.
    pub collapsed_sections: HashSet<&'static str>,
}

impl PreferencesDialogState {
    /// Resets scroll position and ongoing drag gesture to initial zero offset.
    #[inline]
    pub fn reset_scroll(&mut self) {
        self.scroll_state.reset();
    }
}

impl std::ops::Deref for PreferencesDialogState {
    type Target = crate::ui::iris_bridge::types::PanelInteractionState<(), PreferencesAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for PreferencesDialogState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_preferences_tag_invariants() {
        assert_eq!(
            resolve_preferences_tag(PREF_TAG_CLOSE),
            Some(PreferencesTagTarget::Close)
        );
        assert_eq!(
            resolve_preferences_tag(PREF_TAG_TITLEBAR),
            Some(PreferencesTagTarget::Titlebar)
        );
        assert_eq!(
            resolve_preferences_tag(PREF_TAG_SCROLLBAR_THUMB),
            Some(PreferencesTagTarget::ScrollbarThumb)
        );
        assert_eq!(
            resolve_preferences_tag(PREF_TAG_SCROLLBAR_TRACK),
            Some(PreferencesTagTarget::ScrollbarTrack)
        );
        assert_eq!(
            resolve_preferences_tag(encode_tab_tag(2)),
            Some(PreferencesTagTarget::Tab(2))
        );
        assert_eq!(
            resolve_preferences_tag(encode_section_tag("general_scale")),
            Some(PreferencesTagTarget::Section("general_scale"))
        );
        assert_eq!(
            resolve_preferences_tag(encode_toggle_tag(PreferencesToggleId::LiveUpdatesEnabled)),
            Some(PreferencesTagTarget::Toggle(
                PreferencesToggleId::LiveUpdatesEnabled
            ))
        );
        assert_eq!(resolve_preferences_tag(0), None);
        assert_eq!(resolve_preferences_tag(0xFFFF_FFFF), None);
    }
}