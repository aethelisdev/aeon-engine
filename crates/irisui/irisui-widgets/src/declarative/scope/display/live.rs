// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Live / Volatile Canvas & Text Primitives
//!
//! Provides zero-`UiTree`-leakage declarative APIs on [`UiScope`] for high-frequency
//! in-place widgets (telemetry oscilloscopes, compasses, live HUD counters). During initial
//! construction, widgets register their semantic tag in [`UiTree::live_node_tags`](irisui_core::UiTree::live_node_tags).
//! On subsequent `InPlaceRefresh` passes, the same declarative calls or [`UiScope::update_live_canvas`] /
//! [`UiScope::update_live_text`] update primitives and text in place without mutating layout.

use irisui_core::WidgetResponse;
use irisui_core::color::Color;
use irisui_core::geometry::Size;
use irisui_core::node::WidgetRole;
use irisui_core::style::{Style, TextAlign, TextWrap};

use super::super::UiScope;

/// Descriptor configuring a declarative [`UiScope::live_text`] node with full theme/style control.
#[derive(Debug, Clone)]
pub struct LiveTextProps<'a> {
    /// Descriptive widget name for inspection/debugging.
    pub name: &'a str,
    /// Stable 64-bit semantic tag used for in-place text updates.
    pub tag: u64,
    /// Text content to display (or update in place).
    pub text: &'a str,
    /// Font size in logical points.
    pub font_size: f32,
    /// Foreground text color.
    pub color: Color,
    /// Horizontal text alignment.
    pub align: TextAlign,
    /// Full flexbox/positioning style applied when the node is first created.
    pub style: Style,
}

impl<'a> LiveTextProps<'a> {
    /// Creates a new [`LiveTextProps`] descriptor using explicit style parameters.
    #[inline]
    #[must_use]
    pub fn new(
        name: &'a str,
        tag: u64,
        text: &'a str,
        font_size: f32,
        color: Color,
        style: Style,
    ) -> Self {
        Self {
            name,
            tag,
            text,
            font_size,
            color,
            align: TextAlign::Left,
            style,
        }
    }

