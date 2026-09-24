// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Typography, Badges & Display Primitives
//!
//! Provides static text labels, headers, icons, telemetry badges, color swatches,
//! and interactive hyperlinks on [`UiScope`].
//!

use super::core::UiScope;
use crate::declarative::types::{WidgetResponse, hash_label};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Point, Rect, Style, TextAlign, WidgetCursor,
    WidgetId, WidgetRole,
};

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
        name: &'static str,
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
            node.set_style(style);
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
        let tag = hash_label(&label_str);
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
            node.set_text(label_str);
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

    /// Emits a centralized empty state placeholder notice within the developer console viewport.
    ///
    /// # Arguments
    /// * `message` - Explanation text displayed to the developer.
    pub fn console_empty_notice(&mut self, message: &str) -> WidgetId {
        let style = Style::new()
            .padding_insets(Insets::new(24.0, 16.0, 24.0, 16.0))
            .background(Color::rgba(0.0, 0.0, 0.0, 0.0));
        self.container_named("ConsoleEmptyNotice", style, |scope| {
            scope.label(
                message,
                12.0,
                Color::rgba(0.50, 0.53, 0.60, 1.0),
                TextAlign::Center,
            );
        })
    }

    /// Emits a single hardware-accelerated log entry row within the developer console viewport.
    ///
    /// Configures flex-row layout, zebra striping, severity badge, timestamp, target tag, and message
    /// via 100% pure declarative [`UiScope`] container and label compositions.
    ///
    /// # Arguments
    /// * `level` - Severity level determining badge coloring and styling.
    /// * `timestamp` - Formatted time string (e.g. `12:00:00.000`).
    /// * `target` - Subsystem or module identifier generating the message.
    /// * `message` - Log description content.
    /// * `is_hovered` - Whether the row is hovered by the mouse pointer.
    /// * `is_striped` - Whether zebra striping background is active for this index.
    pub fn console_row(
        &mut self,
        level: crate::console::types::ConsoleLogLevel,
        timestamp: &str,
        target: &str,
        message: &str,
        is_hovered: bool,
        is_striped: bool,
    ) -> WidgetId {
        let bg_color = if is_hovered {
            Color::rgba(0.10, 0.13, 0.18, 1.0)
        } else if is_striped {
            Color::rgba(0.04, 0.048, 0.065, 0.98)
        } else {
            Color::rgba(0.05, 0.06, 0.08, 0.98)
        };

        let row_style = Style::new()
            .flex_row()
            .align_items(iris_core::AlignItems::Center)
            .height(26.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .background(bg_color);

        self.container_tagged(
            "ConsoleRow",
            row_style,
            WidgetRole::Default,
            crate::console::types::CONSOLE_TAG_ROW,
            |row_scope| {
                if is_hovered {
                    let accent_style = Style::new()
                        .width(3.0)
                        .height(26.0)
                        .background(Color::rgba(0.0, 0.85, 1.0, 0.95));
                    let accent_id = row_scope.empty_box(accent_style);
                    if let Some(node) = row_scope.tree.get_mut(accent_id) {
                        node.interactive = false;
                    }
                }

                let (badge_text, badge_color, badge_bg) = match level {
                    crate::console::types::ConsoleLogLevel::Error => (
                        "ERR",
                        Color::rgba(0.98, 0.45, 0.45, 1.0),
                        Color::rgba(0.38, 0.08, 0.08, 0.90),
                    ),
                    crate::console::types::ConsoleLogLevel::Warn => (
                        "WRN",
                        Color::rgba(0.98, 0.78, 0.20, 1.0),
                        Color::rgba(0.35, 0.20, 0.02, 0.90),
                    ),
                    crate::console::types::ConsoleLogLevel::Info => (
                        "INF",
                        Color::rgba(0.25, 0.78, 0.98, 1.0),
                        Color::rgba(0.05, 0.20, 0.32, 0.90),
                    ),
                    crate::console::types::ConsoleLogLevel::Debug => (
                        "DBG",
                        Color::rgba(0.75, 0.60, 0.98, 1.0),
                        Color::rgba(0.18, 0.10, 0.32, 0.90),
                    ),
                    crate::console::types::ConsoleLogLevel::Trace => (
                        "TRC",
                        Color::rgba(0.60, 0.66, 0.75, 1.0),
                        Color::rgba(0.12, 0.15, 0.20, 0.90),
                    ),
                };

                let badge_style = Style::new()
                    .width(38.0)
                    .height(18.0)
                    .background(badge_bg)
                    .border_radius(3.5);
                let badge_id =
                    row_scope.container_named("LevelBadge", badge_style, |badge_scope| {
                        let badge_lbl = badge_scope.label_with_width(
                            badge_text,
                            38.0,
                            9.5,
                            badge_color,
                            TextAlign::Center,
                        );
                        if let Some(node) = badge_scope.tree.get_mut(badge_lbl) {
                            node.interactive = false;
                        }
                    });
                if let Some(node) = row_scope.tree.get_mut(badge_id) {
                    node.interactive = false;
                }

                let time_id = row_scope.label_with_width(
                    timestamp,
                    66.0,
                    10.0,
                    Color::rgba(0.50, 0.55, 0.65, 1.0),
                    TextAlign::Left,
                );
                if let Some(node) = row_scope.tree.get_mut(time_id) {
                    node.interactive = false;
                }

                let target_text = format!("[{}]", target);
                let target_w = (target_text.len() as f32 * 6.6 + 6.0).clamp(44.0, 220.0);
                let target_id = row_scope.label_with_width(
                    target_text,
                    target_w,
                    10.5,
                    Color::rgba(0.0, 0.85, 1.0, 0.85),
                    TextAlign::Left,
                );
                if let Some(node) = row_scope.tree.get_mut(target_id) {
                    node.interactive = false;
                }

                let msg_color = if level == crate::console::types::ConsoleLogLevel::Error {
                    Color::rgba(1.0, 0.85, 0.85, 1.0)
                } else {
                    Color::rgba(0.92, 0.94, 0.98, 1.0)
                };
                let msg_id = row_scope.label_flex(message, 11.0, msg_color, TextAlign::Left);
                if let Some(node) = row_scope.tree.get_mut(msg_id) {
                    node.interactive = false;
                }
            },
        )
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

        let track_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(track_id) {
            node.set_name("ScrollBarTrack");
            node.tag = track_tag;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(track_rel_x)
                    .top(track_rel_y)
                    .width(geom.track_rect.width)
                    .height(geom.track_rect.height)
                    .background(if is_track_hovered {
                        Color::rgba(0.08, 0.10, 0.14, 0.50)
                    } else {
                        Color::rgba(0.03, 0.04, 0.06, 0.35)
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
            Color::rgba(0.35, 0.40, 0.50, 0.65)
        };

        let thumb_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(thumb_id) {
            node.set_name("ScrollBarThumb");
            node.tag = thumb_tag;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
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