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

pub use super::key_value::{format_file_size_stack, format_number_stack};
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, TextWrap, WidgetCursor, WidgetRole,
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
    /// Optional foreground text color applied when the parent container is hovered.
    pub hover_color: Option<Color>,
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
            hover_color: None,
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

    /// Sets the foreground text color applied when the parent container is hovered.
    #[inline]
    pub fn hover_color(mut self, hover_color: Color) -> Self {
        self.hover_color = Some(hover_color);
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

/// Descriptor struct configuring typography, prefix sizing, and colors for a labeled status row.
///
/// Prevents parameter explosion on [`UiScope::labeled_status_desc`].
#[derive(Debug, Clone, PartialEq)]
pub struct LabeledStatusDescriptor {
    /// Static prefix text (e.g. `"Status:"`).
    pub prefix: String,
    /// Formatted status text (e.g. `"▶ PLAYING"`).
    pub value: String,
    /// Color of the status value text.
    pub color: Color,
    /// Optional debug node name for the prefix text node.
    pub prefix_name: Option<String>,
    /// Optional debug node name for the value text node.
    pub value_name: Option<String>,
    /// Optional fixed physical pixel width for the prefix label column.
    pub prefix_width: Option<f32>,
    /// Typographic point size for row text nodes.
    pub font_size: f32,
    /// Vertical physical pixel height for the row container.
    pub height: f32,
}

impl LabeledStatusDescriptor {
    /// Constructs a labeled status descriptor with auto-sizing prefix width and standard typography.
    #[inline]
    pub fn new(prefix: impl Into<String>, value: impl Into<String>, color: Color) -> Self {
        Self {
            prefix: prefix.into(),
            value: value.into(),
            color,
            prefix_name: None,
            value_name: None,
            prefix_width: None,
            font_size: 11.0,
            height: 20.0,
        }
    }

    /// Overrides the debug node names for prefix and value text labels.
    #[inline]
    pub fn with_names(
        mut self,
        prefix_name: impl Into<String>,
        value_name: impl Into<String>,
    ) -> Self {
        self.prefix_name = Some(prefix_name.into());
        self.value_name = Some(value_name.into());
        self
    }

    /// Overrides the fixed physical pixel width for the prefix label.
    #[inline]
    pub fn prefix_width(mut self, width: f32) -> Self {
        self.prefix_width = Some(width);
        self
    }

    /// Overrides the font size in logical points.
    #[inline]
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Overrides the vertical container height.
    #[inline]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
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
    pub fn label(&mut self, text: impl Into<String>) {
        let color = self.tree.theme().text_secondary;
        self.label_custom(text, 11.5, color, TextAlign::Left);
    }

    /// Emits a prominent section heading text label node with bold white foreground coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn heading(&mut self, text: impl Into<String>) {
        let color = self.tree.theme().text_primary;
        self.label_custom(text, 13.5, color, TextAlign::Left);
    }

    /// Emits a secondary section subtitle text label node with muted light gray coloring.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    pub fn subheading(&mut self, text: impl Into<String>) {
        let color = self.tree.theme().text_secondary;
        self.label_custom(text, 12.0, color, TextAlign::Left);
    }

    /// Emits a text label node with an explicit foreground color and standard typography sizing.
    ///
    /// # Arguments
    /// * `color` - Foreground text color.
    /// * `text` - Display string content.
    pub fn colored_label(&mut self, color: Color, text: impl Into<String>) {
        self.label_custom(text, 11.5, color, TextAlign::Left);
    }

    /// Emits a static text label node with default light gray foreground coloring.
    ///
    /// Preserved for backward compatibility across existing views; delegates directly to [`Self::label`].
    ///
    /// # Arguments
    /// * `text` - Display string content.
    #[inline]
    pub fn text(&mut self, text: impl Into<String>) {
        self.label(text);
    }

    /// Emits a static text label node with explicit foreground color.
    ///
    /// Preserved for backward compatibility; delegates directly to [`Self::colored_label`].
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `color` - RGBA foreground color.
    #[inline]
    pub fn text_colored(&mut self, text: impl Into<String>, color: Color) {
        self.colored_label(color, text);
    }

    /// Emits a static text label node with explicit foreground color and semantic tag.
    ///
    /// Useful for telemetry metrics, counter displays, and real-time status labels requiring in-place mutation.
    ///
    /// # Arguments
    /// * `text` - Display string content.
    /// * `color` - RGBA foreground color.
    /// * `tag` - 64-bit semantic tag for in-place reactive updates.
    #[inline]
    pub fn text_colored_tagged(&mut self, text: impl Into<String>, color: Color, tag: u64) {
        let text_str = text.into();
        let est_w = (text_str.len() as f32 * 14.0 * 0.65).ceil().max(16.0);
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.tag = tag;
            node.set_text(text_str);
            node.text_color = color;
            node.font_size = 14.0;
            node.line_height = 18.0;
            node.text_align = TextAlign::Left;
            node.set_style(Style::new().width(est_w));
        }
        let _ = self.tree.add_child(self.parent, node_id);
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
    ) {
        let text_str = text.into();
        let char_count = text_str.chars().count().max(1);
        let est_w = (char_count as f32 * font_size * 0.65).ceil().max(16.0);
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Default;
            node.set_text(text_str);
            node.text_color = color;
            node.font_size = font_size;
            let line_h = (font_size * 1.3).max(14.0).round();
            node.line_height = line_h;
            node.text_align = align;
            node.set_style(Style::new().width(est_w).height(line_h));
        }
        let _ = self.tree.add_child(self.parent, node_id);
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
    ) {
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
    ) {
        self.label_with_width(text, width, font_size, color, align);
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
    ) {
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
    ) {
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
    ) {
        let name_str = name.into();
        self.label_styled_passive(name_str.clone(), text, font_size, color, align, style);
        if let Some(node) = self
            .tree
            .find_node_by_name_in_subtree(self.parent, &name_str)
            .and_then(|id| self.tree.get_mut(id))
        {
            node.set_hover_text_color(hover_color);
        }
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
    ) {
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
            if let Some(hover_color) = desc.hover_color {
                node.set_hover_text_color(hover_color);
            }
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a compact, autonomous status badge pill with rounded corners and themed background.
    ///
    /// Ideal for state indicators (e.g. `[ACTIVE]`, `[OFFLINE]`, `[DIRTY]`, category tags)
    /// without requiring manual flexbox container boilerplate in consumer panels.
    ///
    /// # Arguments
    /// * `text` - Badge label string content.
    /// * `tint` - Accent tint color applied to the badge border and text.
    pub fn badge(&mut self, text: impl Into<String>, tint: Color) {
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
        });
    }

    /// Emits a styled status pill row with a named container and passive text label.
    ///
    /// Eliminates deeply nested container closures and multi-level indentation pyramids
    /// when rendering status indicators and state readouts in inspector cards.
    ///
    /// # Arguments
    /// * `container_name` - Descriptive debug identifier for the pill container node.
    /// * `label_name` - Descriptive debug identifier for the inner text label node.
    /// * `text` - Display status text string.
    /// * `color` - Status text foreground color.
    pub fn status_pill(
        &mut self,
        container_name: impl Into<String>,
        label_name: impl Into<String>,
        text: impl Into<String>,
        color: Color,
    ) {
        self.status_pill_styled(container_name, label_name, text, 10.5, color, 18.0);
    }

    /// Emits a compact status badge pill with automatic container and label identifiers.
    ///
    /// Eliminates caller-side node naming boilerplate when displaying status flags (e.g. `[🛡 Grounded]`).
    ///
    /// # Arguments
    /// * `text` - Display status text string.
    /// * `color` - Status text foreground color.
    #[inline]
    pub fn status_badge(&mut self, text: impl Into<String>, color: Color) {
        self.status_pill("StatusBadge", "StatusText", text, color);
    }

    /// Renders a muted, passive description or hint label inside a card container.
    ///
    /// Standardizes font sizing, padding, and muted typography without requiring
    /// complex styling structs or manual line wrapping at caller sites.
    ///
    /// # Arguments
    /// * `text` - Descriptive text string displayed in the card.
    #[inline]
    pub fn card_description(&mut self, text: impl Into<String>) {
        self.label_desc("CardDesc", text);
    }

    /// Renders a muted, passive description label with specified node name and text.
    ///
    /// Eliminates boilerplate styling parameters (theme color, font size, alignment, margins)
    /// when rendering standard inspector description texts.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier for the text node.
    /// * `text` - Display text string.
    #[inline]
    pub fn label_desc(&mut self, name: impl Into<String>, text: impl Into<String>) {
        let color = self.tree.theme().text_muted;
        self.label_desc_colored(name, text, color);
    }

    /// Renders a passive description label with specified node name, text, and custom foreground color.
    ///
    /// Standardizes margins and typography while allowing dynamic informational or warning coloring.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier for the text node.
    /// * `text` - Display text string.
    /// * `color` - Text foreground color.
    #[inline]
    pub fn label_desc_colored(
        &mut self,
        name: impl Into<String>,
        text: impl Into<String>,
        color: Color,
    ) {
        self.label_styled_passive(
            name,
            text,
            10.5,
            color,
            TextAlign::Left,
            Style::new().margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
        );
    }

    /// Emits a configurable labeled status row with custom prefix width, typography size, and row height.
    ///
    /// # Arguments
    /// * `row_name` - Container identifier for the horizontal status row.
    /// * `desc` - Layout and styling descriptor.
    pub fn labeled_status_desc(
        &mut self,
        row_name: impl Into<String>,
        desc: LabeledStatusDescriptor,
    ) {
        let theme = self.theme();
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(desc.height)
            .gap(6.0);
        let mut prefix_style = Style::new().height(desc.height);
        if let Some(w) = desc.prefix_width {
            prefix_style = prefix_style.width(w);
        }
        let prefix_name = desc.prefix_name.as_deref().unwrap_or("LabeledStatusPrefix");
        let value_name = desc.value_name.as_deref().unwrap_or("LabeledStatusValue");
        self.container_named(row_name, style, |row| {
            row.label_styled_passive(
                prefix_name,
                desc.prefix,
                desc.font_size,
                theme.text_muted,
                TextAlign::Left,
                prefix_style,
            );
            row.label_styled_passive(
                value_name,
                desc.value,
                desc.font_size,
                desc.color,
                TextAlign::Left,
                Style::new().flex_grow(1.0).height(desc.height),
            );
        });
    }

    /// Emits a labeled status row containing an auto-sized prefix label and aligned colored status text.
    ///
    /// Automatically adapts the prefix column width to its text content and applies theme typography defaults.
    ///
    /// # Arguments
    /// * `prefix` - Static prefix text (e.g. `"Status:"`).
    /// * `value` - Formatted status text (e.g. `"▶ PLAYING"`).
    /// * `color` - Color of the status value text.
    pub fn labeled_status(
        &mut self,
        prefix: impl Into<String>,
        value: impl Into<String>,
        color: Color,
    ) {
        self.labeled_status_desc(
            "LabeledStatusRow",
            LabeledStatusDescriptor::new(prefix, value, color),
        );
    }

    /// Emits a styled status pill row with custom font size, text color, and row height.
    ///
    /// Eliminates deeply nested container closures and multi-level indentation pyramids
    /// while granting total layout freedom over typographic scale and capsule height.
    ///
    /// # Arguments
    /// * `container_name` - Descriptive debug identifier for the pill container node.
    /// * `label_name` - Descriptive debug identifier for the inner text label node.
    /// * `text` - Display status text string.
    /// * `font_size` - Typographic point size for the label.
    /// * `color` - Status text foreground color.
    /// * `height` - Vertical height of the status capsule row in physical pixels.
    pub fn status_pill_styled(
        &mut self,
        container_name: impl Into<String>,
        label_name: impl Into<String>,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        height: f32,
    ) {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(height);
        self.container_named(container_name, style, |scope| {
            scope.label_styled_passive(
                label_name,
                text,
                font_size,
                color,
                TextAlign::Left,
                Style::new().height(height),
            );
        });
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