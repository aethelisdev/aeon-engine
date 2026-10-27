// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Color Swatch Builder
//!
//! Provides the fluent [`ColorSwatch`] widget implementing [`Widget`].
//! Renders interactive color preview swatches with autonomous hover highlights,
//! active border states, and immediate [`WidgetResponse`] interaction tracking.

use super::Widget;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{Color, Style, WidgetCursor, WidgetRole};

/// Configurable fluent builder for interactive color swatch preview boxes.
///
/// Encapsulates sizing, border highlighting, active states, and persistent hit-testing
/// tags without requiring imperative local variables (`let`) in calling application code.
#[derive(Debug, Clone, Copy)]
pub struct ColorSwatch {
    color: Color,
    tag: Option<u64>,
    width: f32,
    height: f32,
    corner_radius: f32,
    border_width: f32,
    border_color: Option<Color>,
    active_border_color: Option<Color>,
    is_active: bool,
}

impl ColorSwatch {
    /// Creates a new interactive color swatch builder for the specified color.
    ///
    /// # Arguments
    /// * `color` - Color to preview in the swatch box.
    #[inline]
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            tag: None,
            width: 38.0,
            height: 18.0,
            corner_radius: 5.0,
            border_width: 1.0,
            border_color: None,
            active_border_color: None,
            is_active: false,
        }
    }

    /// Sets an explicit persistent 64-bit interaction tag for this swatch.
    ///
    /// # Arguments
    /// * `tag` - Deterministic hit-testing identifier.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets both width and height dimensions in physical pixels.
    ///
    /// # Arguments
    /// * `width` - Width constraint in pixels.
    /// * `height` - Height constraint in pixels.
    #[inline]
    #[must_use]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the width dimension in physical pixels.
    ///
    /// # Arguments
    /// * `width` - Width constraint in pixels.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the height dimension in physical pixels.
    ///
    /// # Arguments
    /// * `height` - Height constraint in pixels.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets the corner rounding radius in physical pixels.
    ///
    /// # Arguments
    /// * `radius` - Corner rounding radius.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Sets the border line width in physical pixels.
    ///
    /// # Arguments
    /// * `width` - Border stroke thickness.
    #[inline]
    #[must_use]
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Configures the active (e.g. color picker popup open) state of the swatch.
    ///
    /// When active or hovered, displays a prominent highlight border.
    ///
    /// # Arguments
    /// * `is_active` - True if the swatch is currently active/selected.
    #[inline]
    #[must_use]
    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Overrides the default idle border color.
    ///
    /// # Arguments
    /// * `color` - Color used when the swatch is idle.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Overrides the active or hovered highlight border color.
    ///
    /// # Arguments
    /// * `color` - Color used when hovered or active.
    #[inline]
    #[must_use]
    pub fn active_border_color(mut self, color: Color) -> Self {
        self.active_border_color = Some(color);
        self
    }
}

impl Widget for ColorSwatch {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> WidgetResponse {
        let tag = self.tag.unwrap_or_else(|| {
            let mut h: u64 = 0xcbf29ce484222325;
            for &comp in &[self.color.r, self.color.g, self.color.b, self.color.a] {
                h ^= comp.to_bits() as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            h
        });

        let node_id = scope.tree.create_node();
        if let Some(node) = scope.tree.get_mut(node_id) {
            node.set_name("ColorSwatchBox");
            node.role = WidgetRole::Button;
            node.tag = tag;
            node.cursor = Some(WidgetCursor::Pointer);
        }
        let _ = scope.tree.add_child(scope.parent, node_id);
        let (clicked, hovered, _) = scope.check_interaction(node_id);

        let idle_border = self
            .border_color
            .unwrap_or_else(|| Color::rgba(0.85, 0.88, 0.95, 0.70));
        let active_border = self
            .active_border_color
            .unwrap_or_else(|| Color::rgba(1.0, 1.0, 1.0, 0.95));

        let current_border = if hovered || self.is_active {
            active_border
        } else {
            idle_border
        };

        let style = Style::new()
            .width(self.width)
            .height(self.height)
            .background(self.color)
            .border(self.border_width, current_border)
            .border_radius(self.corner_radius);

        if let Some(node) = scope.tree.get_mut(node_id) {
            node.style = style;
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, false);
        if let Some(node) = scope.tree.get(node_id) {
            resp.rect = node.computed_rect;
        }
        resp
    }
}