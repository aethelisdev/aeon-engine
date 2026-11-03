// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window layer coordinator for Iris UI.
//!
//! Bridges Aeon Engine panel layout state with the native Iris UI floating window subsystem,
//! providing zero-allocation panel membership queries, workspace boundary constraints,
//! and content rectangle resolution.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::{FloatingWindowClampBounds, FloatingWindowStyle, MIN_VISIBLE_TITLE_BAR};
use irisui::prelude::Rect;

/// Constructs standard editor floating window clamp bounds constraining windows within visible screen extents.
///
/// Enforces that floating windows cannot be dragged above the top menubar or behind the bottom status bar,
/// and preserves at least 60px horizontal grab margin and 32px title bar visibility.
#[inline]
#[must_use]
pub fn editor_floating_clamp_bounds(screen_w: f32, screen_h: f32) -> FloatingWindowClampBounds {
    FloatingWindowClampBounds::new(screen_w, screen_h)
        .with_min_y(IrisEditorOverlay::MENUBAR_HEIGHT)
        .with_bottom_inset(IrisEditorOverlay::STATUS_BAR_HEIGHT)
        .with_min_size(220.0, 140.0)
        .with_min_visible_margin(60.0)
        .with_title_bar_height(irisui::dock::DEFAULT_TAB_BAR_HEIGHT.max(MIN_VISIBLE_TITLE_BAR))
}

/// Checks if a panel currently resides within any active floating window.
///
/// Performs the lookup with zero heap allocations by searching each floating window's dock tree directly.
#[inline]
#[must_use]
pub fn is_panel_in_floating_window(layout_state: &PanelLayoutState, panel: PanelId) -> bool {
    layout_state.dock_state.is_tab_in_floating_window(&panel)
}

/// Returns the Iris content rectangle assigned to an active panel in a floating window.
///
/// Floating panel builders use this rectangle instead of the retired host renderer's bounds,
/// ensuring their content begins below the native title and tab strip rather than covering it.
#[inline]
#[must_use]
pub fn active_panel_content_rect(layout_state: &PanelLayoutState, panel: PanelId) -> Option<Rect> {
    layout_state
        .dock_state
        .find_floating_tab_content_rect(&panel, FloatingWindowStyle::DEFAULT_TITLE_BAR_HEIGHT)
}

/// Resolves the content rectangle of the 3D viewport when detached in a floating window.
#[inline]
#[must_use]
pub fn resolve_floating_viewport_rect(layout_state: &PanelLayoutState) -> Option<Rect> {
    active_panel_content_rect(layout_state, PanelId::Viewport)
}