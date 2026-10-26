// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Toolbar Container Builder
//!
//! Provides the configurable [`Toolbar`] builder for horizontal strips and floating pill toolbars.
//!

use super::scope::ToolbarScope;
use crate::declarative::scope::containers::generic::InnerResponse;
use crate::declarative::scope::core::UiScope;
use iris_core::{AlignItems, Color, CornerRadii, Insets, Style, WidgetRole};

/// Configurable builder for horizontal toolbar strips and floating pill toolbars.
///
/// Supports absolute screen anchoring, pill glass styling, customizable heights and gaps,
/// and nested modular [`ToolbarGroup`](super::group::ToolbarGroup) sections.
#[derive(Debug, Clone)]
pub struct Toolbar {
    name: &'static str,
    position: Option<(f32, f32)>,
    width: Option<f32>,
    height: f32,
    gap: f32,
    tag: u64,
    background: Option<Color>,
    border_color: Option<Color>,
    corner_radius: Option<f32>,
    is_pill: bool,
    is_transparent: bool,
}

impl Toolbar {
    /// Creates a new inline toolbar container builder with a descriptive debug name.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            position: None,
            width: None,
            height: 32.0,
            gap: 8.0,
            tag: 0,
            background: None,
            border_color: None,
            corner_radius: None,
            is_pill: false,
            is_transparent: false,
        }
    }

    /// Creates a floating toolbar container absolutely positioned at `(x, y)` in logical points.
    #[inline]
    pub fn floating(name: &'static str, x: f32, y: f32) -> Self {
        Self {
            name,
            position: Some((x, y)),
            width: None,
            height: 32.0,
            gap: 8.0,
            tag: 0,
            background: None,
            border_color: None,
            corner_radius: None,
            is_pill: false,
            is_transparent: false,
        }
    }

    /// Configures an explicit width in logical points for the toolbar container.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Configures the toolbar container height in logical points.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Configures the gap spacing between nested toolbar groups and standalone tool buttons.
    #[inline]
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Applies rounded pill glass styling with subtle translucent background, border, and elevation shadow.
    #[inline]
    #[must_use]
    pub fn pill(mut self) -> Self {
        self.is_pill = true;
        self
    }

    /// Configures the toolbar to have a transparent background and no outer border (ideal for floating strips).
    #[inline]
    #[must_use]
    pub fn transparent(mut self) -> Self {
        self.is_transparent = true;
        self
    }

    /// Sets an explicit background fill color for the toolbar container.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Sets an explicit border outline color for the toolbar container.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border: Color) -> Self {
        self.border_color = Some(border);
        self
    }

    /// Sets explicit corner radii for the toolbar container.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Renders the toolbar container and provides a specialized [`ToolbarScope`] to child closures.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        f: impl FnOnce(&mut ToolbarScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        let theme = scope.theme();
        let mut style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(self.height)
            .flex_shrink(0.0)
            .gap(self.gap);

        if let Some((x, y)) = self.position {
            style = style.position_absolute().left(x).top(y);
        }

        if let Some(w) = self.width {
            style = style.width(w);
        }

        if self.is_transparent {
            // Transparent floating strip: no background or border
        } else if self.is_pill {
            let bg = self
                .background
                .unwrap_or_else(|| Color::rgba(0.12, 0.13, 0.16, 0.92));
            let border = self
                .border_color
                .unwrap_or_else(|| Color::rgba(0.24, 0.26, 0.32, 0.85));
            let radius = self.corner_radius.unwrap_or(4.0);
            style = style
                .background(bg)
                .border(1.0, border)
                .corner_radii(CornerRadii::all(radius))
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
        } else if let Some(bg) = self.background {
            style = style.background(bg);
            if let Some(border) = self.border_color {
                style = style.border(1.0, border);
            }
            if let Some(radius) = self.corner_radius {
                style = style.corner_radii(CornerRadii::all(radius));
            }
        } else {
            style = style
                .background(theme.toolbar_bg)
                .border(1.0, theme.toolbar_border)
                .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0));
        }

        let mut result = None;
        let response = scope.container_tagged(
            self.name,
            style,
            WidgetRole::Default,
            self.tag,
            |child_scope| {
                let mut tb_scope = ToolbarScope::new(child_scope, self.height);
                result = Some(f(&mut tb_scope));
            },
        );

        InnerResponse::new(
            response.response,
            result.expect("Toolbar content closure must execute synchronously"),
        )
    }
}