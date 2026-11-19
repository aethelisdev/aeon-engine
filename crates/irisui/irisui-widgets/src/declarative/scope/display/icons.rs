// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Texture Atlas Icons & Hover Reactivity Primitives
//!
//! Provides texture atlas icon rendering and dynamic hover tint bindings on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use irisui_core::{AlignItems, Color, Insets, Style};

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

    /// Emits a unified [`irisui_core::UiIcon`] (either GPU texture atlas quad or centered unicode glyph).
    pub fn ui_icon(&mut self, icon: irisui_core::UiIcon, tint: Color, size: f32) {
        match icon {
            irisui_core::UiIcon::Atlas(uv) => self.icon(uv, tint, size),
            irisui_core::UiIcon::Glyph(glyph) => {
                let icon_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(icon_id) {
                    node.set_name("UiIconGlyph");
                    node.interactive = false;
                    node.set_text(glyph);
                    node.text_color = tint;
                    node.font_size = (size * 0.8).max(10.0);
                    node.text_align = irisui_core::TextAlign::Center;
                    node.set_style(Style::new().width(size).height(size));
                }
                let _ = self.tree.add_child(self.parent, icon_id);
            }
        }
    }

    /// Emits a flex-growing, clipped capsule badge displaying a leading atlas icon and colored label.
    ///
    /// Designed for asset slot readouts, texture chips, and compact inline file indicators inside
    /// horizontal inspector rows alongside action buttons.
    ///
    /// # Arguments
    /// * `icon_uv` - Normalized atlas UV bounds `[u_min, v_min, u_max, v_max]` for the leading 12px icon.
    /// * `icon_tint` - Color tint applied to the leading icon quad.
    /// * `text` - Display text rendered immediately after the icon.
    /// * `text_color` - Foreground text color for the badge label.
    pub fn icon_label_badge(
        &mut self,
        icon_uv: [f32; 4],
        icon_tint: Color,
        text: impl Into<String>,
        text_color: Color,
    ) {
        let theme = self.tree.theme();
        let badge_style = Style::new()
            .flex_row()
            .flex_grow(1.0)
            .clip_children(true)
            .align_items(AlignItems::Center)
            .gap(5.0)
            .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
            .background(theme.panel_bg)
            .border(1.0, theme.card_border)
            .border_radius(3.0)
            .height(24.0);

        self.container_named("IconLabelBadge", badge_style, |badge| {
            badge.icon(icon_uv, icon_tint, 12.0);
            badge.colored_label(text_color, text);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_icon_label_badge_emits_clipped_flex_capsule() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.icon_label_badge(
                [0.0, 0.0, 1.0, 1.0],
                Color::rgba(0.0, 0.85, 1.0, 0.85),
                "albedo.png",
                Color::rgba(0.35, 0.75, 0.98, 1.0),
            );
        }

        let root_node = tree.get(root).expect("Root exists");
        assert_eq!(root_node.children.len(), 1);
        let badge = tree.get(root_node.children[0]).expect("Badge node exists");
        assert_eq!(badge.name.as_deref(), Some("IconLabelBadge"));
        assert_eq!(badge.style.flex_grow, 1.0);
        assert!(badge.style.clip_children);
        assert_eq!(badge.children.len(), 2);

        let label = tree.get(badge.children[1]).expect("Label node exists");
        assert_eq!(label.text.as_deref(), Some("albedo.png"));
    }
}