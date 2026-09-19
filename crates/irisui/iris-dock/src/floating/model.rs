// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Data models, visual styling parameters, and interaction evaluation for floating windows.
//!
//! Provides the primary [`FloatingWindow`] container structure, click action evaluators,
//! and style metrics governing title bar heights, shadows, and themes.

use crate::floating::cursor::{FloatingResizeEdge, detect_resize_edge};
use crate::tab_viewer::TabViewer;
use crate::tree::{DockNode, DockNodeId, DockTree};
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};

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

impl FloatingWindowStyle {
    /// Canonical default title bar height in logical pixels.
    pub const DEFAULT_TITLE_BAR_HEIGHT: f32 = 26.0;
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
            title_bar_height: Self::DEFAULT_TITLE_BAR_HEIGHT,
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
    #[inline]
    pub fn title_bar_rect(&self, title_bar_height: f32) -> Rect {
        Rect::new(self.rect.x, self.rect.y, self.rect.width, title_bar_height)
    }

    /// Returns the interior content area below the title bar available for docking layout.
    #[inline]
    pub fn content_rect(&self, title_bar_height: f32) -> Rect {
        let content_h = (self.rect.height - title_bar_height).max(0.0);
        Rect::new(
            self.rect.x,
            self.rect.y + title_bar_height,
            self.rect.width,
            content_h,
        )
    }

    /// Returns the interior content area using the default canonical title bar height.
    #[inline]
    pub fn default_content_rect(&self) -> Rect {
        self.content_rect(FloatingWindowStyle::DEFAULT_TITLE_BAR_HEIGHT)
    }

    /// Checks if a screen cursor coordinate falls inside this floating window.
    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        self.rect.contains_point(point)
    }
}

/// Finds the content rectangle of the floating window currently hosting the specified active tab.
///
/// Searches across all provided floating windows. If a window hosts an active leaf tab matching `tab`,
/// returns the content rectangle positioned directly beneath its title bar.
pub fn find_active_tab_content_rect<Tab: PartialEq>(
    windows: &[FloatingWindow<Tab>],
    tab: &Tab,
    title_bar_height: f32,
) -> Option<Rect> {
    windows.iter().find_map(|window| {
        for (_, node) in window.tree.iter() {
            if let DockNode::Leaf { tabs, active_tab } = node
                && tabs.get(*active_tab).is_some_and(|active| active == tab)
            {
                return Some(window.content_rect(title_bar_height));
            }
        }
        None
    })
}

/// Evaluates mouse clicks against a slice of floating windows ordered from back to front.
///
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
    fn test_floating_window_creation_and_content_rect() {
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

        // Default content rect
        let def_rect = window.default_content_rect();
        assert_eq!(def_rect, Rect::new(100.0, 176.0, 400.0, 274.0));

        // Drag title
        window.drag_title(Point::new(50.0, -20.0));
        assert_eq!(window.rect.x, 150.0);
        assert_eq!(window.rect.y, 130.0);

        // Resize with clamp
        window.resize(50.0, -500.0, 100.0);
        assert_eq!(window.rect.width, 450.0);
        assert_eq!(window.rect.height, 100.0);
    }

    #[test]
    fn test_find_active_tab_content_rect() {
        let windows = vec![
            FloatingWindow::new(
                1,
                "Win1",
                Rect::new(10.0, 20.0, 300.0, 200.0),
                vec!["TabA", "TabB"],
            ),
            FloatingWindow::new(2, "Win2", Rect::new(50.0, 60.0, 400.0, 300.0), vec!["TabC"]),
        ];

        let rect_a = find_active_tab_content_rect(&windows, &"TabA", 26.0);
        assert_eq!(rect_a, Some(Rect::new(10.0, 46.0, 300.0, 174.0)));

        let rect_c = find_active_tab_content_rect(&windows, &"TabC", 26.0);
        assert_eq!(rect_c, Some(Rect::new(50.0, 86.0, 400.0, 274.0)));

        let rect_missing = find_active_tab_content_rect(&windows, &"TabZ", 26.0);
        assert_eq!(rect_missing, None);
    }

    #[test]
    fn test_evaluate_floating_window_click() {
        let window = FloatingWindow::new(
            1,
            "Console",
            Rect::new(50.0, 50.0, 300.0, 200.0),
            vec!["LogTab"],
        );
        let windows = vec![window];
        let viewer = DummyViewer;

        // Click close button
        let close_action =
            evaluate_floating_window_click(&windows, &viewer, Point::new(325.0, 60.0), 6.0, 26.0);
        assert_eq!(
            close_action,
            Some(FloatingWindowClickAction::Close { window_id: 1 })
        );

        // Click dock back button
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
    }
}