// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # High-Level Declarative Docking Area Runner (`DockArea`)
//!
//! Provides a batteries-included, `egui_dock`-style all-in-one docking container.
//! Computes binary split layout trees, renders the docking chrome (tab header strips,
//! active accent lines, close buttons, splitter dividers, and overflow chevrons),
//! establishes layout-clipped content sub-scopes for all active leaf panes, and
//! dispatches into [`TabViewer::ui`] for each tab pane.
//!

use crate::chrome::{DockChromeFrame, DockChromeParams, DockChromeStyle, build_dock_chrome};
use crate::floating::{FloatingWindowStyle, build_floating_windows_layer};
use crate::state::DockState;
use crate::tab_viewer::TabViewer;
use crate::tree::DockNode;
use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetRole;
use iris_core::style::Style;
use iris_widgets::UiScope;

/// High-level declarative runner managing and rendering a complete dock hierarchy.
///
/// Encapsulates layout computation, chrome rendering, pane container isolation,
/// and automated dispatching of active tab content via [`TabViewer::ui`].
///
/// # Example
/// ```rust,no_run
/// use iris_dock::{DockArea, DockState, DockTree, TabViewer};
/// use iris_widgets::UiScope;
/// use iris_core::Rect;
///
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// enum Panel { Hierarchy, Inspector }
///
/// struct MyViewer;
/// impl TabViewer<Panel> for MyViewer {
///     fn title(&self, tab: &Panel) -> String {
///         match tab {
///             Panel::Hierarchy => "Hierarchy".into(),
///             Panel::Inspector => "Inspector".into(),
///         }
///     }
///     fn ui(&mut self, scope: &mut UiScope<'_>, tab: &Panel) {
///         match tab {
///             Panel::Hierarchy => { scope.label("Entities", 12.0, iris_core::Color::WHITE, iris_core::TextAlign::Left); }
///             Panel::Inspector => { scope.label("Properties", 12.0, iris_core::Color::WHITE, iris_core::TextAlign::Left); }
///         }
///     }
/// }
///
/// fn render_workspace(scope: &mut UiScope<'_>, state: &mut DockState<Panel>) {
///     let mut viewer = MyViewer;
///     DockArea::new(state)
///         .bounds(Rect::new(0.0, 0.0, 800.0, 600.0))
///         .show(scope, &mut viewer);
/// }
/// ```
pub struct DockArea<'a, Tab> {
    state: &'a DockState<Tab>,
    bounds: Rect,
    style: DockChromeStyle,
    cursor_pos: Point,
    is_cursor_occluded: bool,
}

impl<'a, Tab: Clone + Copy + PartialEq> DockArea<'a, Tab> {
    /// Initializes a new [`DockArea`] managing the specified mutable [`DockState`].
    ///
    /// # Arguments
    /// * `state` - Mutable reference to the persistent docking state holding the split tree.
    pub fn new(state: &'a mut DockState<Tab>) -> Self {
        Self {
            state: &*state,
            bounds: Rect::ZERO,
            style: DockChromeStyle::default(),
            cursor_pos: Point::new(-1.0, -1.0),
            is_cursor_occluded: false,
        }
    }

    /// Initializes a new [`DockArea`] reading the specified shared [`DockState`].
    pub fn from_ref(state: &'a DockState<Tab>) -> Self {
        Self {
            state,
            bounds: Rect::ZERO,
            style: DockChromeStyle::default(),
            cursor_pos: Point::new(-1.0, -1.0),
            is_cursor_occluded: false,
        }
    }

