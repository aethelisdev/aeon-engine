// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative ScrollAreaView Builder
//!
//! Provides the configurable [`ScrollAreaView`] container builder struct.
//! Replaces legacy `scroll_area_tagged` with an idiomatic builder.

use crate::declarative::scope::containers::generic::InnerResponse;
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::scroll_area::ScrollAreaOptions;
use crate::scroll_area::style::ScrollAreaStyle;
use crate::scroll_area::types::ScrollAreaResponse;
use irisui_core::{Color, Insets, Point, Style, WidgetRole};

/// Configurable builder for declarative scrollable viewport containers.
#[derive(Debug, Clone)]
pub struct ScrollAreaView {
    name: &'static str,
    tag: u64,
    width: Option<f32>,
    height: Option<f32>,
    padding: Insets,
    gap: f32,
    background: Option<Color>,
    scrollbar_style: ScrollAreaStyle,
    scrollbar_tags: Option<(u64, u64)>,
    scroll_offset_y: Option<f32>,
    scroll_key: Option<u64>,
    cursor_pos: Option<Point>,
}

impl ScrollAreaView {
    /// Creates a new scroll area builder with descriptive viewport name.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            tag: 0,
            width: None,
            height: None,
            padding: Insets::ZERO,
            gap: 0.0,
            background: None,
            scrollbar_style: ScrollAreaStyle::dark_default(),
            scrollbar_tags: None,
            scroll_offset_y: None,
            scroll_key: None,
            cursor_pos: None,
        }
    }

    /// Sets a persistent 64-bit semantic tag for interaction and scroll offset tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets an explicit logical viewport width constraint.
    #[inline]
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets an explicit logical viewport height constraint.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets inner padding insets applied around scrolled child content.
    #[inline]
    #[must_use]
    pub const fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Sets vertical spacing gap between child elements.
    #[inline]
    #[must_use]
    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the viewport background fill color.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Overrides the visual scrollbar styling used by [`Self::show_scrolled`].
    #[inline]
    #[must_use]
    pub const fn scrollbar_style(mut self, style: ScrollAreaStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    /// Overrides derived `(track_tag, thumb_tag)` identifiers used by [`Self::show_scrolled`].
    #[inline]
    #[must_use]
    pub const fn scrollbar_tags(mut self, track_tag: u64, thumb_tag: u64) -> Self {
        self.scrollbar_tags = Some((track_tag, thumb_tag));
        self
    }

    /// Sets logical cursor coordinates for scrollbar thumb hover feedback in [`Self::show_scrolled`].
    #[inline]
    #[must_use]
    pub const fn cursor_pos(mut self, cursor_pos: Option<Point>) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Sets an explicit vertical scroll offset overriding the tree-stored offset.
    #[inline]
    #[must_use]
    pub const fn scroll_offset_y(mut self, scroll_y: f32) -> Self {
        self.scroll_offset_y = Some(scroll_y);
        self
    }

    /// Associates a domain `key` with this scroll container so key changes reset scroll to `0.0`.
    #[inline]
    #[must_use]
    pub const fn scroll_key(mut self, key: u64) -> Self {
        self.scroll_key = Some(key);
        self
    }

    /// Renders a simple viewport container scope without emitting an automatic scrollbar.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        f: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> InnerResponse<R> {
        if let Some(key) = self.scroll_key {
            scope.sync_scroll_key(self.tag, key);
        }
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .padding_insets(self.padding)
            .gap(self.gap)
            .background(self.background.unwrap_or(Color::TRANSPARENT));
        scope.container_tagged(
            self.name,
            style,
            WidgetRole::ScrollAreaViewport,
            self.tag,
            f,
        )
    }

    /// Renders an autonomous scrollable viewport that measures child heights, clamps scroll
    /// offsets, and automatically draws a styled vertical scrollbar on overflow.
    pub fn show_scrolled<R>(
        self,
        scope: &mut UiScope<'_>,
        f: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> ScrollAreaResponse<R> {
        let mut opts = ScrollAreaOptions::new(self.name, self.tag);
        opts.width = self.width;
        opts.height = self.height;
        opts.padding = self.padding;
        opts.gap = self.gap;
        opts.background = self.background;
        opts.scrollbar_style = self.scrollbar_style;
        opts.scrollbar_tags = self.scrollbar_tags;
        opts.explicit_scroll_y = self.scroll_offset_y;
        opts.scroll_key = self.scroll_key;
        opts.cursor_pos = self.cursor_pos;
        scope.scroll_area_configured(opts, f)
    }
}