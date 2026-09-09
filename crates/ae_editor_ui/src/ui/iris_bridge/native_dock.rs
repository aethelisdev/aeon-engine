// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native Iris docking chrome construction and computed panel geometry extraction.
//!
//! Renders native GPU SDF split containers, tab bars with compact snug widths,
//! active cyan indicator underlines, divider lines, and 5-way compass navigation drop zones.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::{
    DockNavigatorGeometry, DockNavigatorStyle, DockNodeId, FloatingTabBadgeParams, SplitDirection,
    build_dock_navigator_nodes, build_drop_preview_node, build_floating_tab_badge,
    compute_dock_layout,
};
use irisui::prelude::*;

/// Height of each native docking tab strip in logical editor pixels.
pub const NATIVE_DOCK_TAB_HEIGHT: f32 = 26.0;
/// Width/Thickness of partition splitter lines in logical pixels.
pub const SPLITTER_THICKNESS: f32 = 3.0;

/// Native dock frame geometry consumed by Iris panel builders and editor hit testing.
/// The frame is rebuilt from the authoritative Iris split tree every render frame. It keeps
/// panel content coordinates independent from any legacy immediate-mode dock renderer.
#[derive(Debug, Clone, Default)]
pub struct NativeDockFrame {
    /// Content rectangle assigned to each currently active panel.
    pub panel_rects: Vec<(PanelId, Rect)>,
    /// Click targets for Iris-rendered tab labels and their source leaf/index pairs.
    pub tab_targets: Vec<NativeDockTabTarget>,
    /// Click targets for tab close `✖` buttons.
    pub close_targets: Vec<NativeDockCloseTarget>,
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
    /// Panel kind.
    pub panel: PanelId,
    /// Logical editor-space click bounds.
    pub rect: Rect,
    /// Bounding rectangle of the owning leaf pane.
    pub leaf_rect: Rect,
}

/// Native Iris tab close button hit target.
#[derive(Debug, Clone, Copy)]
pub struct NativeDockCloseTarget {
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
/// The returned geometry is the authoritative panel-coordinate source used by the Iris overlay.
pub fn build_native_dock(
    tree: &mut UiTree,
    parent: WidgetId,
    layout_state: &PanelLayoutState,
    workspace_rect: Rect,
    cursor_pos: Point,
    is_cursor_occluded: bool,
) -> NativeDockFrame {
    let computed = compute_dock_layout(
        &layout_state.dock_state.tree,
        workspace_rect,
        SPLITTER_THICKNESS,
        NATIVE_DOCK_TAB_HEIGHT,
    );
    let mut frame = NativeDockFrame::default();

    let is_dragging_splitter = layout_state.dock_state.active_splitter.is_some();

    // 1. Render all leaf panel backgrounds and tab strips
    for leaf in &computed.leaves {
        if leaf.tabs.is_empty() {
            continue;
        }

        // Base panel background quad with subtle dark border
        add_rect_node(
            tree,
            parent,
            leaf.rect,
            "IrisDockPanel",
            Style::new()
                .background(Color::rgba(0.063, 0.067, 0.086, 1.0))
                .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.70))
                .clip_children(true),
        );

