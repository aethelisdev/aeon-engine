// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Visual Theme & Token System (`iris-core::theme`)
//!
//! Provides a centralized, strongly-typed semantic theme token system ([`Theme`])
//! eliminating scattered hardcoded color literals across UI widgets and editor panels.

use crate::color::Color;

/// Centralized semantic design tokens defining UI surface colors, text hierarchies,
/// status accents, and geometric corner radii.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Background fill color for top-level panels and dockable windows.
    pub panel_bg: Color,
    /// Subtle outline border color for panels.
    pub panel_border: Color,
    /// Background fill color for elevated card containers and property groups.
    pub card_bg: Color,
    /// Outline border color for cards and inspector sections.
    pub card_border: Color,
    /// Background fill color for card header strips and collapsible headers.
    pub card_header_bg: Color,
    /// Background fill color for horizontal toolbars and control strips.
    pub toolbar_bg: Color,
    /// Outline border color for toolbars.
    pub toolbar_border: Color,
    /// Semi-transparent dark backdrop scrim for modal overlays.
    pub modal_scrim: Color,
    /// Elevated background fill color for modal dialog windows.
    pub modal_bg: Color,
    /// Outline border color for modal dialog windows.
    pub modal_border: Color,

    /// Default idle background fill color for standard push buttons.
    pub button_idle_bg: Color,
    /// Hover background fill color for standard push buttons.
    pub button_hover_bg: Color,
    /// Active pressed background fill color for push buttons.
    pub button_active_bg: Color,
    /// Idle outline border color for standard push buttons.
    pub button_border: Color,
    /// Hover outline border color for standard push buttons.
    pub button_hover_border: Color,

    /// Primary high-contrast text color for active headings, labels, and icons.
    pub text_primary: Color,
    /// Secondary comfortable reading color for standard property labels and text.
    pub text_secondary: Color,
    /// Dimmed subtle text color for placeholders, units, and secondary hints.
    pub text_muted: Color,
    /// Vertical blinking caret indicator color for focused text inputs.
    pub text_caret: Color,

    /// Primary brand / interactive cyan accent color.
    pub accent: Color,
    /// Hover state color for interactive cyan accent elements.
    pub accent_hover: Color,
    /// Success indicator color for confirmations, passing checks, and running states.
    pub success: Color,
    /// Warning indicator color for alerts, warnings, and attention items.
    pub warning: Color,
    /// Danger indicator color for destructive actions, errors, and fatal states.
    pub danger: Color,
    /// Information indicator color for neutral informational badges and notifications.
    pub info: Color,

    /// 1-pixel separator line and divider color.
    pub divider: Color,
    /// Background track color for scroll area scrollbars.
    pub scrollbar_track: Color,
    /// Idle thumb pill color for scroll area scrollbars.
    pub scrollbar_thumb: Color,
    /// Hover thumb pill color for scroll area scrollbars.
    pub scrollbar_thumb_hover: Color,

    /// Small corner radius for buttons, chips, and pills (default: 3.0px).
    pub corner_radius_sm: f32,
    /// Medium corner radius for cards, sections, and dropdowns (default: 4.0px).
    pub corner_radius_md: f32,
    /// Large corner radius for modal dialogs and elevated floating overlays (default: 8.0px).
    pub corner_radius_lg: f32,
}

impl Default for Theme {
    #[inline]
    fn default() -> Self {
        Self::dark_default()
    }
}