    /// Sets the available physical rectangular boundaries for the docking workspace.
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    /// Overrides default visual metrics and colors with a custom [`DockChromeStyle`].
    #[must_use]
    pub fn style(mut self, style: DockChromeStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the current mouse cursor coordinates for tab hover and splitter interaction highlights.
    #[must_use]
    pub fn cursor_pos(mut self, cursor_pos: Point) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Sets whether the cursor is currently occluded by an active modal, popup, or floating window.
    #[must_use]
    pub fn cursor_occluded(mut self, is_occluded: bool) -> Self {
        self.is_cursor_occluded = is_occluded;
        self
    }

    /// Renders the complete docking workspace into the provided [`UiScope`].
    ///
    /// Performs full orchestration in a single high-level call:
    /// 1. Computes binary tree layout splits, leaf geometries, and partition splitters.
    /// 2. Renders all chrome elements: tab headers, close buttons, baselines, and splitters.
    /// 3. For each active leaf panel, allocates an isolated container with clipping enabled,
    ///    and executes [`TabViewer::ui`] with a scoped child [`UiScope`].
    ///
    /// Returns the generated [`DockChromeFrame<Tab>`] containing all allocated hit targets.
    pub fn show<V: TabViewer<Tab>>(
        self,
        scope: &mut UiScope<'_>,
        viewer: &mut V,
    ) -> DockChromeFrame<Tab> {
        let is_dragging_splitter = self.state.active_splitter.is_some();
        let active_splitter_node = self.state.active_splitter.as_ref().map(|s| s.node_id);
        let is_dragging_tab = self.state.active_drag.is_some();

        let params = DockChromeParams {
            dock_tree: &self.state.tree,
            workspace_rect: self.bounds,
            cursor_pos: self.cursor_pos,
            is_cursor_occluded: self.is_cursor_occluded,
            is_dragging_splitter,
            active_splitter_node,
            is_dragging_tab,
            viewer,
            style: &self.style,
        };

        let parent_id = scope.parent();
        let mut frame = build_dock_chrome(scope.tree_mut(), parent_id, &params);

        // 1. Render each active docked leaf tab's content into an isolated, clipped panel scope
        for &(tab, panel_rect) in &frame.panel_rects {
            if panel_rect.width <= 0.0 || panel_rect.height <= 0.0 {
                continue;
            }

            let bg_color = if viewer.clear_background(&tab) {
                iris_core::Color::TRANSPARENT
            } else {
                self.style.panel_bg
            };

            let container_style = Style::new()
                .position_absolute()
                .left(panel_rect.x)
                .top(panel_rect.y)
                .width(panel_rect.width)
                .height(panel_rect.height)
                .flex_col()
                .background(bg_color)
                .clip_children(true);

            let container_id =
                scope.container_named("DockLeafPanelContent", container_style, |leaf_scope| {
                    viewer.ui(leaf_scope, &tab);
                });

            iris_widgets::layout_subtree(scope.tree_mut(), container_id, panel_rect);
        }

        // 2. Render all floating windows: frame chrome + active tab content
        if !self.state.floating_windows.is_empty() {
            let floating_style = FloatingWindowStyle::default();
            let (floating_rects, containers) = build_floating_windows_layer(
                scope.tree_mut(),
                parent_id,
                &self.state.floating_windows,
                viewer,
                self.cursor_pos,
                &floating_style,
            );
            frame.floating_rects = floating_rects;

            for win in &self.state.floating_windows {
                let content_rect = win.default_content_rect();
                if content_rect.width <= 0.0 || content_rect.height <= 0.0 {
                    continue;
                }

                // Locate the active tab in this floating window
                let active_tab_opt = win.tree.iter().find_map(|(_, node)| {
                    if let DockNode::Leaf { tabs, active_tab } = node {
                        tabs.get(*active_tab).cloned()
                    } else {
                        None
                    }
                });

                if let Some(tab) = active_tab_opt {
                    let bg_color = if viewer.clear_background(&tab) {
                        iris_core::Color::TRANSPARENT
                    } else {
                        self.style.panel_bg
                    };

                    let container_style = Style::new()
                        .position_absolute()
                        .left(content_rect.x)
                        .top(content_rect.y)
                        .width(content_rect.width)
                        .height(content_rect.height)
                        .flex_col()
                        .background(bg_color)
                        .clip_children(true);

                    // Find corresponding floating window container node in the UI tree
                    let win_container = containers
                        .iter()
                        .find(|(t, _)| t == &tab)
                        .map(|(_, cid)| *cid)
                        .unwrap_or(parent_id);

                    // Attach content container directly to the floating window so it inherits UiLayer::Floating
                    let mut win_scope = scope.child_scope(win_container);
                    let container_id = win_scope.container_named(
                        "FloatingLeafPanelContent",
                        container_style,
                        |leaf_scope| {
                            viewer.ui(leaf_scope, &tab);
                        },
                    );

                    // Mark container with WidgetRole::FloatingWindow to guarantee proper layer z-ordering
                    if let Some(node) = scope.tree_mut().get_mut(container_id) {
                        node.set_role(WidgetRole::FloatingWindow);
                    }

                    iris_widgets::layout_subtree(scope.tree_mut(), container_id, content_rect);

                    frame.panel_rects.push((tab, content_rect));
                }
            }
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{DockTree, SplitDirection};
    use iris_core::{Color, UiTree};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestPanel {
        Hierarchy,
        Inspector,
        Console,
    }

    struct MockViewer {
        rendered_hierarchy: bool,
        rendered_inspector: bool,
        rendered_console: bool,
    }

    impl TabViewer<TestPanel> for MockViewer {
        fn title(&self, tab: &TestPanel) -> String {
            match tab {
                TestPanel::Hierarchy => "Scene Hierarchy".into(),
                TestPanel::Inspector => "Entity Inspector".into(),
                TestPanel::Console => "Debug Console".into(),
            }
        }

        fn ui(&mut self, scope: &mut UiScope<'_>, tab: &TestPanel) {
            match tab {
                TestPanel::Hierarchy => {
                    self.rendered_hierarchy = true;
                    scope.label(
                        "Entities List",
                        11.0,
                        Color::WHITE,
                        iris_core::TextAlign::Left,
                    );
                }
                TestPanel::Inspector => {
                    self.rendered_inspector = true;
                    scope.label(
                        "Component Fields",
                        11.0,
                        Color::WHITE,
                        iris_core::TextAlign::Left,
                    );
                }
                TestPanel::Console => {
                    self.rendered_console = true;
                    scope.label(
                        "System Logs",
                        11.0,
                        Color::WHITE,
                        iris_core::TextAlign::Left,
                    );
                }
            }
        }
    }

    #[test]
    fn test_dock_area_orchestration() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut scope = UiScope::new(&mut tree, root);

        let mut dock_tree = DockTree::new();
        let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy, TestPanel::Console]);
        dock_tree.set_root(center);
        let (_left, _right) = dock_tree
            .split_ordered(
                center,
                SplitDirection::Horizontal,
                0.3,
                vec![TestPanel::Inspector],
                true,
            )
            .expect("Split succeeds");

        let mut dock_state = DockState::new(dock_tree);
        let mut viewer = MockViewer {
            rendered_hierarchy: false,
            rendered_inspector: false,
            rendered_console: false,
        };

        let workspace_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
        let frame = DockArea::new(&mut dock_state)
            .bounds(workspace_rect)
            .show(&mut scope, &mut viewer);

        // Verify that chrome frame was generated with panels and targets
        assert_eq!(frame.panel_rects.len(), 2);
        assert!(!frame.tab_targets.is_empty());
        assert!(!frame.splitter_targets.is_empty());

        // Verify that viewer.ui was invoked for all active tabs
        assert!(viewer.rendered_hierarchy, "Hierarchy UI must be invoked");
        assert!(viewer.rendered_inspector, "Inspector UI must be invoked");
        assert!(
            !viewer.rendered_console,
            "Console UI must not be invoked as it is not in the tree"
        );
    }

