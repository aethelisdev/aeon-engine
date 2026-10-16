// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative ScrollAreaView Builder
//!
//! Provides the configurable [`ScrollAreaView`] container builder struct.
//! Replaces legacy `scroll_area_tagged` with an idiomatic builder.
//!

use crate::declarative::scope::containers::generic::InnerResponse;
use crate::declarative::scope::core::UiScope;
use iris_core::{Color, Style, WidgetRole};

/// Configurable builder for declarative scrollable viewport containers.
#[derive(Debug, Clone)]
pub struct ScrollAreaView {
    name: &'static str,
    tag: u64,
    background: Color,
}

impl ScrollAreaView {
    /// Creates a new scroll area builder with descriptive viewport name.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            tag: 0,
            background: Color::TRANSPARENT,
        }
    }

    /// Sets a persistent 64-bit semantic tag for interaction and scroll offset tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets the viewport background fill color.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = bg;
        self
    }

    /// Renders the scroll container scope around the provided child content closure.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        f: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> InnerResponse<R> {
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .background(self.background);
        scope.container_tagged(
            self.name,
            style,
            WidgetRole::ScrollAreaViewport,
            self.tag,
            f,
        )
    }
}