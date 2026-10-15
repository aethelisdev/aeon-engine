// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Menu Bar & Application Action Routing Primitives
//!
//! Provides top-level application menu bars, cascading dropdown popup menus,
//! and strongly typed action routing without leaking internal widget identifiers.
//!

use super::core::UiScope;
use crate::declarative::types::hash_label_with_seed;
use iris_core::{AlignItems, Color, Insets, Style, WidgetRole};

/// Standard height in logical pixels for top application menu bars.
pub const MENUBAR_DEFAULT_HEIGHT: f32 = 28.0;

/// Standard width in logical pixels for cascading menu dropdown popups.
pub const MENUBAR_DROPDOWN_DEFAULT_WIDTH: f32 = 260.0;

/// Scoped context for constructing items inside an opened menu dropdown popup.
pub struct MenuScope<'a, 'b, A> {
    scope: &'a mut UiScope<'b>,
    selected_action: Option<A>,
    closed: bool,
}

impl<'a, 'b, A> MenuScope<'a, 'b, A> {
    /// Emits an interactive menu item with icon, label, optional shortcut annotation, and bound action.
    ///
    /// When clicked, registers the action and signals the menu bar to close the popup.
    pub fn item(&mut self, icon: &str, label: &str, shortcut: Option<&str>, action: A) {
        let tag = self.scope.tag_for(label);
        let resp = self.scope.dropdown_item(tag, icon, label, shortcut, true);
        if resp.clicked() {
            self.selected_action = Some(action);
            self.closed = true;
        }
    }

    /// Emits a disabled (grayed-out) menu item that cannot be triggered.
    pub fn item_disabled(&mut self, icon: &str, label: &str, shortcut: Option<&str>) {
        let tag = self.scope.tag_for(label);
        self.scope.dropdown_item(tag, icon, label, shortcut, false);
    }

    /// Emits a visual horizontal divider between grouped menu items.
    pub fn separator(&mut self) {
        self.scope.dropdown_separator();
    }
}

/// Scoped context for constructing right-aligned action buttons on the menu bar.
pub struct MenuBarRightScope<'a, 'b, A> {
    scope: UiScope<'b>,
    selected_action: &'a mut Option<A>,
    new_active_menu_tag: &'a mut Option<Option<u64>>,
}

impl<'a, 'b, A> MenuBarRightScope<'a, 'b, A> {
    /// Emits a prominent action button on the right side of the menu bar (e.g. Play / Stop).
    pub fn action_button(&mut self, label: &str, bg: Color, text: Color, action: A) {
        let resp = self.scope.menu_action_button(label, bg, text);
        if resp.clicked() {
            *self.selected_action = Some(action);
            *self.new_active_menu_tag = Some(None);
        }
    }
}

/// Scoped builder context passed to the closure of [`UiScope::menu_bar`].
pub struct MenuBarScope<'a, 'b, A> {
    scope: &'a mut UiScope<'b>,
    left_container: iris_core::WidgetId,
    right_container: iris_core::WidgetId,
    selected_action: Option<A>,
    active_menu_tag: Option<u64>,
    new_active_menu_tag: Option<Option<u64>>,
    menubar_y: f32,
    menubar_height: f32,
    current_menu_x: f32,
}

