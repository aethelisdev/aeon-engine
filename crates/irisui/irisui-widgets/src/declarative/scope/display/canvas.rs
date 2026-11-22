// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Canvas, Texture & Overlay Primitives
//!
//! Provides color swatch boxes, external GPU texture quads, scrollbar overlays, and text caret cursors on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use irisui_core::{
    CanvasOrigin, Color, Point, Rect, Size, Style, TextAlign, WidgetCursor, WidgetResponse,
    WidgetRole,
};

/// Configuration descriptor for declarative [`UiScope::immediate_canvas_with`] emission.
#[derive(Debug, Clone)]
pub struct ImmediateCanvasSpec {
    /// Diagnostic node name.
    pub name: &'static str,
    /// Semantic 64-bit widget tag.
    pub tag: u64,
    /// Flexbox and visual container style.
    pub style: Style,
    /// Coordinate origin used by [`CanvasPrimitive`](irisui_core::CanvasPrimitive) items on this node.
    pub origin: CanvasOrigin,
    /// Whether the canvas participates in pointer hit-testing.
    pub interactive: bool,
    /// Optional hover cursor override.
    pub cursor: Option<WidgetCursor>,
}

impl ImmediateCanvasSpec {
    /// Creates a new interactive canvas spec using screen-space coordinates.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, tag: u64, style: Style) -> Self {
        Self {
            name,
            tag,
            style,
            origin: CanvasOrigin::Screen,
            interactive: true,
            cursor: None,
        }
    }

    /// Sets the coordinate origin mode (`Screen`, `NodeTopLeft`, or `NodeCenter`).
    #[inline]
    #[must_use]
    pub const fn origin(mut self, origin: CanvasOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Configures whether pointer hit-testing captures clicks over this canvas.
    #[inline]
    #[must_use]
    pub const fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Assigns a custom hover cursor icon to the canvas node.
    #[inline]
    #[must_use]
    pub const fn cursor(mut self, cursor: WidgetCursor) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a blinking text input caret cursor node with [`WidgetRole::TextInputCaret`].
    ///
    /// The caret is rendered and collected dynamically in-place by the compiler based on
    /// the container's focus state and frame blink phase without triggering UI tree invalidation.
    ///
    /// # Arguments
    /// * `color` - Caret indicator foreground color.
    pub fn text_caret(&mut self, color: Color) {
        self.text_caret_with_offset(color, 0.0);
    }

    /// Emits an in-place retained text caret cursor with a custom horizontal layout offset.
    ///
    /// Retains the caret in the layout tree while storing the horizontal displacement
    /// in `node.tag` for O(1) in-place positioning without tree reconstruction.
    ///
    /// # Arguments
    /// * `color` - Caret indicator foreground color.
    /// * `offset` - Horizontal pixel offset relative to the text beginning.
    pub fn text_caret_with_offset(&mut self, color: Color, offset: f32) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("TextInputCaret");
            node.role = WidgetRole::TextInputCaret;
            node.interactive = false;
            node.set_text("|");
            node.tag = offset.to_bits() as u64;
            node.text_color = color;
            node.font_size = 11.5;
            node.line_height = 16.0;
            node.text_align = TextAlign::Left;
            node.set_style(Style::new().width(6.0));
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a colored preview swatch box.
    ///
    /// # Arguments
    /// * `color` - Color to display in the swatch.
    /// * `width` - Swatch box width in physical pixels.
    /// * `height` - Swatch box height in physical pixels.
    pub fn color_swatch(&mut self, color: Color, width: f32, height: f32) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::ColorSwatch;
            node.set_style(
                Style::new()
                    .background(color)
                    .border(1.0, Color::rgba(0.85, 0.88, 0.95, 0.70))
                    .border_radius(2.0)
                    .width(width)
                    .height(height),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a textured quad displaying an external host-owned texture (e.g. 3D Viewport RTT framebuffer).
    ///
    /// The emitted node is non-interactive by default so that clicks pass through to child overlays
    /// or to host canvas event handlers.
    ///
    /// # Arguments
    /// * `texture_id` - External texture key recognized by the GPU renderer backend.
    /// * `width` - Quad width constraint in logical points.
    /// * `height` - Quad height constraint in logical points.
    pub fn external_texture(
        &mut self,
        texture_id: irisui_core::ExternalTextureId,
        width: f32,
        height: f32,
    ) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = false;
            node.set_name("ExternalTextureQuad");
            node.set_external_texture(Some(texture_id));
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(width)
                    .height(height),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a vertical scrollbar overlay (track and thumb) for a scrollable container.
    ///
    /// The track and thumb nodes are positioned absolutely relative to `container_rect`.
    ///
    /// # Arguments
    /// * `geom` - Geometric layout of the vertical scrollbar track and thumb.
    /// * `container_rect` - Bounding rectangle of the scroll container used for relative offsets.
    /// * `is_dragging` - Whether the thumb is currently being actively dragged.
    /// * `cursor_pos` - Current mouse cursor coordinate for hover state detection.
    /// * `track_tag` - Semantic hit-testing tag for the track node.
    /// * `thumb_tag` - Semantic hit-testing tag for the thumb node.
    pub fn scrollbar_vertical(
        &mut self,
        geom: crate::scroll_area::ScrollBarGeometry,
        container_rect: Rect,
        is_dragging: bool,
        cursor_pos: Option<Point>,
        track_tag: u64,
        thumb_tag: u64,
    ) {
        let is_thumb_hovered = cursor_pos.is_some_and(|pos| geom.thumb_rect.contains_point(pos));
        let is_track_hovered = cursor_pos.is_some_and(|pos| geom.track_rect.contains_point(pos));

        let track_rel_x = geom.track_rect.x - container_rect.x;
        let track_rel_y = geom.track_rect.y - container_rect.y;
        let track_rel_right = (container_rect.right() - geom.track_rect.right()).max(0.0);

        let parent_layer = self
            .tree
            .get(self.parent)
            .map(|p| {
                if p.layer != irisui_core::UiLayer::Content {
                    p.layer
                } else {
                    p.role.default_layer()
                }
            })
            .unwrap_or_default();

        let track_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(track_id) {
            node.set_name("ScrollBarTrack");
            node.tag = track_tag;
            node.interactive = true;
            node.role = WidgetRole::ScrollBarTrack;
            node.cursor = Some(WidgetCursor::Pointer);
            node.computed_rect = geom.track_rect;
            node.layer = parent_layer;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(track_rel_x)
                    .right(track_rel_right)
                    .top(track_rel_y)
                    .width(geom.track_rect.width)
                    .height(geom.track_rect.height)
                    .background(if is_track_hovered {
                        Color::rgba(0.12, 0.14, 0.20, 0.60)
                    } else {
                        Color::rgba(0.05, 0.06, 0.09, 0.40)
                    })
                    .border_radius(3.0),
            );
        }
        let _ = self.tree.add_child(self.parent, track_id);

        let thumb_rel_x = geom.thumb_rect.x - container_rect.x;
        let thumb_rel_y = geom.thumb_rect.y - container_rect.y;

        let thumb_bg = if is_dragging {
            Color::rgba(0.0, 0.85, 1.0, 0.95)
        } else if is_thumb_hovered {
            Color::rgba(0.55, 0.65, 0.85, 0.90)
        } else {
            Color::rgba(0.40, 0.45, 0.58, 0.75)
        };

        let thumb_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(thumb_id) {
            node.set_name("ScrollBarThumb");
            node.tag = thumb_tag;
            node.interactive = true;
            node.role = WidgetRole::ScrollBarThumb;
            node.cursor = Some(WidgetCursor::Pointer);
            node.computed_rect = geom.thumb_rect;
            node.layer = parent_layer;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(thumb_rel_x)
                    .top(thumb_rel_y)
                    .width(geom.thumb_rect.width)
                    .height(geom.thumb_rect.height)
                    .background(thumb_bg)
                    .border_radius(3.0),
            );
        }
        let _ = self.tree.add_child(self.parent, thumb_id);
    }

    /// Emits a volatile immediate-mode canvas surface for high-frequency dynamic rendering.
    ///
    /// Ideal for oscilloscope curves, profiler telemetry charts, audio waveforms, and timeline scrubbers.
    /// The closure receives an [`ImmediateCanvas`](crate::canvas::ImmediateCanvas) allowing direct emission of geometric primitives
    /// without creating child nodes in the [`UiTree`](irisui_core::UiTree) or triggering Taffy layout dirty flags.
    ///
    /// # Arguments
    /// * `size` - Layout dimensions (width and height) reserved for the canvas container.
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    pub fn immediate_canvas<F>(&mut self, size: Size, draw_fn: F) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        self.immediate_canvas_named("ImmediateCanvas", size, draw_fn)
    }

    /// Emits a volatile immediate-mode canvas surface with a diagnostic debug name.
    ///
    /// # Arguments
    /// * `name` - Diagnostic name assigned to the host widget node.
    /// * `size` - Layout dimensions (width and height) reserved for the canvas container.
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    pub fn immediate_canvas_named<F>(
        &mut self,
        name: &'static str,
        size: Size,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        let tag = self.tag_for(name);
        self.immediate_canvas_tagged(name, tag, size, draw_fn)
    }

    /// Emits a volatile immediate-mode canvas surface with explicit semantic tag and debug name.
    ///
    /// # Arguments
    /// * `name` - Diagnostic name assigned to the host widget node.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag bounding retrieval.
    /// * `size` - Layout dimensions (width and height) reserved for the canvas container.
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    ///
    /// Emits a volatile immediate-mode canvas surface with explicit semantic tag, diagnostic name, and custom layout style.
    ///
    /// Allows absolute positioning, insets, and full style control over the canvas container node.
    ///
    /// # Arguments
    /// * `name` - Diagnostic name assigned to the host widget node.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag bounding retrieval.
    /// * `style` - Full layout style (e.g. absolute positioning, insets, dimensions).
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    pub fn immediate_canvas_styled<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        style: Style,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        self.immediate_canvas_with(ImmediateCanvasSpec::new(name, tag, style), draw_fn)
    }

    /// Emits a declarative canvas configured via [`ImmediateCanvasSpec`], supporting layout-relative
    /// origins (`CanvasOrigin::NodeTopLeft` / `CanvasOrigin::NodeCenter`), pointer passthrough, and
    /// custom hover cursors.
    pub fn immediate_canvas_with<F>(
        &mut self,
        spec: ImmediateCanvasSpec,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        let node_id = self.tree.create_node();
        let parent_layer = self
            .tree
            .get(self.parent)
            .map(|p| p.layer)
            .unwrap_or_default();

        let w = spec
            .style
            .width
            .unwrap_or_else(|| self.available_rect().width);
        let h = spec.style.height.unwrap_or(0.0);
        let bounds = self
            .tree
            .tag_rects
            .get(&spec.tag)
            .copied()
            .unwrap_or_else(|| Rect::new(0.0, 0.0, w, h));

        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(spec.name);
            node.tag = spec.tag;
            node.role = WidgetRole::ImmediateCanvas;
            node.interactive = spec.interactive;
            node.is_boundary = true;
            node.layer = parent_layer;
            node.cursor = spec.cursor;
            node.canvas_origin = spec.origin;
            node.computed_rect = bounds;
            node.set_style(spec.style);
        }

        let _ = self.tree.add_child(self.parent, node_id);
        self.tree.clear_canvas_primitives_for(node_id);

        let mut canvas = crate::canvas::ImmediateCanvas::new(self.tree, node_id, bounds);
        draw_fn(&mut canvas);

        self.response(node_id)
    }

    /// Emits a volatile immediate-mode canvas surface with explicit semantic tag and debug name.
    ///
    /// # Arguments
    /// * `name` - Diagnostic name assigned to the host widget node.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag bounding retrieval.
    /// * `size` - Layout dimensions (width and height) reserved for the canvas container.
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    pub fn immediate_canvas_tagged<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        size: Size,
        draw_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut crate::canvas::ImmediateCanvas),
    {
        self.immediate_canvas_styled(
            name,
            tag,
            Style::new().width(size.width).height(size.height),
            draw_fn,
        )
    }

    /// Measures stacked children inside a tagged scroll container, updates max/safe scroll
    /// offsets on the viewport, and renders a vertical scrollbar overlay when content overflows.
    ///
    /// Returns the resolved maximum vertical scroll offset (`max_scroll_y`).
    pub fn sync_and_render_vertical_scrollbar(&mut self, spec: VerticalScrollbarSpec) -> f32 {
        let current_scroll = self.scroll_offset(spec.viewport_tag);
        let measured_total_h = self
            .node_id_tagged(spec.container_tag)
            .and_then(|id| self.node_ref(id))
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|&cid| self.node_ref(cid))
                    .map(|child| {
                        child.computed_rect.bottom() + current_scroll - spec.content_rect.y
                    })
                    .fold(0.0_f32, f32::max)
            })
            .unwrap_or(0.0);

        let total_h = measured_total_h
            .max(spec.min_virtual_height)
            .max(spec.content_rect.height);
        let max_scroll_y = (total_h - spec.content_rect.height).max(0.0);
        let safe_scroll = current_scroll.clamp(0.0, max_scroll_y);

        self.set_content_size_tagged(spec.viewport_tag, spec.content_rect.width, total_h);
        self.set_max_scroll(spec.viewport_tag, max_scroll_y);
        self.set_scroll_offset(spec.viewport_tag, safe_scroll);

        if total_h > spec.content_rect.height
            && let Some(geom) = crate::scroll_area::ScrollBarGeometry::compute_vertical(
                spec.content_rect,
                total_h,
                safe_scroll,
                &spec.style,
            )
        {
            let is_dragging = self.is_tag_captured(spec.thumb_tag);
            let target_host = spec
                .host_node_tag
                .and_then(|tag| self.node_id_tagged(tag))
                .unwrap_or(self.parent);
            let mut host_scope = self.child_scope(target_host);
            host_scope.scrollbar_vertical(
                geom,
                spec.host_rect,
                is_dragging,
                spec.fallback_cursor,
                spec.track_tag,
                spec.thumb_tag,
            );
        }

        max_scroll_y
    }
}

