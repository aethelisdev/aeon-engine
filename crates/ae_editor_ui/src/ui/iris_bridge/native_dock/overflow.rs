// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native dock tab overflow popup dropdown menu builder bridge.
//!
//! Delegates overflow popup rendering directly to the standardized [`iris_dock::build_dock_overflow_menu`]
//! engine, eliminating manual engine-side quad and typography orchestration.
//!

use crate::ui::panel_layout::PanelTabViewer;
use irisui::dock::{DockNode, DockOverflowMenuStyle, build_dock_overflow_menu};
use irisui::prelude::*;

use super::types::{NativeDockFrame, NativeDockOverflowItemTarget, NativeDockOverflowMenuParams};

/// Renders the floating popup dropdown menu listing all tabs for an active dock leaf with overflow.
/// Dispatches directly into [`iris_dock::build_dock_overflow_menu`] with the engine's [`PanelTabViewer`],
/// caching the resulting menu bounding box and item targets in [`NativeDockFrame`].
pub fn build_native_dock_overflow_menu(
    tree: &mut UiTree,
    parent: WidgetId,
    params: NativeDockOverflowMenuParams<'_>,
    frame: &mut NativeDockFrame,
) {
    let Some(node) = params.layout_state.dock_state.tree.get(params.leaf_id) else {
        return;
    };
    let DockNode::Leaf { tabs, active_tab } = node else {
        return;
    };
    if tabs.is_empty() {
        return;
    }

    let style = DockOverflowMenuStyle::default();
    let Some(overflow_frame) = build_dock_overflow_menu(
        tree,
        parent,
        irisui::dock::DockOverflowMenuParams {
            leaf_id: params.leaf_id,
            tabs,
            active_tab: *active_tab,
            anchor_rect: params.anchor_rect,
            cursor_pos: params.cursor_pos,
        },
        &PanelTabViewer,
        &style,
    ) else {
        return;
    };

    frame.active_overflow_rect = Some(overflow_frame.menu_rect);
    frame.overflow_item_targets.clear();
    for item in overflow_frame.items {
        frame
            .overflow_item_targets
            .push(NativeDockOverflowItemTarget {
                leaf: item.leaf,
                tab_index: item.tab_index,
                panel: tabs[item.tab_index],
                rect: item.rect,
            });
    }
}