        // Tab strip header quad
        add_rect_node(
            tree,
            parent,
            leaf.tab_bar_rect,
            "IrisDockTabStrip",
            Style::new()
                .background(Color::rgba(0.059, 0.059, 0.078, 1.0))
                .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.80)),
        );

        // Compact, snug tab pills
        let mut current_tab_x = leaf.tab_bar_rect.x + 8.0;
        for (index, panel) in leaf.tabs.iter().enumerate() {
            let active = index == leaf.active_tab;
            let atlas_icon = panel.atlas_icon();
            let char_count = panel.title().chars().count();
            let is_closeable = *panel != PanelId::Viewport;
            let close_w = if is_closeable { 18.0 } else { 0.0 };
            let icon_w = if atlas_icon.is_some() { 18.0 } else { 0.0 };
            let title = if atlas_icon.is_some() {
                panel.title().to_string()
            } else {
                format!("{} {}", panel.icon(), panel.title())
            };
            let tab_w =
                (16.0 + icon_w + (char_count as f32) * 6.8 + close_w + 14.0).clamp(52.0, 170.0);

            let tab_rect = Rect::new(
                current_tab_x,
                leaf.tab_bar_rect.y,
                tab_w,
                leaf.tab_bar_rect.height,
            );
            let is_tab_hovered = !is_cursor_occluded && tab_rect.contains_point(cursor_pos);

            let bg_color = if active {
                Color::rgba(0.086, 0.094, 0.118, 1.0)
            } else if is_tab_hovered {
                Color::rgba(0.080, 0.088, 0.110, 1.0)
            } else {
                Color::rgba(0.059, 0.059, 0.078, 1.0)
            };

            // Tab pill background
            add_rect_node(
                tree,
                parent,
                tab_rect,
                "IrisDockTabPill",
                Style::new().background(bg_color).border_radius(4.0),
            );

            // Active 2px cyan line at bottom
            if active {
                add_rect_node(
                    tree,
                    parent,
                    Rect::new(
                        tab_rect.x + 2.0,
                        tab_rect.bottom() - 2.0,
                        tab_rect.width - 4.0,
                        2.0,
                    ),
                    "IrisDockTabActiveLine",
                    Style::new().background(Color::rgba(0.0, 0.898, 1.0, 1.0)),
                );
            }

            // Optional GPU atlas icon quad (16x16 crisp texels, blue when active, white when idle)
            if let Some(uv) = atlas_icon {
                let icon_node = tree.create_node();
                if let Some(node) = tree.get_mut(icon_node) {
                    node.set_name("IrisDockTabAtlasIcon");
                    node.set_texture_uv(uv);
                    node.computed_rect = Rect::new(tab_rect.x + 7.0, tab_rect.y + 5.0, 16.0, 16.0);
                    let tint = if active {
                        Color::rgba(0.0, 0.898, 1.0, 1.0)
                    } else if is_tab_hovered {
                        Color::WHITE
                    } else {
                        Color::rgba(0.85, 0.88, 0.94, 0.85)
                    };
                    node.set_texture_tint(tint);
                }
                let _ = tree.add_child(parent, icon_node);
            }

            // Tab text label
            let text_offset_x = if atlas_icon.is_some() {
                7.0 + 16.0 + 5.0
            } else {
                6.0
            };
            let text_rect = Rect::new(
                tab_rect.x + text_offset_x,
                tab_rect.y + 4.0,
                tab_rect.width - text_offset_x - close_w - 4.0,
                18.0,
            );
            let text_col = if active {
                Color::rgba(0.0, 0.898, 1.0, 1.0)
            } else if is_tab_hovered {
                Color::rgba(0.95, 0.96, 1.0, 1.0)
            } else {
                Color::rgba(0.65, 0.68, 0.75, 1.0)
            };
            add_text_node(tree, parent, text_rect, &title, text_col, TextAlign::Left);

            // Tab close button (if closeable)
            if is_closeable {
                let close_rect = Rect::new(
                    tab_rect.right() - 18.0,
                    tab_rect.y + 3.0,
                    14.0,
                    tab_rect.height - 6.0,
                );
                let is_close_hovered = !is_cursor_occluded && close_rect.contains_point(cursor_pos);
                let close_col = if is_close_hovered {
                    Color::rgba(1.0, 0.35, 0.35, 1.0)
                } else {
                    Color::rgba(0.55, 0.58, 0.65, 0.75)
                };
                add_text_node(tree, parent, close_rect, "✖", close_col, TextAlign::Center);
                frame.close_targets.push(NativeDockCloseTarget {
                    leaf: leaf.node_id,
                    tab_index: index,
                    rect: close_rect,
                });
            }

            frame.tab_targets.push(NativeDockTabTarget {
                leaf: leaf.node_id,
                tab_index: index,
                panel: *panel,
                rect: tab_rect,
                leaf_rect: leaf.rect,
            });

            current_tab_x += tab_w + 2.0;
        }

        if let Some(panel) = leaf.tabs.get(leaf.active_tab) {
            frame.panel_rects.push((*panel, leaf.content_rect));
        }
    }

    // 2. Render all splitter dividers
    for splitter in computed.splitters {
        let is_splitter_active = layout_state
            .dock_state
            .active_splitter
            .as_ref()
            .is_some_and(|s| s.node_id == splitter.node_id);
        let is_hovered = !is_cursor_occluded
            && splitter.rect.contains_point(cursor_pos)
            && !is_dragging_splitter;

        let splitter_col = if is_splitter_active || is_hovered {
            Color::rgba(0.0, 0.898, 1.0, 0.90)
        } else {
            Color::rgba(0.145, 0.161, 0.204, 1.0)
        };

        add_rect_node(
            tree,
            parent,
            splitter.rect,
            "IrisDockSplitter",
            Style::new().background(splitter_col),
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

/// Adds a native docking tab label to the retained Iris tree.
fn add_text_node(
    tree: &mut UiTree,
    parent: WidgetId,
    rect: Rect,
    text: &str,
    color: Color,
    alignment: TextAlign,
) {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name("IrisDockTabLabel");
        node.computed_rect = rect;
        node.set_text(text);
        node.set_text_properties(12.0, 16.0, color, alignment);
    }
    let _ = tree.add_child(parent, id);
}