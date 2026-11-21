// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Stack-Formatted Badge Pill Primitive (`UiScope::badge_fmt`)
//!
//! Emits compact fixed-height pill badges whose width adapts automatically to the formatted
//! [`StackString`] character count without intermediate heap string or `let` allocations.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::display::live::LiveTextProps;
use irisui_core::{
    AlignItems, Color, Insets, JustifyContent, StackString, Style, TextAlign, TextWrap,
    WidgetResponse, WidgetRole,
};

/// Visual configuration descriptor for [`UiScope::badge_fmt`], [`UiScope::badge_live_fmt`], and [`UiScope::badge_text`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgePillSpec {
    /// Debug node name assigned to the badge container.
    pub name: &'static str,
    /// Persistent 64-bit semantic tag for inspection or click queries.
    pub tag: u64,
    /// Optional minimum horizontal capsule width in logical pixels (prevents jitter on live readouts).
    pub min_width: Option<f32>,
    /// Badge capsule height in logical pixels.
    pub height: f32,
    /// Typography point size for badge label text.
    pub font_size: f32,
    /// Estimated average horizontal character advance multiplier for intrinsic capsule sizing.
    pub char_width_est: f32,
    /// Left and right padding inside the badge capsule in logical pixels.
    pub horizontal_padding: f32,
    /// Capsule background fill color.
    pub background: Color,
    /// Capsule border stroke color.
    pub border_color: Color,
    /// Capsule corner rounding radius in logical pixels.
    pub border_radius: f32,
    /// Foreground typography color.
    pub text_color: Color,
}

impl BadgePillSpec {
    /// Creates a compact `24px` pill badge descriptor with explicit semantic tag and palette colors.
    #[inline]
    #[must_use]
    pub const fn new(
        name: &'static str,
        tag: u64,
        background: Color,
        border_color: Color,
        text_color: Color,
    ) -> Self {
        Self {
            name,
            tag,
            min_width: None,
            height: 24.0,
            font_size: 10.0,
            char_width_est: 6.5,
            horizontal_padding: 8.0,
            background,
            border_color,
            border_radius: 4.0,
            text_color,
        }
    }

    /// Sets a minimum capsule width in logical pixels to prevent truncation on dynamic live text updates.
    #[inline]
    #[must_use]
    pub const fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Overrides typography point size and average character advance multiplier.
    #[inline]
    #[must_use]
    pub const fn typography(mut self, font_size: f32, char_width_est: f32) -> Self {
        self.font_size = font_size;
        self.char_width_est = char_width_est;
        self
    }

    /// Overrides the fixed height of the badge capsule in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a compact pill badge formatted into a stack-allocated [`StackString`] buffer,
    /// computing intrinsic capsule width automatically without heap allocations.
    pub fn badge_fmt<const N: usize, F>(&mut self, spec: BadgePillSpec, fmt_fn: F) -> WidgetResponse
    where
        F: FnOnce(&mut StackString<N>),
    {
        let mut buf = StackString::<N>::new();
        fmt_fn(&mut buf);
        self.badge_text(spec, buf.as_str())
    }

    /// Emits a compact pill badge whose inner label is registered as a volatile live text node
    /// (`text_tag`), enabling zero-relayout 60 FPS in-place updates via [`UiScope::update_live_text_fmt`].
    pub fn badge_live_fmt<const N: usize, F>(
        &mut self,
        spec: BadgePillSpec,
        text_tag: u64,
        fmt_fn: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut StackString<N>),
    {
        let mut buf = StackString::<N>::new();
        fmt_fn(&mut buf);
        let text = buf.as_str();
        let text_w = ((text.chars().count() as f32 * spec.char_width_est).ceil()
            + spec.horizontal_padding * 2.0
            + 2.0)
            .max(spec.min_width.unwrap_or(0.0));
        let badge_style = Style::new()
            .width(text_w)
            .height(spec.height)
            .background(spec.background)
            .border(1.0, spec.border_color)
            .border_radius(spec.border_radius)
            .padding_insets(Insets::new(
                2.0,
                spec.horizontal_padding,
                2.0,
                spec.horizontal_padding,
            ))
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .flex_shrink(0.0);

        self.container_tagged(spec.name, badge_style, WidgetRole::Default, spec.tag, |b| {
            b.live_text(
                LiveTextProps::new(
                    "BadgeLiveText",
                    text_tag,
                    text,
                    spec.font_size,
                    spec.text_color,
                    Style::new().flex_grow(1.0),
                )
                .with_align(TextAlign::Center)
                .with_wrap(TextWrap::None),
            );
        })
        .response
    }

    /// Emits a compact pill badge displaying a borrowed string `text`, sizing capsule width from
    /// glyph count and `spec` typography metrics.
    pub fn badge_text(&mut self, spec: BadgePillSpec, text: &str) -> WidgetResponse {
        let text_w = ((text.chars().count() as f32 * spec.char_width_est).ceil()
            + spec.horizontal_padding * 2.0
            + 2.0)
            .max(spec.min_width.unwrap_or(0.0));
        let badge_style = Style::new()
            .width(text_w)
            .height(spec.height)
            .background(spec.background)
            .border(1.0, spec.border_color)
            .border_radius(spec.border_radius)
            .padding_insets(Insets::new(
                2.0,
                spec.horizontal_padding,
                2.0,
                spec.horizontal_padding,
            ))
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .flex_shrink(0.0);

        self.container_tagged(spec.name, badge_style, WidgetRole::Default, spec.tag, |b| {
            b.label_custom(text, spec.font_size, spec.text_color, TextAlign::Center);
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::PanelBuilder;
    use irisui_core::UiTree;
    use std::fmt::Write;

    #[test]
    fn test_badge_fmt_emits_tagged_capsule_and_formatted_text() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let tag = 0xBEE1;
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.badge_fmt::<32, _>(
                BadgePillSpec::new("ClipBadge", tag, Color::BLACK, Color::CYAN, Color::WHITE),
                |b| {
                    let _ = write!(b, "F{} / F{}", 12, 90);
                },
            );
        }

        let badge_id = tree.find_node_by_tag(tag).expect("badge node");
        assert_eq!(badge_id, tree.find_node_by_tag(tag).expect("badge node"));
        assert!(tree.get(badge_id).expect("node").style.width.unwrap_or(0.0) > 40.0);
        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("F12 / F90"))
        );
    }

    #[test]
    fn test_badge_live_fmt_registers_live_text_and_updates_in_place() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let pill_tag = 0xBEE2;
        let text_tag = 0xBEE3;
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.badge_live_fmt::<32, _>(
                BadgePillSpec::new(
                    "LiveTime",
                    pill_tag,
                    Color::BLACK,
                    Color::CYAN,
                    Color::WHITE,
                )
                .min_width(140.0),
                text_tag,
                |b| {
                    let _ = write!(b, "0.00s / 5.00s");
                },
            );
        }

        let pill_node = tree.node_tagged(pill_tag).expect("pill node");
        assert!(pill_node.style.width.unwrap_or(0.0) >= 140.0);
        assert!(tree.is_live_tag(text_tag));

        let changed = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.update_live_text_fmt::<32, _>(text_tag, |b| {
                let _ = write!(b, "2.50s / 5.00s");
            })
        };
        assert!(changed);
        assert_eq!(
            tree.node_tagged(text_tag).and_then(|n| n.text.as_deref()),
            Some("2.50s / 5.00s")
        );
    }
}