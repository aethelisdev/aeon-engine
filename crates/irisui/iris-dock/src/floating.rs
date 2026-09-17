// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window model enabling detachable, free-floating panel surfaces.
//!
//! Each [`FloatingWindow`] hosts an independent [`DockTree`] instance, allowing
//! detached tabs to be repositioned, resized, split into sub-panes, or docked back.
//! Also provides complete native widget tree generation and click interaction evaluation.

use crate::tab_viewer::TabViewer;
use crate::tree::{DockNode, DockNodeId, DockTree};
use iris_core::color::Color;
use iris_core::geometry::{CornerRadii, Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Edge or corner of a floating window used for 8-directional resizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingResizeEdge {
    /// Left border edge.
    Left,
    /// Right border edge.
    Right,
    /// Top border edge.
    Top,
    /// Bottom border edge.
    Bottom,
    /// Top-left corner handle.
    TopLeft,
    /// Top-right corner handle.
    TopRight,
    /// Bottom-left corner handle.
    BottomLeft,
    /// Bottom-right corner handle.
    BottomRight,
}

/// Active drag or resize state for an interactive floating window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatingDragState {
    /// Moving the window across the screen by dragging its title bar.
    Title {
        /// Cursor offset relative to the window origin `(x, y)` when dragging commenced.
        offset: Point,
    },
    /// Resizing the window boundaries along an edge or corner.
    Resize(FloatingResizeEdge),
}

/// Detects whether the specified cursor point falls within the edge or corner resize margin of a rectangle.
/// Returns the corresponding [`FloatingResizeEdge`] if the cursor is within `margin` logical pixels
/// of the border, or `None` if the cursor is inside the window body or entirely outside.
#[inline]
pub fn detect_resize_edge(rect: Rect, point: Point, margin: f32) -> Option<FloatingResizeEdge> {
    if !rect.contains_point(point) {
        return None;
    }
    let on_left = point.x <= rect.x + margin;
    let on_right = point.x >= rect.right() - margin;
    let on_top = point.y <= rect.y + margin;
    let on_bottom = point.y >= rect.bottom() - margin;

    if on_top && on_left {
        Some(FloatingResizeEdge::TopLeft)
    } else if on_top && on_right {
        Some(FloatingResizeEdge::TopRight)
    } else if on_bottom && on_left {
        Some(FloatingResizeEdge::BottomLeft)
    } else if on_bottom && on_right {
        Some(FloatingResizeEdge::BottomRight)
    } else if on_left {
        Some(FloatingResizeEdge::Left)
    } else if on_right {
        Some(FloatingResizeEdge::Right)
    } else if on_top {
        Some(FloatingResizeEdge::Top)
    } else if on_bottom {
        Some(FloatingResizeEdge::Bottom)
    } else {
        None
    }
}

/// Visual theme and metrics configuration for native floating window widgets.
#[derive(Debug, Clone)]
pub struct FloatingWindowStyle {
    /// Background color of the main floating window container.
    pub background: Color,
    /// Border stroke thickness in logical pixels.
    pub border_width: f32,
    /// Border stroke color.
    pub border_color: Color,
    /// Exterior corner radius.
    pub corner_radius: f32,
    /// Drop shadow vertical offset.
    pub shadow_offset_y: f32,
    /// Drop shadow blur radius.
    pub shadow_blur: f32,
    /// Drop shadow color.
    pub shadow_color: Color,
    /// Height of the top title / tab bar header.
    pub title_bar_height: f32,
    /// Background color of the title / tab bar header.
    pub title_bar_background: Color,
    /// Bottom baseline divider line color of the header.
    pub baseline_color: Color,
    /// Accent color used for active indicators and active tab highlights.
    pub accent_color: Color,
    /// Background color of active tab pill.
    pub tab_active_bg: Color,
    /// Background color of hovered tab pill.
    pub tab_hover_bg: Color,
    /// Background color of inactive tab pill.
    pub tab_idle_bg: Color,
    /// Text color for inactive tabs.
    pub text_muted: Color,
    /// Text color for hovered tabs.
    pub text_hover: Color,
    /// Text color for active tabs.
    pub text_active: Color,
    /// Close button icon idle color.
    pub close_btn_idle: Color,
    /// Close button icon hover color.
    pub close_btn_hover: Color,
}

