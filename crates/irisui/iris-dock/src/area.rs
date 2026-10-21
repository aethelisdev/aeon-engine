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
use crate::events::{
    handle_floating_window_event, handle_splitter_event, handle_tab_drag_event,
    handle_tab_strip_event,
};
use crate::floating::{
    FloatingWindowClampBounds, FloatingWindowStyle, build_floating_windows_layer,
};
use crate::state::DockState;
use crate::tab_viewer::TabViewer;
use crate::tree::DockNode;
use iris_core::event::UiEvent;
use iris_core::geometry::{Point, Rect};
use iris_core::node::{WidgetCursor, WidgetRole};
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
///             Panel::Hierarchy => { scope.label("Entities"); }
///             Panel::Inspector => { scope.label("Properties"); }
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

            let panel_content = scope.container_rect(
                "DockLeafPanelContent",
                container_style,
                panel_rect,
                |leaf_scope| {
                    viewer.ui(leaf_scope, &tab);
                },
            );
            scope
                .tree_mut()
                .set_boundary(panel_content.response.id, true);
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
                    let floating_content = win_scope.container_rect_with_role(
                        "FloatingLeafPanelContent",
                        container_style,
                        content_rect,
                        WidgetRole::FloatingWindow,
                        |leaf_scope| {
                            viewer.ui(leaf_scope, &tab);
                        },
                    );
                    scope
                        .tree_mut()
                        .set_boundary(floating_content.response.id, true);

                    frame.panel_rects.push((tab, content_rect));
                }
            }
        }

        // 3. Autonomous tab drag-and-drop overlays (compass navigator, drop preview, floating tab badge)
        if let Some(ref drag) = self.state.active_drag {
            let computed = crate::layout::compute_dock_layout(
                &self.state.tree,
                self.bounds,
                self.style.splitter_thickness,
                self.style.tab_bar_height,
            );

            let hovered_leaf = computed
                .leaves
                .iter()
                .find(|leaf| leaf.contains_point(drag.cursor_pos));

            if let Some(leaf) = hovered_leaf {
                let nav_style = crate::navigator::DockNavigatorStyle::default();
                let geometry = crate::navigator::DockNavigatorGeometry::from_content_rect(
                    leaf.content_rect,
                    40.0,
                    4.0,
                );
                let drop_zone = geometry.hit_test(drag.cursor_pos);

                if let Some(zone) = drop_zone {
                    crate::navigator::build_drop_preview_node(
                        scope.tree_mut(),
                        parent_id,
                        leaf.content_rect,
                        zone,
                        &nav_style,
                    );
                }

                crate::navigator::build_dock_navigator_nodes(
                    scope.tree_mut(),
                    parent_id,
                    &geometry,
                    drop_zone,
                    &nav_style,
                );
            }

            let title_str = viewer.title(&drag.tab_data);
            let badge_params = crate::navigator::FloatingTabBadgeParams {
                cursor_pos: drag.cursor_pos,
                title: &title_str,
                icon: None,
            };
            crate::navigator::build_floating_tab_badge(scope.tree_mut(), parent_id, badge_params);
        }

        frame
    }

    /// Orchestrates incoming user interface events across all docking sub-systems:
    /// 1. Floating windows (title dragging, 8-way edge/corner resizing, close, and tab selection).
    /// 2. Active tab drag-and-drop or threshold motion detection.
    /// 3. Tab headers (selection, middle-click close, close button, and overflow menu).
    /// 4. Splitter dividers (drag resizing and hover cursor feedback).
    ///
    /// Returns a [`DockResponse`] indicating event consumption, requested cursor override,
    /// and whether visual layout reconstruction is necessary.
    ///
    /// # Arguments
    /// * `state` - Mutable reference to the central [`DockState`].
    /// * `viewer` - Reference to the tab viewer providing tab titles and custom metrics.
    /// * `event` - The incoming user interface event.
    /// * `workspace_rect` - Absolute screen boundaries of the primary docking area.
    /// * `config` - Interaction parameters configuring divider thickness, header heights, and window clamping.
    pub fn handle_event<V: TabViewer<Tab>>(
        state: &mut DockState<Tab>,
        viewer: &V,
        event: &UiEvent,
        workspace_rect: Rect,
        config: &DockInteractionConfig,
    ) -> DockResponse {
        let mut response = DockResponse::default();

        // 1. Process Floating Windows (highest visual layer)
        let floating_res = handle_floating_window_event(state, viewer, event, config.clamp_bounds);

        if floating_res.consumed {
            response.consumed = true;
        }
        if floating_res.layout_changed {
            response.layout_changed = true;
        }
        if let Some(cursor) = floating_res.cursor {
            response.cursor_override = Some(cursor);
        }

        // Apply floating window lifecycle requests
        if let Some(win_id) = floating_res.closed_window {
            let _ = state.close_floating_window(win_id);
            response.layout_changed = true;
        }
        if let Some(win_id) = floating_res.dock_back_window {
            if let Some(target_leaf) = state.tree.find_first_leaf() {
                let _ = state.dock_floating_window(win_id, target_leaf, crate::DropZone::Center);
            } else {
                let _ = state.close_floating_window(win_id);
            }
            response.layout_changed = true;
        }
        if let Some(pending) = floating_res.pending_tab_drag {
            state.pending_tab_drag = Some(pending);
        }

        if response.consumed {
            return response;
        }

        // 2. Process Tab Drag and Drop (active drag or pending threshold activation)
        let computed_layout = crate::layout::compute_dock_layout_with_viewer(
            &state.tree,
            workspace_rect,
            config.splitter_thickness,
            config.tab_bar_height,
            viewer,
        );

        let mut pending = state.pending_tab_drag;
        let drag_res =
            handle_tab_drag_event(state, event, &mut pending, workspace_rect, &computed_layout);
        state.pending_tab_drag = pending;

        if drag_res.consumed {
            response.consumed = true;
        }
        if drag_res.layout_changed {
            response.layout_changed = true;
        }
        if let Some(cursor) = drag_res.cursor {
            response.cursor_override = Some(cursor);
        }

        if response.consumed {
            return response;
        }

        // Compute current dock chrome hit targets (tab headers, close buttons, chevrons, splitters)
        let targets_frame = crate::events::compute_dock_interaction_targets(
            &state.tree,
            workspace_rect,
            config.splitter_thickness,
            config.tab_bar_height,
            viewer,
        );

        // 3. Process Tab Headers, Close Buttons, and Overflow Menus
        let tab_strip_res = handle_tab_strip_event(
            state,
            event,
            &targets_frame.tab_targets,
            &targets_frame.close_targets,
            &targets_frame.chevron_targets,
            &[],
        );

        if tab_strip_res.consumed {
            response.consumed = true;
        }
        if tab_strip_res.layout_changed {
            response.layout_changed = true;
        }
        if let Some(cursor) = tab_strip_res.cursor {
            response.cursor_override = Some(cursor);
        }
        if let Some((leaf, tab_index, tab, press_pos, leaf_rect)) = tab_strip_res.pending_drag {
            state.pending_tab_drag = Some(crate::events::PendingTabDrag {
                leaf,
                tab_index,
                tab,
                press_pos,
                leaf_rect,
                floating_window_id: None,
            });
        }

        if response.consumed {
            return response;
        }

        // 4. Process Splitter Dividers
        let splitter_res = handle_splitter_event(state, event, &targets_frame.splitter_targets);

        if splitter_res.consumed {
            response.consumed = true;
        }
        if splitter_res.layout_changed {
            response.layout_changed = true;
        }
        if let Some(cursor) = splitter_res.cursor {
            response.cursor_override = Some(cursor);
        }

        response
    }
}

/// High-level response summarizing the results of processing an event in [`DockArea::handle_event`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DockResponse {
    /// Whether the event was consumed by the docking hierarchy, stopping propagation.
    pub consumed: bool,
    /// Requested cursor override (e.g. `ColResize`, `RowResize`, `NwseResize`, `Grabbing`, `Pointer`).
    pub cursor_override: Option<WidgetCursor>,
    /// Whether docking layout or floating window geometry changed, requiring visual rebuild.
    pub layout_changed: bool,
}

/// Configuration parameters for dock event handling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockInteractionConfig {
    /// Thickness of split dividers in logical pixels. Defaults to 4.0.
    pub splitter_thickness: f32,
    /// Height of tab headers in logical pixels. Defaults to 26.0.
    pub tab_bar_height: f32,
    /// Screen clamping boundaries for floating windows (top inset, bottom statusbar inset, min visible margin).
    pub clamp_bounds: FloatingWindowClampBounds,
}

impl Default for DockInteractionConfig {
    fn default() -> Self {
        Self {
            splitter_thickness: 4.0,
            tab_bar_height: 26.0,
            clamp_bounds: FloatingWindowClampBounds::default(),
        }
    }
}

#[cfg(test)]
#[path = "area_tests.rs"]
mod tests;