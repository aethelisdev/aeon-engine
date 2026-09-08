// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Iris docking chrome construction and computed panel geometry extraction.

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::{DockNodeId, SplitDirection, compute_dock_layout};
use irisui::prelude::*;

/// Height of each native docking tab strip in logical editor pixels.
pub const NATIVE_DOCK_TAB_HEIGHT: f32 = 24.0;
const SPLITTER_THICKNESS: f32 = 3.0;

/// Native dock frame geometry consumed by Iris panel builders and editor hit testing.
/// The frame is rebuilt from the authoritative Iris split tree every render frame. It keeps
/// panel content coordinates independent from the retired immediate-mode dock renderer.
#[derive(Debug, Clone, Default)]
pub struct NativeDockFrame {
    /// Content rectangle assigned to each currently active panel.
    pub panel_rects: Vec<(PanelId, Rect)>,
    /// Click targets for Iris-rendered tab labels and their source leaf/index pairs.
    pub tab_targets: Vec<NativeDockTabTarget>,
    /// Drag targets for Iris-rendered split dividers.
    pub splitter_targets: Vec<NativeDockSplitterTarget>,
}

impl NativeDockFrame {
    /// Returns the active content rectangle for a panel, if its tab is selected in an Iris leaf.
    pub fn panel_rect(&self, panel: PanelId) -> Option<Rect> {
        self.panel_rects
            .iter()
            .find_map(|(candidate, rect)| (*candidate == panel).then_some(*rect))
    }
}

/// Native Iris tab hit target linked to a stable dock tree leaf and tab index.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockTabTarget {
    /// Leaf that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Logical editor-space click bounds.
    pub rect: Rect,
}

/// Native Iris splitter hit target linked to a stable split node.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockSplitterTarget {
    /// Split node that owns the divider.
    pub node: DockNodeId,
    /// Axis along which the divider moves.
    pub direction: SplitDirection,
    /// Logical editor-space drag bounds.
    pub rect: Rect,
    /// Parent-axis dimension used to normalize drag distance into a split ratio.
    pub total_dimension: f32,
}

/// Builds Iris-rendered tab strips, panel backgrounds, and split dividers for the native tree.
/// The returned geometry is the only panel-coordinate source used by the Iris overlay. Content
/// builders receive rectangles below their tab strip, so native chrome remains visible and never
/// overlaps panel controls.
pub fn build_native_dock(
    tree: &mut UiTree,
    parent: WidgetId,
    layout_state: &PanelLayoutState,
    workspace_rect: Rect,
) -> NativeDockFrame {
    let computed = compute_dock_layout(
        &layout_state.dock_state.tree,
        workspace_rect,
        SPLITTER_THICKNESS,
        NATIVE_DOCK_TAB_HEIGHT,
    );
    let mut frame = NativeDockFrame::default();

    for leaf in computed.leaves {
        add_rect_node(
            tree,
            parent,
            leaf.rect,
            "IrisDockPanel",
            Style::new()
                .background(Color::hex("#101116"))
                .border(1.0, Color::hex("#2d303c"))
                .clip_children(true),
        );
        add_rect_node(
            tree,
            parent,
            leaf.tab_bar_rect,
            "IrisDockTabStrip",
            Style::new()
                .background(Color::hex("#0d0e13"))
                .border(1.0, Color::hex("#252934")),
        );

        let tab_count = leaf.tabs.len().max(1) as f32;
        let tab_width = (leaf.tab_bar_rect.width / tab_count).max(1.0);
        for (index, panel) in leaf.tabs.iter().enumerate() {
            let tab_rect = Rect::new(
                leaf.tab_bar_rect.x + tab_width * index as f32,
                leaf.tab_bar_rect.y,
                tab_width,
                leaf.tab_bar_rect.height,
            );
            let active = index == leaf.active_tab;
            add_rect_node(
                tree,
                parent,
                tab_rect,
                "IrisDockTab",
                Style::new()
                    .background(if active {
                        Color::hex("#171a23")
                    } else {
                        Color::hex("#0d0e13")
                    })
                    .border(
                        if active { 1.0 } else { 0.0 },
                        if active {
                            Color::hex("#00bfe8")
                        } else {
                            Color::TRANSPARENT
                        },
                    ),
            );
            add_text_node(
                tree,
                parent,
                tab_rect,
                panel.title(),
                if active {
                    Color::hex("#00d7ff")
                } else {
                    Color::hex("#9ca3af")
                },
            );
            frame.tab_targets.push(NativeDockTabTarget {
                leaf: leaf.node_id,
                tab_index: index,
                rect: tab_rect,
            });
        }

        if let Some(panel) = leaf.tabs.get(leaf.active_tab) {
            frame.panel_rects.push((*panel, leaf.content_rect));
        }
    }

    for splitter in computed.splitters {
        add_rect_node(
            tree,
            parent,
            splitter.rect,
            "IrisDockSplitter",
            Style::new().background(Color::hex("#252934")),
        );
        frame.splitter_targets.push(NativeDockSplitterTarget {
            node: splitter.node_id,
            direction: splitter.direction,
            rect: splitter.rect,
            total_dimension: match splitter.direction {
                SplitDirection::Horizontal => workspace_rect.width,
                SplitDirection::Vertical => workspace_rect.height,
            }
            .max(1.0),
        });
    }

    frame
}

/// Adds an absolutely positioned native docking rectangle to the retained Iris tree.
fn add_rect_node(
    tree: &mut UiTree,
    parent: WidgetId,
    rect: Rect,
    name: &str,
    style: Style,
) -> WidgetId {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name(name);
        node.computed_rect = rect;
        node.set_style(style);
    }
    let _ = tree.add_child(parent, id);
    id
}

/// Adds a centered native docking tab label to the retained Iris tree.
fn add_text_node(tree: &mut UiTree, parent: WidgetId, rect: Rect, text: &str, color: Color) {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name("IrisDockTabLabel");
        node.computed_rect = rect;
        node.set_text(text);
        node.set_text_properties(12.0, 16.0, color, TextAlign::Center);
    }
    let _ = tree.add_child(parent, id);
}