// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window layer coordinator for Iris UI.
//!
//! Bridges Aeon Engine panel layout state with the native Iris UI floating window subsystem,
//! tracking active floating window rectangles to support hardware occlusion culling,
//! and resolving whether specific panels reside in docked leaves or independent floating surfaces.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState, PanelTabViewer};
use irisui::dock::{DockNode, FloatingWindowStyle, build_floating_windows_layer};
use irisui::prelude::*;

/// Checks if a panel currently resides within any active floating window.
pub fn is_panel_in_floating_window(layout_state: &PanelLayoutState, panel: PanelId) -> bool {
    layout_state
        .dock_state
        .floating_windows
        .iter()
        .any(|w| w.tree.all_tabs().contains(&panel))
}

/// Returns the Iris content rectangle assigned to an active panel in a floating window.
/// Floating panel builders use this rectangle instead of the retired host renderer's bounds,
/// ensuring their content begins below the native title and tab strip rather than covering it.
pub fn active_panel_content_rect(layout_state: &PanelLayoutState, panel: PanelId) -> Option<Rect> {
    const TITLE_BAR_HEIGHT: f32 = 26.0;

    layout_state
        .dock_state
        .floating_windows
        .iter()
        .find_map(|window| {
            for (_, node) in window.tree.iter() {
                if let DockNode::Leaf { tabs, active_tab } = node
                    && tabs.get(*active_tab).is_some_and(|active| *active == panel)
                {
                    return Some(Rect::new(
                        window.rect.x,
                        window.rect.y + TITLE_BAR_HEIGHT,
                        window.rect.width,
                        (window.rect.height - TITLE_BAR_HEIGHT).max(0.0),
                    ));
                }
            }
            None
        })
}

/// Resolves the content rectangle of the 3D viewport when detached in a floating window.
pub fn resolve_floating_viewport_rect(layout_state: &PanelLayoutState) -> Option<Rect> {
    active_panel_content_rect(layout_state, PanelId::Viewport)
}

/// Builds the complete native Iris UI floating window hierarchy in the UI tree.
/// Delegates to the core Iris UI docking framework (`irisui::dock::build_floating_windows_layer`)
/// which instantiates standard floating window containers, styling quads, tab pills, and controls.
/// Returns:
/// - List of floating window bounding rectangles for hardware occlusion culling.
/// - Active panel mapping `(PanelId, WidgetId)` where `WidgetId` is the floating window container.
pub fn build_floating_windows(
    tree: &mut UiTree,
    root_id: WidgetId,
    layout_state: &PanelLayoutState,
    cursor_pos: Point,
) -> (Vec<Rect>, Vec<(PanelId, WidgetId)>) {
    build_floating_windows_layer(
        tree,
        root_id,
        &layout_state.dock_state.floating_windows,
        &PanelTabViewer,
        cursor_pos,
        &FloatingWindowStyle::default(),
    )
}