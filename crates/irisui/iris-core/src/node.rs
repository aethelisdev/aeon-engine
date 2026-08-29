// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Individual widget node representations stored within the generational arena.

use crate::color::Color;
use crate::dirty::DirtyFlags;
use crate::geometry::{Point, Rect, Size};
use crate::id::WidgetId;
use crate::style::{Style, TextAlign};

/// A single node in the Retained-Mode UI tree stored in the central arena.
/// Each node holds hierarchical relationships (parent and children references via `WidgetId`),
/// current style parameters, fine-grained dirty flags, cached layout coordinates, and optional text payload.
#[derive(Debug, Clone)]
pub struct WidgetNode {
    /// The unique generational key of this node.
    pub id: WidgetId,
    /// Optional parent node key. `None` if this is the root node or a detached branch.
    pub parent: Option<WidgetId>,
    /// Ordered list of child node keys.
    pub children: Vec<WidgetId>,
    /// Visual and layout styling attributes.
    pub style: Style,
    /// Dirty tracking flags for selective recomputation.
    pub dirty: DirtyFlags,
    /// Cached absolute layout bounding rectangle in screen-space pixels.
    pub computed_rect: Rect,
    /// Intrinsic content size (e.g. measured text dimensions or image resolution).
    pub content_size: Size,
    /// Optional text payload displayed by this node.
    pub text: Option<String>,
    /// Font size in pixels.
    pub font_size: f32,
    /// Line height in pixels.
    pub line_height: f32,
    /// Text RGBA foreground color.
    pub text_color: Color,
    /// Text alignment.
    pub text_align: TextAlign,
    /// Whether this node and its subtree are visible.
    pub visible: bool,
    /// Whether this node can receive mouse and keyboard interaction events.
    pub interactive: bool,
    /// Optional debug name for inspection and profiling.
    pub name: Option<String>,
}

impl WidgetNode {
    /// Default standard font size in pixels.
    pub const DEFAULT_FONT_SIZE: f32 = 14.0;
    /// Default standard line height in pixels.
    pub const DEFAULT_LINE_HEIGHT: f32 = 18.0;

    /// Creates a new `WidgetNode` with default properties and full dirty flags.
    #[inline]
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            style: Style::default(),
            dirty: DirtyFlags::ALL,
            computed_rect: Rect::ZERO,
            content_size: Size::ZERO,
            text: None,
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::DEFAULT_LINE_HEIGHT,
            text_color: Color::WHITE,
            text_align: TextAlign::Left,
            visible: true,
            interactive: true,
            name: None,
        }
    }

    /// Sets the debug name of the node (builder style).
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the debug name of the node on an existing mutable reference.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sets the text content and marks `TEXT`, `LAYOUT`, and `PAINT` dirty flags.
    #[inline]
    pub fn set_text(&mut self, text: impl Into<String>) {
        let new_text = text.into();
        if self.text.as_deref() != Some(&new_text) {
            self.text = Some(new_text);
            self.dirty |= DirtyFlags::TEXT | DirtyFlags::LAYOUT | DirtyFlags::PAINT;
        }
    }

    /// Sets the text styling properties (font size, line height, color, alignment).
    #[inline]
    pub fn set_text_properties(
        &mut self,
        font_size: f32,
        line_height: f32,
        color: Color,
        align: TextAlign,
    ) {
        self.font_size = font_size;
        self.line_height = line_height;
        self.text_color = color;
        self.text_align = align;
        self.dirty |= DirtyFlags::TEXT | DirtyFlags::LAYOUT | DirtyFlags::PAINT;
    }

    /// Sets the style of the node and marks styling flags as dirty.
    #[inline]
    pub fn set_style(&mut self, style: Style) {
        if self.style != style {
            self.style = style;
            self.dirty |= DirtyFlags::STYLE | DirtyFlags::LAYOUT | DirtyFlags::PAINT;
        }
    }

    /// Marks specified dirty flags on this node.
    #[inline]
    pub fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.dirty |= flags;
    }

    /// Clears specified dirty flags after processing.
    #[inline]
    pub fn clear_dirty(&mut self, flags: DirtyFlags) {
        self.dirty.remove(flags);
    }

    /// Tests if a given screen-space point falls within this node's cached bounding box.
    #[inline]
    pub fn hit_test(&self, point: Point) -> bool {
        self.visible && self.interactive && self.computed_rect.contains_point(point)
    }
}