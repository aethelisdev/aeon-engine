// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toolbar Container Builder
//!
//! Provides the configurable [`Toolbar`] container builder struct.
//! Replaces legacy `toolbar_tagged` with an idiomatic builder.
//!

use crate::declarative::scope::containers::generic::InnerResponse;
use crate::declarative::scope::core::UiScope;
use iris_core::{AlignItems, Color, Insets, Style, WidgetRole};

/// Configurable builder for horizontal toolbar strips.
#[derive(Debug, Clone)]
pub struct Toolbar {
    name: &'static str,
    height: f32,
    tag: u64,
    background: Color,
}

impl Toolbar {
    /// Creates a new toolbar container builder with descriptive name.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            height: 34.0,
            tag: 0,
            background: Color::rgba(0.08, 0.09, 0.11, 0.95),
        }
    }

    /// Sets the toolbar height in physical pixels.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets the toolbar background fill color.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = bg;
        self
    }

    /// Renders the toolbar container scope around the provided child content closure.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        f: impl FnOnce(&mut UiScope<'_>) -> R,
    ) -> InnerResponse<R> {
        let style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(self.height)
            .background(self.background)
            .gap(6.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08));
        scope.container_tagged(self.name, style, WidgetRole::Default, self.tag, f)
    }
}