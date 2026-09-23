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
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
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
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(16.0).round();
            node.text_align = align;
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

    /// Emits an icon quad textured from an atlas UV rectangle.
    ///
    /// # Arguments
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon(&mut self, icon_uv: [f32; 4], tint: Color, size: f32) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
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
}