impl Default for FloatingWindowStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.055, 0.063, 0.086, 1.0), // #0e1016
            border_width: 1.0,
            border_color: Color::rgba(0.18, 0.20, 0.26, 0.8),
            corner_radius: 19.0,
            shadow_offset_y: 8.0,
            shadow_blur: 28.0,
            shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.75),
            title_bar_height: 26.0,
            title_bar_background: Color::rgba(0.059, 0.059, 0.078, 1.0), // #0f0f14
            baseline_color: Color::rgba(1.0, 1.0, 1.0, 0.06),
            accent_color: Color::rgba(0.2, 0.8, 0.95, 1.0),
            tab_active_bg: Color::rgba(0.118, 0.125, 0.165, 1.0),
            tab_hover_bg: Color::rgba(0.14, 0.15, 0.20, 0.9),
            tab_idle_bg: Color::rgba(0.059, 0.059, 0.078, 1.0),
            text_muted: Color::rgba(0.55, 0.58, 0.65, 1.0),
            text_hover: Color::rgba(0.9, 0.92, 0.96, 1.0),
            text_active: Color::rgba(0.2, 0.8, 0.95, 1.0),
            close_btn_idle: Color::rgba(0.55, 0.58, 0.65, 1.0),
            close_btn_hover: Color::rgba(1.0, 0.35, 0.35, 1.0),
        }
    }
}

/// Click action outcome evaluated from an interactive click on a floating window.
#[derive(Debug, Clone, PartialEq)]
pub enum FloatingWindowClickAction<Tab: Clone> {
    /// Clicked the dock-back button `⤢`.
    DockBack {
        /// Target floating window identifier.
        window_id: u64,
    },
    /// Clicked the close button `✖`.
    Close {
        /// Target floating window identifier.
        window_id: u64,
    },
    /// Clicked a specific tab pill.
    TabSelected {
        /// Target floating window identifier.
        window_id: u64,
        /// Leaf node identifier hosting the tab within the window's dock tree.
        leaf_id: DockNodeId,
        /// Index of the tab within the leaf's tab list.
        tab_index: usize,
        /// Cloned tab item.
        tab: Tab,
        /// Full window bounding rectangle.
        window_rect: Rect,
    },
    /// Clicked the title bar background to initiate dragging.
    TitleDragStart {
        /// Target floating window identifier.
        window_id: u64,
        /// Initial cursor offset relative to window top-left.
        offset: Point,
    },
    /// Clicked window boundary margin to initiate edge resizing.
    ResizeStart {
        /// Target floating window identifier.
        window_id: u64,
        /// Border edge or corner being resized.
        edge: FloatingResizeEdge,
    },
}

