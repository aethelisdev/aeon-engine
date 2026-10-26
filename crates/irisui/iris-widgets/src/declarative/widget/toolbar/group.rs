// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ToolbarGroup Container Builder
//!
//! Provides the configurable [`ToolbarGroup`] builder for modular toolbar sub-groups.
//!

use super::group_scope::ToolbarGroupScope;
use crate::declarative::scope::containers::generic::InnerResponse;
use crate::declarative::scope::core::UiScope;
use iris_core::{AlignItems, Color, CornerRadii, Style};

/// Configurable builder for modular toolbar sub-groups.
#[derive(Debug, Clone)]
pub struct ToolbarGroup {
    width: Option<f32>,
    gap: f32,
    background: Option<Color>,
    border_color: Option<Color>,
    corner_radius: f32,
    is_transparent: bool,
}

impl Default for ToolbarGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolbarGroup {
    /// Creates a new modular toolbar sub-group with default dark pill styling.
    #[inline]
    pub fn new() -> Self {
        Self {
            width: None,
            gap: 4.0,
            background: None,
            border_color: None,
            corner_radius: 4.0,
            is_transparent: false,
        }
    }

    /// Configures an explicit width in logical points for this toolbar group.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Configures the group to have a transparent background and no border.
    #[inline]
    #[must_use]
    pub fn transparent(mut self) -> Self {
        self.is_transparent = true;
        self
    }

    /// Configures the gap spacing between items in this group.
    #[inline]
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Configures the background color of this group.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Configures the border color of this group.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border: Color) -> Self {
        self.border_color = Some(border);
        self
    }

    /// Configures the corner radius of this group container.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Renders this modular sub-group inside the parent scope.
    pub fn show<R>(
        self,
        scope: &mut UiScope<'_>,
        parent_height: f32,
        f: impl FnOnce(&mut ToolbarGroupScope<'_, '_>) -> R,
    ) -> InnerResponse<R> {
        let mut style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(parent_height)
            .flex_shrink(0.0)
            .gap(self.gap);

        if let Some(w) = self.width {
            style = style.width(w);
        }

        if self.is_transparent {
            // Transparent group: no background, border, or shadow
        } else {
            let bg = self
                .background
                .unwrap_or_else(|| Color::rgba(0.12, 0.13, 0.16, 0.92));
            let border = self
                .border_color
                .unwrap_or_else(|| Color::rgba(0.24, 0.26, 0.32, 0.85));
            style = style
                .background(bg)
                .border(1.0, border)
                .corner_radii(CornerRadii::all(self.corner_radius))
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
        }

        let mut result = None;
        let response = scope.container_named("ToolbarGroup", style, |group_scope| {
            let mut g_scope = ToolbarGroupScope::new(group_scope, parent_height);
            result = Some(f(&mut g_scope));
        });

        InnerResponse::new(
            response.response,
            result.expect("ToolbarGroup content closure must execute synchronously"),
        )
    }
}