// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Texture Atlas Icons & Hover Reactivity Primitives
//!
//! Provides texture atlas icon rendering and dynamic hover tint bindings on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use iris_core::{Color, Style, WidgetId};

impl<'a> UiScope<'a> {
    /// Emits an icon quad textured from an atlas UV rectangle.
    ///
    /// # Arguments
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon(&mut self, icon_uv: [f32; 4], tint: Color, size: f32) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
        icon_id
    }

    /// Emits a named icon quad textured from an atlas UV rectangle with a debug identifier.
    ///
    /// # Arguments
    /// * `name` - Static debug identifier assigned to the UI node.
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon_named(
        &mut self,
        name: &'static str,
        icon_uv: [f32; 4],
        tint: Color,
        size: f32,
    ) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.set_name(name);
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
        icon_id
    }

    /// Emits an icon quad from an atlas UV and tint pair without intermediate let bindings.
    #[inline]
    pub fn icon_pair(&mut self, (icon_uv, tint): ([f32; 4], Color), size: f32) -> WidgetId {
        self.icon(icon_uv, tint, size)
    }

    /// Emits a named icon quad from an atlas UV and tint pair with a debug identifier.
    #[inline]
    pub fn icon_pair_named(
        &mut self,
        name: &'static str,
        (icon_uv, tint): ([f32; 4], Color),
        size: f32,
    ) -> WidgetId {
        self.icon_named(name, icon_uv, tint, size)
    }

    /// Emits an icon quad with reactive parent hover texture tinting.
    ///
    /// Combines icon quad creation and hover tint reactivity in a single fluent call,
    /// eliminating intermediate [`WidgetId`] tracking bindings.
    ///
    /// # Arguments
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Default color tint applied across the icon quad.
    /// * `hover_tint` - Dynamic color tint applied while the parent container is hovered.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon_hover(
        &mut self,
        icon_uv: [f32; 4],
        tint: Color,
        hover_tint: Color,
        size: f32,
    ) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_hover_texture_tint(hover_tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
        icon_id
    }

    /// Emits a named icon quad with reactive parent hover texture tinting and a debug identifier.
    ///
    /// Combines icon quad creation and hover tint reactivity in a single fluent call,
    /// eliminating intermediate [`WidgetId`] tracking bindings.
    ///
    /// # Arguments
    /// * `name` - Static debug identifier assigned to the UI node.
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Default color tint applied across the icon quad.
    /// * `hover_tint` - Dynamic color tint applied while the parent container is hovered.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon_hover_named(
        &mut self,
        name: &'static str,
        icon_uv: [f32; 4],
        tint: Color,
        hover_tint: Color,
        size: f32,
    ) -> WidgetId {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.set_name(name);
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_hover_texture_tint(hover_tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
        icon_id
    }

    /// Sets the declarative hover text color on an existing widget node.
    ///
    /// When the target node or an enclosing interactive container is hovered,
    /// the text collector dynamically overrides the foreground color in-place
    /// without triggering tree invalidation or allocations.
    ///
    /// # Arguments
    /// * `node_id` - Target widget node identifier.
    /// * `color` - Color applied to text while hovered.
    pub fn set_hover_text_color(&mut self, node_id: WidgetId, color: Color) {
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_hover_text_color(color);
        }
    }

    /// Sets the declarative hover texture tint on an existing widget node.
    ///
    /// When the target node or an enclosing interactive container is hovered,
    /// the tree compiler dynamically overrides the quad texture tint in-place
    /// without triggering tree invalidation or allocations.
    ///
    /// # Arguments
    /// * `node_id` - Target widget node identifier.
    /// * `tint` - Color tint applied across texture quads while hovered.
    pub fn set_hover_texture_tint(&mut self, node_id: WidgetId, tint: Color) {
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_hover_texture_tint(tint);
        }
    }
}