impl Theme {
    /// Creates the standard production dark theme for professional workstation UI.
    #[must_use]
    pub fn dark_default() -> Self {
        Self {
            panel_bg: Color::rgba(0.047, 0.052, 0.068, 1.0),
            panel_border: Color::rgba(0.18, 0.21, 0.28, 0.85),
            card_bg: Color::rgba(0.078, 0.088, 0.118, 0.85),
            card_border: Color::rgba(0.18, 0.21, 0.28, 0.60),
            card_header_bg: Color::rgba(0.068, 0.076, 0.100, 0.95),
            toolbar_bg: Color::rgba(0.068, 0.076, 0.100, 1.0),
            toolbar_border: Color::rgba(0.18, 0.21, 0.28, 0.85),
            modal_scrim: Color::rgba(0.0, 0.0, 0.0, 0.65),
            modal_bg: Color::rgba(0.075, 0.086, 0.133, 0.98),
            modal_border: Color::rgba(0.22, 0.25, 0.34, 0.90),

            button_idle_bg: Color::rgba(0.078, 0.088, 0.118, 0.85),
            button_hover_bg: Color::rgba(0.105, 0.120, 0.160, 1.0),
            button_active_bg: Color::rgba(0.0, 0.60, 0.90, 0.90),
            button_border: Color::rgba(0.18, 0.21, 0.28, 0.85),
            button_hover_border: Color::rgba(0.0, 0.898, 1.0, 0.85),

            text_primary: Color::rgba(0.98, 0.99, 1.0, 1.0),
            text_secondary: Color::rgba(0.85, 0.88, 0.95, 1.0),
            text_muted: Color::rgba(0.55, 0.58, 0.68, 1.0),
            text_caret: Color::rgba(0.24, 0.52, 0.95, 1.0),

            accent: Color::rgba(0.0, 0.898, 1.0, 1.0),
            accent_hover: Color::rgba(0.25, 0.94, 1.0, 1.0),
            success: Color::rgba(0.20, 0.80, 0.45, 1.0),
            warning: Color::rgba(0.95, 0.75, 0.20, 1.0),
            danger: Color::rgba(0.95, 0.28, 0.30, 1.0),
            info: Color::rgba(0.25, 0.60, 1.0, 1.0),

            divider: Color::rgba(0.18, 0.21, 0.28, 0.85),
            scrollbar_track: Color::rgba(0.035, 0.040, 0.055, 0.60),
            scrollbar_thumb: Color::rgba(0.18, 0.21, 0.28, 0.70),
            scrollbar_thumb_hover: Color::rgba(0.28, 0.33, 0.44, 0.90),

            corner_radius_sm: 3.0,
            corner_radius_md: 4.0,
            corner_radius_lg: 8.0,
        }
    }

    /// Creates a deep obsidian black theme optimized for high-contrast dark environments.
    #[must_use]
    pub fn obsidian() -> Self {
        Self {
            panel_bg: Color::rgba(0.028, 0.031, 0.043, 1.0),
            panel_border: Color::rgba(0.14, 0.16, 0.22, 0.85),
            card_bg: Color::rgba(0.047, 0.052, 0.068, 0.95),
            card_border: Color::rgba(0.16, 0.18, 0.24, 0.60),
            card_header_bg: Color::rgba(0.038, 0.042, 0.055, 0.95),
            toolbar_bg: Color::rgba(0.038, 0.042, 0.055, 1.0),
            toolbar_border: Color::rgba(0.14, 0.16, 0.22, 0.85),
            modal_scrim: Color::rgba(0.0, 0.0, 0.0, 0.80),
            modal_bg: Color::rgba(0.047, 0.052, 0.068, 1.0),
            modal_border: Color::rgba(0.20, 0.23, 0.30, 0.85),

            button_idle_bg: Color::rgba(0.055, 0.062, 0.082, 0.95),
            button_hover_bg: Color::rgba(0.085, 0.095, 0.125, 1.0),
            button_active_bg: Color::rgba(0.0, 0.55, 0.85, 0.95),
            button_border: Color::rgba(0.16, 0.18, 0.24, 0.80),
            button_hover_border: Color::rgba(0.0, 0.898, 1.0, 0.85),

            text_primary: Color::rgba(0.98, 0.99, 1.0, 1.0),
            text_secondary: Color::rgba(0.85, 0.88, 0.95, 1.0),
            text_muted: Color::rgba(0.55, 0.58, 0.68, 1.0),
            text_caret: Color::rgba(0.24, 0.52, 0.95, 1.0),

            accent: Color::rgba(0.0, 0.898, 1.0, 1.0),
            accent_hover: Color::rgba(0.25, 0.94, 1.0, 1.0),
            success: Color::rgba(0.20, 0.80, 0.45, 1.0),
            warning: Color::rgba(0.95, 0.75, 0.20, 1.0),
            danger: Color::rgba(0.95, 0.28, 0.30, 1.0),
            info: Color::rgba(0.25, 0.60, 1.0, 1.0),

            divider: Color::rgba(0.14, 0.16, 0.22, 0.85),
            scrollbar_track: Color::rgba(0.02, 0.02, 0.03, 0.60),
            scrollbar_thumb: Color::rgba(0.14, 0.16, 0.22, 0.70),
            scrollbar_thumb_hover: Color::rgba(0.22, 0.26, 0.35, 0.90),

            corner_radius_sm: 3.0,
            corner_radius_md: 4.0,
            corner_radius_lg: 8.0,
        }
    }

    /// Sets the vertical blinking text caret color in-place.
    pub fn set_text_caret(&mut self, color: Color) {
        self.text_caret = color;
    }

    /// Builder method to customize the vertical blinking text caret color.
    #[must_use]
    pub fn with_text_caret(mut self, color: Color) -> Self {
        self.text_caret = color;
        self
    }
}