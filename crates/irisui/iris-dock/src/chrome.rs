// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained-mode docking chrome rendering, tab strip generation, and splitter management.
//!
//! Provides the primary visual chrome generation for docked pane layouts:
//! - Leaf background quads and tab header strips.
//! - Continuous 1px baseline dividers.
//! - Tab buttons with rounded top corners, active bottom accent lines, atlas/emoji icons, labels, and close buttons.
//! - Tab overflow chevron buttons when tabs exceed available strip width.
//! - Interactive partition splitter dividers between split panes.
//!

use crate::layout::compute_dock_layout;
use crate::tab_viewer::TabViewer;
use crate::tree::{DockNodeId, DockTree, SplitDirection};
use iris_core::color::Color;
use iris_core::geometry::{CornerRadii, Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

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

/// Native Iris tab hit target linked to a stable dock tree leaf and tab index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockTabTarget<T> {
    /// Leaf node that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Payload identifier of the tab.
    pub tab: T,
    /// Logical editor-space click bounds.
    pub rect: Rect,
    /// Bounding rectangle of the owning leaf pane.
    pub leaf_rect: Rect,
}

/// Native Iris tab close button hit target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockCloseTarget {
    /// Leaf that owns the tab.
    pub leaf: DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Logical editor-space click bounds.
    pub rect: Rect,
}

/// Native Iris dock tab overflow chevron button hit target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockChevronTarget {
    /// Leaf that owns the overflowing tab bar.
    pub leaf: DockNodeId,
    /// Logical editor-space click bounds of the chevron button.
    pub rect: Rect,
}

/// Native Iris splitter hit target linked to a stable split node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockSplitterTarget {
    /// Split node that owns the divider.
    pub node: DockNodeId,
    /// Axis along which the divider moves.
    pub direction: SplitDirection,
    /// Interactive hit rectangle of the splitter divider.
    pub rect: Rect,
    /// Total width or height of the parent split container.
    pub total_dimension: f32,
}

/// Native dock chrome geometry consumed by host panel builders and editor hit testing.
///
/// Rebuilt from the authoritative split tree every render frame.
#[derive(Debug, Clone)]
pub struct DockChromeFrame<T> {
    /// Content rectangle assigned to each currently active panel.
    pub panel_rects: Vec<(T, Rect)>,
    /// Click targets for rendered tab labels and their source leaf/index pairs.
    pub tab_targets: Vec<DockTabTarget<T>>,
    /// Click targets for tab close `✖` buttons.
    pub close_targets: Vec<DockCloseTarget>,
    /// Drag targets for rendered split dividers.
    pub splitter_targets: Vec<DockSplitterTarget>,
    /// Click targets for tab strip overflow chevron buttons (`▾`).
    pub chevron_targets: Vec<DockChevronTarget>,
}

impl<T> Default for DockChromeFrame<T> {
    fn default() -> Self {
        Self {
            panel_rects: Vec::new(),
            tab_targets: Vec::new(),
            close_targets: Vec::new(),
            splitter_targets: Vec::new(),
            chevron_targets: Vec::new(),
        }
    }
}

impl<T: PartialEq + Copy> DockChromeFrame<T> {
    /// Returns the active content rectangle for a panel, if its tab is selected in a leaf.
    pub fn panel_rect(&self, target: T) -> Option<Rect> {
        self.panel_rects
            .iter()
            .find_map(|(candidate, rect)| (*candidate == target).then_some(*rect))
    }
}

/// Context parameters passed into [`build_dock_chrome`].
///
/// Groups dimensional metrics, tree data, interaction states, and delegates into a single descriptor.
pub struct DockChromeParams<'a, T, V> {
    /// Generational split tree governing the pane hierarchy.
    pub dock_tree: &'a DockTree<T>,
    /// Bounding rectangle available for the entire dock workspace.
    pub workspace_rect: Rect,
    /// Current logical mouse cursor position.
    pub cursor_pos: Point,
    /// Whether the cursor is occluded by modals or foreground dropdown menus.
    pub is_cursor_occluded: bool,
    /// Whether an active divider partition is currently being dragged.
    pub is_dragging_splitter: bool,
    /// Identifier of the specific split node being dragged, if any.
    pub active_splitter_node: Option<DockNodeId>,
    /// Whether an individual tab is currently being dragged.
    pub is_dragging_tab: bool,
    /// Tab viewer lifecycle delegate providing titles, icons, and closeability.
    pub viewer: &'a V,
    /// Visual styling and dimensional metrics.
    pub style: &'a DockChromeStyle,
}

