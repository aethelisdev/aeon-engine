// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Tooltip Primitives & Declarative Overlay Extension
//!
//! Provides hardware-accelerated declarative tooltips on [`UiScope`],
//! [`WidgetResponse`], and [`TreeRowResponse`].
//!
//! Enforces zero-allocation delayed activation via [`iris_core::tree::UiTree`]'s internal hover timer,
//! smart boundary clamping, elevated non-clipped rendering on [`UiLayer::Tooltip`],
//! and rich modern typography.

use super::core::UiScope;
use crate::declarative::scope::tree_view::TreeRowResponse;
use crate::declarative::types::WidgetResponse;
use iris_core::{Color, Insets, Point, Rect, Style, TextAlign, UiLayer, WidgetId, WidgetRole};

/// Extension trait providing declarative tooltip chaining on UI widget responses.
pub trait TooltipExt {
    /// Attaches a simple text tooltip to this widget response, displayed on [`UiLayer::Tooltip`]
    /// after the autonomous hover delay has elapsed.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    fn tooltip(self, scope: &mut UiScope<'_>, text: impl Into<String>) -> Self;

    /// Attaches a custom UI tooltip builder closure to this widget response, displayed on [`UiLayer::Tooltip`]
    /// after the autonomous hover delay has elapsed.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    fn tooltip_ui<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>);

    /// Alias for [`Self::tooltip`] conforming to egui standard convention.
    fn on_hover_text(self, scope: &mut UiScope<'_>, text: impl Into<String>) -> Self;

    /// Alias for [`Self::tooltip_ui`] conforming to egui standard convention.
    fn on_hover_ui<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>);
}

impl TooltipExt for WidgetResponse {
    #[inline]
    fn tooltip(self, scope: &mut UiScope<'_>, text: impl Into<String>) -> Self {
        let text_str = text.into();
        scope.tooltip_for(self.id, move |ui| {
            ui.tooltip_label(text_str);
        });
        self
    }

    #[inline]
    fn tooltip_ui<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        scope.tooltip_for(self.id, add_contents);
        self
    }

    #[inline]
    fn on_hover_text(self, scope: &mut UiScope<'_>, text: impl Into<String>) -> Self {
        self.tooltip(scope, text)
    }

    #[inline]
    fn on_hover_ui<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.tooltip_ui(scope, add_contents)
    }
}

impl TooltipExt for TreeRowResponse {
    #[inline]
    fn tooltip(self, scope: &mut UiScope<'_>, text: impl Into<String>) -> Self {
        let text_str = text.into();
        scope.tooltip_for(self.id, move |ui| {
            ui.tooltip_label(text_str);
        });
        self
    }

    #[inline]
    fn tooltip_ui<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        scope.tooltip_for(self.id, add_contents);
        self
    }

    #[inline]
    fn on_hover_text(self, scope: &mut UiScope<'_>, text: impl Into<String>) -> Self {
        self.tooltip(scope, text)
    }

    #[inline]
    fn on_hover_ui<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.tooltip_ui(scope, add_contents)
    }
}

impl<'a> UiScope<'a> {
    /// Renders an autonomous tooltip overlay anchored to the specified target if currently active.
    ///
    /// Checks the tree's persistent interaction state to verify if the hover delay threshold has
    /// elapsed for `target_id`. If active, calculates smart viewport placement and renders
    /// the tooltip container on [`UiLayer::Tooltip`].
    pub fn tooltip_for<F>(&mut self, target_id: WidgetId, add_contents: F)
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let target_tag = self.tree.get(target_id).map_or(0, |n| n.tag);
        if !self.tree.is_tooltip_open_for(target_id, target_tag) {
            return;
        }

        let anchor_pos = self
            .tree
            .event_state
            .active_tooltip
            .as_ref()
            .map(|t| t.anchor_pos)
            .or(self.tree.event_state.last_pointer_pos)
            .unwrap_or(Point::ZERO);

        self.render_tooltip_card(anchor_pos, add_contents);
    }

    /// Renders an elevated tooltip card positioned relative to `anchor_pos` on [`UiLayer::Tooltip`].
    ///
    /// Clamps position against window boundaries and bypasses parent clipping scissors.
    pub fn render_tooltip_card<F>(&mut self, anchor_pos: Point, add_contents: F)
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let (window_w, window_h) = if let Some(root_id) = self.tree.root() {
            let r = self
                .tree
                .get(root_id)
                .map(|n| n.computed_rect)
                .unwrap_or(Rect::ZERO);
            if r.width > 0.0 && r.height > 0.0 {
                (r.width, r.height)
            } else {
                (1920.0, 1080.0)
            }
        } else {
            (1920.0, 1080.0)
        };

        // Offset slightly below and to the right of the cursor
        let mut x = anchor_pos.x + 10.0;
        let mut y = anchor_pos.y + 18.0;

        let root_parent = self.tree.root().unwrap_or(self.parent);
        let card_id = self.tree.create_node();
        if let Some(card) = self.tree.get_mut(card_id) {
            card.set_name("AutonomousTooltip");
            card.set_role(WidgetRole::Tooltip);
            card.set_layer(UiLayer::Tooltip);
            card.interactive = false;
            card.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .flex_col()
                    .padding_insets(Insets::new(4.0, 8.0, 4.0, 8.0))
                    .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
                    .border(1.0, Color::rgba(0.24, 0.28, 0.38, 0.85))
                    .border_radius(4.0)
                    .box_shadow(0.0, 3.0, 10.0, Color::rgba(0.0, 0.0, 0.0, 0.55)),
            );
        }
        let _ = self.tree.add_child(root_parent, card_id);

        let mut child_scope = self.child_scope(card_id);
        add_contents(&mut child_scope);

        // Measure dynamic content dimensions and apply boundary clamping
        let width = crate::declarative::layout::measure_width(self.tree, card_id).max(40.0);
        let height = crate::declarative::layout::measure_height(self.tree, card_id).max(20.0);

        if x + width > window_w - 8.0 {
            x = (window_w - width - 8.0).max(4.0);
        }
        if y + height > window_h - 8.0 {
            y = (anchor_pos.y - height - 6.0).max(4.0);
        }

        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, card_id, bounds);
        if let Some(node) = self.tree.get_mut(card_id) {
            node.computed_rect = bounds;
            node.style.inset_left = Some(x);
            node.style.inset_top = Some(y);
        }
    }

    /// Emits a standard single-line or multi-line text label inside a tooltip container.
    pub fn tooltip_label(&mut self, text: impl Into<String>) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TooltipLabel");
            node.set_role(WidgetRole::Tooltip);
            node.set_layer(UiLayer::Tooltip);
            node.interactive = false;
            node.set_text(text.into());
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Left;
            node.text_color = Color::rgba(0.90, 0.92, 0.96, 1.0);
            node.set_style(Style::new().flex_shrink(0.0));
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }
}