    #[test]
    fn test_dock_area_floating_window_content_orchestration() {
        use crate::floating::FloatingWindow;
        use iris_core::UiLayer;

        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut scope = UiScope::new(&mut tree, root);

        let mut dock_tree = DockTree::new();
        let center = dock_tree.create_leaf(vec![TestPanel::Hierarchy]);
        dock_tree.set_root(center);

        let mut dock_state = DockState::new(dock_tree);
        let floating_win = FloatingWindow::new(
            101,
            "Inspector",
            Rect::new(100.0, 100.0, 300.0, 400.0),
            vec![TestPanel::Inspector],
        );
        dock_state.floating_windows.push(floating_win);

        let mut viewer = MockViewer {
            rendered_hierarchy: false,
            rendered_inspector: false,
            rendered_console: false,
        };

        let workspace_rect = Rect::new(0.0, 0.0, 1000.0, 600.0);
        let frame = DockArea::new(&mut dock_state)
            .bounds(workspace_rect)
            .show(&mut scope, &mut viewer);

        assert!(
            viewer.rendered_hierarchy,
            "Docked hierarchy must be rendered"
        );
        assert!(
            viewer.rendered_inspector,
            "Floating inspector must be rendered"
        );
        assert_eq!(frame.floating_rects.len(), 1);

        // Find the FloatingLeafPanelContent node in the tree
        let (_id, floating_content_node) = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("FloatingLeafPanelContent"))
            .expect("FloatingLeafPanelContent node must exist");

        // Verify layer is UiLayer::Floating so it is never occluded by FloatingWindowBase
        assert_eq!(
            floating_content_node.layer,
            UiLayer::Floating,
            "Floating panel content must be in UiLayer::Floating"
        );

        // Verify parent is the FloatingWindow container, not the root
        let parent_id = floating_content_node.parent.expect("must have parent");
        let parent_node = tree.get(parent_id).expect("parent node exists");
        assert_eq!(
            parent_node.name.as_deref(),
            Some("FloatingWindow_101"),
            "Floating content must be a child of FloatingWindow_101 container"
        );
    }
}