    /// Sets horizontal text alignment.
    #[inline]
    #[must_use]
    pub const fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl UiScope<'_> {
    /// Emits a fixed-size live/volatile immediate canvas or updates its primitives in place
    /// when a node with `tag` already exists in the retained tree.
    pub fn live_canvas_tagged<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        size: Size,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        self.live_canvas_styled(
            name,
            tag,
            Style {
                width: Some(size.width),
                height: Some(size.height),
                flex_shrink: 0.0,
                ..Default::default()
            },
            draw_fn,
        )
    }

    /// Emits a styled live/volatile immediate canvas or updates its primitives in place
    /// when a node with `tag` already exists in the retained tree.
    pub fn live_canvas_styled<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        style: Style,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        self.live_canvas_anchored(name, tag, irisui_core::CanvasOrigin::Screen, style, draw_fn)
    }

    /// Emits a layout-anchored live/volatile immediate canvas with an explicit [`irisui_core::CanvasOrigin`]
    /// or updates its primitives in place when `tag` already exists in the retained tree.
    ///
    /// Pairing this with [`irisui_core::CanvasOrigin::NodeTopLeft`] or [`irisui_core::CanvasOrigin::NodeCenter`]
    /// lets Taffy absolute anchors (such as `.right(8.0).top(8.0)`) position the canvas cleanly on Frame #0
    /// without manual consumer `viewport_rect` coordinate subtraction or redundant `finish_layout` calls.
    pub fn live_canvas_anchored<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        origin: irisui_core::CanvasOrigin,
        style: Style,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        if let Some(existing_id) = self.tree.node_tagged(tag).map(|n| n.id) {
            self.tree.register_live_tag(tag);
            let _ = self.update_live_canvas(tag, draw_fn);
            return self.response(existing_id);
        }
        self.tree.register_live_tag(tag);
        self.immediate_canvas_with(
            crate::declarative::scope::display::canvas::ImmediateCanvasSpec::new(name, tag, style)
                .origin(origin),
            draw_fn,
        )
    }

    /// Emits a styled live/volatile text node or updates its text in place when `props.tag`
    /// already exists in the retained tree.
    pub fn live_text(&mut self, props: LiveTextProps<'_>) -> WidgetResponse {
        if let Some(existing_id) = self.tree.node_tagged(props.tag).map(|n| n.id) {
            self.tree.register_live_tag(props.tag);
            let _ = self.update_live_text(props.tag, props.text);
            return self.response(existing_id);
        }

        self.tree.register_live_tag(props.tag);
        let node_id = self.tree.create_node();
        let parent_layer = self
            .tree
            .get(self.parent)
            .map(|p| p.layer)
            .unwrap_or_default();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(props.name);
            node.tag = props.tag;
            node.role = WidgetRole::Default;
            node.layer = parent_layer;
            node.interactive = false;
            node.set_text(props.text);
            node.text_color = props.color;
            node.font_size = props.font_size;
            node.line_height = (props.font_size * 1.3).max(12.0).round();
            node.text_align = props.align;
            node.set_text_wrap(TextWrap::Word);
            node.set_style(props.style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        self.response(node_id)
    }

    /// Updates an existing live/volatile canvas tagged `tag` in place without mutating layout.
    ///
    /// Automatically records internal quad re-upload flags on the retained tree.
    pub fn update_live_canvas<F>(&mut self, tag: u64, draw_fn: F) -> bool
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        let updated = crate::canvas::update_canvas_by_tag(self.tree, tag, draw_fn);
        if updated {
            self.tree.record_live_quad_update();
        }
        updated
    }

    /// Updates an existing live/volatile text node tagged `tag` in place without mutating layout.
    ///
    /// Automatically marks text preparation and quad re-upload flags only when the text changed.
    pub fn update_live_text(&mut self, tag: u64, text: &str) -> bool {
        self.tree.update_live_text_by_tag(tag, text)
    }

    /// Updates both the text and foreground color of a live/volatile node tagged `tag` in place.
    pub fn update_live_text_colored(&mut self, tag: u64, text: &str, color: Color) -> bool {
        self.tree.update_live_text_colored_by_tag(tag, text, color)
    }

    /// Emits a styled live/volatile text node formatted into a zero-heap [`irisui_core::StackString`].
    pub fn live_text_fmt<const N: usize, F>(
        &mut self,
        name: &'static str,
        tag: u64,
        font_size: f32,
        color: Color,
        style: Style,
        fmt_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut irisui_core::StackString<N>),
    {
        let mut buf = irisui_core::StackString::<N>::new();
        fmt_fn(&mut buf);
        self.live_text(LiveTextProps::new(
            name,
            tag,
            buf.as_str(),
            font_size,
            color,
            style,
        ))
    }

    /// Formats into a stack [`irisui_core::StackString`] and updates the live text node tagged `tag`.
    pub fn update_live_text_fmt<const N: usize, F>(&mut self, tag: u64, fmt_fn: F) -> bool
    where
        F: FnOnce(&mut irisui_core::StackString<N>),
    {
        let mut buf = irisui_core::StackString::<N>::new();
        fmt_fn(&mut buf);
        self.tree.update_live_text_by_tag(tag, buf.as_str())
    }

    /// Formats into a stack [`irisui_core::StackString`] and updates text and color in place.
    pub fn update_live_text_colored_fmt<const N: usize, F>(
        &mut self,
        tag: u64,
        color: Color,
        fmt_fn: F,
    ) -> bool
    where
        F: FnOnce(&mut irisui_core::StackString<N>),
    {
        let mut buf = irisui_core::StackString::<N>::new();
        fmt_fn(&mut buf);
        self.tree
            .update_live_text_colored_by_tag(tag, buf.as_str(), color)
    }

    /// Returns `true` when `tag` is registered as a live/volatile node in the retained tree.
    #[inline]
    #[must_use]
    pub fn has_live_tag(&self, tag: u64) -> bool {
        self.tree.is_live_tag(tag)
    }

    /// Returns `true` when at least one live/volatile node is registered in the retained tree.
    #[inline]
    #[must_use]
    pub fn has_live_nodes(&self) -> bool {
        self.tree.has_live_nodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::{CanvasOrigin, Point, Rect, UiTree};

    #[test]
    fn test_live_canvas_anchored_preserves_origin_and_updates_in_place() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        const TAG_ANCHORED: u64 = 0xAA10;

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.live_canvas_anchored(
                "AnchoredCompass",
                TAG_ANCHORED,
                CanvasOrigin::NodeTopLeft,
                Style::new()
                    .position_absolute()
                    .right(8.0)
                    .top(8.0)
                    .width(72.0)
                    .height(72.0),
                |c| c.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::RED, 0.0),
            );
        }

        let node = tree.node_tagged(TAG_ANCHORED).expect("node must exist");
        assert_eq!(node.canvas_origin, CanvasOrigin::NodeTopLeft);
        assert!(tree.is_live_tag(TAG_ANCHORED));

        // Second call updates primitives in place without creating duplicate nodes
        let initial_len = tree.len();
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.live_canvas_anchored(
                "AnchoredCompass",
                TAG_ANCHORED,
                CanvasOrigin::NodeTopLeft,
                Style::new().width(72.0).height(72.0),
                |c| {
                    c.draw_line(
                        Point::new(0.0, 0.0),
                        Point::new(20.0, 20.0),
                        Color::GREEN,
                        2.0,
                    );
                },
            );
        }

        assert_eq!(tree.len(), initial_len);
        let id = tree.find_node_by_tag(TAG_ANCHORED).expect("id");
        assert!(matches!(
            tree.canvas_primitives(id).and_then(|p| p.first()),
            Some(irisui_core::CanvasPrimitive::Line { .. })
        ));
    }

    #[test]
    fn test_live_text_fmt_and_colored_update_in_place() {
        use std::fmt::Write;
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        const TAG_TEXT: u64 = 0xBB11;

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.live_text_fmt::<32, _>(
                "FpsText",
                TAG_TEXT,
                14.0,
                Color::GREEN,
                Style::new(),
                |buf| {
                    let _ = write!(buf, "{:.0} FPS", 144.0_f32);
                },
            );
        }

        assert_eq!(
            tree.node_tagged(TAG_TEXT).and_then(|n| n.text.as_deref()),
            Some("144 FPS")
        );
        assert_eq!(
            tree.node_tagged(TAG_TEXT).map(|n| n.text_color),
            Some(Color::GREEN)
        );

        {
            let mut scope = UiScope::new(&mut tree, root);
            assert!(
                scope.update_live_text_colored_fmt::<32, _>(TAG_TEXT, Color::RED, |buf| {
                    let _ = write!(buf, "{:.0} FPS", 24.0_f32);
                })
            );
            // Repeating with identical text and color returns false (no redundant dirty mark)
            assert!(
                !scope.update_live_text_colored_fmt::<32, _>(TAG_TEXT, Color::RED, |buf| {
                    let _ = write!(buf, "{:.0} FPS", 24.0_f32);
                })
            );
        }

        assert_eq!(
            tree.node_tagged(TAG_TEXT).and_then(|n| n.text.as_deref()),
            Some("24 FPS")
        );
        assert_eq!(
            tree.node_tagged(TAG_TEXT).map(|n| n.text_color),
            Some(Color::RED)
        );
    }
}