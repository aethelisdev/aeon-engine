// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Typography, Text Labels & Status Badge Primitives
//!
//! Provides clean, ergonomic typography helpers on [`UiScope`]:
//! - 1-argument standard [`UiScope::label`] with consistent theme typography.
//! - Prominent headings ([`UiScope::heading`], [`UiScope::subheading`]).
//! - Colored text labels ([`UiScope::colored_label`], [`UiScope::text_colored`]).
//! - Autonomous status badges ([`UiScope::badge`], [`UiScope::key_value_badge`]).
//! - Custom typography and layout wrappers ([`UiScope::label_custom`], [`UiScope::label_with_width`]).
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, TextWrap, WidgetCursor, WidgetId,
    WidgetRole,
};

/// Descriptor struct configuring word-wrapping, style layout, and typography for wrapped text labels.
///
/// Prevents parameter explosion on [`UiScope::label_styled_passive_wrapped`].
#[derive(Debug, Clone, Copy)]
pub struct WrappedLabelDescriptor {
    /// Font size in logical points.
    pub font_size: f32,
    /// Foreground text color.
    pub color: Color,
    /// Horizontal text alignment (`Left`, `Center`, `Right`).
    pub align: TextAlign,
    /// Word-wrapping mode applied across multi-line runs.
    pub wrap: TextWrap,
    /// Explicit layout and dimension styling.
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

    /// Overrides the container layout style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
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
    /// Emits a standard text label node with default light gray foreground coloring and left alignment.
    ///
    /// Provides the fundamental crates.io-grade 1-argument typography primitive, eliminating
    /// verbose 4-argument boilerplate across declarative views and inspector rows.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn label(&mut self, text: impl Into<String>) -> WidgetId {
        self.label_custom(
            text,
            11.5,
            Color::rgba(0.78, 0.80, 0.85, 1.0),
            TextAlign::Left,
        )
    }

    /// Emits a prominent section heading text label node with bold white foreground coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn heading(&mut self, text: impl Into<String>) -> WidgetId {
        self.label_custom(text, 13.5, Color::WHITE, TextAlign::Left)
    }

    /// Emits a secondary section subtitle text label node with muted light gray coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn subheading(&mut self, text: impl Into<String>) -> WidgetId {
        self.label_custom(
            text,
            12.0,
            Color::rgba(0.90, 0.92, 0.96, 1.0),
            TextAlign::Left,
        )
    }

    /// Emits a text label node with an explicit foreground color and standard typography sizing.
    ///
    /// # Arguments
    /// * `color` - Foreground text color.
    /// * `text` - Display string content.
    pub fn colored_label(&mut self, color: Color, text: impl Into<String>) -> WidgetId {
        self.label_custom(text, 11.5, color, TextAlign::Left)
    }

    /// Emits a static text label node with default light gray foreground coloring.
    ///
    /// Preserved for backward compatibility across existing views; delegates directly to [`Self::label`].
    ///
    /// # Arguments
    /// * `text` - Display string content.
    #[inline]
    pub fn text(&mut self, text: impl Into<String>) -> WidgetId {
        self.label(text)
    }

    /// Emits a static text label node with explicit foreground color.
    ///
    /// Preserved for backward compatibility; delegates directly to [`Self::colored_label`].
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `color` - RGBA foreground color.
    #[inline]
    pub fn text_colored(&mut self, text: impl Into<String>, color: Color) -> WidgetId {
        self.colored_label(color, text)
    }

    /// Emits a fully customized text label node with explicit font size, foreground color, and horizontal alignment.
    ///
    /// Serves as the low-level foundation for all declarative label and heading primitives.
    /// Automatically estimates intrinsic width based on character count and glyph proportions.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Foreground text color.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    pub fn label_custom(
        &mut self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) -> WidgetId {
        let text_str = text.into();
        let est_w = (text_str.len() as f32 * font_size * 0.65).ceil().max(16.0);
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text_str);
            node.text_color = color;
            node.font_size = font_size;
            node.line_height = (font_size * 1.3).max(14.0).round();
            node.text_align = align;
            node.set_style(Style::new().width(est_w));
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a styled text label node with an explicit fixed width constraint.
    ///
    /// Useful for structured tabular rows and key-value columns requiring uniform width.
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
            node.line_height = (font_size * 1.3).max(14.0).round();
            node.text_align = align;
            node.set_style(Style::new().width(width));
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a styled text label node with an explicit width constraint.
    #[inline]
    pub fn label_fixed(
        &mut self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        width: f32,
        align: TextAlign,
    ) -> WidgetId {
        self.label_with_width(text, width, font_size, color, align)
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
            node.line_height = (font_size * 1.3).max(14.0).round();
            node.text_align = align;
            node.set_style(Style::new().flex_grow(1.0));
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

    /// Emits a non-interactive, styled text label with reactive hover text coloring.
    ///
    /// Combines label emission and parent hover reactivity in a single fluent call,
    /// eliminating intermediate [`WidgetId`] tracking bindings.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the label node.
    /// * `text` - Display string content.
    /// * `font_size` - Font size in logical points.
    /// * `color` - Default foreground text color.
    /// * `hover_color` - Foreground text color when the parent container is hovered.
    /// * `align` - Horizontal text alignment (`Left`, `Center`, `Right`).
    /// * `style` - Explicit layout style applied to the label.
    pub fn label_styled_hover_passive(
        &mut self,
        name: impl Into<String>,
        text: impl Into<String>,
        font_size: f32,
        (color, hover_color): (Color, Color),
        align: TextAlign,
        style: Style,
    ) -> WidgetId {
        let node_id = self.label_styled_passive(name, text, font_size, color, align, style);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_hover_text_color(hover_color);
        }
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

    /// Emits a compact, autonomous status badge pill with rounded corners and themed background.
    ///
    /// Ideal for state indicators (e.g. `[ACTIVE]`, `[OFFLINE]`, `[DIRTY]`, category tags)
    /// without requiring manual flexbox container boilerplate in consumer panels.
    ///
    /// # Arguments
    /// * `text` - Badge label string content.
    /// * `tint` - Accent tint color applied to the badge border and text.
    pub fn badge(&mut self, text: impl Into<String>, tint: Color) -> WidgetId {
        let text_str = text.into();
        let bg_color = Color::rgba(tint.r * 0.15, tint.g * 0.15, tint.b * 0.15, 0.85);
        let border_color = Color::rgba(tint.r * 0.45, tint.g * 0.45, tint.b * 0.45, 0.70);

        let badge_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
            .background(bg_color)
            .border(1.0, border_color)
            .border_radius(4.0)
            .height(18.0);

        self.container(badge_style, |badge_scope| {
            badge_scope.label_custom(text_str, 9.5, tint, TextAlign::Center);
        })
    }

    /// Emits a styled telemetry or key-value pill badge.
    ///
    /// # Arguments
    /// * `label` - Left title label text.
    /// * `value` - Right value text.
    /// * `val_color` - Color of the value readout.
    pub fn key_value_badge(&mut self, label: &str, value: &str, val_color: Color) -> WidgetId {
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
            badge_scope.label_custom(
                label,
                10.0,
                Color::rgba(0.60, 0.63, 0.70, 1.0),
                TextAlign::Left,
            );
            badge_scope.label_custom(value, 10.0, val_color, TextAlign::Right);
        })
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
}