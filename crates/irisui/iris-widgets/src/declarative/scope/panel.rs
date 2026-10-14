// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Builder & Autonomous Subtree Lifecycle
//!
//! Provides the fluent [`PanelBuilder`] on [`UiScope`]. Automatically resolves container boundaries,
//! applies vertical flexbox flow and clipping, executes recursive layout upon exit, and evaluates
//! zero-latency hover updates directly using [`iris_core::UiTree::cursor_position`] without requiring
//! consumer coordinates or manual layout finalization calls.
//!

use super::core::UiScope;
use iris_core::{Color, Rect, Style, WidgetId};

/// Canonical default dark theme background fill color for panel containers.
pub const DEFAULT_PANEL_BACKGROUND: Color = Color::rgba(0.08, 0.09, 0.11, 0.98);

/// Canonical default border outline color for panel containers.
pub const DEFAULT_PANEL_BORDER: Color = Color::rgba(0.12, 0.13, 0.16, 0.90);

/// Fluent builder for constructing dockable panels with autonomous layout and optional styling.
pub struct PanelBuilder<'a, 'b> {
    scope: &'a mut UiScope<'b>,
    name: &'static str,
    tag: u64,
    bg: Option<Color>,
    border: Option<(f32, Color)>,
    bounds: Option<Rect>,
}

impl<'a, 'b> PanelBuilder<'a, 'b> {
    /// Creates a new [`PanelBuilder`] initialized with default panel parameters.
    ///
    /// # Arguments
    /// * `scope` - Mutable reference to the parent UI scope.
    /// * `name` - Descriptive debug identifier assigned to the panel node.
    #[inline]
    pub fn new(scope: &'a mut UiScope<'b>, name: &'static str) -> Self {
        Self {
            scope,
            name,
            tag: 0,
            bg: None,
            border: None,
            bounds: None,
        }
    }

    /// Sets the semantic 64-bit interaction tag assigned to the panel root node.
    #[inline]
    #[must_use]
    pub fn tagged(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets the background fill color of the panel container.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    /// Sets the border stroke width and color of the panel container.
    #[inline]
    #[must_use]
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some((width, color));
        self
    }

    /// Explicitly constrains the panel bounding geometry in local physical pixels.
    /// If not specified, the panel automatically adopts the active parent's available geometry.
    #[inline]
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Finalizes the panel container, executes the child builder closure within the panel scope,
    /// and autonomously recalculates recursive subtree layout across the resolved bounds upon exit.
    ///
    /// # Arguments
    /// * `f` - Child builder closure receiving the panel's interior [`UiScope`].
    pub fn show<F>(self, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.show_inner(|scope| {
            f(scope);
        })
        .0
    }

    /// Finalizes the panel container, executes the child builder closure within the panel scope,
    /// and autonomously recalculates recursive subtree layout upon exit, returning both the panel's
    /// [`WidgetId`] and the closure's inner return value.
    ///
    /// # Arguments
    /// * `f` - Child builder closure receiving the panel's interior [`UiScope`].
    pub fn show_inner<F, R>(self, f: F) -> (WidgetId, R)
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let rect = self.bounds.unwrap_or_else(|| self.scope.available_rect());
        let bg = self.bg.unwrap_or(DEFAULT_PANEL_BACKGROUND);
        let mut style = Style::new()
            .flex_col()
            .width(rect.width)
            .height(rect.height)
            .background(bg)
            .clip_children(true);

        if let Some((bw, bc)) = self.border {
            style = style.border(bw, bc);
        } else {
            style = style.border(1.0, DEFAULT_PANEL_BORDER);
        }

        let node_id = self.scope.tree.create_node();
        if let Some(node) = self.scope.tree.get_mut(node_id) {
            node.set_name(self.name);
            node.tag = self.tag;
            node.computed_rect = rect;
            node.interactive = true;
            node.set_style(style);
        }
        let _ = self.scope.tree.add_child(self.scope.parent, node_id);

        let mut child_scope = self.scope.child_scope(node_id);
        let result = f(&mut child_scope);
        crate::declarative::layout_subtree(self.scope.tree, node_id, rect);
        if let Some(cursor) = self.scope.tree.cursor_position() {
            super::hover::update_hover_styles_recursive(self.scope.tree, node_id, cursor);
        }
        (node_id, result)
    }
}

impl<'a> UiScope<'a> {
    /// Initiates a fluent [`PanelBuilder`] for constructing a dockable panel container.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the panel root node.
    #[inline]
    pub fn panel(&mut self, name: &'static str) -> PanelBuilder<'_, 'a> {
        PanelBuilder::new(self, name)
    }
}