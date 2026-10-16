// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Builder & Autonomous Subtree Lifecycle
//!
//! Provides the fluent [`PanelBuilder`] on [`UiScope`]. Automatically resolves container boundaries,
//! applies vertical flexbox flow and clipping, executes recursive layout upon exit, and evaluates
//! zero-latency hover updates directly using [`iris_core::UiTree::cursor_position`] without requiring
//! consumer coordinates or manual layout finalization calls.
//!

use super::containers::InnerResponse;
use super::core::UiScope;
use iris_core::{Color, Rect, Style};

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

    /// Sets the background fill color using the active theme tokens.
    #[inline]
    #[must_use]
    pub fn background_themed<F: FnOnce(&iris_core::Theme) -> Color>(mut self, f: F) -> Self {
        let theme = self.scope.tree.theme();
        self.bg = Some(f(&theme));
        self
    }

    /// Sets the border stroke width and color of the panel container.
    #[inline]
    #[must_use]
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some((width, color));
        self
    }

    /// Sets the border stroke width and color using the active theme tokens.
    #[inline]
    #[must_use]
    pub fn border_themed<F: FnOnce(&iris_core::Theme) -> Color>(
        mut self,
        width: f32,
        f: F,
    ) -> Self {
        let theme = self.scope.tree.theme();
        self.border = Some((width, f(&theme)));
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
    pub fn show<R, F>(self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let rect = self.bounds.unwrap_or_else(|| self.scope.available_rect());
        let theme = self.scope.tree.theme();
        let bg = self.bg.unwrap_or(theme.panel_bg);
        let mut style = Style::new()
            .flex_col()
            .width(rect.width)
            .height(rect.height)
            .background(bg)
            .clip_children(true);

        if let Some((bw, bc)) = self.border {
            style = style.border(bw, bc);
        } else {
            style = style.border(1.0, theme.panel_border);
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

        let panel_seed = crate::declarative::types::combine_seeds(
            self.scope.seed,
            crate::declarative::types::hash_label(self.name),
        );
        let mut child_scope = self.scope.child_scope_seeded(node_id, panel_seed);
        let inner = f(&mut child_scope);
        crate::declarative::layout_subtree(self.scope.tree, node_id, rect);
        if let Some(cursor) = self.scope.tree.cursor_position() {
            super::hover::update_hover_styles_recursive(self.scope.tree, node_id, cursor);
        }
        let response = self.scope.response(node_id);
        InnerResponse::new(response, inner)
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

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_panel_builder_theme_defaults() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let resp = scope.panel("TestThemePanel").show(|_| 123);
        assert_eq!(resp.inner, 123);

        let node = tree.get(resp.response.id).expect("Panel node exists");
        assert_eq!(node.style.background_color, tree.theme().panel_bg);
        assert_eq!(
            node.style.border,
            iris_core::Border::uniform(1.0, tree.theme().panel_border)
        );
    }

    #[test]
    fn test_panel_builder_themed_overrides() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        let resp = scope
            .panel("TestCustomThemedPanel")
            .background_themed(|t| t.card_bg)
            .border_themed(2.0, |t| t.card_border)
            .show(|_| ());

        let node = tree.get(resp.response.id).expect("Panel node exists");
        assert_eq!(node.style.background_color, tree.theme().card_bg);
        assert_eq!(
            node.style.border,
            iris_core::Border::uniform(2.0, tree.theme().card_border)
        );
    }
}