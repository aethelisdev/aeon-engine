// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toolbar Buttons & Mode Selectors
//!
//! Provides specialized toolbar strip buttons, square tool icons, and mode dropdowns on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, CornerRadii, Insets, JustifyContent, Style, TextAlign, WidgetCursor,
    WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits a 32×32 pixel square toolbar icon button for tools, modes, and action toggles.
    ///
    /// Provides built-in hover lifting, active state coloring, and pointer cursor styling.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the icon.
    /// * `tag` - Semantic tag for tracking interactions.
    /// * `is_active` - Whether this tool is currently selected or active.
    pub fn toolbar_icon_button(
        &mut self,
        icon_uv: [f32; 4],
        tag: u64,
        is_active: bool,
    ) -> WidgetResponse {
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("ToolbarIconButton");
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        let is_effective_active = is_active ^ clicked;
        let (bg, border_color) = if is_effective_active {
            (
                Color::rgba(0.06, 0.46, 0.92, 1.0),
                Color::rgba(0.25, 0.60, 1.0, 0.95),
            )
        } else if hovered {
            (
                Color::rgba(0.20, 0.23, 0.30, 0.95),
                Color::rgba(0.35, 0.40, 0.50, 0.90),
            )
        } else {
            (
                Color::rgba(0.12, 0.13, 0.16, 0.92),
                Color::rgba(0.24, 0.26, 0.32, 0.85),
            )
        };

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(32.0)
                .height(32.0)
                .background(bg)
                .border(1.0, border_color)
                .border_radius(4.0)
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
            if !is_effective_active {
                style = style
                    .hover_background(Color::rgba(0.20, 0.23, 0.30, 0.95))
                    .hover_border(1.0, Color::rgba(0.35, 0.40, 0.50, 0.90));
            }
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_name("ToolbarIcon");
            node.set_texture_uv(icon_uv);
            let tint = if is_effective_active {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else if hovered {
                Color::rgba(0.95, 0.98, 1.0, 1.0)
            } else {
                Color::rgba(0.80, 0.84, 0.90, 0.90)
            };
            node.set_texture_tint(tint);
            node.hover_texture_tint = Some(Color::rgba(0.95, 0.98, 1.0, 1.0));
            node.set_style(Style::new().width(22.0).height(22.0));
        }
        let _ = self.tree.add_child(btn_id, icon_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits a toolbar mode selector button with optional leading icon and text label (e.g. for dropdown triggers).
    ///
    /// # Arguments
    /// * `icon_uv` - Optional leading icon texture UV coordinates.
    /// * `label` - Caption text displayed on the mode button.
    /// * `tag` - Semantic tag for tracking interactions.
    /// * `is_open` - Whether the dropdown popup associated with this mode is open.
    /// * `width` - Explicit button width in logical points, or `0.0` for auto-sizing with padding.
    /// * `radii` - Corner radii styling.
    pub fn toolbar_mode_button(
        &mut self,
        icon_uv: Option<[f32; 4]>,
        label: impl Into<String>,
        tag: u64,
        is_open: bool,
        width: f32,
        radii: CornerRadii,
    ) -> WidgetResponse {
        let label_str = label.into();
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("ToolbarModeButton");
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        let is_highlighted = is_open || hovered || clicked;
        let bg = if is_highlighted {
            Color::rgba(0.20, 0.23, 0.30, 0.90)
        } else {
            Color::TRANSPARENT
        };
        let text_color = if is_highlighted {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.85, 0.88, 0.94, 1.0)
        };

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(4.0)
                .height(30.0)
                .background(bg)
                .corner_radii(radii);
            if width > 0.0 {
                style = style.width(width);
            } else {
                style = style.padding_insets(Insets::new(0.0, 10.0, 0.0, 10.0));
            }
            if !is_open {
                style = style.hover_background(Color::rgba(0.20, 0.23, 0.30, 0.90));
            }
            node.set_style(style);
        }

        if let Some(uv) = icon_uv {
            let icon_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(icon_id) {
                node.interactive = false;
                node.set_name("ToolbarModeIcon");
                node.set_texture_uv(uv);
                node.set_texture_tint(text_color);
                node.hover_texture_tint = Some(Color::WHITE);
                node.set_style(Style::new().width(18.0).height(18.0));
            }
            let _ = self.tree.add_child(btn_id, icon_id);
        }

        let approx_text_w = (label_str.chars().count() as f32 * 6.8).ceil();
        let txt_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(txt_id) {
            node.interactive = false;
            node.set_name("ToolbarModeText");
            node.set_style(Style::new().width(approx_text_w).height(14.0));
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.hover_text_color = Some(Color::WHITE);
        }
        let _ = self.tree.add_child(btn_id, txt_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }
}