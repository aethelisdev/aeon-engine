// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scroll Area Configuration Types
//!
//! Defines [`ScrollAreaOptions`] and [`ScrollAreaConfig`] descriptors used by
//! [`UiScope::scroll_area_configured`](super::core::UiScope::scroll_area_configured)
//! and [`ScrollAreaView`](crate::declarative::widget::ScrollAreaView).

use crate::scroll_area::style::ScrollAreaStyle;
use crate::scroll_area::types::ScrollBarVisibility;
use irisui_core::color::Color;
use irisui_core::geometry::{Insets, Point};

/// Declarative viewport configuration options for [`crate::UiScope::scroll_area_configured`] and [`crate::declarative::widget::scroll_area::ScrollAreaView`].
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollAreaOptions {
    /// Debug identifier assigned to the viewport node in the UI tree.
    pub name: &'static str,
    /// Persistent semantic tag tracking viewport scroll offsets and interactions.
    pub tag: u64,
    /// Optional explicit width in logical pixels (defaults to flex expansion).
    pub width: Option<f32>,
    /// Optional explicit height in logical pixels (defaults to flex growth).
    pub height: Option<f32>,
    /// Inner insets padding around child content.
    pub padding: Insets,
    /// Uniform spacing gap between child elements.
    pub gap: f32,
    /// Optional background fill color for the viewport container.
    pub background: Option<Color>,
    /// Visual style applied when rendering the automatic vertical scrollbar.
    pub scrollbar_style: ScrollAreaStyle,
    /// Optional `(track_tag, thumb_tag)` overriding derived scrollbar tags.
    pub scrollbar_tags: Option<(u64, u64)>,
    /// Optional explicit scroll offset overriding tree-managed offset.
    pub explicit_scroll_y: Option<f32>,
    /// Optional domain key; when it changes between frames for `tag`, scroll offset auto-resets to `0.0`.
    pub scroll_key: Option<u64>,
    /// Optional cursor position in logical coordinates for scrollbar hover evaluation.
    pub cursor_pos: Option<Point>,
}

impl ScrollAreaOptions {
    /// Creates options for an autonomous scrollable container with `name` and semantic `tag`.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, tag: u64) -> Self {
        Self {
            name,
            tag,
            width: None,
            height: None,
            padding: Insets::ZERO,
            gap: 0.0,
            background: None,
            scrollbar_style: ScrollAreaStyle::dark_default(),
            scrollbar_tags: None,
            explicit_scroll_y: None,
            scroll_key: None,
            cursor_pos: None,
        }
    }

    /// Associates a domain key with this scroll viewport so key transitions auto-reset scroll offset.
    #[inline]
    #[must_use]
    pub const fn with_scroll_key(mut self, key: u64) -> Self {
        self.scroll_key = Some(key);
        self
    }
}

/// Configuration options customizing a declarative scroll area container.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollAreaConfig {
    /// Debug identifier assigned to the container node in the UI tree.
    pub name: &'static str,
    /// Current vertical scroll position in logical pixels.
    pub scroll_y: f32,
    /// Total virtual or measured height of all contained content in logical pixels.
    pub content_height: f32,
    /// Visual style parameters controlling scrollbar geometry, colors, and margins.
    pub style: ScrollAreaStyle,
    /// Visibility policy determining when the scrollbar indicator is rendered.
    pub visibility: ScrollBarVisibility,
    /// Optional background fill color applied to the viewport container node.
    pub background: Option<Color>,
    /// Optional cursor position in logical pixels for thumb hover evaluation.
    pub cursor_pos: Option<Point>,
    /// Whether the scrollbar thumb is actively undergoing dragging.
    pub is_dragging: bool,
}

impl Default for ScrollAreaConfig {
    #[inline]
    fn default() -> Self {
        Self {
            name: "ScrollArea",
            scroll_y: 0.0,
            content_height: 0.0,
            style: ScrollAreaStyle::default(),
            visibility: ScrollBarVisibility::Auto,
            background: None,
            cursor_pos: None,
            is_dragging: false,
        }
    }
}

impl ScrollAreaConfig {
    /// Creates a default scroll area configuration with the given content height and scroll offset.
    #[inline]
    pub fn new(content_height: f32, scroll_y: f32) -> Self {
        Self {
            content_height: content_height.max(0.0),
            scroll_y: scroll_y.max(0.0),
            ..Default::default()
        }
    }

    /// Sets the debug and telemetry name for the container node.
    #[inline]
    pub const fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets an explicit background color for the viewport container.
    #[inline]
    pub const fn with_background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Overrides the visual style of the scrollbar indicator.
    #[inline]
    pub const fn with_style(mut self, style: ScrollAreaStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the scrollbar visibility policy.
    #[inline]
    pub const fn with_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets the active dragging flag for the scrollbar thumb.
    #[inline]
    pub const fn with_dragging(mut self, is_dragging: bool) -> Self {
        self.is_dragging = is_dragging;
        self
    }
}