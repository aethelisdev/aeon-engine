// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Canvas, Texture & Overlay Primitives
//!
//! Provides color swatch boxes, external GPU texture quads, scrollbar overlays, and text caret cursors on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use iris_core::{Color, Point, Rect, Style, TextAlign, WidgetCursor, WidgetRole};

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
        texture_id: iris_core::ExternalTextureId,
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

        let parent_layer = self
            .tree
            .get(self.parent)
            .map(|p| p.layer)
            .unwrap_or_default();

        let track_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(track_id) {
            node.set_name("ScrollBarTrack");
            node.tag = track_tag;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.computed_rect = geom.track_rect;
            node.layer = parent_layer;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(track_rel_x)
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
            node.role = WidgetRole::Button;
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
}