impl<'a, 'b, A> MenuBarScope<'a, 'b, A> {
    /// Declares a top-level cascading menu item (e.g., "File", "Edit", "View").
    ///
    /// Manages hover highlighting, open/closed toggling on click,  hover-switch
    /// when another menu is already open, and renders child dropdown popups on [`iris_core::UiLayer::Popup`].
    pub fn menu<F>(&mut self, title: &str, add_items: F)
    where
        F: FnOnce(&mut MenuScope<'_, '_, A>),
    {
        let tag = hash_label_with_seed(0, title);
        let was_active = self.active_menu_tag == Some(tag);
        let mut is_active = was_active;

        let mut left_scope = self.scope.child_scope(self.left_container);
        let resp = left_scope.menu_bar_item_tagged(title, tag, was_active);

        // 1. Hover-switch: When any menu is already open and pointer hovers over another header
        if self.active_menu_tag.is_some() && !was_active && resp.hovered() {
            self.new_active_menu_tag = Some(Some(tag));
            self.active_menu_tag = Some(tag);
            is_active = true;
        }

        // 2. Click toggle: Clicking an active header closes it; clicking an inactive header opens it immediately
        if resp.clicked() {
            if was_active {
                self.new_active_menu_tag = Some(None);
                self.active_menu_tag = None;
                is_active = false;
            } else {
                self.new_active_menu_tag = Some(Some(tag));
                self.active_menu_tag = Some(tag);
                is_active = true;
            }
        }

        // Apply instant effective highlight on the header node if state changed this frame
        if is_active != was_active {
            if let Some(node) = self.scope.tree.get_mut(resp.id) {
                if is_active {
                    node.text_color = Color::hex("#00e5ff");
                    node.style.background_color = Color::hex("#1e2230");
                } else {
                    node.text_color = Color::hex("#dcdce2");
                    node.style.background_color = Color::TRANSPARENT;
                }
            }
        }

        // Measure menu button anchor coordinate
        let popup_x = self.current_menu_x;
        let btn_width = (title.len() as f32 * 7.5) + 16.0;
        self.current_menu_x += btn_width + 2.0;

        // 3. Render floating dropdown popup if currently active directly to root popup layer on this very frame!
        if is_active {
            let popup_y = self.menubar_y + self.menubar_height;
            let root_id = self.scope.tree.root().unwrap_or(self.scope.parent);
            let mut root_scope = self.scope.child_scope(root_id);

            let dropdown_resp = root_scope.dropdown_menu_card(
                popup_x,
                popup_y,
                MENUBAR_DROPDOWN_DEFAULT_WIDTH,
                |card_scope| {
                    let mut menu_scope = MenuScope {
                        scope: card_scope,
                        selected_action: None,
                        closed: false,
                    };
                    add_items(&mut menu_scope);
                    (menu_scope.selected_action, menu_scope.closed)
                },
            );

            let (selected, closed) = dropdown_resp.inner;
            if let Some(act) = selected {
                self.selected_action = Some(act);
            }
            if closed {
                self.new_active_menu_tag = Some(None);
                self.active_menu_tag = None;
            }
        }
    }

    /// Emits a prominent action button on the menu bar (e.g. Play / Stop).
    pub fn action_button(&mut self, label: &str, bg: Color, text: Color, action: A) {
        let mut right_scope = self.scope.child_scope(self.right_container);
        let resp = right_scope.menu_action_button(label, bg, text);
        if resp.clicked() {
            self.selected_action = Some(action);
            self.new_active_menu_tag = Some(None);
        }
    }

    /// Emits right-aligned items inside the dedicated right flex group of the menu bar.
    ///
    /// Items emitted here automatically align to the far right edge of the window via flexbox space-between.
    pub fn right<F>(&mut self, add_right_contents: F)
    where
        F: FnOnce(&mut MenuBarRightScope<'_, '_, A>),
    {
        let right_scope = self.scope.child_scope(self.right_container);
        let mut right_builder = MenuBarRightScope {
            scope: right_scope,
            selected_action: &mut self.selected_action,
            new_active_menu_tag: &mut self.new_active_menu_tag,
        };
        add_right_contents(&mut right_builder);
    }
}

