// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top Application Menu Bar & Window Controls (`iris-widgets::declarative::scope::menu::bar_controls`)
//!
//! Provides top menu bar item buttons, prominent action buttons, and window
//! chrome control buttons on [`UiScope`].

use super::super::core::UiScope;
use super::super::menu_bar::MENU_ITEM_PAD_H;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits a top application menu bar button (e.g. "File", "Edit", "View", "Window", "Help").
    ///
    /// Resolves hover and interaction state internally via [`UiScope::check_interaction`]. The semantic
    /// interaction tag is automatically derived from the caption string via [`UiScope::tag_for`].
    ///
    /// # Arguments
    /// * `label` - Caption text displayed on the menu bar button.
    /// * `is_active` - Whether the dropdown associated with this menu item is currently open.
    pub fn menu_bar_item(&mut self, label: impl Into<String>, is_active: bool) -> WidgetResponse {
        let label_str = label.into();
        let tag = self.tag_for(&label_str);
        self.menu_bar_item_tagged(label_str, tag, is_active)
    }

    /// Emits a top application menu bar button with explicit tag and custom horizontal padding.
    ///
    /// # Arguments
    /// * `label` - Caption text displayed on the menu bar button.
    /// * `tag` - Explicit semantic identifier constant inspected during menu activation routing.
    /// * `is_active` - Whether the dropdown associated with this menu item is currently open.
    /// * `pad_h` - Horizontal padding in physical pixels applied symmetrically on left and right.
    pub fn menu_bar_item_styled(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        is_active: bool,
        pad_h: f32,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("MenuBarItem");
            node.set_role(WidgetRole::MenuBarItem);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let theme = self.theme();
        let (text_color, bg) = if is_active {
            (theme.accent, theme.button_active_bg)
        } else if hovered {
            (theme.text_primary, theme.button_hover_bg)
        } else {
            (theme.text_secondary, Color::TRANSPARENT)
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 12.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.hover_text_color = Some(theme.text_primary);
            let hover_bg = if is_active {
                theme.button_active_bg
            } else {
                theme.button_hover_bg
            };
            node.set_style(
                Style::new()
                    .height(24.0)
                    .padding_insets(Insets::new(3.0, pad_h, 3.0, pad_h))
                    .background(bg)
                    .hover_background(hover_bg)
                    .border_radius(theme.corner_radius_sm)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_shrink(0.0),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a top application menu bar button with an explicit semantic interaction tag.
    ///
    /// # Arguments
    /// * `label` - Caption text displayed on the menu bar button.
    /// * `tag` - Explicit semantic identifier constant inspected during menu activation routing.
    /// * `is_active` - Whether the dropdown associated with this menu item is currently open.
    pub fn menu_bar_item_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        is_active: bool,
    ) -> WidgetResponse {
        self.menu_bar_item_styled(label, tag, is_active, MENU_ITEM_PAD_H)
    }

    /// Emits a prominent top menu action button (e.g. "▶ Play" or "⏹ Stop").
    ///
    /// The semantic interaction tag is automatically derived from the caption string via
    /// [`UiScope::tag_for`]. Applies a background brightness lift when hovered.
    ///
    /// # Arguments
    /// * `label` - Caption text on the action button.
    /// * `bg_color` - Base idle background color.
    /// * `text_color` - Label text color.
    pub fn menu_action_button(
        &mut self,
        label: impl Into<String>,
        bg_color: Color,
        text_color: Color,
    ) -> WidgetResponse {
        let label_str = label.into();
        let tag = self.tag_for(&label_str);
        self.menu_action_button_tagged(label_str, tag, bg_color, text_color)
    }

    /// Emits a prominent top menu action button with an explicit semantic interaction tag.
    ///
    /// # Arguments
    /// * `label` - Caption text on the action button.
    /// * `tag` - Explicit semantic identifier constant inspected during execution dispatch.
    /// * `bg_color` - Base idle background color.
    /// * `text_color` - Label text color.
    pub fn menu_action_button_tagged(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        bg_color: Color,
        text_color: Color,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("MenuBarActionButton");
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let hover_bg = Color::lerp(bg_color, Color::WHITE, 0.18);

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 13.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.hover_text_color = Some(Color::WHITE);
            node.set_style(
                Style::new()
                    .height(24.0)
                    .padding_insets(Insets::new(3.0, 12.0, 3.0, 12.0))
                    .background(bg_color)
                    .hover_background(hover_bg)
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_shrink(0.0),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits an industrial desktop window control button (Minimize, Maximize, Close).
    ///
    /// Close buttons apply an elevated danger hover tint, while standard controls
    /// apply subtle toolbar hover lightening derived from theme tokens.
    ///
    /// # Arguments
    /// * `tag` - Interaction tag identifier for hit-testing and event tracking.
    /// * `label` - Text glyph on the control button (e.g. `—`, `□`, `✕`).
    /// * `is_close` - Whether this button represents the window close action.
    pub fn menu_window_button(
        &mut self,
        tag: u64,
        label: impl Into<String>,
        is_close: bool,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(if is_close {
                "WindowCloseButton"
            } else {
                "WindowControlButton"
            });
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let theme = self.theme();
        let hover_bg = if is_close {
            theme.danger
        } else {
            theme.button_hover_bg
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 13.0;
            node.text_align = TextAlign::Center;
            node.text_color = theme.text_secondary;
            node.hover_text_color = Some(theme.text_primary);
            node.set_style(
                Style::new()
                    .width(42.0)
                    .height(28.0)
                    .background(Color::TRANSPARENT)
                    .hover_background(hover_bg)
                    .border_radius(theme.corner_radius_sm)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_shrink(0.0),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }
}