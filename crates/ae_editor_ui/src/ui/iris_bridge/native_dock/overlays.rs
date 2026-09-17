// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dock drag-and-drop overlays (compass navigator, drop zone preview, floating tab badge).
//!

use crate::ui::panel_layout::PanelLayoutState;
use irisui::dock::{
    DockNavigatorGeometry, DockNavigatorStyle, FloatingTabBadgeParams, build_dock_navigator_nodes,
    build_drop_preview_node, build_floating_tab_badge, compute_dock_layout,
};
use irisui::prelude::*;

use super::types::{NATIVE_DOCK_TAB_HEIGHT, SPLITTER_THICKNESS};

/// Renders 5-way compass dock navigator, drop zone preview, and floating tab badge overlays.
/// Rendered as topmost floating overlays so they are drawn above the 3D Viewport texture,
/// docked panels, and floating windows.
pub fn build_native_dock_drag_overlays(
    tree: &mut UiTree,
    parent: WidgetId,
    layout_state: &PanelLayoutState,
    workspace_rect: Rect,
) {
    let Some(ref drag) = layout_state.dock_state.active_drag else {
        return;
    };

    let computed = compute_dock_layout(
        &layout_state.dock_state.tree,
        workspace_rect,
        SPLITTER_THICKNESS,
        NATIVE_DOCK_TAB_HEIGHT,
    );

    // Find leaf hovered by cursor from precomputed leaves
    let hovered_leaf = computed
        .leaves
        .iter()
        .find(|leaf| leaf.rect.contains_point(drag.cursor_pos));

    if let Some(leaf) = hovered_leaf {
        let nav_style = DockNavigatorStyle::default();
        let geometry = DockNavigatorGeometry::from_content_rect(leaf.content_rect, 40.0, 4.0);
        let drop_zone = geometry.hit_test(drag.cursor_pos);

        // Drop preview rectangle overlay
        if let Some(zone) = drop_zone {
            build_drop_preview_node(tree, parent, leaf.content_rect, zone, &nav_style);
        }

        // 5-way compass buttons
        build_dock_navigator_nodes(tree, parent, &geometry, drop_zone, &nav_style);
    }

    // Floating tab badge following the cursor
    let badge_params = FloatingTabBadgeParams {
        cursor_pos: drag.cursor_pos,
        title: drag.tab_data.title(),
        icon: Some(drag.tab_data.icon()),
    };
    build_floating_tab_badge(tree, parent, badge_params);
}