// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Tab Context and Type Definitions
//!
//! Provides context data structures and option mappings for the Editor preferences submodules.

use crate::ui::iris_bridge::preferences::types::{PreferencesDropdownId, PreferencesSliderId};
use ae_editor::snapping::SnapMode;
use irisui::prelude::*;
use std::collections::HashSet;

/// Snapping mode options.
pub const SNAP_MODE_OPTIONS: [(SnapMode, &str); 3] = [
    (SnapMode::Off, "Off"),
    (SnapMode::Hold, "Hold (Ctrl)"),
    (SnapMode::Toggle, "Toggle"),
];

/// Shared layout parameters for building editor preference cards.
#[derive(Clone, Copy)]
pub struct EditorCardContext<'a> {
    /// Left horizontal origin in content area.
    pub base_x: f32,
    /// Top vertical origin in content area.
    pub content_y: f32,
    /// Usable content width.
    pub content_w: f32,
    /// Vertical scroll offset.
    pub scroll_y: f32,
    /// Width of the number input pill box.
    pub val_box_w: f32,
    /// Height of the number input pill box.
    pub val_box_h: f32,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Set of currently collapsed card identifiers.
    pub collapsed_sections: &'a HashSet<&'static str>,
    /// Active number input editing state: `(slider_id, editing_buffer)`.
    pub active_number_input: Option<(PreferencesSliderId, &'a str)>,
    /// Caret blink toggle state for text cursor.
    pub blink_caret: bool,
    /// Active dropdown menu currently open.
    pub active_dropdown: Option<PreferencesDropdownId>,
}