/// Builds complete retained Iris UI widgets for docked pane backgrounds, tab strips, and splitters.
pub fn build_dock_chrome<T: Clone + Copy, V: TabViewer<T>>(
    tree: &mut UiTree,
    parent: WidgetId,
    params: &DockChromeParams<'_, T, V>,
) -> DockChromeFrame<T> {
    let style = params.style;
    let computed = compute_dock_layout(
        params.dock_tree,
        params.workspace_rect,
        style.splitter_thickness,
        style.tab_bar_height,
    );
    let mut frame = DockChromeFrame::default();

    // 1. Render all leaf panel backgrounds and tab strips
    for leaf in &computed.leaves {
        if leaf.tabs.is_empty() {
            if params.is_dragging_tab {
                add_rect_node(
                    tree,
                    parent,
                    leaf.rect,
                    "IrisDockPanelEmpty",
                    Style::new()
                        .background(style.empty_panel_bg)
                        .border(1.0, style.empty_panel_border)
                        .clip_children(true),
                );
            }
            continue;
        }

        // Base panel background quad
        add_rect_node(
            tree,
            parent,
            leaf.rect,
            "IrisDockPanel",
            Style::new().background(style.panel_bg).clip_children(true),
        );

        // Tab strip header quad
        let strip_id = add_rect_node(
            tree,
            parent,
            leaf.tab_bar_rect,
            "IrisDockTabStrip",
            Style::new()
                .background(style.tab_bar_bg)
                .clip_children(true),
        );

        // Continuous baseline divider running full width of the tab bar
        add_rect_node(
            tree,
            strip_id,
            Rect::new(
                leaf.tab_bar_rect.x,
                leaf.tab_bar_rect.bottom() - 1.0,
                leaf.tab_bar_rect.width,
                1.0,
            ),
            "IrisDockTabStripBaseline",
            Style::new().background(style.tab_bar_baseline_color),
        );

        // Pre-measure all tabs in the current leaf to determine layout tier
        let tab_count = leaf.tabs.len();
        let bar_width = leaf.tab_bar_rect.width;

        let measurements: Vec<(f32, Option<[f32; 4]>, String, bool)> = leaf
            .tabs
            .iter()
            .map(|tab| {
                let atlas_icon = params.viewer.atlas_icon(tab);
                let title = if atlas_icon.is_some() {
                    params.viewer.raw_title(tab)
                } else {
                    params.viewer.title(tab)
                };
                let char_count = title.chars().count();
                let is_closeable = params.viewer.closeable(tab);
                let close_w = if is_closeable { 18.0f32 } else { 0.0f32 };
                let icon_w = if atlas_icon.is_some() {
                    18.0f32
                } else {
                    0.0f32
                };
                let natural_w: f32 =
                    (16.0f32 + icon_w + (char_count as f32) * 6.8f32 + close_w + 14.0f32)
                        .clamp(52.0f32, 170.0f32);
                (natural_w, atlas_icon, title, is_closeable)
            })
            .collect();

        let total_natural_w: f32 = measurements.iter().map(|m| m.0).sum();

        // Determine hybrid layout tier: Natural vs Proportional Shrink vs Overflow Chevron
        let (has_chevron, visible_start, visible_end, tab_widths) = if total_natural_w <= bar_width
        {
            (
                false,
                0,
                tab_count,
                measurements.iter().map(|m| m.0).collect(),
            )
        } else if (tab_count as f32 * style.min_shrunk_tab_width) <= bar_width {
            let shrink_ratio = (bar_width / total_natural_w.max(1.0)).clamp(0.0, 1.0);
            (
                false,
                0,
                tab_count,
                measurements
                    .iter()
                    .map(|m| (m.0 * shrink_ratio).max(style.min_shrunk_tab_width))
                    .collect(),
            )
        } else {
            let available_for_tabs = (bar_width - style.chevron_width).max(0.0);
            let max_visible = ((available_for_tabs / style.min_shrunk_tab_width).floor() as usize)
                .clamp(1, tab_count);
            let active_idx = leaf.active_tab.min(tab_count.saturating_sub(1));
            let (start, end) = if active_idx < max_visible {
                (0, max_visible)
            } else {
                let end = (active_idx + 1).min(tab_count);
                (end.saturating_sub(max_visible), end)
            };
            let count = (end - start).max(1);
            (
                true,
                start,
                end,
                vec![(available_for_tabs / count as f32).max(style.min_shrunk_tab_width); count],
            )
        };

        // Tabs start flush at the left boundary of the tab bar
        let mut current_tab_x = leaf.tab_bar_rect.x;
        let max_tab_strip_right = if has_chevron {
            leaf.tab_bar_rect.right() - style.chevron_width
        } else {
            leaf.tab_bar_rect.right()
        };

        for (offset, index) in (visible_start..visible_end).enumerate() {
            let tab = &leaf.tabs[index];
            let &(desired_natural_w, atlas_icon, ref title, is_closeable) = &measurements[index];
            let active = index == leaf.active_tab;

            let desired_w = tab_widths.get(offset).copied().unwrap_or(desired_natural_w);
            let tab_w = desired_w.min((max_tab_strip_right - current_tab_x).max(0.0));
            if tab_w <= 1.0 {
                continue;
            }

            let tab_rect = Rect::new(
                current_tab_x,
                leaf.tab_bar_rect.y,
                tab_w,
                leaf.tab_bar_rect.height,
            );
            let is_tab_hovered =
                !params.is_cursor_occluded && tab_rect.contains_point(params.cursor_pos);

            let bg_color = if active {
                style.tab_active_bg
            } else if is_tab_hovered {
                style.tab_hovered_bg
            } else {
                style.tab_idle_bg
            };

            // Tab studio background with rounded top corners, flat on bottom baseline
            let tab_id = add_rect_node(
                tree,
                strip_id,
                tab_rect,
                "IrisDockTab",
                Style::new()
                    .background(bg_color)
                    .corner_radii(style.tab_corner_radii)
                    .clip_children(true),
            );

            // Active accent line anchored flush to the bottom baseline and spanning full tab width
            if active {
                add_rect_node(
                    tree,
                    tab_id,
                    Rect::new(
                        tab_rect.x,
                        tab_rect.bottom() - style.tab_active_line_height,
                        tab_rect.width,
                        style.tab_active_line_height,
                    ),
                    "IrisDockTabActiveLine",
                    Style::new().background(style.tab_active_line_color),
                );
            }

            // Optional GPU atlas icon quad
            if let Some(uv) = atlas_icon
                && tab_w >= 36.0
            {
                let tint = if active {
                    style.icon_active_tint
                } else if is_tab_hovered {
                    style.icon_hovered_tint
                } else {
                    style.icon_idle_tint
                };
                add_icon_node(
                    tree,
                    tab_id,
                    Rect::new(tab_rect.x + 7.0, tab_rect.y + 5.0, 16.0, 16.0),
                    "IrisDockTabAtlasIcon",
                    uv,
                    tint,
                );
            }

            // Tab text label (clamped within remaining tab width)
            let text_offset_x = if atlas_icon.is_some() { 26.0 } else { 6.0 };
            let render_close = is_closeable && (active || is_tab_hovered || tab_w >= 84.0);
            let close_reserved = if render_close { 22.0 } else { 4.0 };
            let text_width = (tab_rect.width - text_offset_x - close_reserved).max(0.0);
            if text_width >= 8.0 {
                let text_rect = Rect::new(
                    tab_rect.x + text_offset_x,
                    tab_rect.y + 4.0,
                    text_width,
                    18.0,
                );
                let text_col = if active {
                    style.text_active_color
                } else if is_tab_hovered {
                    style.text_hovered_color
                } else {
                    style.text_idle_color
                };
                add_text_node(tree, tab_id, text_rect, title, text_col, TextAlign::Left);
            }

            // Tab close button (if closeable and tab width is sufficient)
            if render_close {
                let close_rect = Rect::new(
                    tab_rect.right() - 19.0,
                    tab_rect.y + 3.0,
                    15.0,
                    tab_rect.height - 6.0,
                );
                let is_close_hovered =
                    !params.is_cursor_occluded && close_rect.contains_point(params.cursor_pos);
                if is_close_hovered {
                    add_rect_node(
                        tree,
                        tab_id,
                        close_rect,
                        "IrisDockTabCloseHoverPill",
                        Style::new()
                            .background(style.close_btn_hover_bg)
                            .corner_radii(CornerRadii::all(3.0)),
                    );
                }
                let close_col = if is_close_hovered {
                    style.close_btn_hover_color
                } else {
                    style.close_btn_idle_color
                };
                add_text_node(tree, tab_id, close_rect, "✖", close_col, TextAlign::Center);
                frame.close_targets.push(DockCloseTarget {
                    leaf: leaf.node_id,
                    tab_index: index,
                    rect: close_rect,
                });
            }

            frame.tab_targets.push(DockTabTarget {
                leaf: leaf.node_id,
                tab_index: index,
                tab: *tab,
                rect: tab_rect,
                leaf_rect: leaf.rect,
            });

            current_tab_x += tab_w;
        }

        // Render tab overflow chevron button if tabs exceeded available tab bar width
        if has_chevron {
            let chevron_rect = Rect::new(
                leaf.tab_bar_rect.right() - style.chevron_width,
                leaf.tab_bar_rect.y,
                style.chevron_width,
                leaf.tab_bar_rect.height,
            );
            let is_chevron_hovered =
                !params.is_cursor_occluded && chevron_rect.contains_point(params.cursor_pos);
            let chevron_bg = if is_chevron_hovered {
                style.chevron_hovered_bg
            } else {
                style.chevron_idle_bg
            };
            add_rect_node(
                tree,
                strip_id,
                chevron_rect,
                "IrisDockTabChevron",
                Style::new().background(chevron_bg),
            );
            let chevron_text_col = if is_chevron_hovered {
                style.chevron_hovered_icon_col
            } else {
                style.chevron_idle_icon_col
            };

            if let Some(uv) = style.chevron_icon_uv {
                let icon_size = 12.0;
                let icon_rect = Rect::new(
                    chevron_rect.x + (chevron_rect.width - icon_size) * 0.5,
                    chevron_rect.y + (chevron_rect.height - icon_size) * 0.5,
                    icon_size,
                    icon_size,
                );
                add_icon_node(
                    tree,
                    strip_id,
                    icon_rect,
                    "IrisDockTabChevronIcon",
                    uv,
                    chevron_text_col,
                );
            } else {
                add_text_node(
                    tree,
                    strip_id,
                    chevron_rect,
                    "▾",
                    chevron_text_col,
                    TextAlign::Center,
                );
            }

            frame.chevron_targets.push(DockChevronTarget {
                leaf: leaf.node_id,
                rect: chevron_rect,
            });
        }

        if let Some(panel) = leaf.tabs.get(leaf.active_tab) {
            frame.panel_rects.push((*panel, leaf.content_rect));
        }
    }

    // 2. Render all splitter dividers
    for splitter in computed.splitters {
        let is_splitter_active = params.active_splitter_node == Some(splitter.node_id);
        let is_hovered = !params.is_cursor_occluded
            && splitter.rect.contains_point(params.cursor_pos)
            && !params.is_dragging_splitter;
        let splitter_col = if is_splitter_active || is_hovered {
            style.splitter_active_color
        } else {
            style.splitter_idle_color
        };

        let visual_rect = if splitter.direction == SplitDirection::Horizontal {
            Rect::new(
                splitter.rect.x,
                splitter.rect.y + style.tab_bar_height,
                splitter.rect.width,
                (splitter.rect.height - style.tab_bar_height).max(0.0),
            )
        } else {
            splitter.rect
        };

        add_rect_node(
            tree,
            parent,
            visual_rect,
            "IrisDockSplitter",
            Style::new().background(splitter_col),
        );
        let total_dim = if splitter.direction == SplitDirection::Horizontal {
            params.workspace_rect.width
        } else {
            params.workspace_rect.height
        }
        .max(1.0);
        frame.splitter_targets.push(DockSplitterTarget {
            node: splitter.node_id,
            direction: splitter.direction,
            rect: splitter.rect,
            total_dimension: total_dim,
        });
    }

    frame
}

