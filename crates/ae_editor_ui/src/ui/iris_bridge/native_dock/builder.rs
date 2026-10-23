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

/// Returns the default engine-themed [`DockChromeStyle`] used across the editor.
pub fn editor_dock_style() -> DockChromeStyle {
    editor_dock_style_with_tokens(&EditorThemeTokens::default())
}

/// Returns a [`DockChromeStyle`] resolved from the specified [`EditorThemeTokens`].
pub fn editor_dock_style_with_tokens(tokens: &EditorThemeTokens) -> DockChromeStyle {
    DockChromeStyle {
        empty_panel_bg: tokens.canvas_bg,
        empty_panel_border: tokens.splitter_idle,
        panel_bg: tokens.panel_bg,
        tab_bar_bg: tokens.tab_bar_bg,
        tab_bar_height: NATIVE_DOCK_TAB_HEIGHT,
        tab_bar_baseline_color: tokens.border_micron,
        tab_corner_radii: CornerRadii::new(4.0, 4.0, 0.0, 0.0),
        tab_active_bg: tokens.tab_active_bg,
        tab_hovered_bg: tokens.tab_hover_bg,
        tab_idle_bg: tokens.tab_idle_bg,
        tab_active_line_color: tokens.accent,
        tab_active_line_height: 2.0,
        icon_active_tint: Color::WHITE,
        icon_hovered_tint: Color::WHITE,
        icon_idle_tint: tokens.text_muted,
        text_active_color: tokens.text_active,
        text_hovered_color: Color::WHITE,
        text_idle_color: tokens.text_muted,
        close_btn_hover_bg: Color::rgba(0.9, 0.2, 0.2, 0.25),
        close_btn_hover_color: Color::rgba(1.0, 0.45, 0.45, 1.0),
        close_btn_idle_color: tokens.text_muted,
        splitter_thickness: SPLITTER_THICKNESS,
        splitter_active_color: tokens.splitter_active,
        splitter_idle_color: tokens.splitter_idle,
        min_shrunk_tab_width: MIN_SHRUNK_TAB_WIDTH,
        chevron_width: CHEVRON_WIDTH,
        chevron_hovered_bg: tokens.tab_hover_bg,
        chevron_idle_bg: tokens.tab_bar_bg,
        chevron_hovered_icon_col: tokens.accent,
        chevron_idle_icon_col: tokens.text_muted,
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