impl<'a> UiScope<'a> {
    /// Emits a top-level application menu bar adhering to desktop standards.
    ///
    /// Autonomous features:
    /// - Manages opened/closed dropdown popup state internally via [`iris_core::UiTree::active_menu_bar_tag`].
    /// - Smooth hover-switching between menus when any dropdown is active.
    /// - Elevation of dropdown items to [`iris_core::UiLayer::Popup`] with proper scissor clipping bypass.
    /// - Returns strongly typed `Option<A>` representing the selected action without leaking `WidgetId`s.
    pub fn menu_bar<A, F>(&mut self, width: f32, height: f32, add_contents: F) -> Option<A>
    where
        F: FnOnce(&mut MenuBarScope<'_, '_, A>),
    {
        let active_tag = self.tree.active_menu_bar_tag;

        let (selected_action, new_active_tag) = self
            .row_with()
            .width(width)
            .height(height)
            .space_between()
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 10.0))
            .background(Color::rgba(0.035, 0.040, 0.050, 1.0))
            .show(|bar| {
                let left_container = bar.tree.create_node();
                if let Some(left) = bar.tree.get_mut(left_container) {
                    left.set_name("MenuBarLeft");
                    left.set_style(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .gap(2.0),
                    );
                }
                let _ = bar.tree.add_child(bar.parent, left_container);

                let right_container = bar.tree.create_node();
                if let Some(right) = bar.tree.get_mut(right_container) {
                    right.set_name("MenuBarRight");
                    right.set_style(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .gap(6.0),
                    );
                }
                let _ = bar.tree.add_child(bar.parent, right_container);

                let mut bar_scope = MenuBarScope {
                    scope: bar,
                    left_container,
                    right_container,
                    selected_action: None,
                    active_menu_tag: active_tag,
                    new_active_menu_tag: None,
                    menubar_y: 0.0,
                    menubar_height: height,
                    current_menu_x: 6.0,
                };

                add_contents(&mut bar_scope);

                (bar_scope.selected_action, bar_scope.new_active_menu_tag)
            })
            .inner;

        // Outside-click dismissal: if a menu is open and a click occurred outside menu bar items/popups
        if active_tag.is_some()
            && self.tree.has_frame_interactions()
            && new_active_tag.is_none()
            && selected_action.is_none()
        {
            if let Some(click_pos) = self
                .tree
                .event_state
                .frame_clicked_pos
                .or(self.tree.event_state.last_pointer_pos)
            {
                if let Some(hit) = self.tree.hit_test_target(click_pos) {
                    if hit.role != WidgetRole::MenuBarItem && hit.role != WidgetRole::DropdownItem {
                        self.tree.close_menu();
                        return None;
                    }
                } else {
                    self.tree.close_menu();
                    return None;
                }
            }
        }

        if let Some(target) = new_active_tag {
            self.tree.set_active_menu(target);
        }

        selected_action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Point, UiTree};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestAction {
        NewFile,
        SaveFile,
        Exit,
    }

    #[test]
    fn test_menu_bar_structure_and_action_dispatch() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut scope = UiScope::new(&mut tree, root);
        let action = scope.menu_bar(1920.0, 28.0, |bar| {
            bar.menu("File", |m| {
                m.item("🗎", "New", Some("Ctrl N"), TestAction::NewFile);
                m.item("🖫", "Save", Some("Ctrl S"), TestAction::SaveFile);
                m.separator();
                m.item("⏻", "Exit", Some("Alt F4"), TestAction::Exit);
            });
            bar.menu("Edit", |m| {
                m.item_disabled("↩", "Undo", Some("Ctrl Z"));
            });
        });

        assert_eq!(action, None, "No action should be triggered without clicks");
        assert_eq!(tree.active_menu_bar_tag, None, "Menus should start closed");
    }

    #[test]
    fn test_menu_bar_click_to_open() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        // Simulate click on "File"
        let file_tag = hash_label_with_seed(0, "File");
        tree.event_state.frame_clicked_tag = Some(file_tag);
        tree.event_state.frame_clicked_pos = Some(Point::new(10.0, 10.0));

        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.menu_bar::<TestAction, _>(1920.0, 28.0, |bar| {
            bar.menu("File", |m| {
                m.item("🗎", "New", None, TestAction::NewFile);
            });
        });

        assert_eq!(
            tree.active_menu_bar_tag,
            Some(file_tag),
            "Clicking menu header must activate it"
        );

        let has_popup = tree
            .iter()
            .any(|(_, n)| n.role == WidgetRole::DropdownPopup);
        assert!(
            has_popup,
            "Dropdown popup card must be rendered immediately on the first click frame!"
        );
    }
}