/// Adds an absolutely positioned dock rectangle to the retained Iris tree.
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
        node.set_role(WidgetRole::Default);
        node.computed_rect = rect;
        node.set_style(style);
    }
    let _ = tree.add_child(parent, id);
    id
}

/// Adds a dock tab text label to the retained Iris tree.
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
        node.set_role(WidgetRole::Default);
        node.computed_rect = rect;
        node.set_text(text);
        node.set_text_properties(12.0, 16.0, color, alignment);
        node.set_style(Style::new().clip_children(true));
        node.interactive = false;
    }
    let _ = tree.add_child(parent, id);
}

/// Adds a textured GPU atlas icon node to the retained Iris tree.
fn add_icon_node(
    tree: &mut UiTree,
    parent: WidgetId,
    rect: Rect,
    name: &str,
    uv: [f32; 4],
    tint: Color,
) {
    let id = tree.create_node();
    if let Some(node) = tree.get_mut(id) {
        node.set_name(name);
        node.set_role(WidgetRole::Default);
        node.set_texture_uv(uv);
        node.computed_rect = rect;
        node.set_texture_tint(tint);
        node.interactive = false;
    }
    let _ = tree.add_child(parent, id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::DockTree;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MockTab {
        Viewport,
        Hierarchy,
    }

    struct MockViewer;
    impl TabViewer<MockTab> for MockViewer {
        fn title(&self, tab: &MockTab) -> String {
            match tab {
                MockTab::Viewport => "Viewport".to_string(),
                MockTab::Hierarchy => "Hierarchy".to_string(),
            }
        }

        fn closeable(&self, tab: &MockTab) -> bool {
            *tab != MockTab::Viewport
        }
    }

    #[test]
    fn test_build_dock_chrome_single_pane_and_tabs() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut dock_tree = DockTree::new();
        let root_leaf = dock_tree.create_leaf(vec![MockTab::Viewport, MockTab::Hierarchy]);
        dock_tree.set_root(root_leaf);

        let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);
        let params = DockChromeParams {
            dock_tree: &dock_tree,
            workspace_rect: workspace,
            cursor_pos: Point::new(10.0, 10.0),
            is_cursor_occluded: false,
            is_dragging_splitter: false,
            active_splitter_node: None,
            is_dragging_tab: false,
            viewer: &MockViewer,
            style: &DockChromeStyle::default(),
        };
        let frame = build_dock_chrome(&mut tree, root, &params);

        assert_eq!(frame.panel_rects.len(), 1);
        assert_eq!(frame.panel_rects[0].0, MockTab::Viewport);
        assert_eq!(frame.tab_targets.len(), 2);
        assert_eq!(frame.close_targets.len(), 1); // Only Hierarchy is closeable
        assert_eq!(frame.splitter_targets.len(), 0);
    }

    #[test]
    fn test_build_dock_chrome_splitters() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut dock_tree = DockTree::new();
        let root_leaf = dock_tree.create_leaf(vec![MockTab::Viewport]);
        dock_tree.set_root(root_leaf);
        let (_left, _right) = dock_tree
            .split(
                root_leaf,
                SplitDirection::Horizontal,
                0.3,
                vec![MockTab::Hierarchy],
            )
            .unwrap();

        let workspace = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let params = DockChromeParams {
            dock_tree: &dock_tree,
            workspace_rect: workspace,
            cursor_pos: Point::new(50.0, 50.0),
            is_cursor_occluded: false,
            is_dragging_splitter: false,
            active_splitter_node: None,
            is_dragging_tab: false,
            viewer: &MockViewer,
            style: &DockChromeStyle::default(),
        };
        let frame = build_dock_chrome(&mut tree, root, &params);

        assert_eq!(frame.panel_rects.len(), 2);
        assert_eq!(frame.tab_targets.len(), 2);
        assert_eq!(frame.splitter_targets.len(), 1);
        assert_eq!(
            frame.splitter_targets[0].direction,
            SplitDirection::Horizontal
        );
        assert!(frame.panel_rect(MockTab::Hierarchy).is_some());
        assert!(frame.panel_rect(MockTab::Viewport).is_some());
    }
}