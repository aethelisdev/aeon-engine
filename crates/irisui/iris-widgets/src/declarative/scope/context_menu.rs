// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Context Menu & Floating Popup Primitives
//!
//! Provides hardware-accelerated declarative context menu popups on [`UiScope`],
//! [`WidgetResponse`], and [`TreeRowResponse`].
//!
//! Manages autonomous menu open/dismiss lifecycles, pointer position anchoring,
//! viewport boundary clamping, and Z-order elevation to [`UiLayer::Popup`].
//!

use super::core::UiScope;
use crate::declarative::scope::tree_view::TreeRowResponse;
use crate::declarative::types::WidgetResponse;
use iris_core::dispatch::PopupMenuState;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Point, Rect, Style, TextAlign, UiLayer,
    WidgetCursor, WidgetId, WidgetRole,
};

/// Standard width constraint for autonomous floating context menu cards in logical pixels.
pub const CONTEXT_MENU_DEFAULT_WIDTH: f32 = 160.0;

/// Standard row height for context menu item buttons in logical pixels.
pub const CONTEXT_MENU_ITEM_HEIGHT: f32 = 24.0;

impl<'a> UiScope<'a> {
    /// Opens or maintains an autonomous context menu popup anchored to a specific widget node.
    ///
    /// When `is_secondary_clicked` is `true`, activates the context menu for `target_id`
    /// at `pointer_pos`. While the menu is active, renders an elevated floating container on
    /// [`UiLayer::Popup`], automatically clamps its boundaries to the viewport window, and
    /// runs `add_contents` to emit child action items.
    ///
    /// # Arguments
    /// * `target_id` - Unique widget node identifier owning this context menu.
    /// * `is_secondary_clicked` - Whether a secondary (right) click was registered on the target this frame.
    /// * `pointer_pos` - Screen coordinates of the triggering click, if captured.
    /// * `add_contents` - Child builder closure receiving the scoped context of the menu container.
    pub fn context_menu_for<F>(
        &mut self,
        target_id: WidgetId,
        is_secondary_clicked: bool,
        pointer_pos: Option<Point>,
        add_contents: F,
    ) where
        F: FnOnce(&mut UiScope<'_>),
    {
        let target_tag = self.tree.get(target_id).map_or(0, |n| n.tag);

        // 1. Activation trigger on secondary click
        if is_secondary_clicked {
            let pos = pointer_pos
                .or(self.tree.event_state.frame_clicked_pos)
                .or(self.tree.event_state.last_pointer_pos)
                .unwrap_or(Point::ZERO);

            self.tree.event_state.active_context_menu = Some(PopupMenuState {
                owner_id: target_id,
                owner_tag: target_tag,
                anchor_pos: pos,
                container_id: None,
            });
        }

        // 2. Active render pass if this target currently owns the open menu
        let active_anchor = self.tree.event_state.active_context_menu.and_then(|menu| {
            if menu.owner_id == target_id || (target_tag != 0 && menu.owner_tag == target_tag) {
                Some(menu.anchor_pos)
            } else {
                None
            }
        });

        if let Some(anchor) = active_anchor {
            self.render_context_menu_popup(anchor, add_contents);
        }
    }

    /// Emits a floating context menu popup card container at the given screen anchor.
    fn render_context_menu_popup<F>(&mut self, anchor_pos: Point, add_contents: F)
    where
        F: FnOnce(&mut UiScope<'_>),
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

        let width = CONTEXT_MENU_DEFAULT_WIDTH;
        let mut x = anchor_pos.x;
        let mut y = anchor_pos.y;

        // Viewport horizontal clamping
        if x + width > window_w - 6.0 {
            x = (x - width).max(6.0);
        }

        let container_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(container_id) {
            node.set_name("ContextMenuPopup");
            node.set_role(WidgetRole::DropdownPopup);
            node.set_layer(UiLayer::Popup);
            node.interactive = true;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .flex_col()
                    .width(width)
                    .padding_insets(Insets::new(4.0, 4.0, 4.0, 4.0))
                    .background(Color::rgba(0.04, 0.045, 0.06, 0.98))
                    .border(1.0, Color::rgba(0.22, 0.25, 0.34, 0.90))
                    .border_radius(5.0)
                    .box_shadow(0.0, 6.0, 20.0, Color::rgba(0.0, 0.0, 0.0, 0.75)),
            );
            node.computed_rect.x = x;
            node.computed_rect.y = y;
            node.computed_rect.width = width;
        }
        let popup_parent = self.tree.root().unwrap_or(self.parent);
        let _ = self.tree.add_child(popup_parent, container_id);

        // Store container ID on active state for outside-click dismiss hit testing
        if let Some(active) = self.tree.event_state.active_context_menu.as_mut() {
            active.container_id = Some(container_id);
        }

        // Emit child action items within this elevated scope
        let mut child_scope = self.child_scope(container_id);
        add_contents(&mut child_scope);

        // Measure actual content height and apply vertical viewport clamping
        let height = crate::declarative::layout::measure_height(self.tree, container_id);
        if y + height > window_h - 6.0 {
            y = (y - height).max(6.0);
        }

        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, container_id, bounds);
        if let Some(node) = self.tree.get_mut(container_id) {
            node.computed_rect = bounds;
            node.style.inset_left = Some(x);
            node.style.inset_top = Some(y);
            node.style.height = Some(height);
        }
    }

    /// Emits an individual interactive action item row inside a context menu.
    ///
    /// When clicked, automatically closes the parent context menu autonomously
    /// so callers do not need to manually manage dismiss states.
    ///
    /// # Arguments
    /// * `label` - Text displayed on the menu row.
    pub fn context_menu_item(&mut self, label: impl Into<String>) -> WidgetResponse {
        self.context_menu_item_with_icon("", label, None)
    }

    /// Emits an individual interactive action item row with leading icon and trailing shortcut.
    ///
    /// # Arguments
    /// * `icon` - Optional leading icon text or glyph.
    /// * `label` - Primary action label.
    /// * `shortcut` - Optional trailing shortcut hint (e.g. "Ctrl+D", "Del").
    pub fn context_menu_item_with_icon(
        &mut self,
        icon: &str,
        label: impl Into<String>,
        shortcut: Option<&str>,
    ) -> WidgetResponse {
        let label_str = label.into();
        let tag = self.tag_for(&label_str);
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ContextMenuItem");
            node.set_role(WidgetRole::Button);
            node.set_layer(UiLayer::Popup);
            node.tag = tag;
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let (clicked, hovered, _) = self.check_interaction(node_id);

        // Autonomous close upon item activation
        if clicked {
            self.tree.event_state.close_context_menu();
        }

        let text_color = if hovered {
            Color::WHITE
        } else {
            Color::rgba(0.85, 0.88, 0.94, 1.0)
        };

        if let Some(node) = self.tree.get_mut(node_id) {
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .height(CONTEXT_MENU_ITEM_HEIGHT)
                .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
                .background(Color::TRANSPARENT)
                .border_radius(3.0);
            if hovered {
                style = style.background(Color::rgba(0.0, 0.55, 0.85, 0.25));
            }
            node.set_style(style);
        }

        // Optional Icon Column
        if !icon.is_empty() {
            let icon_id = self.tree.create_node();
            if let Some(n) = self.tree.get_mut(icon_id) {
                n.set_name("ContextMenuIcon");
                n.set_layer(UiLayer::Popup);
                n.interactive = false;
                n.set_text(icon);
                n.font_size = 11.5;
                n.line_height = 13.0;
                n.text_align = TextAlign::Center;
                n.text_color = text_color;
                n.set_style(Style::new().width(16.0).height(13.0).flex_shrink(0.0));
            }
            let _ = self.tree.add_child(node_id, icon_id);
        }

        // Label Column
        let label_id = self.tree.create_node();
        if let Some(n) = self.tree.get_mut(label_id) {
            n.set_name("ContextMenuLabel");
            n.set_layer(UiLayer::Popup);
            n.interactive = false;
            n.set_text(label_str);
            n.font_size = 11.5;
            n.line_height = 13.0;
            n.text_align = TextAlign::Left;
            n.text_color = text_color;
            n.set_style(Style::new().flex_grow(1.0).height(13.0));
        }
        let _ = self.tree.add_child(node_id, label_id);

        // Optional Trailing Shortcut
        if let Some(sc) = shortcut {
            let sc_id = self.tree.create_node();
            if let Some(n) = self.tree.get_mut(sc_id) {
                n.set_name("ContextMenuShortcut");
                n.set_layer(UiLayer::Popup);
                n.interactive = false;
                n.set_text(sc);
                n.font_size = 10.5;
                n.line_height = 13.0;
                n.text_align = TextAlign::Right;
                n.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
                n.set_style(Style::new().width(48.0).height(13.0).flex_shrink(0.0));
            }
            let _ = self.tree.add_child(node_id, sc_id);
        }

        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a subtle horizontal separator line dividing context menu action groups.
    pub fn context_menu_separator(&mut self) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name("ContextMenuSeparator");
            node.set_role(WidgetRole::Separator);
            node.set_layer(UiLayer::Popup);
            node.interactive = false;
            node.set_style(
                Style::new()
                    .height(5.0)
                    .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
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

/// Extension trait adding autonomous context menu capabilities to interaction responses.
pub trait ContextMenuExt {
    /// Opens or maintains an autonomous context menu popup anchored to this response.
    ///
    /// When secondary (right) clicked, activates the menu at the captured click position.
    /// While open, renders the elevated popup card on [`UiLayer::Popup`] and executes `add_contents`.
    /// Automatically closes when an action item is clicked or when clicking outside.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    fn context_menu<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>);
}

impl ContextMenuExt for WidgetResponse {
    #[inline]
    fn context_menu<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        scope.context_menu_for(
            self.id,
            self.secondary_clicked(),
            self.pointer_pos,
            add_contents,
        );
        self
    }
}

impl ContextMenuExt for TreeRowResponse {
    #[inline]
    fn context_menu<F>(self, scope: &mut UiScope<'_>, add_contents: F) -> Self
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        scope.context_menu_for(
            self.id,
            self.secondary_clicked(),
            self.pointer_pos,
            add_contents,
        );
        self
    }
}