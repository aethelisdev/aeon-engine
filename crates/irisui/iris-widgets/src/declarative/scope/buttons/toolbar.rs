// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toolbar Buttons & Mode Selectors
//!
//! Provides specialized toolbar strip buttons, square tool icons, and mode dropdowns on [`UiScope`].

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
        self.toolbar_icon_button_styled(icon_uv, tag, is_active, None, None)
    }

    /// Emits a square 32×32 pixel icon tool button with customizable active background and border colors.
    ///
    /// When optional color overrides are `None`, falls back to the ambient [`Theme`](iris_core::Theme)
    /// tokens (`theme.button_active_bg` and `theme.button_hover_border`).
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the icon.
    /// * `tag` - Semantic tag for tracking interactions.
    /// * `is_active` - Whether this tool is currently selected or active.
    /// * `active_bg` - Optional custom background color for active state.
    /// * `active_border` - Optional custom outline border color for active state.
    pub fn toolbar_icon_button_styled(
        &mut self,
        icon_uv: [f32; 4],
        tag: u64,
        is_active: bool,
        active_bg: Option<Color>,
        active_border: Option<Color>,
    ) -> WidgetResponse {
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("ToolbarIconButton");
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        let theme = self.theme();
        let (bg, border_color) = if is_active {
            (
                active_bg.unwrap_or(theme.button_active_bg),
                active_border.unwrap_or(theme.button_hover_border),
            )
        } else if hovered {
            (theme.button_hover_bg, theme.button_hover_border)
        } else {
            (theme.button_idle_bg, theme.button_border)
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
                .border_radius(theme.corner_radius_md)
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
            if !is_active {
                style = style
                    .hover_background(theme.button_hover_bg)
                    .hover_border(1.0, theme.button_hover_border);
            }
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_name("ToolbarIcon");
            node.set_texture_uv(icon_uv);
            let tint = if is_active || hovered {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            node.set_texture_tint(tint);
            node.hover_texture_tint = Some(theme.text_primary);
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

        let theme = self.theme();
        let is_highlighted = is_open || hovered || clicked;
        let bg = if is_highlighted {
            theme.button_hover_bg
        } else {
            Color::TRANSPARENT
        };
        let text_color = if is_highlighted {
            theme.text_primary
        } else {
            theme.text_secondary
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
                style = style.hover_background(theme.button_hover_bg);
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
                node.hover_texture_tint = Some(theme.text_primary);
                node.set_style(Style::new().width(18.0).height(18.0));
            }
            let _ = self.tree.add_child(btn_id, icon_id);
        }

        let txt_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(txt_id) {
            node.interactive = false;
            node.set_name("ToolbarModeText");
            node.set_style(Style::new().height(14.0));
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.hover_text_color = Some(theme.text_primary);
        }
        let _ = self.tree.add_child(btn_id, txt_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits a standalone toolbar dropdown button displaying a text caption with full framing.
    ///
    /// Delegates directly to [`crate::declarative::widget::ToolbarButton`] ensuring unified
    /// intrinsic text layout, full theme token compliance, and zero magic number approximations.
    pub fn toolbar_dropdown_button(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        is_open: bool,
        width: f32,
    ) -> WidgetResponse {
        self.add(
            crate::declarative::widget::ToolbarButton::new(label)
                .tag(tag)
                .width(width)
                .height(32.0)
                .open(is_open),
        )
    }
}