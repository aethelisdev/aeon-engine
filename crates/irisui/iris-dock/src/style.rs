// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Centralized theme, color palette, and geometric sizing configuration for docking (`DockStyle`).
//!
//! Controls tab bar heights, tab button paddings, separator thicknesses, close button visuals,
//! and drop zone highlight colors across the docking engine.

use crate::navigator::DockNavigatorStyle;
use iris_core::Color;

/// Visual styling and sizing parameters governing the appearance of docking interfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct DockStyle {
    /// Height of the top tab bar strip in logical pixels.
    pub tab_bar_height: f32,
    /// Minimum allowable tab button width in logical pixels.
    pub tab_min_width: f32,
    /// Maximum allowable tab button width before clamping in logical pixels.
    pub tab_max_width: f32,
    /// Horizontal internal padding for each tab button.
    pub tab_padding_x: f32,
    /// Visual thickness of partition splitter divider lines.
    pub splitter_thickness: f32,
    /// Invisible extra hover hit margin on each side of splitters for easy grabbing.
    pub splitter_hit_margin: f32,
    /// Minimum allowable pane dimension in logical pixels.
    pub min_pane_size: f32,
    /// Outer threshold margin from window borders for screen-edge docking.
    pub screen_drop_margin: f32,
    /// Whether tab close buttons (`x`) should be rendered by default.
    pub show_close_buttons: bool,
    /// Whether the add tab button (`+`) should be rendered on tab strips.
    pub show_add_buttons: bool,

    // --- Colors ---
    /// Background fill color for the tab bar strip.
    pub tab_bar_bg: Color,
    /// Background color of an inactive/unselected tab.
    pub tab_bg_normal: Color,
    /// Background color of the currently active tab.
    pub tab_bg_active: Color,
    /// Background color of a tab under cursor hover.
    pub tab_bg_hover: Color,
    /// Text color for inactive tabs.
    pub tab_text_normal: Color,
    /// Text color for active tabs.
    pub tab_text_active: Color,
    /// Default color of tab close buttons (`x`).
    pub close_btn_color: Color,
    /// Highlight color of tab close buttons on cursor hover.
    pub close_btn_hover_color: Color,
    /// Default resting color of splitter divider lines.
    pub splitter_color: Color,
    /// Splitter color when hovered by cursor.
    pub splitter_hover_color: Color,
    /// Splitter color while actively being dragged.
    pub splitter_drag_color: Color,
    /// Fill color of the semi-transparent drop zone preview box.
    pub drop_preview_fill: Color,
    /// Border color of the semi-transparent drop zone preview box.
    pub drop_preview_border: Color,

    /// Nested visual style for the 5-way blueprint drop navigator.
    pub navigator: DockNavigatorStyle,
}

impl Default for DockStyle {
    fn default() -> Self {
        Self::dark()
    }
}

impl DockStyle {
    /// Constructs a standard dark-slate and cyan theme matching Aeon Engine defaults.
    pub fn dark() -> Self {
        Self {
            tab_bar_height: 26.0,
            tab_min_width: 56.0,
            tab_max_width: 180.0,
            tab_padding_x: 10.0,
            splitter_thickness: 4.0,
            splitter_hit_margin: 8.0,
            min_pane_size: 60.0,
            screen_drop_margin: 32.0,
            show_close_buttons: true,
            show_add_buttons: true,

            tab_bar_bg: Color::rgba(0.08, 0.09, 0.11, 1.0),
            tab_bg_normal: Color::rgba(0.12, 0.13, 0.16, 0.8),
            tab_bg_active: Color::rgba(0.18, 0.20, 0.25, 1.0),
            tab_bg_hover: Color::rgba(0.15, 0.17, 0.22, 1.0),
            tab_text_normal: Color::rgba(0.60, 0.64, 0.72, 1.0),
            tab_text_active: Color::rgba(0.95, 0.97, 1.00, 1.0),
            close_btn_color: Color::rgba(0.55, 0.58, 0.65, 0.8),
            close_btn_hover_color: Color::rgba(0.95, 0.35, 0.35, 1.0),
            splitter_color: Color::rgba(0.15, 0.16, 0.20, 0.7),
            splitter_hover_color: Color::rgba(0.0, 0.85, 1.0, 0.6),
            splitter_drag_color: Color::rgba(0.0, 0.90, 1.0, 0.95),
            drop_preview_fill: Color::rgba(0.0, 0.85, 1.0, 0.18),
            drop_preview_border: Color::rgba(0.0, 0.90, 1.0, 0.80),

            navigator: DockNavigatorStyle::default(),
        }
    }

    /// Sets the height of the tab bar in logical pixels.
    pub fn with_tab_bar_height(mut self, height: f32) -> Self {
        self.tab_bar_height = height.max(16.0);
        self
    }

    /// Sets the minimum pane size in logical pixels.
    pub fn with_min_pane_size(mut self, min_size: f32) -> Self {
        self.min_pane_size = min_size.max(10.0);
        self
    }

    /// Toggles the rendering of close buttons on tab bars.
    pub fn with_close_buttons(mut self, enabled: bool) -> Self {
        self.show_close_buttons = enabled;
        self
    }

    /// Toggles the rendering of the add tab button on tab bars.
    pub fn with_add_buttons(mut self, enabled: bool) -> Self {
        self.show_add_buttons = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_style_defaults_and_builders() {
        let style = DockStyle::default()
            .with_tab_bar_height(32.0)
            .with_min_pane_size(80.0)
            .with_close_buttons(false);

        assert_eq!(style.tab_bar_height, 32.0);
        assert_eq!(style.min_pane_size, 80.0);
        assert!(!style.show_close_buttons);
        assert!(style.show_add_buttons);
    }
}