/// Evaluates mouse clicks against a slice of floating windows ordered from back to front.
/// Checks front-to-back (reverse iteration) for highest visual priority:
/// 1. Dock-back button `⤢` (top-right)
/// 2. Close button `✖` (top-right)
/// 3. Tab pills (selecting active tab)
/// 4. Title bar background (initiates window drag)
/// 5. Window boundary resize margins (initiates edge/corner resize)
pub fn evaluate_floating_window_click<Tab: Clone, V: TabViewer<Tab>>(
    windows: &[FloatingWindow<Tab>],
    viewer: &V,
    cursor: Point,
    resize_margin: f32,
    title_bar_height: f32,
) -> Option<FloatingWindowClickAction<Tab>> {
    for win in windows.iter().rev() {
        let bar_rect = Rect::new(win.rect.x, win.rect.y, win.rect.width, title_bar_height);

        // 1. Dock-back button `⤢`
        let dock_btn_rect = Rect::new(bar_rect.right() - 56.0, bar_rect.y + 2.0, 20.0, 22.0);
        if dock_btn_rect.contains_point(cursor) {
            return Some(FloatingWindowClickAction::DockBack { window_id: win.id });
        }

        // 2. Close button `✖`
        let close_btn_rect = Rect::new(bar_rect.right() - 32.0, bar_rect.y + 2.0, 20.0, 22.0);
        if close_btn_rect.contains_point(cursor) {
            return Some(FloatingWindowClickAction::Close { window_id: win.id });
        }

        // 3. Tab pills
        let mut current_tab_x = bar_rect.x;
        for (leaf_id, node) in win.tree.iter() {
            if let DockNode::Leaf { tabs, .. } = node {
                for (tab_idx, tab) in tabs.iter().enumerate() {
                    let has_atlas_icon = viewer.atlas_icon(tab).is_some();
                    let icon_w = if has_atlas_icon { 18.0 } else { 0.0 };
                    let title_text = if has_atlas_icon {
                        viewer.raw_title(tab)
                    } else {
                        viewer.title(tab)
                    };
                    let char_count = title_text.chars().count();
                    let tab_w =
                        (16.0 + icon_w + (char_count as f32) * 6.8 + 14.0).clamp(52.0, 160.0);
                    let tab_rect = Rect::new(current_tab_x, bar_rect.y, tab_w, title_bar_height);

                    if tab_rect.contains_point(cursor) {
                        return Some(FloatingWindowClickAction::TabSelected {
                            window_id: win.id,
                            leaf_id,
                            tab_index: tab_idx,
                            tab: tab.clone(),
                            window_rect: win.rect,
                        });
                    }
                    current_tab_x += tab_w;
                }
            }
        }

        // 4. Title bar drag
        if bar_rect.contains_point(cursor) {
            return Some(FloatingWindowClickAction::TitleDragStart {
                window_id: win.id,
                offset: Point::new(cursor.x - win.rect.x, cursor.y - win.rect.y),
            });
        }

        // 5. Window boundary edge resize
        if let Some(edge) = detect_resize_edge(win.rect, cursor, resize_margin) {
            return Some(FloatingWindowClickAction::ResizeStart {
                window_id: win.id,
                edge,
            });
        }
    }

    None
}

