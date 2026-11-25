// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating Dropdown Menu Cards & Items (`irisui-widgets::declarative::scope::menu::dropdown`)
//!
//! Provides floating dropdown popup containers, selectable menu item rows with text or
//! GPU atlas icons, shortcut annotations, and horizontal dividers on [`UiScope`].

use super::super::containers::InnerResponse;
use super::super::core::UiScope;
use super::super::scoped_popup::DropdownCardConfig;
use super::types::DropdownIcon;
use crate::declarative::types::WidgetResponse;
use irisui_core::{
    AlignItems, Color, Insets, JustifyContent, Rect, Style, TextAlign, UiLayer, WidgetCursor,
    WidgetId, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits an elevated floating dropdown popup container positioned at `(x, y)` on [`UiLayer::Popup`].
    ///
    /// # Arguments
    /// * `x` - Left horizontal screen offset in physical pixels.
    /// * `y` - Top vertical screen offset in physical pixels.
    /// * `width` - Fixed card width constraint in physical pixels.
    /// * `f` - Closure emitting dropdown items inside this popup scope.
    pub fn dropdown_menu_card<R, F>(&mut self, x: f32, y: f32, width: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = self.tree.create_node();
        let theme = self.theme();
        let bg = theme.modal_bg;
        let border = theme.modal_border;
        let radius = theme.corner_radius_md;
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("DropdownMenu");
            node.set_role(WidgetRole::DropdownPopup);
            node.set_layer(UiLayer::Popup);
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .flex_col()
                    .width(width)
                    .padding_insets(Insets::new(4.0, 4.0, 4.0, 4.0))
                    .background(bg)
                    .border(1.0, border)
                    .border_radius(radius)
                    .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70)),
            );
            node.computed_rect.x = x;
            node.computed_rect.y = y;
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let child_seed = crate::declarative::types::combine_seeds(
            self.seed,
            crate::declarative::types::hash_label("DropdownMenu"),
        );
        let mut child_scope = self.child_scope_seeded(node_id, child_seed);
        let inner = f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);
        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.height = Some(height);
        }
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits an elevated floating dropdown popup container with an explicit debug name at `(x, y)` on [`UiLayer::Popup`].
    pub fn dropdown_menu_card_named<R, F>(
        &mut self,
        name: &'static str,
        x: f32,
        y: f32,
        width: f32,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.dropdown_menu_card_styled(name, DropdownCardConfig::new(x, y, width), f)
    }

    /// Emits a selectable menu item row inside a floating dropdown menu.
    ///
    /// Automatically handles icon, label, and keyboard shortcut alignment.
    /// Interaction and hover highlights are resolved internally via [`UiScope::check_interaction`].
    pub fn dropdown_item(
        &mut self,
        tag: u64,
        icon: &str,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
    ) -> WidgetResponse {
        let icon_val = if icon.is_empty() {
            DropdownIcon::None
        } else {
            DropdownIcon::Text(icon)
        };
        self.render_dropdown_item_impl(tag, icon_val, label, shortcut, enabled)
    }

    /// Emits an interactive item row inside an elevated dropdown menu card using a unified [`irisui_core::UiIcon`].
    pub fn dropdown_item_with_ui_icon(
        &mut self,
        tag: irisui_core::UiTag,
        icon: Option<irisui_core::UiIcon>,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
    ) -> WidgetResponse {
        self.render_dropdown_item_impl(tag.raw(), icon.into(), label, shortcut, enabled)
    }

    /// Emits a [`irisui_core::UiIcon`]-backed dropdown item that autonomously closes all open popups when clicked.
    ///
    /// Preserves the returned [`WidgetResponse`] so consumers can attach fluent `.on_click(...)` callbacks
    /// with zero manual popup boolean resets or intermediate `let` bindings.
    #[inline]
    pub fn dropdown_item_autoclose_ui_icon(
        &mut self,
        tag: irisui_core::UiTag,
        icon: Option<irisui_core::UiIcon>,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
    ) -> WidgetResponse {
        let resp = self.dropdown_item_with_ui_icon(tag, icon, label, shortcut, enabled);
        if resp.clicked() {
            self.tree.close_all_popups();
        }
        resp
    }

    /// Emits an interactive item row inside an elevated dropdown menu card with an optional text or texture icon.
    pub fn dropdown_item_with_icon(
        &mut self,
        tag: u64,
        icon: Option<crate::cascading_menu::CascadingMenuIcon>,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
    ) -> WidgetResponse {
        self.render_dropdown_item_impl(tag, icon.into(), label, shortcut, enabled)
    }

    /// Internal shared rendering implementation for dropdown item rows.
    pub(super) fn render_dropdown_item_impl(
        &mut self,
        tag: u64,
        icon: DropdownIcon<'_>,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("DropdownItem");
            node.set_role(WidgetRole::DropdownItem);
            node.set_layer(UiLayer::Popup);
            node.set_tag(tag);
            node.interactive = enabled;
            node.cursor = if enabled {
                Some(WidgetCursor::Pointer)
            } else {
                None
            };
        }
        let (parent_rect, cur_y) = if let Some(parent_node) = self.tree.get(self.parent) {
            let p_rect = parent_node.computed_rect;
            let mut y_offset = p_rect.y + 4.0;
            for child_id in &parent_node.children {
                if let Some(child) = self.tree.get(*child_id) {
                    if child.role == WidgetRole::Separator {
                        y_offset += 7.0;
                    } else {
                        y_offset += 24.0;
                    }
                }
            }
            (p_rect, y_offset)
        } else {
            (Rect::ZERO, 0.0)
        };

        let initial_rect = if parent_rect.width > 0.0 {
            Rect::new(
                parent_rect.x + 4.0,
                cur_y,
                (parent_rect.width - 8.0).max(0.0),
                24.0,
            )
        } else {
            Rect::ZERO
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = initial_rect;
        }

        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, mut hovered, _) = self.check_interaction(node_id);

        if !hovered
            && enabled
            && let Some(cursor) = self.tree.cursor_position()
            && initial_rect.contains_point(cursor)
        {
            hovered = true;
        }

        if hovered && enabled {
            let parent_tag = if let Some(n) = self.tree.get(self.parent) {
                if n.tag != 0 {
                    n.tag
                } else {
                    let name = n.name.as_deref().unwrap_or("MenuCard");
                    crate::declarative::types::hash_label(name)
                }
            } else {
                0
            };
            self.tree.close_submenu(parent_tag);
        }

        let theme = self.theme();
        let text_color = if !enabled {
            theme.text_muted
        } else {
            theme.text_secondary
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .height(24.0)
                .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
                .background(Color::TRANSPARENT)
                .border_radius(theme.corner_radius_sm);
            if enabled {
                style = style.hover_background(theme.button_hover_bg);
            }
            node.set_style(style);
        }

        match icon {
            DropdownIcon::Text(t) if !t.is_empty() => {
                let icon_id = self.tree.create_node();
                if let Some(n) = self.tree.get_mut(icon_id) {
                    n.set_name("DropdownIcon");
                    n.set_role(WidgetRole::DropdownIcon);
                    n.set_layer(UiLayer::Popup);
                    n.interactive = false;
                    n.set_text(t);
                    n.font_size = 12.0;
                    n.line_height = 14.0;
                    n.text_align = TextAlign::Center;
                    n.text_color = text_color;
                    n.set_style(Style::new().width(18.0).height(14.0).flex_shrink(0.0));
                }
                let _ = self.tree.add_child(node_id, icon_id);
            }
            DropdownIcon::Texture(uv) => {
                let icon_id = self.tree.create_node();
                if let Some(n) = self.tree.get_mut(icon_id) {
                    n.set_name("DropdownIcon");
                    n.set_role(WidgetRole::DropdownIcon);
                    n.set_layer(UiLayer::Popup);
                    n.interactive = false;
                    n.set_texture_uv(uv);
                    n.set_texture_tint(text_color);
                    if enabled {
                        n.hover_texture_tint = Some(theme.text_primary);
                    }
                    n.set_style(
                        Style::new()
                            .width(14.0)
                            .height(14.0)
                            .margin_insets(Insets::new(0.0, 2.0, 0.0, 2.0))
                            .flex_shrink(0.0),
                    );
                }
                let _ = self.tree.add_child(node_id, icon_id);
            }
            _ => {}
        }

        let label_id = self.tree.create_node();
        if let Some(n) = self.tree.get_mut(label_id) {
            n.set_name("DropdownLabel");
            n.set_role(WidgetRole::DropdownLabel);
            n.set_layer(UiLayer::Popup);
            n.interactive = false;
            n.set_text(label);
            n.font_size = 12.0;
            n.line_height = 14.0;
            n.text_align = TextAlign::Left;
            n.text_color = text_color;
            n.set_style(Style::new().flex_grow(1.0).height(14.0));
            if enabled {
                n.set_hover_text_color(theme.text_primary);
            }
        }
        let _ = self.tree.add_child(node_id, label_id);

        if let Some(sc) = shortcut {
            let sc_color = if sc == "✓" {
                theme.accent
            } else if !enabled {
                theme.text_muted
            } else {
                theme.text_secondary
            };
            let sc_id = self.tree.create_node();
            if let Some(n) = self.tree.get_mut(sc_id) {
                n.set_name("DropdownShortcut");
                n.set_role(WidgetRole::DropdownShortcut);
                n.set_layer(UiLayer::Popup);
                n.interactive = false;
                n.set_text(sc);
                n.font_size = 11.0;
                n.line_height = 14.0;
                n.text_align = TextAlign::Right;
                n.text_color = sc_color;
                let sc_w = if sc.len() <= 4 { 18.0 } else { 74.0 };
                n.set_style(Style::new().width(sc_w).height(14.0).flex_shrink(0.0));
                if enabled && sc != "✓" {
                    n.set_hover_text_color(theme.text_secondary);
                }
            }
            let _ = self.tree.add_child(node_id, sc_id);
        }

        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a standard dropdown menu item row from a [`crate::cascading_menu::CascadingMenuItem`].
    #[inline]
    pub fn dropdown_menu_item(
        &mut self,
        item: &crate::cascading_menu::CascadingMenuItem,
    ) -> WidgetResponse {
        self.dropdown_item_with_icon(
            item.tag,
            item.icon,
            &item.label,
            item.shortcut.as_deref(),
            item.is_enabled,
        )
    }

    /// Emits a selectable dropdown item row that autonomously closes all open popups when clicked.
    ///
    /// Preserves the returned [`WidgetResponse`] (including `clicked() == true`) so consumers can
    /// chain domain actions (e.g., `.on_click(...)` / `.then_click(...)`) without intermediate `let`
    /// statements or manual popup boolean resets.
    #[inline]
    pub fn dropdown_item_autoclose(
        &mut self,
        tag: u64,
        icon: &str,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
    ) -> WidgetResponse {
        let resp = self.dropdown_item(tag, icon, label, shortcut, enabled);
        if resp.clicked() {
            self.tree.close_all_popups();
        }
        resp
    }

    /// Emits a [`crate::cascading_menu::CascadingMenuItem`] row that autonomously closes all open popups when clicked.
    ///
    /// Preserves the returned [`WidgetResponse`] (including `clicked() == true`) so consumers can
    /// attach fluent action handlers with zero manual boolean popup flags.
    #[inline]
    pub fn dropdown_menu_item_autoclose(
        &mut self,
        item: &crate::cascading_menu::CascadingMenuItem,
    ) -> WidgetResponse {
        let resp = self.dropdown_menu_item(item);
        if resp.clicked() {
            self.tree.close_all_popups();
        }
        resp
    }

    /// Emits a subtle horizontal divider line separating dropdown item groups.
    pub fn dropdown_separator(&mut self) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("DropdownSeparator");
            node.set_role(WidgetRole::Separator);
            node.set_layer(UiLayer::Popup);
            node.interactive = false;
            node.set_style(
                Style::new()
                    .height(7.0)
                    .padding_insets(Insets::new(3.0, 4.0, 3.0, 4.0))
                    .flex_col()
                    .justify_content(JustifyContent::Center),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let divider_color = self.theme().divider;
        let line_id = self.tree.create_node();
        if let Some(line) = self.tree.get_mut(line_id) {
            line.set_layer(UiLayer::Popup);
            line.interactive = false;
            line.set_style(Style::new().height(1.0).background(divider_color));
        }
        let _ = self.tree.add_child(node_id, line_id);
        node_id
    }
}