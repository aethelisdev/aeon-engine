// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Styling definition and Fluent builder API for UI nodes.

use crate::color::Color;
use crate::geometry::{Border, BoxShadow, CornerRadii, Insets, Size};

/// Flexbox layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    /// Items are placed horizontally from left to right.
    #[default]
    Row,
    /// Items are placed vertically from top to bottom.
    Column,
    /// Items are placed horizontally in reverse order.
    RowReverse,
    /// Items are placed vertically in reverse order.
    ColumnReverse,
}

/// Alignment along the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    /// Items are aligned to the start of the cross axis.
    FlexStart,
    /// Items are aligned to the end of the cross axis.
    FlexEnd,
    /// Items are centered along the cross axis.
    Center,
    /// Items are stretched to fill the cross axis.
    #[default]
    Stretch,
}

/// Alignment along the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    /// Items are packed toward the start of the main axis.
    #[default]
    FlexStart,
    /// Items are packed toward the end of the main axis.
    FlexEnd,
    /// Items are centered along the main axis.
    Center,
    /// Items are evenly distributed with equal space between them.
    SpaceBetween,
    /// Items are evenly distributed with equal space around them.
    SpaceAround,
    /// Items are evenly distributed with equal space between and on edges.
    SpaceEvenly,
}

/// Text horizontal alignment within its container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// Text is aligned to the left edge.
    #[default]
    Left,
    /// Text is centered horizontally within the available width.
    Center,
    /// Text is aligned to the right edge.
    Right,
}

/// Complete styling specification for a widget node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background color of the widget.
    pub background_color: Color,
    /// Rounded corner radii for SDF rendering.
    pub corner_radii: CornerRadii,
    /// Border stroke thickness and color.
    pub border: Border,
    /// Drop shadow specification.
    pub box_shadow: Option<BoxShadow>,
    /// Inner padding insets.
    pub padding: Insets,
    /// Outer margin insets.
    pub margin: Insets,
    /// Explicit width constraint in pixels (if specified).
    pub width: Option<f32>,
    /// Explicit height constraint in pixels (if specified).
    pub height: Option<f32>,
    /// Minimum width constraint in pixels.
    pub min_width: Option<f32>,
    /// Minimum height constraint in pixels.
    pub min_height: Option<f32>,
    /// Maximum width constraint in pixels.
    pub max_width: Option<f32>,
    /// Maximum height constraint in pixels.
    pub max_height: Option<f32>,
    /// Direction of flex items.
    pub flex_direction: FlexDirection,
    /// Space between flex children in pixels.
    pub gap: f32,
    /// Alignment of flex items along the cross axis.
    pub align_items: AlignItems,
    /// Justification of flex items along the main axis.
    pub justify_content: JustifyContent,
    /// Flex grow factor.
    pub flex_grow: f32,
    /// Flex shrink factor.
    pub flex_shrink: f32,
    /// Overall opacity multiplier in range `[0.0, 1.0]`.
    pub opacity: f32,
    /// Whether children exceeding this widget's bounds should be clipped.
    pub clip_children: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: Color::TRANSPARENT,
            corner_radii: CornerRadii::ZERO,
            border: Border::NONE,
            box_shadow: None,
            padding: Insets::ZERO,
            margin: Insets::ZERO,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            flex_direction: FlexDirection::Row,
            gap: 0.0,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::FlexStart,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            opacity: 1.0,
            clip_children: false,
        }
    }
}

impl Style {
    /// Creates a new default style.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the background color.
    #[inline]
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Sets uniform corner radii for all four corners.
    #[inline]
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.corner_radii = CornerRadii::all(radius);
        self
    }

    /// Sets individual corner radii.
    #[inline]
    pub fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.corner_radii = radii;
        self
    }

    /// Sets uniform border width and color.
    #[inline]
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Border::uniform(width, color);
        self
    }

    /// Sets drop shadow parameters.
    #[inline]
    pub fn box_shadow(mut self, offset_x: f32, offset_y: f32, blur: f32, color: Color) -> Self {
        self.box_shadow = Some(BoxShadow::new(offset_x, offset_y, blur, 0.0, color));
        self
    }

    /// Sets uniform inner padding.
    #[inline]
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Insets::all(padding);
        self
    }

    /// Sets directional inner padding insets.
    #[inline]
    pub fn padding_insets(mut self, insets: Insets) -> Self {
        self.padding = insets;
        self
    }

    /// Sets uniform outer margin.
    #[inline]
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = Insets::all(margin);
        self
    }

    /// Sets directional outer margin insets.
    #[inline]
    pub fn margin_insets(mut self, insets: Insets) -> Self {
        self.margin = insets;
        self
    }

    /// Sets explicit fixed width in pixels.
    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets explicit fixed height in pixels.
    #[inline]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets explicit fixed size dimensions.
    #[inline]
    pub fn size(mut self, size: Size) -> Self {
        self.width = Some(size.width);
        self.height = Some(size.height);
        self
    }

    /// Sets layout direction to horizontal flex row.
    #[inline]
    pub fn flex_row(mut self) -> Self {
        self.flex_direction = FlexDirection::Row;
        self
    }

    /// Sets layout direction to vertical flex column.
    #[inline]
    pub fn flex_col(mut self) -> Self {
        self.flex_direction = FlexDirection::Column;
        self
    }

    /// Sets space between children in pixels.
    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets flex cross-axis alignment.
    #[inline]
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    /// Sets flex main-axis justification.
    #[inline]
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }

    /// Sets the flex grow factor.
    #[inline]
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = grow;
        self
    }

    /// Sets the flex shrink factor.
    #[inline]
    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = shrink;
        self
    }

    /// Sets whether children exceeding bounds are clipped.
    #[inline]
    pub fn clip_children(mut self, clip: bool) -> Self {
        self.clip_children = clip;
        self
    }

    /// Sets overall opacity multiplier in `[0.0, 1.0]`.
    #[inline]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}