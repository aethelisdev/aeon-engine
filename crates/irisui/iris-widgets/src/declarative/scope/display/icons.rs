// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Texture Atlas Icons & Hover Reactivity Primitives
//!
//! Provides texture atlas icon rendering and dynamic hover tint bindings on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use iris_core::{Color, Style};

impl<'a> UiScope<'a> {
    /// Emits an icon quad textured from an atlas UV rectangle.
    ///
    /// # Arguments
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon(&mut self, icon_uv: [f32; 4], tint: Color, size: f32) {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
    }

    /// Emits a named icon quad textured from an atlas UV rectangle with a debug identifier.
    ///
    /// # Arguments
    /// * `name` - Static debug identifier assigned to the UI node.
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]`.
    /// * `tint` - Color tint applied across the icon quad.
    /// * `size` - Width and height of the icon quad in physical pixels.
    pub fn icon_named(&mut self, name: &'static str, icon_uv: [f32; 4], tint: Color, size: f32) {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.set_name(name);
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
    }

    /// Emits an icon quad from an atlas UV and tint pair without intermediate let bindings.
    #[inline]
    pub fn icon_pair(&mut self, (icon_uv, tint): ([f32; 4], Color), size: f32) {
        self.icon(icon_uv, tint, size);
    }

    /// Emits a named icon quad from an atlas UV and tint pair with a debug identifier.
    #[inline]
    pub fn icon_pair_named(
        &mut self,
        name: &'static str,
        (icon_uv, tint): ([f32; 4], Color),
        size: f32,
    ) {
        self.icon_named(name, icon_uv, tint, size);
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
    pub fn icon_hover(&mut self, icon_uv: [f32; 4], tint: Color, hover_tint: Color, size: f32) {
        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(tint);
            node.set_hover_texture_tint(hover_tint);
            node.set_style(Style::new().width(size).height(size));
        }
        let _ = self.tree.add_child(self.parent, icon_id);
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
    ) {
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
    }
}