/// Builds the complete native Iris UI floating window widget hierarchy in the UI tree.
/// For each floating window:
/// 1. Creates a container node with `WidgetRole::FloatingWindow` (ensuring correct z-ordering in `UiLayer::Floating`).
/// 2. Adds the base background quad with drop shadow, border, and rounded corners.
/// 3. Adds the title bar header quad.
/// 4. Adds dock-back `⤢` and close `✖` control buttons.
/// 5. Adds header baseline divider.
/// 6. Renders tab pill items with icons, titles, and active indicator line.
/// Returns:
/// - List of window bounding rectangles (for hardware occlusion culling).
/// - Mapping from active `Tab` to its floating window container [`WidgetId`].
pub fn build_floating_windows_layer<Tab: Clone, V: TabViewer<Tab>>(
    tree: &mut UiTree,
    root_id: WidgetId,
    windows: &[FloatingWindow<Tab>],
    viewer: &V,
    cursor_pos: Point,
    style: &FloatingWindowStyle,
) -> (Vec<Rect>, Vec<(Tab, WidgetId)>) {
    let mut rects = Vec::with_capacity(windows.len());
    let mut active_tab_containers = Vec::new();

    for win in windows {
        let win_rect = win.rect;
        rects.push(win_rect);

        // 1. Floating Window Container Node
        let win_container = tree.create_node();
        if let Some(node) = tree.get_mut(win_container) {
            node.set_name(format!("FloatingWindow_{}", win.id));
            node.set_role(WidgetRole::FloatingWindow);
            node.computed_rect = win_rect;
            node.style = Style::new().width(win.rect.width).height(win.rect.height);
        }
        let _ = tree.add_child(root_id, win_container);

        // 2. Base Background Quad with Shadow and Border
        let bg_id = tree.create_node();
        if let Some(node) = tree.get_mut(bg_id) {
            node.set_name("FloatingWindowBase");
            node.computed_rect = win_rect;
            node.style = Style::new()
                .background(style.background)
                .border(style.border_width, style.border_color)
                .border_radius(style.corner_radius)
                .box_shadow(
                    0.0,
                    style.shadow_offset_y,
                    style.shadow_blur,
                    style.shadow_color,
                );
        }
        let _ = tree.add_child(win_container, bg_id);

        // 3. Tab Bar Header Quad
        let bar_rect = Rect::new(
            win.rect.x,
            win.rect.y,
            win.rect.width,
            style.title_bar_height,
        );
        let bar_id = tree.create_node();
        if let Some(node) = tree.get_mut(bar_id) {
            node.set_name("FloatingWindowTabBar");
            node.computed_rect = bar_rect;
            node.style = Style::new()
                .background(style.title_bar_background)
                .border_radius(style.corner_radius);
        }
        let _ = tree.add_child(win_container, bar_id);

        // 4. Tab Bar Buttons (Dock-back `⤢` and Close `✖`)
        let dock_btn_rect = Rect::new(bar_rect.right() - 56.0, bar_rect.y + 2.0, 20.0, 22.0);
        let is_dock_hovered = dock_btn_rect.contains_point(cursor_pos);
        let dock_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(dock_btn_id) {
            node.set_name("FloatingWindowDockBtn");
            node.computed_rect = dock_btn_rect;
            node.text = Some("⤢".to_string());
            node.font_size = 12.0;
            node.text_color = if is_dock_hovered {
                Color::WHITE
            } else {
                style.accent_color
            };
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(win_container, dock_btn_id);

        let close_btn_rect = Rect::new(bar_rect.right() - 32.0, bar_rect.y + 2.0, 20.0, 22.0);
        let is_close_hovered = close_btn_rect.contains_point(cursor_pos);
        let close_btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(close_btn_id) {
            node.set_name("FloatingWindowCloseBtn");
            node.computed_rect = close_btn_rect;
            node.text = Some("✖".to_string());
            node.font_size = 10.0;
            node.text_color = if is_close_hovered {
                style.close_btn_hover
            } else {
                style.close_btn_idle
            };
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(win_container, close_btn_id);

        // 5. Header Baseline Divider
        let baseline_id = tree.create_node();
        if let Some(line) = tree.get_mut(baseline_id) {
            line.set_name("FloatingWindowHeaderBaseline");
            line.computed_rect =
                Rect::new(bar_rect.x, bar_rect.bottom() - 1.0, bar_rect.width, 1.0);
            line.style = Style::new().background(style.baseline_color);
        }
        let _ = tree.add_child(win_container, baseline_id);

        // 6. Render Tab Titles & Active Highlights
        let mut current_tab_x = bar_rect.x;
        for (_leaf_id, node) in win.tree.iter() {
            if let DockNode::Leaf { tabs, active_tab } = node {
                for (tab_idx, tab) in tabs.iter().enumerate() {
                    let is_active = tab_idx == *active_tab;
                    let atlas_icon = viewer.atlas_icon(tab);
                    let title = if atlas_icon.is_some() {
                        viewer.raw_title(tab)
                    } else {
                        viewer.title(tab)
                    };
                    let char_count = title.chars().count();
                    let icon_w = if atlas_icon.is_some() { 18.0 } else { 0.0 };
                    let tab_w =
                        (16.0 + icon_w + (char_count as f32) * 6.8 + 14.0).clamp(52.0, 160.0);
                    let tab_rect =
                        Rect::new(current_tab_x, bar_rect.y, tab_w, style.title_bar_height);
                    let is_tab_hovered = tab_rect.contains_point(cursor_pos);

                    let tab_node_id = tree.create_node();
                    if let Some(tab_node) = tree.get_mut(tab_node_id) {
                        tab_node.set_name("FloatingWindowTab");
                        tab_node.computed_rect = tab_rect;
                        let bg_color = if is_active {
                            style.tab_active_bg
                        } else if is_tab_hovered {
                            style.tab_hover_bg
                        } else {
                            style.tab_idle_bg
                        };
                        tab_node.style = Style::new()
                            .background(bg_color)
                            .corner_radii(CornerRadii::new(5.0, 5.0, 0.0, 0.0));
                    }
                    let _ = tree.add_child(win_container, tab_node_id);

                    if is_active {
                        let active_line_id = tree.create_node();
                        if let Some(line) = tree.get_mut(active_line_id) {
                            line.set_name("FloatingWindowTabActiveLine");
                            line.computed_rect =
                                Rect::new(tab_rect.x, tab_rect.bottom() - 2.0, tab_rect.width, 2.0);
                            line.style = Style::new().background(style.accent_color);
                        }
                        let _ = tree.add_child(win_container, active_line_id);
                    }

                    if let Some(uv) = atlas_icon {
                        let icon_node = tree.create_node();
                        if let Some(node) = tree.get_mut(icon_node) {
                            node.set_name("FloatingWindowTabIcon");
                            node.computed_rect =
                                Rect::new(tab_rect.x + 7.0, tab_rect.y + 5.0, 16.0, 16.0);
                            node.set_texture_uv(uv);
                            let tint = if is_active {
                                style.accent_color
                            } else if is_tab_hovered {
                                Color::WHITE
                            } else {
                                style.text_muted
                            };
                            node.set_texture_tint(tint);
                        }
                        let _ = tree.add_child(win_container, icon_node);
                    }

                    let text_offset_x = if atlas_icon.is_some() {
                        7.0 + 16.0 + 5.0
                    } else {
                        6.0
                    };
                    let text_id = tree.create_node();
                    if let Some(text_node) = tree.get_mut(text_id) {
                        text_node.set_name("FloatingWindowTabTitle");
                        text_node.computed_rect = Rect::new(
                            tab_rect.x + text_offset_x,
                            tab_rect.y + 4.0,
                            (tab_rect.width - text_offset_x - 6.0).max(0.0),
                            18.0,
                        );
                        text_node.text = Some(title);
                        text_node.font_size = 12.0;
                        text_node.text_color = if is_active {
                            style.text_active
                        } else if is_tab_hovered {
                            style.text_hover
                        } else {
                            style.text_muted
                        };
                        text_node.text_align = TextAlign::Left;
                    }
                    let _ = tree.add_child(win_container, text_id);

                    current_tab_x += tab_w;
                }

                if *active_tab < tabs.len() {
                    active_tab_containers.push((tabs[*active_tab].clone(), win_container));
                }
            }
        }
    }

    (rects, active_tab_containers)
}

/// Independent floating panel surface detached from the main dock hierarchy.
#[derive(Debug, Clone)]
pub struct FloatingWindow<T> {
    /// Unique identifier for this floating window.
    pub id: u64,
    /// Human-readable title displayed on the floating window title bar.
    pub title: String,
    /// Absolute bounding rectangle in screen/viewport logical coordinates.
    pub rect: Rect,
    /// Isolated docking tree hosted within this floating window.
    pub tree: DockTree<T>,
    /// Whether this window currently has keyboard/mouse focus.
    pub is_focused: bool,
    /// Whether this window is collapsed or minimized to title bar.
    pub is_minimized: bool,
}

impl<T> FloatingWindow<T> {
    /// Creates a new floating window hosting the specified initial tabs in a single root leaf.
    pub fn new(id: u64, title: impl Into<String>, rect: Rect, initial_tabs: Vec<T>) -> Self {
        let mut tree = DockTree::new();
        if !initial_tabs.is_empty() {
            let root = tree.create_leaf(initial_tabs);
            tree.set_root(root);
        }

        Self {
            id,
            title: title.into(),
            rect,
            tree,
            is_focused: true,
            is_minimized: false,
        }
    }

    /// Creates a new floating window hosting an existing pre-configured [`DockTree`].
    pub fn from_tree(id: u64, title: impl Into<String>, rect: Rect, tree: DockTree<T>) -> Self {
        Self {
            id,
            title: title.into(),
            rect,
            tree,
            is_focused: true,
            is_minimized: false,
        }
    }

    /// Translates the floating window position by the specified drag delta.
    pub fn drag_title(&mut self, delta: Point) {
        self.rect.x += delta.x;
        self.rect.y += delta.y;
    }

    /// Resizes the floating window dimensions, enforcing the specified minimum size.
    pub fn resize(&mut self, delta_w: f32, delta_h: f32, min_size: f32) {
        self.rect.width = (self.rect.width + delta_w).max(min_size);
        self.rect.height = (self.rect.height + delta_h).max(min_size);
    }

    /// Returns the bounding rectangle of the floating window's top title bar.
    pub fn title_bar_rect(&self, title_bar_height: f32) -> Rect {
        Rect::new(self.rect.x, self.rect.y, self.rect.width, title_bar_height)
    }

    /// Returns the interior content area below the title bar available for docking layout.
    pub fn content_rect(&self, title_bar_height: f32) -> Rect {
        let content_h = (self.rect.height - title_bar_height).max(0.0);
        Rect::new(
            self.rect.x,
            self.rect.y + title_bar_height,
            self.rect.width,
            content_h,
        )
    }

    /// Checks if a screen cursor coordinate falls inside this floating window.
    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains_point(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyViewer;
    impl TabViewer<&'static str> for DummyViewer {
        fn title(&self, tab: &&'static str) -> String {
            tab.to_string()
        }
    }

    #[test]
    fn test_floating_window_creation_and_geometry() {
        let mut window = FloatingWindow::new(
            1,
            "Inspector Window",
            Rect::new(100.0, 150.0, 400.0, 300.0),
            vec!["InspectorTab"],
        );

        assert_eq!(window.id, 1);
        assert_eq!(window.title, "Inspector Window");
        assert!(window.contains_point(Point::new(150.0, 200.0)));
        assert!(!window.contains_point(Point::new(50.0, 50.0)));

        // Title bar & Content rect
        let title_rect = window.title_bar_rect(28.0);
        assert_eq!(title_rect, Rect::new(100.0, 150.0, 400.0, 28.0));
        let content_rect = window.content_rect(28.0);
        assert_eq!(content_rect, Rect::new(100.0, 178.0, 400.0, 272.0));

        // Drag title
        window.drag_title(Point::new(50.0, -20.0));
        assert_eq!(window.rect.x, 150.0);
        assert_eq!(window.rect.y, 130.0);

        // Resize with clamp
        window.resize(50.0, -500.0, 100.0);
        assert_eq!(window.rect.width, 450.0);
        assert_eq!(window.rect.height, 100.0); // clamped to min_size
    }

    #[test]
    fn test_detect_resize_edge() {
        let rect = Rect::new(100.0, 100.0, 200.0, 200.0);
        const MARGIN: f32 = 6.0;

        assert_eq!(
            detect_resize_edge(rect, Point::new(102.0, 102.0), MARGIN),
            Some(FloatingResizeEdge::TopLeft)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(298.0, 102.0), MARGIN),
            Some(FloatingResizeEdge::TopRight)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(102.0, 298.0), MARGIN),
            Some(FloatingResizeEdge::BottomLeft)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(298.0, 298.0), MARGIN),
            Some(FloatingResizeEdge::BottomRight)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(102.0, 150.0), MARGIN),
            Some(FloatingResizeEdge::Left)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(298.0, 150.0), MARGIN),
            Some(FloatingResizeEdge::Right)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(200.0, 102.0), MARGIN),
            Some(FloatingResizeEdge::Top)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(200.0, 298.0), MARGIN),
            Some(FloatingResizeEdge::Bottom)
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(200.0, 200.0), MARGIN),
            None
        );
        assert_eq!(
            detect_resize_edge(rect, Point::new(50.0, 50.0), MARGIN),
            None
        );
    }

    #[test]
    fn test_evaluate_floating_window_click_and_layer() {
        let window = FloatingWindow::new(
            1,
            "Console",
            Rect::new(50.0, 50.0, 300.0, 200.0),
            vec!["LogTab"],
        );
        let windows = vec![window];
        let viewer = DummyViewer;

        // Click close button: right() = 350.0, close btn is [350 - 32 = 318, 52, 20, 22]
        let close_action =
            evaluate_floating_window_click(&windows, &viewer, Point::new(325.0, 60.0), 6.0, 26.0);
        assert_eq!(
            close_action,
            Some(FloatingWindowClickAction::Close { window_id: 1 })
        );

        // Click dock back button: dock btn is [350 - 56 = 294, 52, 20, 22]
        let dock_action =
            evaluate_floating_window_click(&windows, &viewer, Point::new(300.0, 60.0), 6.0, 26.0);
        assert_eq!(
            dock_action,
            Some(FloatingWindowClickAction::DockBack { window_id: 1 })
        );

        // Click title bar drag
        let drag_action =
            evaluate_floating_window_click(&windows, &viewer, Point::new(200.0, 60.0), 6.0, 26.0);
        assert_eq!(
            drag_action,
            Some(FloatingWindowClickAction::TitleDragStart {
                window_id: 1,
                offset: Point::new(150.0, 10.0),
            })
        );

        // Build layer
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let (rects, containers) = build_floating_windows_layer(
            &mut tree,
            root,
            &windows,
            &viewer,
            Point::new(0.0, 0.0),
            &FloatingWindowStyle::default(),
        );

        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], Rect::new(50.0, 50.0, 300.0, 200.0));
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].0, "LogTab");
    }
}