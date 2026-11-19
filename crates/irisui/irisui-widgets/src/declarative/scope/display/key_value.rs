// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Key-Value Rows, Formatted Numbers & Telemetry Badges
//!
//! Provides zero-allocation, structured key-value display primitives on [`UiScope`]:
//! - Horizontal key-value row emitters ([`UiScope::key_value`], [`UiScope::key_value_styled`]).
//! - Zero-heap stack formatted file size rows ([`UiScope::key_value_file_size`]).
//! - Zero-heap stack formatted floating-point number labels and rows ([`UiScope::label_number`], [`UiScope::key_value_number`]).
//! - Structured telemetry and status key-value badges ([`UiScope::key_value_badge`]).

use crate::declarative::scope::core::UiScope;
use irisui_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign};

impl<'a> UiScope<'a> {
    /// Emits a styled telemetry or key-value pill badge.
    ///
    /// # Arguments
    /// * `label` - Left title label text.
    /// * `value` - Right value text.
    /// * `val_color` - Color of the value readout.
    pub fn key_value_badge(&mut self, label: &str, value: &str, val_color: Color) {
        let theme = self.tree.theme();
        let badge_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0))
            .background(theme.card_bg)
            .border(1.0, theme.card_border)
            .border_radius(4.0)
            .height(22.0);

        self.container(badge_style, |badge_scope| {
            badge_scope.label_custom(label, 10.0, theme.text_secondary, TextAlign::Left);
            badge_scope.label_custom(value, 10.0, val_color, TextAlign::Right);
        });
    }

    /// Emits a structured horizontal key-value row with secondary key and primary value text.
    ///
    /// Eliminates `format!("{key}: {val}")` heap allocations in inspector, stats, and preview dialogs
    /// by creating two independent text nodes in a lightweight flex row container.
    ///
    /// # Arguments
    /// * `key` - Left property or metadata key title (rendered with muted text color).
    /// * `val` - Right value text content (rendered with primary text color).
    pub fn key_value(&mut self, key: &str, val: &str) {
        let theme = self.tree.theme();
        self.key_value_styled(key, val, theme.text_muted, theme.text_primary, 11.5);
    }

    /// Emits a structured horizontal key-value row with custom container identifier.
    ///
    /// # Arguments
    /// * `row_name` - Container node identifier.
    /// * `key` - Left property or metadata key title.
    /// * `val` - Right value text content.
    pub fn key_value_named(&mut self, row_name: impl Into<String>, key: &str, val: &str) {
        let theme = self.tree.theme();
        self.key_value_styled_named(
            row_name,
            key,
            val,
            theme.text_muted,
            theme.text_primary,
            11.5,
        );
    }

    /// Emits a structured horizontal key-value row with customized text colors and font size.
    ///
    /// # Arguments
    /// * `key` - Left property or metadata key title.
    /// * `val` - Right value text content.
    /// * `key_color` - Foreground color for the key label and colon separator.
    /// * `val_color` - Foreground color for the value text.
    /// * `font_size` - Size in points for all text nodes in the row.
    pub fn key_value_styled(
        &mut self,
        key: &str,
        val: &str,
        key_color: Color,
        val_color: Color,
        font_size: f32,
    ) {
        self.key_value_styled_named("KeyValueRow", key, val, key_color, val_color, font_size);
    }

    /// Emits a structured horizontal key-value row with custom container identifier, text colors, and font size.
    ///
    /// # Arguments
    /// * `row_name` - Container node identifier.
    /// * `key` - Left property or metadata key title.
    /// * `val` - Right value text content.
    /// * `key_color` - Foreground color for the key label and colon separator.
    /// * `val_color` - Foreground color for the value text.
    /// * `font_size` - Size in points for all text nodes in the row.
    pub fn key_value_styled_named(
        &mut self,
        row_name: impl Into<String>,
        key: &str,
        val: &str,
        key_color: Color,
        val_color: Color,
        font_size: f32,
    ) {
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(6.0);

        self.container_named(row_name, row_style, |row| {
            row.label_styled_passive(
                "KeyValueLabel",
                key,
                font_size,
                key_color,
                TextAlign::Left,
                Style::new(),
            );
            row.label_styled_passive(
                "KeyValueSeparator",
                ":",
                font_size,
                key_color,
                TextAlign::Left,
                Style::new(),
            );
            row.label_styled_passive(
                "KeyValueContent",
                val,
                font_size,
                val_color,
                TextAlign::Left,
                Style::new(),
            );
        });
    }

    /// Emits a key-value row where the value is a byte count formatted as human-readable size (B, KB, MB, GB).
    ///
    /// Operates completely on a stack buffer with zero heap allocations.
    ///
    /// # Arguments
    /// * `key` - Left property key title.
    /// * `bytes` - Integer file or buffer size in bytes.
    pub fn key_value_file_size(&mut self, key: &str, bytes: u64) {
        let mut buf = [0u8; 32];
        let val_str = format_file_size_stack(bytes, &mut buf);
        self.key_value(key, val_str);
    }

    /// Emits a floating-point number label formatted to `precision` decimal digits without heap allocation.
    ///
    /// Uses an internal stack buffer to format the number directly into the node text slice.
    ///
    /// # Arguments
    /// * `val` - Floating-point number to render.
    /// * `precision` - Number of fractional decimal places to retain.
    pub fn label_number(&mut self, val: f64, precision: usize) {
        let mut buf = [0u8; 32];
        let num_str = format_number_stack(val, precision, &mut buf);
        self.label(num_str);
    }

    /// Emits a structured horizontal key-value row formatting a float value without heap allocations.
    ///
    /// # Arguments
    /// * `key` - Left property key title.
    /// * `val` - Numeric float value to format.
    /// * `precision` - Fractional precision places.
    pub fn key_value_number(&mut self, key: &str, val: f64, precision: usize) {
        self.key_value_number_named("KeyValueNumberRow", key, val, precision);
    }

    /// Emits a structured horizontal key-value row formatting a float value with custom container identifier.
    ///
    /// Operates completely on a stack buffer with zero heap allocations.
    ///
    /// # Arguments
    /// * `row_name` - Container node identifier.
    /// * `key` - Left property key title.
    /// * `val` - Numeric float value to format.
    /// * `precision` - Fractional precision places.
    pub fn key_value_number_named(
        &mut self,
        row_name: impl Into<String>,
        key: &str,
        val: f64,
        precision: usize,
    ) {
        let mut buf = [0u8; 32];
        let num_str = format_number_stack(val, precision, &mut buf);
        self.key_value_named(row_name, key, num_str);
    }

    /// Emits a styled passive label displaying a formatted number via [`NumberLabelDescriptor`],
    /// operating entirely on a stack buffer with zero heap allocations.
    ///
    /// # Arguments
    /// * `name` - Container node identifier.
    /// * `desc` - Descriptor configuring the number format, prefix, suffix, and typography.
    pub fn label_number_desc(&mut self, name: impl Into<String>, desc: NumberLabelDescriptor<'_>) {
        let mut buf = [0u8; 64];
        let text = format_prefix_number_suffix_stack(
            desc.prefix,
            desc.val,
            desc.precision,
            desc.suffix,
            &mut buf,
        );
        let text_color = desc.color.unwrap_or(self.theme().text_primary);
        self.label_styled_passive(
            name,
            text,
            desc.font_size,
            text_color,
            TextAlign::Left,
            Style::new().height(desc.height),
        );
    }

    /// Emits a two-line title and muted description row paired with a right-aligned tinted pill badge.
    ///
    /// Designed for keybinding tables, command reference lists, and annotated setting rows
    /// without manual flexbox column/pill container nesting at consumer call sites.
    ///
    /// # Arguments
    /// * `title` - Primary action or item title displayed in primary text color.
    /// * `note` - Secondary explanatory subtitle displayed in muted text color below the title.
    /// * `badge` - Compact pill text rendered on the trailing edge (e.g., `"Ctrl + Z"`, `"Default"`).
    /// * `badge_tint` - Foreground text color used to derive the badge's translucent fill and border.
    pub fn titled_badge_row(&mut self, title: &str, note: &str, badge: &str, badge_tint: Color) {
        let theme = self.theme();
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding_insets(Insets::new(4.0, 0.0, 4.0, 0.0));
        let badge_style = Style::new()
            .background(Color::rgba(badge_tint.r, badge_tint.g, badge_tint.b, 0.15))
            .border(
                1.0,
                Color::rgba(badge_tint.r, badge_tint.g, badge_tint.b, 0.40),
            )
            .border_radius(4.0)
            .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0));

        self.container_named("TitledBadgeRow", row_style, |row| {
            row.container_named(
                "TitledBadgeLeftCol",
                Style::new().flex_col().flex_grow(1.0),
                |left| {
                    left.label_styled_passive(
                        "TitledBadgeTitle",
                        title,
                        11.5,
                        theme.text_primary,
                        TextAlign::Left,
                        Style::new(),
                    );
                    left.label_styled_passive(
                        "TitledBadgeNote",
                        note,
                        10.5,
                        theme.text_muted,
                        TextAlign::Left,
                        Style::new(),
                    );
                },
            );

            row.container_named("TitledBadgePill", badge_style, |pill| {
                pill.label_styled_passive(
                    "TitledBadgeText",
                    badge,
                    11.0,
                    badge_tint,
                    TextAlign::Center,
                    Style::new(),
                );
            });
        });
    }
}

