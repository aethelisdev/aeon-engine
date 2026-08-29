// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top application menu bar and dropdown menu item builders.

use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, UiTree, WidgetId};

/// Helper builder for the top horizontal application menu bar.
pub struct MenuBarBuilder<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
    left_group: WidgetId,
    right_group: WidgetId,
}

impl<'a> MenuBarBuilder<'a> {
    /// Creates a full-width top menu bar with left and right layout partitions matching egui geometry.
    pub fn new(tree: &'a mut UiTree, width: f32) -> Self {
        let node_id = tree.create_node();
        let left_group = tree.create_node();
        let right_group = tree.create_node();

        if let Some(node) = tree.get_mut(node_id) {
            node.set_name("TopMenuBar");
            node.set_style(
                Style::new()
                    .flex_row()
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Center)
                    .width(width)
                    .height(26.0)
                    .padding_insets(Insets::new(0.0, 6.0, 0.0, 10.0))
                    .background(Color::hex("#0f0f14"))
                    .border(1.0, Color::hex("#2d303c")),
            );
        }

        if let Some(left) = tree.get_mut(left_group) {
            left.set_name("MenuBarLeftGroup");
            left.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(2.0),
            );
        }

        if let Some(right) = tree.get_mut(right_group) {
            right.set_name("MenuBarRightGroup");
            right.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(6.0),
            );
        }

        let _ = tree.add_child(node_id, left_group);
        let _ = tree.add_child(node_id, right_group);

        Self {
            tree,
            node_id,
            left_group,
            right_group,
        }
    }

    /// Appends a clickable text menu item (e.g. "File", "Edit", "View") to the left group.
    pub fn add_menu_button(
        &mut self,
        label: impl Into<String>,
        is_active: bool,
        is_hovered: bool,
    ) -> WidgetId {
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_text(label);
            node.font_size = 12.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;

            let (text_color, bg) = if is_active {
                (Color::hex("#00e5ff"), Color::hex("#202432"))
            } else if is_hovered {
                (Color::WHITE, Color::hex("#282c38"))
            } else {
                (Color::hex("#dcdce2"), Color::TRANSPARENT)
            };

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

        let _ = self.tree.add_child(self.left_group, btn_id);
        btn_id
    }

    /// Appends an action control button (e.g. "▶ Play" or "⏹ Stop") to the right group.
    pub fn add_action_button(
        &mut self,
        label: impl Into<String>,
        bg_color: Color,
        text_color: Color,
        is_hovered: bool,
    ) -> WidgetId {
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 13.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;

            let bg = if is_hovered {
                Color::lerp(bg_color, Color::WHITE, 0.15)
            } else {
                bg_color
            };

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

        let _ = self.tree.add_child(self.right_group, btn_id);
        btn_id
    }

    /// Consumes the builder and returns the root `WidgetId` of the menu bar.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}

/// Helper builder for standalone floating dropdown popup menus.
pub struct DropdownMenuBuilder<'a> {
    tree: &'a mut UiTree,
    node_id: WidgetId,
}

impl<'a> DropdownMenuBuilder<'a> {
    /// Creates a new floating dropdown menu positioned at the specified screen coordinates.
    pub fn new(tree: &'a mut UiTree, x: f32, y: f32, width: f32) -> Self {
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.set_name("DropdownMenu");
            node.set_style(
                Style::new()
                    .flex_col()
                    .width(width)
                    .padding_insets(Insets::new(4.0, 4.0, 4.0, 4.0))
                    .background(Color::hex("#14141c"))
                    .border(1.0, Color::hex("#2d303c"))
                    .border_radius(4.0)
                    .box_shadow(0.0, 4.0, 12.0, Color::rgba(0.0, 0.0, 0.0, 0.6)),
            );
            node.computed_rect.x = x;
            node.computed_rect.y = y;
        }

        Self { tree, node_id }
    }

    /// Appends a clickable menu item row with left icon, label, and right shortcut in distinct columns.
    pub fn add_item(
        &mut self,
        icon: &str,
        label: &str,
        shortcut: Option<&str>,
        enabled: bool,
        is_hovered: bool,
    ) -> WidgetId {
        let row_id = self.tree.create_node();

        let (text_color, bg) = if !enabled {
            (Color::hex("#646470"), Color::TRANSPARENT)
        } else if is_hovered {
            (Color::WHITE, Color::hex("#282c38"))
        } else {
            (Color::hex("#dcdce2"), Color::TRANSPARENT)
        };

        if !icon.is_empty() {
            let icon_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(icon_id) {
                node.set_name("DropdownIcon");
                node.set_text(icon);
                node.font_size = 12.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = text_color;
            }
            let _ = self.tree.add_child(row_id, icon_id);
        }

        let label_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(label_id) {
            node.set_name("DropdownLabel");
            node.set_text(label);
            node.font_size = 12.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Left;
            node.text_color = text_color;
        }
        let _ = self.tree.add_child(row_id, label_id);

        if let Some(sc) = shortcut {
            let sc_id = self.tree.create_node();
            if let Some(sc_node) = self.tree.get_mut(sc_id) {
                sc_node.set_name("DropdownShortcut");
                sc_node.set_text(sc);
                sc_node.font_size = 11.0;
                sc_node.line_height = 14.0;
                sc_node.text_align = TextAlign::Right;
                sc_node.text_color = if sc == "✓" {
                    Color::hex("#00e5ff")
                } else if !enabled {
                    Color::hex("#50505a")
                } else if is_hovered {
                    Color::hex("#b0b0be")
                } else {
                    Color::hex("#828292")
                };
            }
            let _ = self.tree.add_child(row_id, sc_id);
        }

        if let Some(row_node) = self.tree.get_mut(row_id) {
            row_node.set_name("DropdownItem");
            row_node.set_style(
                Style::new()
                    .height(24.0)
                    .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0))
                    .background(bg)
                    .border_radius(3.0)
                    .flex_row()
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Center)
                    .flex_shrink(0.0),
            );
        }

        let _ = self.tree.add_child(self.node_id, row_id);
        row_id
    }

    /// Appends a subtle 1px horizontal separator divider line.
    pub fn add_separator(&mut self) -> WidgetId {
        let sep_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(sep_id) {
            node.set_name("DropdownSeparator");
            node.set_style(
                Style::new()
                    .height(1.0)
                    .margin_insets(Insets::new(3.0, 4.0, 3.0, 4.0))
                    .background(Color::hex("#2d303c")),
            );
        }

        let _ = self.tree.add_child(self.node_id, sep_id);
        sep_id
    }

    /// Consumes the builder and returns the root `WidgetId` of the dropdown.
    #[inline]
    pub fn build(self) -> WidgetId {
        self.node_id
    }
}