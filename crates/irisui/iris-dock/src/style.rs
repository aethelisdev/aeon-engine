// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Centralized theme, color palette, and geometric sizing configuration for docking (`DockStyle`).
//!
//! Controls tab bar heights, tab button paddings, separator thicknesses, close button visuals,
//! and drop zone highlight colors across the docking engine.

use crate::navigator::DockNavigatorStyle;
use iris_core::{Color, CornerRadii};

/// Visual styling and dimensional metrics for docked chrome, tab strips, and splitters.
#[derive(Debug, Clone, PartialEq)]
pub struct DockChromeStyle {
    /// Background color of empty dock pane leaves.
    pub empty_panel_bg: Color,
    /// Border stroke color of empty dock pane leaves.
    pub empty_panel_border: Color,
    /// Background color of leaf panel content regions.
    pub panel_bg: Color,
    /// Background color of leaf tab bar header strips.
    pub tab_bar_bg: Color,
    /// Height of each tab bar strip in logical pixels.
    pub tab_bar_height: f32,
    /// Color of the 1px continuous baseline divider running along the bottom of the tab strip.
    pub tab_bar_baseline_color: Color,
    /// Background color for the currently active/selected tab.
    pub tab_active_bg: Color,
    /// Background color for an unselected tab hovered by the cursor.
    pub tab_hovered_bg: Color,
    /// Background color for an idle unselected tab.
    pub tab_idle_bg: Color,
    /// Corner radii applied to tab button top borders.
    pub tab_corner_radii: CornerRadii,
    /// Accent color of the 2px line anchored flush to the bottom baseline of active tabs.
    pub tab_active_line_color: Color,
    /// Height in logical pixels of the active tab indicator line.
    pub tab_active_line_height: f32,
    /// Accent color tint for icons on active tabs.
    pub icon_active_tint: Color,
    /// Hovered color tint for icons on hovered tabs.
    pub icon_hovered_tint: Color,
    /// Idle color tint for icons on unselected tabs.
    pub icon_idle_tint: Color,
    /// Text color for the active tab label.
    pub text_active_color: Color,
    /// Text color for a hovered tab label.
    pub text_hovered_color: Color,
    /// Text color for an idle unselected tab label.
    pub text_idle_color: Color,
    /// Background pill color when hovering over the tab close button.
    pub close_btn_hover_bg: Color,
    /// Text/icon color for the tab close button when hovered.
    pub close_btn_hover_color: Color,
    /// Text/icon color for the tab close button in idle state.
    pub close_btn_idle_color: Color,
    /// Thickness of partition splitter lines in logical pixels.
    pub splitter_thickness: f32,
    /// Color of partition splitter dividers when active or hovered.
    pub splitter_active_color: Color,
    /// Color of partition splitter dividers in idle state.
    pub splitter_idle_color: Color,
    /// Minimum proportional width in logical pixels before triggering overflow handling.
    pub min_shrunk_tab_width: f32,
    /// Width in logical pixels reserved for the tab overflow chevron button.
    pub chevron_width: f32,
    /// Background color for the overflow chevron button when hovered.
    pub chevron_hovered_bg: Color,
    /// Background color for the overflow chevron button in idle state.
    pub chevron_idle_bg: Color,
    /// Icon color for the overflow chevron button when hovered.
    pub chevron_hovered_icon_col: Color,
    /// Icon color for the overflow chevron button in idle state.
    pub chevron_idle_icon_col: Color,
    /// Texture atlas UV coordinates for the chevron down icon `[u0, v0, u1, v1]`, if used.
    pub chevron_icon_uv: Option<[f32; 4]>,
}

impl Default for DockChromeStyle {
    fn default() -> Self {
        Self {
            empty_panel_bg: Color::from_u8(16, 20, 28, 160),
            empty_panel_border: Color::from_u8(35, 42, 55, 120),
            panel_bg: Color::from_u8(16, 18, 24, 255),
            tab_bar_bg: Color::from_u8(20, 24, 33, 255),
            tab_bar_height: 26.0,
            tab_bar_baseline_color: Color::from_u8(36, 42, 56, 180),
            tab_active_bg: Color::from_u8(30, 36, 50, 255),
            tab_hovered_bg: Color::from_u8(25, 30, 42, 255),
            tab_idle_bg: Color::from_u8(20, 24, 33, 255),
            tab_corner_radii: CornerRadii::new(5.0, 5.0, 0.0, 0.0),
            tab_active_line_color: Color::from_u8(0, 229, 255, 255),
            tab_active_line_height: 2.0,
            icon_active_tint: Color::from_u8(0, 229, 255, 255),
            icon_hovered_tint: Color::WHITE,
            icon_idle_tint: Color::from_u8(156, 163, 175, 255),
            text_active_color: Color::from_u8(0, 229, 255, 255),
            text_hovered_color: Color::from_u8(241, 245, 249, 255),
            text_idle_color: Color::from_u8(148, 163, 184, 255),
            close_btn_hover_bg: Color::rgba(0.9, 0.2, 0.2, 0.25),
            close_btn_hover_color: Color::rgba(1.0, 0.45, 0.45, 1.0),
            close_btn_idle_color: Color::rgba(0.65, 0.68, 0.75, 0.85),
            splitter_thickness: 3.0,
            splitter_active_color: Color::from_u8(0, 229, 255, 255),
            splitter_idle_color: Color::from_u8(30, 36, 48, 255),
            min_shrunk_tab_width: 68.0,
            chevron_width: 24.0,
            chevron_hovered_bg: Color::from_u8(30, 36, 50, 255),
            chevron_idle_bg: Color::from_u8(20, 24, 33, 255),
            chevron_hovered_icon_col: Color::from_u8(0, 229, 255, 255),
            chevron_idle_icon_col: Color::from_u8(148, 163, 184, 255),
            chevron_icon_uv: None,
        }
    }
}

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