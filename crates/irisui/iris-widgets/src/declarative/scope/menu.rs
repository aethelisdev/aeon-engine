// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Menu Bar & Floating Dropdown Primitives
//!
//! Provides top application menu bar items, play/stop action buttons, floating
//! dropdown popup cards, selectable menu item rows with icons and shortcut annotations,
//! and dividers on [`UiScope`].
//!

use super::core::UiScope;
use crate::declarative::scope::containers::InnerResponse;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Rect, Style, TextAlign, UiLayer, WidgetCursor,
    WidgetId, WidgetRole,
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
                    .height(20.0)
                    .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
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

    /// Emits a prominent top menu action button (e.g. "▶ Play" or "⏹ Stop").
    ///
    /// The semantic interaction tag is automatically derived from the caption string via
    /// [`UiScope::tag_for`]. Applies a ~15% background brightness lift when hovered.
    ///
    /// # Arguments
    /// * `label` - Caption text on the action button.
    /// * `bg_color` - Base idle background color (e.g. emerald green or crimson red).
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
    /// * `bg_color` - Base idle background color (e.g. emerald green or crimson red).
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
                    .height(20.0)
                    .padding_insets(Insets::new(2.0, 10.0, 2.0, 10.0))
                    .background(bg_color)
                    .hover_background(hover_bg)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .flex_shrink(0.0),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits an industrial desktop window control button (Minimize, Maximize, Close).
    ///
    /// Close buttons apply an elevated danger hover tint (`#E81123`), while standard controls
    /// apply subtle toolbar hover lightening (`rgba(1.0, 1.0, 1.0, 0.12)`).
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
            Color::hex("#E81123")
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.12)
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 13.0;
            node.text_align = TextAlign::Center;
            node.text_color = theme.text_secondary;
            node.hover_text_color = Some(Color::WHITE);
            node.set_style(
                Style::new()
                    .width(36.0)
                    .height(24.0)
                    .background(Color::TRANSPARENT)
                    .hover_background(hover_bg)
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
    ///
    /// # Arguments
    /// * `name` - Static debug name assigned to the dropdown container.
    /// * `x` - Left horizontal screen offset in physical pixels.
    /// * `y` - Top vertical screen offset in physical pixels.
    /// * `width` - Fixed card width constraint in physical pixels.
    /// * `f` - Closure emitting dropdown items inside this popup scope.
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

        let child_seed = crate::declarative::types::combine_seeds(
            self.seed,
            crate::declarative::types::hash_label(name),
        );
        let mut child_scope = self.child_scope_seeded(node_id, child_seed);
        let inner = f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);
        let window_h = self
            .tree
            .root()
            .and_then(|r| self.tree.get(r))
            .map(|n| n.computed_rect.height)
            .filter(|&h| h > 10.0)
            .unwrap_or(1080.0);
        let clamped_y = if y + height > window_h - 6.0 {
            (window_h - height - 6.0).max(6.0)
        } else {
            y
        };
        let bounds = Rect::new(x, clamped_y, width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.inset_top = Some(clamped_y);
            node.style.height = Some(height);
        }
        if let Some(cursor) = self.tree.cursor_position() {
            super::hover::update_hover_styles_recursive(self.tree, node_id, cursor);
        }
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
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

        let theme = self.theme();
        let (text_color, bg) = if !enabled {
            (theme.text_muted, Color::TRANSPARENT)
        } else {
            (theme.text_secondary, Color::TRANSPARENT)
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .height(24.0)
                .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
                .background(bg)
                .border_radius(theme.corner_radius_sm);
            if enabled {
                style = style.hover_background(theme.button_hover_bg);
            }
            node.set_style(style);
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
            if enabled {
                n.set_hover_text_color(theme.text_primary);
            }
        }
        let _ = self.tree.add_child(node_id, label_id);

        // 3. Trailing Shortcut / Checkmark (fixed width 74.0, right-aligned)
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

    /// Emits an elevated, cascading branching submenu item row with a custom popup container debug name.
    ///
    /// When the pointer hovers over this row or when its submenu branch is active,
    /// a secondary floating popup card container is autonomously positioned and rendered
    /// using smart placement (auto-flip right/left and vertical boundary clamping).
    ///
    /// # Arguments
    /// * `popup_name` - Debug node name assigned to the child submenu popup card.
    /// * `label` - Primary menu action text displayed on this row.
    /// * `icon` - Optional leading icon symbol.
    /// * `tag` - Semantic identifier constant assigned to this submenu branch.
    /// * `f` - Scoped closure populating items inside the child submenu popup card.
    pub fn submenu_named<R, F>(
        &mut self,
        popup_name: &'static str,
        label: impl Into<String>,
        icon: Option<&str>,
        tag: u64,
        f: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let label_str = label.into();
        let icon_str = icon.unwrap_or("");
        let parent_card_id = self.parent;
        let parent_card_rect = self
            .tree
            .get(parent_card_id)
            .map(|n| n.computed_rect)
            .unwrap_or(Rect::ZERO);

        let parent_tag = if let Some(n) = self.tree.get(parent_card_id) {
            if n.tag != 0 {
                n.tag
            } else {
                let name = n.name.as_deref().unwrap_or("MenuCard");
                crate::declarative::types::hash_label(name)
            }
        } else {
            0
        };

        // 1. Emit the interactive row in the current menu card
        let resp = self.dropdown_item(tag, icon_str, &label_str, Some("▶"), true);

        // 2. Check autonomous hover activation
        let is_hovered = resp.hovered()
            || self.tree.cursor_position().is_some_and(|c| {
                self.tree
                    .get(resp.id)
                    .is_some_and(|n| n.computed_rect.contains_point(c))
            });

        if is_hovered {
            self.tree.set_active_submenu(parent_tag, tag);
        }

        let is_branch_active = self.tree.is_submenu_open(parent_tag, tag);

        // 3. If active, autonomously compute smart placement and render child popup
        if is_branch_active {
            let item_rect = self
                .tree
                .get(resp.id)
                .map(|n| n.computed_rect)
                .unwrap_or(Rect::ZERO);

            let (window_w, _window_h) = if let Some(root_id) = self.tree.root() {
                let r = self
                    .tree
                    .get(root_id)
                    .map(|n| n.computed_rect)
                    .unwrap_or(Rect::ZERO);
                if r.width > 0.0 && r.height > 0.0 {
                    (r.width, r.height)
                } else {
                    (1920.0, 1080.0)
                }
            } else {
                (1920.0, 1080.0)
            };

            let sub_w = if parent_card_rect.width > 0.0 {
                parent_card_rect.width
            } else {
                185.0
            };

            // Smart horizontal placement: try right first, auto-flip to left if overflowing
            let mut sub_x = parent_card_rect.right() + 2.0;
            if sub_x + sub_w > window_w - 6.0 {
                let left_candidate = parent_card_rect.x - sub_w - 2.0;
                if left_candidate >= 6.0 {
                    sub_x = left_candidate;
                } else {
                    sub_x = (window_w - sub_w - 6.0).max(6.0);
                }
            }

            // Smart vertical placement: align with item top, clamp if overflowing
            let sub_y = (item_rect.y - 4.0).max(6.0);

            // Submenu card node created as an elevated overlay attached to root
            let root_anchor = self.tree.root().unwrap_or(parent_card_id);
            let mut sub_scope = UiScope::new(self.tree, root_anchor);
            sub_scope.dropdown_menu_card_named(popup_name, sub_x, sub_y, sub_w, |card| {
                if let Some(n) = card.tree.get_mut(card.parent) {
                    n.tag = tag;
                }
                f(card)
            });
        }

        resp
    }

    /// Emits an elevated, cascading branching submenu item row.
    ///
    /// When the pointer hovers over this row or when its submenu branch is active,
    /// a secondary floating popup card container is autonomously positioned and rendered
    /// using smart placement (auto-flip right/left and vertical boundary clamping).
    ///
    /// # Arguments
    /// * `label` - Primary menu action text displayed on this row.
    /// * `icon` - Optional leading icon symbol.
    /// * `tag` - Semantic identifier constant assigned to this submenu branch.
    /// * `f` - Scoped closure populating items inside the child submenu popup card.
    pub fn submenu<R, F>(
        &mut self,
        label: impl Into<String>,
        icon: Option<&str>,
        tag: u64,
        f: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.submenu_named("CascadingSubmenuPopup", label, icon, tag, f)
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

    /// Emits an elevated floating popup card container dynamically anchored to `anchor_rect`.
    ///
    /// Automatically performs horizontal and vertical viewport boundary clamping and
    /// flips direction if screen borders are exceeded, eliminating manual coordinate
    /// guessing and tree iteration in editor panels.
    ///
    /// # Arguments
    /// * `anchor_rect` - Absolute screen rectangle of the trigger widget or button.
    /// * `width` - Preferred popup width in physical pixels.
    /// * `f` - Scoped closure populating items inside the popup card.
    pub fn popup_anchored<R, F>(&mut self, anchor_rect: Rect, width: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.popup_anchored_with_placement(
            "AnchoredPopup",
            anchor_rect,
            width,
            PopupPlacement::Below,
            f,
        )
    }

    /// Emits an elevated floating popup card container with an explicit debug name anchored to `anchor_rect`.
    ///
    /// # Arguments
    /// * `name` - Static debug name assigned to the container node.
    /// * `anchor_rect` - Absolute screen rectangle of the trigger widget or button.
    /// * `width` - Preferred popup width in physical pixels.
    /// * `f` - Scoped closure populating items inside the popup card.
    pub fn popup_anchored_named<R, F>(
        &mut self,
        name: &'static str,
        anchor_rect: Rect,
        width: f32,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.popup_anchored_with_placement(name, anchor_rect, width, PopupPlacement::Below, f)
    }

    /// Emits an elevated floating popup card container with explicit placement preference and boundary clamping.
    ///
    /// # Arguments
    /// * `name` - Static debug name assigned to the container node.
    /// * `anchor_rect` - Absolute screen rectangle of the trigger widget or button.
    /// * `width` - Preferred popup width in physical pixels.
    /// * `placement` - Vertical orientation preference ([`PopupPlacement::Below`] or [`PopupPlacement::Above`]).
    /// * `f` - Scoped closure populating items inside the popup card.
    pub fn popup_anchored_with_placement<R, F>(
        &mut self,
        name: &'static str,
        anchor_rect: Rect,
        width: f32,
        placement: PopupPlacement,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let (window_w, window_h) = if let Some(root_id) = self.tree.root() {
            let r = self
                .tree
                .get(root_id)
                .map(|n| n.computed_rect)
                .unwrap_or(Rect::ZERO);
            if r.width > 0.0 && r.height > 0.0 {
                (r.width, r.height)
            } else {
                (1920.0, 1080.0)
            }
        } else {
            (1920.0, 1080.0)
        };

        // Horizontal placement: align left with anchor, clamp to right screen boundary
        let mut x = anchor_rect.x;
        if x + width > window_w - 6.0 {
            x = (anchor_rect.right() - width).clamp(6.0, window_w - width - 6.0);
        }

        // Vertical initial placement: below or above
        let mut y = match placement {
            PopupPlacement::Below => anchor_rect.bottom() + 2.0,
            PopupPlacement::Above => (anchor_rect.y - 200.0).max(6.0),
        };

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

        let child_seed = crate::declarative::types::combine_seeds(
            self.seed,
            crate::declarative::types::hash_label(name),
        );
        let mut child_scope = self.child_scope_seeded(node_id, child_seed);
        let inner = f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);

        // Re-evaluate vertical position with accurate measured height
        y = match placement {
            PopupPlacement::Below => {
                if anchor_rect.bottom() + height + 4.0 > window_h - 6.0 {
                    (anchor_rect.y - height - 2.0).max(6.0)
                } else {
                    anchor_rect.bottom() + 2.0
                }
            }
            PopupPlacement::Above => {
                if anchor_rect.y - height - 2.0 < 6.0 {
                    anchor_rect.bottom() + 2.0
                } else {
                    anchor_rect.y - height - 2.0
                }
            }
        };

        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.inset_left = Some(x);
            node.style.inset_top = Some(y);
            node.style.height = Some(height);
        }
        if let Some(cursor) = self.tree.cursor_position() {
            super::hover::update_hover_styles_recursive(self.tree, node_id, cursor);
        }
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }
}

/// Placement direction for anchored floating popups relative to their anchor rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupPlacement {
    /// Positions the popup card directly below the anchor rectangle (default).
    /// If bottom viewport overflow occurs, dynamically flips above the anchor.
    #[default]
    Below,
    /// Positions the popup card directly above the anchor rectangle.
    /// If top viewport overflow occurs, dynamically flips below the anchor.
    Above,
}

/// Extension trait adding autonomous popup capabilities to widget responses.
pub trait PopupExt {
    /// Opens or maintains an autonomous dropdown popup card anchored below this response.
    ///
    /// Returns the inner closure output if open.
    fn popup_below<R, F>(
        self,
        scope: &mut UiScope<'_>,
        width: f32,
        add_contents: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R;
}

impl PopupExt for WidgetResponse {
    #[inline]
    fn popup_below<R, F>(
        self,
        scope: &mut UiScope<'_>,
        width: f32,
        add_contents: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if self.rect.width > 0.0 && self.rect.height > 0.0 {
            Some(scope.popup_anchored_with_placement(
                "PopupBelow",
                self.rect,
                width,
                PopupPlacement::Below,
                add_contents,
            ))
        } else {
            None
        }
    }
}