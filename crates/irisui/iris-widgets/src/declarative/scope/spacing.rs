// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Layout Spacing, Dividers & Separator Primitives
//!
//! Provides idiomatic layout spacing and divider primitives on [`UiScope`]:
//! - Fixed layout gaps via [`UiScope::spacing`].
//! - Visual horizontal and vertical hair-line dividers via [`UiScope::separator`] and [`UiScope::divider`].
//! - Dynamic flex-grow spacers (springs) via [`UiScope::spacer`].
//! - Non-interactive decorative leaf boxes via [`UiScope::empty_box_passive`].
//!

use super::core::UiScope;
use iris_core::{Color, FlexDirection, Style, WidgetRole};

impl<'a> UiScope<'a> {
    /// Emits a passive layout spacing element of the specified size in logical pixels.
    ///
    /// Respects the parent container's flex direction: if the parent is a row, applies
    /// `width(px)`; if a column, applies `height(px)`.
    ///
    /// # Arguments
    /// * `px` - Spacing gap extent in logical pixels.
    pub fn spacing(&mut self, px: f32) {
        self.spacing_named("Spacing", px);
    }

    /// Emits a passive layout spacing element with a custom node name for debugging and tree inspection.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier for the UI node.
    /// * `px` - Spacing gap extent in logical pixels.
    pub fn spacing_named(&mut self, name: impl Into<String>, px: f32) {
        let is_row = self
            .tree
            .get(self.parent)
            .map(|n| n.style.flex_direction == FlexDirection::Row)
            .unwrap_or(false);

        let style = if is_row {
            Style::new().width(px).height(1.0)
        } else {
            Style::new().width(1.0).height(px)
        };
        self.empty_box_passive_named(name, style);
    }

    /// Emits a horizontal divider separator line.
    pub fn separator(&mut self) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("Separator");
            node.role = WidgetRole::Separator;
            node.set_style(
                Style::new()
                    .height(1.0)
                    .background(Color::rgba(0.24, 0.25, 0.30, 0.6)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a styled leaf element (empty container node without child scopes, e.g. solid box or backdrop).
    ///
    /// # Arguments
    /// * `style` - Visual bounds and styling properties applied to the leaf node.
    pub fn empty_box(&mut self, style: Style) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a styled leaf element with `interactive = false` so it does not intercept hit-testing.
    ///
    /// Used for background grids, visual outlines, guide lines, and purely decorative graphics.
    ///
    /// # Arguments
    /// * `style` - Visual bounds and styling properties applied to the passive leaf node.
    pub fn empty_box_passive(&mut self, style: Style) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = false;
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a named, non-interactive leaf box node with custom styling.
    ///
    /// Ideal for decorative lines, pins, markers, and visual outlines that should not block
    /// mouse events or register in hit-testing.
    ///
    /// # Arguments
    /// * `name` - Descriptive name of the widget node for tree debugging.
    /// * `style` - Visual bounds and styling properties applied to the leaf node.
    pub fn empty_box_passive_named(&mut self, name: impl Into<String>, style: Style) {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.interactive = false;
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
    }

    /// Emits a 1-pixel horizontal rule divider container with a custom color.
    ///
    /// # Arguments
    /// * `color` - RGBA color of the divider line.
    pub fn divider(&mut self, color: Color) {
        let div_style = Style::new().background(color).height(1.0);
        self.empty_box(div_style);
    }

    /// Emits a 1-pixel vertical divider container with custom height and color.
    ///
    /// # Arguments
    /// * `height` - Explicit vertical height of the divider line.
    /// * `color` - RGBA color of the divider line.
    pub fn vertical_divider(&mut self, height: f32, color: Color) {
        let div_style = Style::new().background(color).width(1.0).height(height);
        self.empty_box(div_style);
    }

    /// Emits an empty flex-grow spacer node to expand and push subsequent items.
    pub fn spacer(&mut self) {
        self.empty_box(Style::new().flex_grow(1.0));
    }

    /// Emits a fixed-size empty spacing element of the specified extent in physical pixels.
    ///
    /// Delegates directly to [`UiScope::spacing_named`] using the `"SpacerPx"` node label,
    /// dynamically conforming to the parent container's layout axis (width for rows, height for columns).
    ///
    /// # Arguments
    /// * `size` - Fixed spacing extent in physical pixels.
    pub fn spacer_px(&mut self, size: f32) {
        self.spacing_named("SpacerPx", size);
    }
}