/// Configuration descriptor for declarative [`UiScope::sync_and_render_vertical_scrollbar`].
#[derive(Debug, Clone, Copy)]
pub struct VerticalScrollbarSpec {
    /// Semantic tag of the scroll viewport node storing scroll offset and content size.
    pub viewport_tag: u64,
    /// Semantic tag of the column/stack child container whose laid-out items determine content height.
    pub container_tag: u64,
    /// Semantic hit-testing tag assigned to the emitted scrollbar track node.
    pub track_tag: u64,
    /// Semantic hit-testing tag assigned to the emitted scrollbar thumb node.
    pub thumb_tag: u64,
    /// Scrollable viewport rectangle in logical screen space.
    pub content_rect: Rect,
    /// Host container rectangle used for relative absolute positioning of the scrollbar overlay.
    pub host_rect: Rect,
    /// Optional tagged ancestor node receiving the scrollbar overlay (defaults to current parent).
    pub host_node_tag: Option<u64>,
    /// Minimum virtual content height floor applied alongside measured child bounds.
    pub min_virtual_height: f32,
    /// Optional fallback pointer coordinate for scrollbar track/thumb hover highlights.
    pub fallback_cursor: Option<Point>,
    /// Visual scrollbar thickness, inset, and color style tokens.
    pub style: crate::scroll_area::ScrollAreaStyle,
}

