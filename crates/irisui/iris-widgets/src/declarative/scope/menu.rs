// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Menu Bar & Floating Dropdown Primitives
//!
//! Provides top application menu bar items, play/stop action buttons, floating
//! dropdown popup cards, selectable menu item rows with icons and shortcut annotations,
//! and dividers on [`UiScope`].
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Rect, Style, TextAlign, UiLayer, WidgetCursor,
    WidgetId, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits a top application menu bar button (e.g. "File", "Edit", "View", "Window", "Help").
    ///
    /// Resolves hover and interaction state internally via [`UiScope::check_interaction`] without
    /// requiring manual external coordinate hit tests.
    ///
    /// # Arguments
    /// * `label` - Caption text displayed on the menu bar button.
    /// * `tag` - Semantic identifier constant inspected during menu activation routing.
    /// * `is_active` - Whether the dropdown associated with this menu item is currently open.
    pub fn menu_bar_item(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        is_active: bool,
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

        let (text_color, bg) = if is_active {
            (Color::hex("#00e5ff"), Color::hex("#1e2230"))
        } else if hovered {
            (Color::WHITE, Color::hex("#222634"))
        } else {
            (Color::hex("#dcdce2"), Color::TRANSPARENT)
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 12.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.set_style(
                Style::new()
                    .height(20.0)
                    .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                    .background(bg)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_shrink(0.0),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a prominent top menu action button (e.g. "▶ Play" or "⏹ Stop").
    ///
    /// Applies a ~15% background brightness lift when hovered, resolved internally by the framework.
    ///
    /// # Arguments
    /// * `label` - Caption text on the action button.
    /// * `tag` - Semantic identifier constant inspected during execution dispatch.
    /// * `bg_color` - Base idle background color (e.g. emerald green or crimson red).
    /// * `text_color` - Label text color.
    pub fn menu_action_button(
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

        let bg = if hovered {
            Color::lerp(bg_color, Color::WHITE, 0.15)
        } else {
            bg_color
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 13.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.set_style(
                Style::new()
                    .height(20.0)
                    .padding_insets(Insets::new(2.0, 10.0, 2.0, 10.0))
                    .background(bg)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_shrink(0.0),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits an elevated floating dropdown popup container positioned at `(x, y)` on [`UiLayer::Popup`].
    ///
    /// # Arguments
    /// * `x` - Left horizontal screen offset in physical pixels.
    /// * `y` - Top vertical screen offset in physical pixels.
    /// * `width` - Fixed card width constraint in physical pixels.
    /// * `f` - Closure emitting dropdown items inside this popup scope.
    pub fn dropdown_menu_card<F>(&mut self, x: f32, y: f32, width: f32, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
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
                    .background(Color::rgba(0.035, 0.040, 0.050, 0.98)) // Deep sleek obsidian black
                    .border(1.0, Color::rgba(0.20, 0.22, 0.28, 0.85))
                    .border_radius(4.0)
                    .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70)),
            );
            node.computed_rect.x = x;
            node.computed_rect.y = y;
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);
        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.height = Some(height);
        }
        node_id
    }

    /// Emits an elevated floating dropdown popup container with an explicit debug name at `(x, y)` on [`UiLayer::Popup`].
    ///
    /// # Arguments
    /// * `name` - Static debug name assigned to the dropdown container.
    /// * `x` - Left horizontal screen offset in physical pixels.
    /// * `y` - Top vertical screen offset in physical pixels.
    /// * `width` - Fixed card width constraint in physical pixels.
    /// * `f` - Closure emitting dropdown items inside this popup scope.
    pub fn dropdown_menu_card_named<F>(
        &mut self,
        name: &'static str,
        x: f32,
        y: f32,
        width: f32,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
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
                    .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
                    .border(1.0, Color::rgba(0.20, 0.23, 0.30, 0.90))
                    .border_radius(6.0)
                    .box_shadow(0.0, 6.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.75)),
            );
            node.computed_rect.x = x;
            node.computed_rect.y = y;
            node.computed_rect.width = width;
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);
        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.height = Some(height);
        }
        node_id
    }

    /// Emits a selectable menu item row inside a floating dropdown menu.
    ///
    /// Automatically handles icon, label, and keyboard shortcut alignment.
    /// Interaction and hover highlights are resolved internally via [`UiScope::check_interaction`].
    ///
    /// # Arguments
    /// * `tag` - Semantic tag constant mapped to an execution action.
    /// * `icon` - Optional leading symbol.
    /// * `label` - Primary menu action text.
    /// * `shortcut` - Optional trailing keyboard shortcut or checkmark indicator.
    /// * `enabled` - Whether the item is interactive or disabled/grayed-out.
    pub fn dropdown_item(
        &mut self,
        tag: u64,
        icon: &str,
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
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);

        let is_hovered = enabled && hovered;
        let (text_color, bg) = if !enabled {
            (Color::hex("#646470"), Color::TRANSPARENT)
        } else if is_hovered {
            (Color::WHITE, Color::hex("#222634"))
        } else {
            (Color::hex("#dcdce2"), Color::TRANSPARENT)
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(6.0)
                    .height(24.0)
                    .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
                    .background(bg)
                    .border_radius(3.0),
            );
        }

        // 1. Icon column (fixed width 18.0)
        if !icon.is_empty() {
            let icon_id = self.tree.create_node();
            if let Some(n) = self.tree.get_mut(icon_id) {
                n.set_name("DropdownIcon");
                n.set_role(WidgetRole::DropdownIcon);
                n.set_layer(UiLayer::Popup);
                n.interactive = false;
                n.set_text(icon);
                n.font_size = 12.0;
                n.line_height = 14.0;
                n.text_align = TextAlign::Center;
                n.text_color = text_color;
                n.set_style(Style::new().width(18.0).height(14.0).flex_shrink(0.0));
            }
            let _ = self.tree.add_child(node_id, icon_id);
        }

        // 2. Primary Label (flexible width: style.width is None, taking all available middle space)
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
        }
        let _ = self.tree.add_child(node_id, label_id);

        // 3. Trailing Shortcut / Checkmark (fixed width 74.0, right-aligned)
        if let Some(sc) = shortcut {
            let sc_color = if sc == "✓" {
                Color::hex("#00e5ff")
            } else if !enabled {
                Color::hex("#50505a")
            } else if is_hovered {
                Color::hex("#b0b0be")
            } else {
                Color::hex("#828292")
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
            }
            let _ = self.tree.add_child(node_id, sc_id);
        }

        WidgetResponse::new(node_id, clicked, hovered, false)
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

        let line_id = self.tree.create_node();
        if let Some(line) = self.tree.get_mut(line_id) {
            line.set_layer(UiLayer::Popup);
            line.interactive = false;
            line.set_style(
                Style::new()
                    .height(1.0)
                    .background(Color::rgba(0.20, 0.22, 0.28, 0.70)),
            );
        }
        let _ = self.tree.add_child(node_id, line_id);
        node_id
    }
}