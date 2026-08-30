// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Types & Parameters
//!
//! Option constants, descriptors, and parameter structs for the Graphics settings tab.

use super::super::super::types::{PreferencesDropdownId, PreferencesSliderId, PreferencesToggleId};
use ae_renderer::graphics_settings::{FpsLimit, PcfQuality, ShadowResolution, SkyQuality};
use irisui::prelude::*;

/// Pre-configured shadow resolution options.
pub const SHADOW_RES_OPTIONS: [ShadowResolution; 4] = [
    ShadowResolution::Low,
    ShadowResolution::Medium,
    ShadowResolution::High,
    ShadowResolution::Ultra,
];

/// Pre-configured shadow cascade count options.
pub const CASCADE_OPTIONS: [(u32, &str); 2] = [
    (3, "3 Cascades (Default)"),
    (4, "4 Cascades (High Fidelity)"),
];

/// Pre-configured PCF shadow filter options.
pub const PCF_OPTIONS: [PcfQuality; 3] = [PcfQuality::Off, PcfQuality::Soft, PcfQuality::UltraSoft];

/// Pre-configured framerate limiter options.
pub const FPS_OPTIONS: [FpsLimit; 3] = [FpsLimit::Limit60, FpsLimit::Limit120, FpsLimit::Uncapped];

/// Pre-configured hardware MSAA sample options.
pub const MSAA_OPTIONS: [(u32, &str); 3] = [(1, "Off (1x)"), (2, "2x"), (4, "4x (Default)")];

/// Pre-configured atmospheric sky quality options.
pub const SKY_OPTIONS: [SkyQuality; 3] = [SkyQuality::Low, SkyQuality::Medium, SkyQuality::High];

/// Parameters for rendering an interactive checkbox toggle.
pub struct CheckboxParams<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub is_checked: bool,
    pub toggle_id: PreferencesToggleId,
    pub cursor_pos: Point,
}

/// Parameters for rendering a continuous numerical slider row.
pub struct SliderRowParams<'a> {
    pub base_x: f32,
    pub y: f32,
    pub width: f32,
    pub label: &'a str,
    pub val_text: &'a str,
    pub current_val: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub slider_id: PreferencesSliderId,
    pub cursor_pos: Point,
    pub is_editing: bool,
    pub editing_buffer: &'a str,
    pub blink_caret: bool,
}

/// Parameters for rendering an interactive dropdown row.
pub struct DropdownRowParams<'a> {
    pub base_x: f32,
    pub y: f32,
    pub width: f32,
    pub label: &'a str,
    pub selected_text: &'a str,
    pub dropdown_id: PreferencesDropdownId,
    pub cursor_pos: Point,
    pub is_open: bool,
}

/// Layout context passed down to individual preference card builders.
#[derive(Debug, Clone, Copy)]
pub struct CardLayoutContext {
    /// Left horizontal coordinate for cards.
    pub base_x: f32,
    /// Vertical top offset inside the tab scroll area.
    pub y_offset: f32,
    /// Usable content width.
    pub content_w: f32,
    /// Content area top position in screen coordinates.
    pub content_rect_y: f32,
}

/// Parameters for rendering a labeled collapsible section header.
pub struct SectionHeaderParams<'a> {
    pub base_x: f32,
    pub y: f32,
    pub width: f32,
    pub section_id: &'static str,
    pub title: &'a str,
    pub is_collapsed: bool,
    pub cursor_pos: Point,
}