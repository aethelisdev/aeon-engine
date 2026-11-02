// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Builder & Autonomous Subtree Lifecycle
//!
//! Provides the fluent [`PanelBuilder`] on [`UiScope`]. Automatically resolves container boundaries,
//! applies vertical flexbox flow and clipping, executes recursive layout upon exit, and evaluates
//! zero-latency hover updates directly using [`iris_core::UiTree::cursor_position`] without requiring
//! consumer coordinates or manual layout finalization calls.

use super::containers::InnerResponse;
use super::core::UiScope;
use crate::scroll_area::{ScrollAreaStyle, ScrollBarGeometry};
use iris_core::node::WidgetRole;
use iris_core::{Color, Rect, Style};

/// Fluent builder for constructing dockable panels with autonomous layout and optional styling.
pub struct PanelBuilder<'a, 'b> {
    scope: &'a mut UiScope<'b>,
    name: &'static str,
    tag: u64,
    bg: Option<Color>,
    border: Option<(f32, Color)>,
    bounds: Option<Rect>,
    scrollable: bool,
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
            scrollable: false,
        }
    }

    /// Enables or disables autonomous vertical scrolling with auto-appearing scrollbar.
    ///
    /// When enabled, if panel contents overflow the allocated vertical bounds, a hardware-clipped
    /// viewport and autonomous GPU SDF scrollbar indicator are emitted automatically.
    #[inline]
    #[must_use]
    pub fn scrollable(mut self, enabled: bool) -> Self {
        self.scrollable = enabled;
        self
    }

    /// Convenience shortcut enabling autonomous vertical scrolling (`overflow-y: auto`).
    #[inline]
    #[must_use]
    pub fn auto_scroll(self) -> Self {
        self.scrollable(true)
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

        if self.scrollable {
            let panel_tag = if self.tag != 0 {
                self.tag
            } else {
                crate::declarative::types::hash_label(self.name)
            };
            let scroll_y = self.scope.tree.scroll_offset(panel_tag);

            let vp_id = self.scope.tree.create_node();
            if let Some(node) = self.scope.tree.get_mut(vp_id) {
                node.set_name("PanelViewport");
                node.tag = panel_tag;
                node.role = WidgetRole::ScrollAreaViewport;
                node.interactive = true;
                node.computed_rect = rect;
                node.style.scroll_offset_y = scroll_y;
                node.set_style(
                    Style::new()
                        .flex_col()
                        .width(rect.width)
                        .flex_grow(1.0)
                        .clip_children(true)
                        .scroll_offset_y(scroll_y),
                );
            }
            let _ = self.scope.tree.add_child(node_id, vp_id);

            let mut child_scope = self.scope.child_scope_seeded(vp_id, panel_seed);
            let inner = f(&mut child_scope);

            let content_h = crate::declarative::measure_content_height(self.scope.tree, vp_id);
            let max_scroll = (content_h - rect.height).max(0.0);
            let safe_scroll = if max_scroll > 0.0 {
                scroll_y.clamp(0.0, max_scroll)
            } else {
                0.0
            };

            self.scope.tree.set_max_scroll(panel_tag, max_scroll);
            self.scope.tree.set_scroll_offset(panel_tag, safe_scroll);

            if let Some(node) = self.scope.tree.get_mut(vp_id) {
                node.content_size = iris_core::geometry::Size::new(rect.width, content_h);
                node.style.scroll_offset_y = safe_scroll;
            }

            if max_scroll > 0.0 {
                let track_tag = panel_tag.wrapping_add(0x1000);
                let thumb_tag = panel_tag.wrapping_add(0x2000);
                let scroll_style = ScrollAreaStyle::dark_default();

                if let Some(geom) =
                    ScrollBarGeometry::compute_vertical(rect, content_h, safe_scroll, &scroll_style)
                {
                    let is_dragging =
                        self.scope
                            .tree
                            .event_state
                            .pointer_capture
                            .is_some_and(|cid| {
                                self.scope.tree.get(cid).is_some_and(|n| n.tag == thumb_tag)
                            });
                    let cursor_pos = self.scope.tree.event_state.last_pointer_pos;

                    let mut panel_scope = self.scope.child_scope(node_id);
                    panel_scope.scrollbar_vertical(
                        geom,
                        rect,
                        is_dragging,
                        cursor_pos,
                        track_tag,
                        thumb_tag,
                    );
                }
            }

            crate::declarative::layout_subtree(self.scope.tree, vp_id, rect);
            if let Some(cursor) = self.scope.tree.cursor_position() {
                super::hover::update_hover_styles_recursive(self.scope.tree, vp_id, cursor);
            }
            let response = self.scope.response(node_id);
            InnerResponse::new(response, inner)
        } else {
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
        let (resp, pid) = {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.panel("TestThemePanel").show(|_| 123);
            let pid = resp.response.id;
            (resp, pid)
        };
        assert_eq!(resp.inner, 123);

        let node = tree.get(pid).expect("Panel node exists");
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
        let pid = {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope
                .panel("TestCustomThemedPanel")
                .background_themed(|t| t.card_bg)
                .border_themed(2.0, |t| t.card_border)
                .show(|_| ());
            resp.response.id
        };

        let node = tree.get(pid).expect("Panel node exists");
        assert_eq!(node.style.background_color, tree.theme().card_bg);
        assert_eq!(
            node.style.border,
            iris_core::Border::uniform(2.0, tree.theme().card_border)
        );
    }

    #[test]
    fn test_panel_builder_autonomous_scroll_overflow() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let tag = 0xAA01;
        let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);

        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = scope
                .panel("TestScrollPanel")
                .tagged(tag)
                .bounds(bounds)
                .auto_scroll()
                .show(|ui| {
                    // 10 items of 40px = 400px > 200px
                    for _ in 0..10 {
                        ui.empty_box_passive_named("Item", Style::new().height(40.0));
                    }
                });
        }

        assert_eq!(tree.max_scroll(tag), 200.0);
        let has_thumb = tree
            .iter()
            .any(|(_, n)| n.role == WidgetRole::ScrollBarThumb);
        let has_track = tree
            .iter()
            .any(|(_, n)| n.role == WidgetRole::ScrollBarTrack);
        assert!(has_thumb, "Must emit scrollbar thumb when overflowing");
        assert!(has_track, "Must emit scrollbar track when overflowing");

        // Simulate mouse wheel scroll
        let scroll_event = iris_core::UiEvent::MouseScroll {
            delta_x: 0.0,
            delta_y: -50.0,
            point: iris_core::geometry::Point::new(100.0, 100.0),
        };
        let disp = tree.dispatch_event(&scroll_event);
        assert!(disp.consumed);
        assert_eq!(tree.scroll_offset(tag), 50.0);
    }

    #[test]
    fn test_panel_builder_no_scrollbar_when_content_fits() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let tag = 0xAA02;
        let bounds = Rect::new(0.0, 0.0, 300.0, 400.0);

        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = scope
                .panel("TestFitPanel")
                .tagged(tag)
                .bounds(bounds)
                .auto_scroll()
                .show(|ui| {
                    // 2 items of 40px = 80px <= 400px
                    for _ in 0..2 {
                        ui.empty_box_passive_named("SmallItem", Style::new().height(40.0));
                    }
                });
        }

        assert_eq!(tree.max_scroll(tag), 0.0);
        let has_thumb = tree
            .iter()
            .any(|(_, n)| n.role == WidgetRole::ScrollBarThumb);
        assert!(
            !has_thumb,
            "Must NOT emit scrollbar thumb when content fits"
        );
    }
}