/// Configuration descriptor for formatting a floating-point number label with prefix, suffix, and typography.
#[derive(Debug, Clone, Copy)]
pub struct NumberLabelDescriptor<'a> {
    /// Static prefix text preceding the formatted number.
    pub prefix: &'a str,
    /// Numeric float value to format.
    pub val: f64,
    /// Fractional decimal precision places.
    pub precision: usize,
    /// Static suffix text trailing the formatted number.
    pub suffix: &'a str,
    /// Font size in points.
    pub font_size: f32,
    /// Optional custom text foreground color. When `None`, dynamically defaults to `theme.text_primary`.
    pub color: Option<Color>,
    /// Container height in logical pixels.
    pub height: f32,
}

impl<'a> NumberLabelDescriptor<'a> {
    /// Constructs a [`NumberLabelDescriptor`] with 1 fractional digit precision, default 11pt font size,
    /// and theme-adaptive text color.
    #[inline]
    #[must_use]
    pub const fn new(prefix: &'a str, val: f64, suffix: &'a str) -> Self {
        Self {
            prefix,
            val,
            precision: 1,
            suffix,
            font_size: 11.0,
            color: None,
            height: 20.0,
        }
    }

    /// Sets the fractional decimal precision places.
    #[inline]
    #[must_use]
    pub const fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Sets the label font size in points.
    #[inline]
    #[must_use]
    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Sets a custom label text color.
    #[inline]
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the container height in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

/// Formats a prefix string, float number, and suffix string into a stack buffer without heap allocations.
#[inline]
pub fn format_prefix_number_suffix_stack<'b>(
    prefix: &str,
    val: f64,
    precision: usize,
    suffix: &str,
    buf: &'b mut [u8; 64],
) -> &'b str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{prefix}{val:.precision$}{suffix}");
    let pos = cursor.position() as usize;
    std::str::from_utf8(&buf[..pos]).unwrap_or("")
}

