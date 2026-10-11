// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Numeric Input Style & State Types
//!
//! Provides standardized styling metrics and editing state containers
//! for numeric input fields, property controls, and vector editors in Iris UI.
//!
//! Encapsulates visual colors, border geometry, font metrics, active keyboard editing
//! text buffers, selection highlights, and caret blinking phases.
//!

use iris_core::color::Color;

/// Visual styling configuration for a numeric input pill.
///
/// Encapsulates colors for idle, hover, and active states, border stroke
/// dimensions, corner radiuses, text colors, and font sizing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericInputStyle {
    /// Background color in normal/idle state.
    pub bg_idle: Color,
    /// Background color when hovered by cursor.
    pub bg_hover: Color,
    /// Background color when actively focused/editing.
    pub bg_active: Color,
    /// Border color in normal/idle state.
    pub border_idle: Color,
    /// Border color when hovered by cursor.
    pub border_hover: Color,
    /// Border color when actively focused/editing.
    pub border_active: Color,
    /// Border width in logical pixels.
    pub border_width: f32,
    /// Corner rounding radius in logical pixels.
    pub border_radius: f32,
    /// Text color in normal/idle state.
    pub text_color_idle: Color,
    /// Text color when hovered by cursor.
    pub text_color_hover: Color,
    /// Text color when actively focused/editing.
    pub text_color_active: Color,
    /// Background color of the selection pill when all text is selected.
    pub selection_bg: Color,
    /// Corner radius of the selection pill.
    pub selection_radius: f32,
    /// Font size in logical points.
    pub font_size: f32,
}

impl Default for NumericInputStyle {
    fn default() -> Self {
        Self {
            bg_idle: Color::rgba(0.125, 0.133, 0.153, 0.98),
            bg_hover: Color::rgba(0.157, 0.169, 0.200, 1.0),
            bg_active: Color::rgba(0.118, 0.125, 0.145, 1.0),
            border_idle: Color::rgba(0.180, 0.192, 0.227, 0.85),
            border_hover: Color::rgba(0.235, 0.247, 0.286, 0.95),
            border_active: Color::rgba(0.0, 0.80, 1.00, 0.95),
            border_width: 1.0,
            border_radius: 5.0,
            text_color_idle: Color::rgba(0.886, 0.894, 0.918, 1.0),
            text_color_hover: Color::WHITE,
            text_color_active: Color::WHITE,
            selection_bg: Color::rgba(0.14, 0.46, 0.88, 0.95),
            selection_radius: 3.0,
            font_size: 10.5,
        }
    }
}

/// Active text editing and cursor selection state for numeric inputs.
///
/// Encapsulates the user's active keyboard buffer, caret index, select-all state,
/// and cursor blink timer state during text entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericInputEditState<'a> {
    /// In-progress text buffer entered by the user.
    pub buffer: &'a str,
    /// Caret insertion index within the string buffer.
    pub cursor_idx: usize,
    /// Whether all characters in the buffer are currently selected.
    pub is_all_selected: bool,
    /// Caret blinking phase (true = visible cursor pipe, false = hidden).
    pub blink_caret: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_input_style_defaults() {
        let style = NumericInputStyle::default();
        assert_eq!(style.border_width, 1.0);
        assert_eq!(style.border_radius, 5.0);
        assert_eq!(style.font_size, 10.5);
    }

    #[test]
    fn test_numeric_input_edit_state() {
        let state = NumericInputEditState {
            buffer: "123.45",
            cursor_idx: 3,
            is_all_selected: false,
            blink_caret: true,
        };
        assert_eq!(state.buffer, "123.45");
        assert_eq!(state.cursor_idx, 3);
        assert!(!state.is_all_selected);
        assert!(state.blink_caret);
    }
}