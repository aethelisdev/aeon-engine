// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Typography, Badges & Display Primitives
//!
//! Provides static text labels, headers, icons, telemetry badges, color swatches,
//! and interactive hyperlinks on [`UiScope`].
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Point, Rect, Style, TextAlign, TextWrap,
    WidgetCursor, WidgetId, WidgetRole,
};

/// Layout and styling descriptor for rich wrapped text labels.
///
/// Encapsulates typography properties (size, color, alignment, wrapping) and container layout style
/// to eliminate parameter bloat and uphold clean API boundaries.
#[derive(Debug, Clone)]
pub struct WrappedLabelDescriptor {
    /// Font size in logical points.
    pub font_size: f32,
    /// Foreground text color.
    pub color: Color,
    /// Horizontal text alignment (`Left`, `Center`, `Right`).
    pub align: TextAlign,
    /// Text wrap mode (`NoWrap`, `Word`, `Char`, `Auto`).
    pub wrap: TextWrap,
    /// Explicit layout style applied to the label.
    pub style: Style,
}

impl WrappedLabelDescriptor {
    /// Constructs a wrapped label descriptor with canonical word-wrap and left-align defaults.
    #[inline]
    pub fn new(font_size: f32, color: Color, style: Style) -> Self {
        Self {
            font_size,
            color,
            align: TextAlign::Left,
            wrap: TextWrap::Word,
            style,
        }
    }

    /// Overrides the horizontal text alignment.
    #[inline]
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Overrides the text wrap mode.
    #[inline]
    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a static text label node with default light gray foreground coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn text(&mut self, text: impl Into<String>) -> WidgetId {
        self.text_colored(text, Color::rgba(0.78, 0.80, 0.85, 1.0))
    }

    /// Emits a styled text label node with customizable font size, color, and text alignment.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Foreground text color.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    pub fn label(
        &mut self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> WidgetId {
        let text_str = text.into();
        let est_w = (text_str.len() as f32 * font_size * 0.65).ceil().max(20.0);
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text_str);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(16.0).round();
            node.text_align = align;
            node.set_style(Style::new().width(est_w));
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a styled text label node with an explicit fixed width, font size, color, and text alignment.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `width` - Explicit physical pixel width constraint.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Foreground text color.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    pub fn label_with_width(
        &mut self,
        text: impl Into<String>,
        width: f32,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(16.0).round();
            node.text_align = align;
            node.set_style(Style::new().width(width));
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a styled text label node with an explicit style layout (e.g. absolute position, size).
    ///
    /// The node is passive (`interactive = false`) by default so it does not block hit-testing.
    ///
    /// # Arguments
    /// * `name` - Descriptive name of the widget node for tree debugging.
    /// * `text` - Display string content.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Foreground text color.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    /// * `style` - Explicit layout style applied to the label.
    pub fn label_styled_passive(
        &mut self,
        name: impl Into<String>,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        align: TextAlign,
        style: Style,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.role = WidgetRole::Default;
            node.interactive = false;
            node.set_text(text);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(12.0).round();
            node.text_align = align;
            node.set_text_wrap(TextWrap::Word);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a non-interactive, styled text label with explicit text wrapping and layout styling.
    ///
    /// Ideal for multi-line description paragraphs, form hints, and responsive dialog subtitles.
    ///
    /// # Arguments
    /// * `name` - Static debug identifier for profiling and inspection.
    /// * `text` - Display string content.
    /// * `desc` - Typography and layout descriptor ([`WrappedLabelDescriptor`]).
    pub fn label_styled_passive_wrapped(
        &mut self,
        name: &'static str,
        text: impl Into<String>,
        desc: WrappedLabelDescriptor,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.role = WidgetRole::Default;
            node.interactive = false;
            node.set_text(text);
            node.text_color = desc.color;
            node.font_size = desc.font_size;
            node.line_height = (desc.font_size * 1.3).max(12.0).round();
            node.text_align = desc.align;
            node.set_text_wrap(desc.wrap);
            node.set_style(desc.style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a static text label node with explicit text coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `color` - RGBA foreground color.
    pub fn text_colored(&mut self, text: impl Into<String>, color: Color) -> WidgetId {
        self.label(text, 12.0, color, TextAlign::Left)
    }

    /// Emits a prominent section heading text label node.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn heading(&mut self, text: impl Into<String>) -> WidgetId {
        self.label(
            text,
            13.0,
            Color::rgba(0.95, 0.96, 1.0, 1.0),
            TextAlign::Left,
        )
    }

    /// Emits a styled text label node with an explicit width constraint.
    ///
    /// Useful for structured form rows where labels must maintain fixed alignment.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Foreground text color.
    /// * `width` - Explicit width allocated for the text bounding box.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    pub fn label_fixed(
        &mut self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        width: f32,
        align: TextAlign,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(16.0).round();
            node.text_align = align;
            node.style.width = Some(width);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a text label that expands to fill remaining row/column flex space (`flex_grow = 1.0`).
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Foreground text color.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    pub fn label_flex(
        &mut self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(16.0).round();
            node.text_align = align;
            node.set_style(Style::new().flex_grow(1.0));
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits an icon quad textured from an atlas UV rectangle.
    ///
    /// # Arguments
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon(&mut self, icon_uv: [f32; 4], tint: Color, size: f32) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
        icon_id
    }

    /// Emits a named icon quad textured from an atlas UV rectangle with a debug identifier.
    ///
    /// # Arguments
    /// * `name` - Static debug identifier assigned to the UI node.
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon_named(
        &mut self,
        name: &'static str,
        icon_uv: [f32; 4],
        tint: Color,
        size: f32,
    ) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.set_name(name);
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
        icon_id
    }

    /// Emits a styled telemetry or key-value pill badge.
    ///
    /// # Arguments
    /// * `label` - Left title label text.
    /// * `value` - Right value text.
    /// * `val_color` - Color of the value readout.
    pub fn badge(&mut self, label: &str, value: &str, val_color: Color) -> WidgetId {
        let badge_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0))
            .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
            .border(1.0, Color::rgba(0.18, 0.20, 0.24, 0.85))
            .border_radius(4.0)
            .height(22.0);

        self.container(badge_style, |badge_scope| {
            badge_scope.label(
                label,
                10.0,
                Color::rgba(0.60, 0.63, 0.70, 1.0),
                TextAlign::Left,
            );
            badge_scope.label(value, 10.0, val_color, TextAlign::Right);
        })
    }

    /// Emits a colored preview swatch box.
    ///
    /// # Arguments
    /// * `color` - Color to display in the swatch.
    /// * `width` - Swatch box width in physical pixels.
    /// * `height` - Swatch box height in physical pixels.
    pub fn color_swatch(&mut self, color: Color, width: f32, height: f32) -> WidgetId {
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
        node_id
    }

    /// Emits a clickable text hyperlink with persistent tag and automatic hover styling.
    ///
    /// Changes text color dynamically on hover and returns an interactive [`WidgetResponse`].
    ///
    /// # Arguments
    /// * `label` - Hyperlink anchor text.
    /// * `font_size` - Size of the typography in logical points.
    pub fn link(&mut self, label: impl Into<String>, font_size: f32) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(visible);
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(16.0).round();
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::rgba(0.35, 0.95, 1.0, 1.0)
            } else {
                Color::rgba(0.0, 0.85, 0.95, 1.0)
            };
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
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
    ) -> WidgetId {
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
        node_id
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