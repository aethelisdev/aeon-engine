// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Iris docking chrome layout calculation and retained node generation.
//!
//! Dispatches directly into [`irisui::dock::build_dock_chrome`] with the engine's theme styling and [`PanelTabViewer`].
//!

use crate::ui::panel_layout::{PanelLayoutState, PanelTabViewer};
use irisui::dock::{DockChromeParams, DockChromeStyle, build_dock_chrome};
use irisui::prelude::*;

use super::super::icons::*;
use super::super::theme::*;
use super::types::*;

/// Builds Iris-rendered tab strips, panel backgrounds, and split dividers for the native tree.
///
/// The returned geometry is the authoritative panel-coordinate source used by the Iris overlay.
pub fn build_native_dock(
    tree: &mut UiTree,
    parent: WidgetId,
    layout_state: &PanelLayoutState,
    workspace_rect: Rect,
    cursor_pos: Point,
    is_cursor_occluded: bool,
) -> NativeDockFrame {
    let is_dragging_splitter = layout_state.dock_state.active_splitter.is_some();
    let active_splitter_node = layout_state
        .dock_state
        .active_splitter
        .as_ref()
        .map(|s| s.node_id);
    let is_dragging_tab = layout_state.dock_state.active_drag.is_some();

    let style = DockChromeStyle {
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
    };

    let params = DockChromeParams {
        dock_tree: &layout_state.dock_state.tree,
        workspace_rect,
        cursor_pos,
        is_cursor_occluded,
        is_dragging_splitter,
        active_splitter_node,
        is_dragging_tab,
        viewer: &PanelTabViewer,
        style: &style,
    };

    let chrome = build_dock_chrome(tree, parent, &params);

    chrome.into()
}