impl VerticalScrollbarSpec {
    /// Creates a new vertical scrollbar spec targeting `viewport_tag` and `container_tag`.
    #[inline]
    #[must_use]
    pub fn new(
        viewport_tag: u64,
        container_tag: u64,
        track_tag: u64,
        thumb_tag: u64,
        content_rect: Rect,
    ) -> Self {
        Self {
            viewport_tag,
            container_tag,
            track_tag,
            thumb_tag,
            content_rect,
            host_rect: content_rect,
            host_node_tag: None,
            min_virtual_height: 0.0,
            fallback_cursor: None,
            style: crate::scroll_area::ScrollAreaStyle::default(),
        }
    }

    /// Sets the host container bounding rectangle and optional tagged host node for scrollbar overlay attachment.
    #[inline]
    #[must_use]
    pub const fn host(mut self, host_rect: Rect, host_node_tag: Option<u64>) -> Self {
        self.host_rect = host_rect;
        self.host_node_tag = host_node_tag;
        self
    }

    /// Sets a minimum virtual content height floor in pixels.
    #[inline]
    #[must_use]
    pub const fn min_virtual_height(mut self, height: f32) -> Self {
        self.min_virtual_height = height;
        self
    }

    /// Sets the fallback pointer cursor position for hover visual state detection.
    #[inline]
    #[must_use]
    pub const fn cursor(mut self, cursor_pos: Option<Point>) -> Self {
        self.fallback_cursor = cursor_pos;
        self
    }

