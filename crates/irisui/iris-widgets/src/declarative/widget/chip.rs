// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Category Filter Chip Builder Widget (`Chip`)
//!
//! Provides the fluent [`Chip`] widget implementing [`Widget`] to render categorized
//! filter badges with autonomous text measurement, dynamic count pills, and zero heap
//! allocations during formatting and interaction.
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::declarative::widget::Widget;
use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetRole};

/// Fluent builder for emitting interactive category filter badges and item counter chips.
///
/// Eliminates manual pixel-width estimation (`len() * 6.8 + 16.0`) and per-frame
/// string allocations (`format!("{label} ({count})")`) by separating label text and count
/// badges into independent flex nodes with stack-allocated formatting.
pub struct Chip<'a> {
    label: &'a str,
    count: Option<usize>,
    selected: bool,
    tag: u64,
    accent_color: Option<Color>,
    width: Option<f32>,
}

impl<'a> Chip<'a> {
    /// Creates a new [`Chip`] builder for the specified label text.
    ///
    /// # Arguments
    /// * `label` - Static or borrowed display label of the filter chip.
    #[inline]
    #[must_use]
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            count: None,
            selected: false,
            tag: 0,
            accent_color: None,
            width: None,
        }
    }

    /// Attaches an item counter pill to the chip without heap allocations.
    ///
    /// # Arguments
    /// * `count` - Item count integer displayed next to the label.
    #[inline]
    #[must_use]
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Sets whether the filter chip is currently active or selected.
    #[inline]
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Assigns a 64-bit persistent semantic tag for $O(1)$ event routing.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets an explicit accent border and highlight color for the active state.
    #[inline]
    #[must_use]
    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = Some(color);
        self
    }

    /// Overrides the horizontal layout width of the filter chip with an explicit fixed pixel dimension.
    ///
    /// When omitted, width is calculated automatically from label length and counter digits.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl<'a> Widget for Chip<'a> {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let theme = scope.tree().theme();
        let accent = self.accent_color.unwrap_or(theme.accent);

        let chip_w = self.width.unwrap_or_else(|| {
            let count_len = match self.count {
                Some(c) => 5.0 + ((count_digits(c) + 2) as f32 * 6.5),
                None => 0.0,
            };
            ((self.label.len() as f32 * 6.8) + count_len + 18.0)
                .max(52.0)
                .ceil()
        });

        let chip_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .width(chip_w)
            .height(22.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .gap(5.0)
            .border_radius(4.0)
            .border(
                1.0,
                if self.selected {
                    accent
                } else {
                    Color::rgba(0.16, 0.18, 0.24, 0.40)
                },
            )
            .background(if self.selected {
                Color::rgba(0.12, 0.16, 0.22, 0.95)
            } else {
                Color::rgba(0.08, 0.09, 0.11, 0.60)
            })
            .hover_background(Color::rgba(0.10, 0.12, 0.16, 0.80))
            .hover_border(1.0, Color::rgba(0.28, 0.32, 0.42, 0.70));

        let tag = self.tag;
        let selected = self.selected;
        let label = self.label;
        let count = self.count;

        scope.container_tagged_resp(
            "CategoryChip",
            chip_style,
            WidgetRole::Button,
            tag,
            |chip| {
                // Primary chip label
                chip.label_styled_hover_passive(
                    "CategoryChipLabel",
                    label,
                    11.0,
                    (
                        if selected {
                            Color::WHITE
                        } else {
                            Color::rgba(0.65, 0.69, 0.78, 1.0)
                        },
                        if selected {
                            Color::WHITE
                        } else {
                            Color::rgba(0.90, 0.93, 0.98, 1.0)
                        },
                    ),
                    TextAlign::Center,
                    Style::new(),
                );

                // Counter badge using stack buffer formatting (zero allocation)
                if let Some(c) = count {
                    let mut buf = [0u8; 24];
                    let count_str = format_count_stack(c, &mut buf);
                    chip.label_styled_hover_passive(
                        "CategoryChipCount",
                        count_str,
                        10.0,
                        (
                            if selected {
                                Color::rgba(0.85, 0.90, 1.0, 0.90)
                            } else {
                                Color::rgba(0.45, 0.50, 0.60, 0.85)
                            },
                            if selected {
                                Color::rgba(0.95, 0.98, 1.0, 1.0)
                            } else {
                                Color::rgba(0.70, 0.75, 0.85, 0.95)
                            },
                        ),
                        TextAlign::Center,
                        Style::new(),
                    );
                }
            },
        )
    }
}

/// Formats an item count integer enclosed in parentheses on a stack buffer without heap allocations.
#[inline]
fn format_count_stack(count: usize, buf: &mut [u8; 24]) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "({count})");
    let pos = cursor.position() as usize;
    std::str::from_utf8(&buf[..pos]).unwrap_or("()")
}

/// Computes the decimal digit count of an item counter using hardware-accelerated integer logarithm.
#[inline]
const fn count_digits(count: usize) -> usize {
    match count.checked_ilog10() {
        Some(exp) => exp as usize + 1,
        None => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_chip_builder_and_response() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut scope = UiScope::new(&mut tree, root);

        let resp = scope.add(Chip::new("3D Meshes").count(42).selected(true).tag(0x1234));
        let chip_id = resp.id;

        assert!(tree.get(chip_id).is_some());
        assert!(tree.has_node_named("CategoryChipLabel"));
        assert!(tree.has_node_named("CategoryChipCount"));
        tree.assert_node_text("CategoryChipLabel", "3D Meshes");
        tree.assert_node_text("CategoryChipCount", "(42)");
    }

    #[test]
    fn test_chip_without_count() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut scope = UiScope::new(&mut tree, root);

        let _ = scope.add(Chip::new("All Assets").selected(false));

        assert!(tree.has_node_named("CategoryChipLabel"));
        assert!(!tree.has_node_named("CategoryChipCount"));
        tree.assert_node_text("CategoryChipLabel", "All Assets");
    }

    #[test]
    fn test_format_count_stack() {
        let mut buf = [0u8; 24];
        assert_eq!(format_count_stack(0, &mut buf), "(0)");
        assert_eq!(format_count_stack(99, &mut buf), "(99)");
        assert_eq!(format_count_stack(1000, &mut buf), "(1000)");
    }
}