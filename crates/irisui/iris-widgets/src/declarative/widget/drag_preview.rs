// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Generic Floating Drag Preview Widget (`FloatingDragPreview`)
//!
//! Provides the configurable [`FloatingDragPreview`] builder and [`FloatingDragPreviewStyle`]
//! for rendering non-interactive floating feedback cards beneath the cursor during active
//! drag-and-drop gestures (assets, scene nodes, dock tabs, and palette items).
//!
//! Evaluates directly into the elevated [`UiLayer::Tooltip`] stacking context with zero-clipping
//! and non-interactive pass-through semantics, ensuring underlying drop targets receive full
//! hover and drop events.

use super::Widget;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, BoxShadow, Color, CornerRadii, Insets, Point, Style, TextAlign, UiLayer, WidgetRole,
};

/// Default horizontal and vertical offset displacement from the pointer cursor in screen pixels.
pub const DEFAULT_DRAG_PREVIEW_OFFSET: Point = Point::new(14.0, 14.0);

/// Visual styling attributes governing the appearance and geometry of a [`FloatingDragPreview`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingDragPreviewStyle {
    /// Background fill color of the floating preview capsule card.
    pub background: Color,
    /// Perimeter border stroke outline color.
    pub border_color: Color,
    /// Perimeter border outline width in screen pixels.
    pub border_width: f32,
    /// Corner radii curvature for rounded rectangle SDF shading.
    pub corner_radius: CornerRadii,
    /// Analytic drop shadow cast beneath the capsule card.
    pub shadow: BoxShadow,
    /// Internal padding insets surrounding content elements in pixels.
    pub padding: Insets,
    /// Horizontal spacing between the badge, title label, and secondary text in pixels.
    pub gap: f32,
    /// Font size for the primary title label in physical pixels.
    pub font_size: f32,
    /// Font size for the optional sub-label text in physical pixels.
    pub sub_font_size: f32,
    /// Primary foreground text color.
    pub text_color: Color,
    /// Secondary foreground text color for descriptions and counters.
    pub sub_text_color: Color,
    /// Relative offset displacement from the cursor pointer coordinate in pixels.
    pub offset: Point,
}

impl Default for FloatingDragPreviewStyle {
    #[inline]
    fn default() -> Self {
        Self {
            background: Color::from_u8(24, 26, 32, 230),
            border_color: Color::from_u8(65, 70, 85, 200),
            border_width: 1.0,
            corner_radius: CornerRadii::all(6.0),
            shadow: BoxShadow::new(0.0, 4.0, 12.0, 0.0, Color::from_u8(0, 0, 0, 140)),
            padding: Insets::new(5.0, 10.0, 5.0, 10.0),
            gap: 8.0,
            font_size: 13.0,
            sub_font_size: 11.0,
            text_color: Color::WHITE,
            sub_text_color: Color::from_u8(160, 165, 185, 255),
            offset: DEFAULT_DRAG_PREVIEW_OFFSET,
        }
    }
}

/// Declarative floating preview card displayed beneath the mouse cursor during active drag gestures.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingDragPreview {
    /// Main title label identifying the item being dragged (e.g. asset name, entity name).
    label: String,
    /// Cursor pointer coordinate in screen pixels.
    cursor: Point,
    /// Relative offset displacement from the cursor pointer coordinate in pixels.
    offset: Option<Point>,
    /// Optional category or type badge pill displayed to the left of the label: `(badge_text, color)`.
    badge: Option<(String, Color)>,
    /// Optional secondary description or sub-label text.
    sub_label: Option<String>,
    /// Configurable visual styling attributes.
    style: FloatingDragPreviewStyle,
}

impl FloatingDragPreview {
    /// Constructs a new floating drag preview card initialized with the given title label.
    #[must_use]
    #[inline]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            cursor: Point::ZERO,
            offset: None,
            badge: None,
            sub_label: None,
            style: FloatingDragPreviewStyle::default(),
        }
    }

    /// Sets the current screen-space mouse cursor coordinate.
    #[must_use]
    #[inline]
    pub const fn with_cursor(mut self, cursor: Point) -> Self {
        self.cursor = cursor;
        self
    }

    /// Overrides the relative offset displacement from the cursor pointer position.
    #[must_use]
    #[inline]
    pub const fn with_offset(mut self, offset: Point) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Attaches an optional category or type badge pill displayed with a custom tint color.
    #[must_use]
    #[inline]
    pub fn with_badge(mut self, badge: impl Into<String>, color: Color) -> Self {
        self.badge = Some((badge.into(), color));
        self
    }

    /// Attaches an optional secondary description or sub-label text.
    #[must_use]
    #[inline]
    pub fn with_sub_label(mut self, sub_label: impl Into<String>) -> Self {
        self.sub_label = Some(sub_label.into());
        self
    }

    /// Overrides the visual styling parameters governing this preview card.
    #[must_use]
    #[inline]
    pub const fn with_style(mut self, style: FloatingDragPreviewStyle) -> Self {
        self.style = style;
        self
    }
}

impl Widget for FloatingDragPreview {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let effective_offset = self.offset.unwrap_or(self.style.offset);
        let pos = Point::new(
            self.cursor.x + effective_offset.x,
            self.cursor.y + effective_offset.y,
        );

        let root_parent = scope.tree().root().unwrap_or(scope.parent());
        let card_id = scope.tree_mut().create_node();

        let s = self.style;
        if let Some(card) = scope.tree_mut().get_mut(card_id) {
            card.set_name("FloatingDragPreview");
            card.set_role(WidgetRole::Tooltip);
            card.set_layer(UiLayer::Tooltip);
            card.interactive = false;
            card.set_style(
                Style::new()
                    .position_absolute()
                    .left(pos.x)
                    .top(pos.y)
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .padding_insets(s.padding)
                    .gap(s.gap)
                    .background(s.background)
                    .border(s.border_width, s.border_color)
                    .corner_radii(s.corner_radius)
                    .box_shadow(
                        s.shadow.offset.x,
                        s.shadow.offset.y,
                        s.shadow.blur,
                        s.shadow.color,
                    ),
            );
        }

        let _ = scope.tree_mut().add_child(root_parent, card_id);

        let mut child_scope = UiScope::new(scope.tree_mut(), card_id);

        // 1. Optional Category/Type Badge Pill
        if let Some((badge_text, badge_color)) = self.badge {
            let badge_bg = Color::rgba(
                badge_color.r * 0.25,
                badge_color.g * 0.25,
                badge_color.b * 0.25,
                0.9,
            );
            let badge_border = Color::rgba(
                badge_color.r * 0.7,
                badge_color.g * 0.7,
                badge_color.b * 0.7,
                0.9,
            );
            let badge_style = Style::new()
                .padding_insets(Insets::new(1.0, 5.0, 1.0, 5.0))
                .background(badge_bg)
                .border(1.0, badge_border)
                .corner_radii(CornerRadii::all(4.0));

            child_scope.container_passive_named("DragBadgePill", badge_style, |pill| {
                pill.label_custom(&badge_text, s.sub_font_size, badge_color, TextAlign::Left);
            });
        }

        // 2. Primary Title Label
        child_scope.label_custom(&self.label, s.font_size, s.text_color, TextAlign::Left);

        // 3. Optional Sub-Label Text
        if let Some(sub) = self.sub_label {
            child_scope.label_custom(&sub, s.sub_font_size, s.sub_text_color, TextAlign::Left);
        }

        WidgetResponse::default()
    }
}