/// Formats a 64-bit integer file size in bytes to a human-readable string without heap allocations.
#[inline]
pub fn format_file_size_stack(bytes: u64, buf: &mut [u8; 32]) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    if bytes < 1024 {
        let _ = write!(cursor, "{bytes} B");
    } else if bytes < 1024 * 1024 {
        let _ = write!(cursor, "{:.1} KB", bytes as f64 / 1024.0);
    } else if bytes < 1024 * 1024 * 1024 {
        let _ = write!(cursor, "{:.2} MB", bytes as f64 / (1024.0 * 1024.0));
    } else {
        let _ = write!(
            cursor,
            "{:.2} GB",
            bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        );
    }
    let pos = cursor.position() as usize;
    std::str::from_utf8(&buf[..pos]).unwrap_or("")
}

/// Formats a float number with specified precision onto a stack buffer without heap allocations.
#[inline]
pub fn format_number_stack(val: f64, precision: usize, buf: &mut [u8; 32]) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{val:.precision$}");
    let pos = cursor.position() as usize;
    std::str::from_utf8(&buf[..pos]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_titled_badge_row_layout_and_badge_text() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.titled_badge_row(
                "Undo",
                "Revert last transform action.",
                "Ctrl + Z",
                Color::rgba(0.0, 0.90, 1.0, 1.0),
            );
        }

        let root_node = tree.get(root).expect("Root node exists");
        assert_eq!(root_node.children.len(), 1);
        let row = tree.get(root_node.children[0]).expect("Row exists");
        assert_eq!(row.name.as_deref(), Some("TitledBadgeRow"));
        assert_eq!(row.children.len(), 2);

        let left = tree.get(row.children[0]).expect("Left column exists");
        assert_eq!(left.children.len(), 2);
        let title = tree.get(left.children[0]).expect("Title exists");
        let note = tree.get(left.children[1]).expect("Note exists");
        assert_eq!(title.text.as_deref(), Some("Undo"));
        assert_eq!(note.text.as_deref(), Some("Revert last transform action."));

        let pill = tree.get(row.children[1]).expect("Badge pill exists");
        assert_eq!(pill.name.as_deref(), Some("TitledBadgePill"));
        let pill_text = tree.get(pill.children[0]).expect("Pill text exists");
        assert_eq!(pill_text.text.as_deref(), Some("Ctrl + Z"));
    }
}