    /// Sets custom [`ScrollAreaStyle`](crate::scroll_area::ScrollAreaStyle) metrics and colors.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: crate::scroll_area::ScrollAreaStyle) -> Self {
        self.style = style;
        self
    }
}

#[cfg(test)]
mod scrollbar_tests {
    use super::*;
    use irisui_core::tree::UiTree;

    #[test]
    fn test_sync_and_render_vertical_scrollbar_clamps_and_emits_nodes() {
        const VP: u64 = 101;
        const CONT: u64 = 102;
        const TRACK: u64 = 103;
        const THUMB: u64 = 104;

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        tree.scroll_offsets.insert(VP, 500.0);

        let max_scroll = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.container_tagged(
                "Viewport",
                Style::new()
                    .width(200.0)
                    .height(100.0)
                    .scroll_offset_y(500.0),
                WidgetRole::ScrollAreaViewport,
                VP,
                |vp| {
                    vp.container_tagged(
                        "Body",
                        Style::new().flex_col(),
                        WidgetRole::Default,
                        CONT,
                        |col| {
                            col.empty_box(Style::new().height(250.0));
                        },
                    );
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 200.0, 100.0));
            scope.sync_and_render_vertical_scrollbar(
                VerticalScrollbarSpec::new(
                    VP,
                    CONT,
                    TRACK,
                    THUMB,
                    Rect::new(0.0, 0.0, 200.0, 100.0),
                )
                .min_virtual_height(300.0),
            )
        };

        assert!((max_scroll - 200.0).abs() < f32::EPSILON);
        assert!((*tree.scroll_offsets.get(&VP).unwrap_or(&0.0) - 200.0).abs() < f32::EPSILON);
        assert!(tree.find_node_by_tag(TRACK).is_some());
        assert!(tree.find_node_by_tag(THUMB).is_some());
    }
}