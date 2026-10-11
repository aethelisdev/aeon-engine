// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Iris docking chrome layout calculation and retained node generation.
//!
//! Dispatches directly into [`irisui::dock::DockArea`] with the engine's theme styling and [`crate::ui::panel_layout::PanelTabViewer`].
//!

use crate::ui::panel_layout::PanelLayoutState;
use irisui::dock::DockChromeStyle;
use irisui::prelude::*;

use super::super::icons::*;
use super::super::theme::*;
use super::types::*;

/// Returns the engine-themed [`DockChromeStyle`] used across the editor.
pub fn editor_dock_style() -> DockChromeStyle {
    DockChromeStyle {
        empty_panel_bg: Color::from_u8(16, 20, 28, 160),
        empty_panel_border: Color::from_u8(35, 42, 55, 120),
        panel_bg: ELEVATION_1_PANEL,
        tab_bar_bg: ELEVATION_2_HEADER,
        tab_bar_height: NATIVE_DOCK_TAB_HEIGHT,
        tab_bar_baseline_color: BORDER_MICRON,
        tab_corner_radii: CornerRadii::new(4.0, 4.0, 0.0, 0.0),
        tab_active_bg: ELEVATION_1_PANEL,
        tab_hovered_bg: ELEVATION_3_HOVERED_PILL,
        tab_idle_bg: ELEVATION_2_HEADER,
        tab_active_line_color: ACCENT_CYAN,
        tab_active_line_height: 2.0,
        icon_active_tint: ACCENT_CYAN,
        icon_hovered_tint: Color::WHITE,
        icon_idle_tint: TEXT_REGULAR,
        text_active_color: ACCENT_CYAN,
        text_hovered_color: TEXT_BRIGHT,
        text_idle_color: TEXT_MUTED,
        close_btn_hover_bg: Color::rgba(0.9, 0.2, 0.2, 0.25),
        close_btn_hover_color: Color::rgba(1.0, 0.45, 0.45, 1.0),
        close_btn_idle_color: Color::rgba(0.65, 0.68, 0.75, 0.85),
        splitter_thickness: SPLITTER_THICKNESS,
        splitter_active_color: SPLITTER_ACTIVE,
        splitter_idle_color: SPLITTER_IDLE,
        min_shrunk_tab_width: MIN_SHRUNK_TAB_WIDTH,
        chevron_width: CHEVRON_WIDTH,
        chevron_hovered_bg: ELEVATION_3_HOVERED_PILL,
        chevron_idle_bg: ELEVATION_2_HEADER,
        chevron_hovered_icon_col: ACCENT_CYAN,
        chevron_idle_icon_col: TEXT_MUTED,
        chevron_icon_uv: Some(ICON_CHEVRON_DOWN),
    }
}

/// Builds Iris-rendered tab strips, panel backgrounds, and split dividers using [`DockArea`].
pub fn build_native_dock<V: irisui::dock::TabViewer<crate::ui::panel_layout::PanelId>>(
    scope: &mut UiScope<'_>,
    layout_state: &mut PanelLayoutState,
    workspace_rect: Rect,
    cursor_pos: Point,
    is_cursor_occluded: bool,
    viewer: &mut V,
) -> NativeDockFrame {
    let style = editor_dock_style();
    let chrome = irisui::dock::DockArea::new(&mut layout_state.dock_state)
        .bounds(workspace_rect)
        .style(style)
        .cursor_pos(cursor_pos)
        .cursor_occluded(is_cursor_occluded)
        .show(scope, viewer);

    chrome.into()
}