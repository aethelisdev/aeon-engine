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
use std::collections::HashSet;

/// Interactive dropdown menu identifiers in the Preferences dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesDropdownId {
    /// UI scale selection dropdown in General tab.
    UiScale,
    /// Shadow resolution selection in Graphics tab.
    ShadowResolution,
    /// Shadow cascade count selection in Graphics tab.
    ShadowCascades,
    /// PCF filtering quality selection in Graphics tab.
    ShadowPcf,
    /// Framerate limit selection in Graphics tab.
    FpsLimit,
    /// MSAA sample count selection in Graphics tab.
    MsaaSamples,
    /// Skybox rendering quality selection in Graphics tab.
    SkyQuality,
    /// Snapping mode selection in Editor tab.
    SnapMode,
}

/// Interactive slider identifiers for drag handling in the Preferences dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesSliderId {
    /// Shadow depth bias slider (0.0001..=0.05).
    ShadowBias,
    /// Post-processing Bloom intensity slider (0.0..=3.0).
    BloomIntensity,
    /// Sun pitch angle slider (-PI..=PI).
    SunPitch,
    /// Sun yaw angle slider (-PI..=PI).
    SunYaw,
    /// Atmosphere scattering density slider (0.0..=5.0).
    AtmosphereDensity,
    /// Ozone Chappuis layer absorption slider (0.0..=3.0).
    OzoneDensity,
    /// Sun disk angular size slider (0.1..=5.0).
    SunDiscSize,
    /// Sun corona glow strength slider (0.0..=5.0).
    SunGlowStrength,
    /// Procedural cloud coverage slider (0.0..=1.0).
    CloudCoverage,
    /// Procedural cloud density slider (0.1..=3.0).
    CloudDensity,
    /// Procedural cloud wind speed slider (0.0..=5.0).
    CloudSpeed,
    /// Procedural cloud turbulence evolution slider (0.0..=3.0).
    CloudEvolution,
    /// Procedural cloud base altitude slider (500.0..=5000.0).
    CloudAltitude,
    /// Depth fog distance slider (100.0..=2000.0).
    FogDistance,
    /// Snapping grid step size slider (0.1..=10.0).
    GridSize,
    /// Maximum undo/redo history RAM limit slider (10..=5000).
    UndoHistoryLimit,
    /// Fixed physics update rate frequency in Hz slider (30.0..=240.0).
    PhysicsFrequency,
}

/// Interactive checkbox / toggle identifiers in the Preferences dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesToggleId {
    /// Directional shadow rendering master toggle.
    ShadowsEnabled,
    /// Post-processing Bloom effect toggle.
    BloomEnabled,
    /// Atmospheric depth fog toggle.
    FogEnabled,
    /// Live hot-reload editor updates toggle.
    LiveUpdatesEnabled,
    /// Core engine system module toggle.
    Module(EngineModule),
}

/// Standard discrete physics simulation frequency presets in Hz.
pub const PHYSICS_HZ_PRESETS: [f32; 7] = [30.0, 60.0, 90.0, 120.0, 144.0, 180.0, 240.0];

/// Interactive hit targets collected during the Preferences UI tree construction.
#[derive(Debug, Clone, Default)]
pub struct PreferencesTargets {
    /// Titlebar dragging area rect.
    pub title_bar_rect: Rect,
    /// Main preferences card rect.
    pub card_rect: Rect,
    /// Titlebar close '✖' button rect.
    pub close_button: Rect,
    /// Tab buttons in the left sidebar: `(tab_index, button_rect)`.
    pub tabs: Vec<(u8, Rect)>,
    /// Content area bounding box for mouse wheel scrolling and clipping.
    pub content_rect: Rect,
    /// Total virtual scrollable height of the current tab content.
    pub total_content_height: f32,
    /// Interactive checkbox / toggle target rects: `(toggle_id, rect)`.
    pub toggles: Vec<(PreferencesToggleId, Rect)>,
    /// Interactive continuous slider target tracks: `(slider_id, track_rect, min_val, max_val, current_val)`.
    pub sliders: Vec<(PreferencesSliderId, Rect, f32, f32, f32)>,
    /// Interactive dropdown combobox buttons: `(dropdown_id, button_rect)`.
    pub dropdowns: Vec<(PreferencesDropdownId, Rect)>,
    /// Active open dropdown item list: `(item_index, rect, label)`.
    pub active_dropdown_items: Vec<(usize, Rect, String)>,
    /// Active open dropdown menu popup bounding box.
    pub active_dropdown_popup_rect: Option<Rect>,
    /// Collapsible card / section header hit targets: `(section_id, header_rect)`.
    pub section_toggles: Vec<(&'static str, Rect)>,
    /// Interactive direct numeric input box targets: `(slider_id, box_rect, min_val, max_val, current_val)`.
    pub number_inputs: Vec<(PreferencesSliderId, Rect, f32, f32, f32)>,
}

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
    /// Vertical scroll offset in physical pixels for the content area.
    pub scroll_offset_y: f32,
    /// Currently open dropdown menu identifier, if any.
    pub active_dropdown: Option<PreferencesDropdownId>,
    /// Set of currently collapsed card/section identifiers.
    pub collapsed_sections: &'a HashSet<&'static str>,
    /// Currently active inline number input editing state: `(slider_id, typed_buffer)`.
    pub active_number_input: Option<(PreferencesSliderId, &'a str)>,
    /// Whether the blinking caret cursor should be visible in active inputs.
    pub blink_caret: bool,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Current display/UI zoom factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
    /// Reference to graphics settings.
    pub graphics_settings: &'a GraphicsSettings,
    /// Reference to snapping settings.
    pub snapping_settings: &'a SnapSettings,
    /// Reference to editor configuration.
    pub editor_config: &'a EditorConfig,
    /// Whether live hot-reload editor updates are active.
    pub enable_live_updates: bool,
    /// Set of currently enabled engine core modules.
    pub enabled_modules: &'a HashSet<EngineModule>,
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
    /// Open or close a dropdown ComboBox.
    ToggleDropdown(Option<PreferencesDropdownId>),
    /// Set UI scale factor (e.g. 0.75, 1.0, 1.25).
    SetUiScale(f32),
    /// Toggle a boolean setting or engine module.
    Toggle(PreferencesToggleId),
    /// Set slider continuous numerical value.
    SetSliderValue(PreferencesSliderId, f32),
    /// Select item index in an open dropdown.
    SelectDropdownItem(PreferencesDropdownId, usize),
    /// Content area scrolled via mouse wheel.
    Scroll(f32),
}