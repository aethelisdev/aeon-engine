// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Context Menu Widget
//!
//! Provides a standardized, hardware-accelerated right-click popup context menu builder
//! for Iris UI applications and tools.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual icon representation for a context menu item or header.
/// Supports either a Unicode glyph or a GPU texture atlas UV rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextMenuIcon {
    /// Unicode text or emoji glyph (e.g. `"🗑"`, `"👁"`).
    Text(&'static str),
    /// GPU texture atlas UV quad `[u_min, v_min, u_max, v_max]`.
    Texture([f32; 4]),
}

/// Optional header displayed at the very top of the context menu.
/// Useful for identifying the subject under the cursor (e.g. filename, entity name, folder).
#[derive(Debug, Clone)]
pub struct ContextMenuHeader {
    /// Title text displayed in the header bar.
    pub title: String,
    /// Optional icon displayed before the title.
    pub icon: Option<ContextMenuIcon>,
}

/// An individual item within a context menu.
/// Can represent an actionable row, a destructive action (rendered in danger tones), or a visual separator line.
#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    /// Human-readable label text.
    pub label: String,
    /// Numerical tag assigned to `node.tag` for $O(1)$ dispatch.
    pub tag: u64,
    /// Optional leading visual icon.
    pub icon: Option<ContextMenuIcon>,
    /// Optional trailing keyboard shortcut string.
    pub shortcut: Option<String>,
    /// Whether this item represents a destructive action (e.g. delete) styled with danger colors.
    pub is_destructive: bool,
    /// Whether this row renders as a visual separator line.
    pub is_separator: bool,
    /// Whether this item is interactive and selectable.
    pub is_enabled: bool,
}

impl ContextMenuItem {
    /// Constructs an actionable menu item with the specified label and tag.
    #[inline]
    pub fn item(tag: u64, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tag,
            icon: None,
            shortcut: None,
            is_destructive: false,
            is_separator: false,
            is_enabled: true,
        }
    }

    /// Constructs an actionable menu item with an icon, label, and tag.
    #[inline]
    pub fn item_with_icon(tag: u64, icon: ContextMenuIcon, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tag,
            icon: Some(icon),
            shortcut: None,
            is_destructive: false,
            is_separator: false,
            is_enabled: true,
        }
    }

    /// Constructs a destructive actionable menu item (highlighted with danger colors on hover).
    #[inline]
    pub fn destructive(tag: u64, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tag,
            icon: None,
            shortcut: None,
            is_destructive: true,
            is_separator: false,
            is_enabled: true,
        }
    }

    /// Constructs a destructive actionable menu item with an icon.
    #[inline]
    pub fn destructive_with_icon(
        tag: u64,
        icon: ContextMenuIcon,
        label: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            tag,
            icon: Some(icon),
            shortcut: None,
            is_destructive: true,
            is_separator: false,
            is_enabled: true,
        }
    }

    /// Constructs a visual horizontal separator item.
    #[inline]
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            tag: u64::MAX,
            icon: None,
            shortcut: None,
            is_destructive: false,
            is_separator: true,
            is_enabled: false,
        }
    }

    /// Sets an optional keyboard shortcut hint.
    #[inline]
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// Visual style configuration for a context menu popup card.
#[derive(Debug, Clone)]
pub struct ContextMenuStyle {
    /// Background fill color of the menu container card.
    pub bg_color: Color,
    /// 1px border outline color.
    pub border_color: Color,
    /// Corner border radius in logical pixels.
    pub border_radius: f32,
    /// Height of an actionable row in logical pixels.
    pub row_height: f32,
    /// Minimum width of the menu popup card in logical pixels.
    pub min_width: f32,
    /// Text font size for item labels.
    pub font_size: f32,
    /// Background color applied to an actionable item when hovered.
    pub hover_bg: Color,
    /// Text color for regular non-destructive items.
    pub text_color: Color,
    /// Text color for destructive items in default state.
    pub destructive_text: Color,
    /// Background fill color applied to destructive items when hovered.
    pub destructive_hover_bg: Color,
    /// Text color applied to destructive items when hovered.
    pub destructive_hover_text: Color,
    /// Color of visual horizontal separator lines.
    pub separator_color: Color,
}

impl Default for ContextMenuStyle {
    fn default() -> Self {
        Self {
            bg_color: Color::rgba(0.082, 0.090, 0.106, 0.98),
            border_color: Color::rgba(0.173, 0.180, 0.208, 0.90),
            border_radius: 5.0,
            row_height: 24.0,
            min_width: 170.0,
            font_size: 11.5,
            hover_bg: Color::rgba(0.161, 0.188, 0.235, 0.95),
            text_color: Color::rgba(0.85, 0.87, 0.92, 1.0),
            destructive_text: Color::rgba(0.95, 0.40, 0.40, 0.90),
            destructive_hover_bg: Color::rgba(0.40, 0.10, 0.10, 0.90),
            destructive_hover_text: Color::rgba(1.0, 0.50, 0.50, 1.0),
            separator_color: Color::rgba(0.18, 0.20, 0.25, 0.80),
        }
    }
}

/// Standardized right-click context menu builder.
#[derive(Debug, Clone)]
pub struct ContextMenuBuilder {
    click_pos: Point,
    cursor_pos: Point,
    header: Option<ContextMenuHeader>,
    items: Vec<ContextMenuItem>,
    viewport_bounds: Option<Rect>,
    width: Option<f32>,
    style: ContextMenuStyle,
}

impl ContextMenuBuilder {
    /// Initiates a context menu at the designated screen coordinates.
    #[inline]
    pub fn new(click_pos: Point) -> Self {
        Self {
            click_pos,
            cursor_pos: click_pos,
            header: None,
            items: Vec::new(),
            viewport_bounds: None,
            width: None,
            style: ContextMenuStyle::default(),
        }
    }

    /// Sets the current cursor position used for instant hover styling.
    #[inline]
    pub fn cursor_pos(mut self, pos: Point) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Sets the clipping / screen boundaries used to clamp the menu from extending off-screen.
    #[inline]
    pub fn viewport_bounds(mut self, bounds: Rect) -> Self {
        self.viewport_bounds = Some(bounds);
        self
    }

    /// Overrides the fixed menu width in logical pixels.
    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides the visual styling parameters.
    #[inline]
    pub fn style(mut self, style: ContextMenuStyle) -> Self {
        self.style = style;
        self
    }

    /// Adds a descriptive header at the top of the context menu.
    #[inline]
    pub fn header(mut self, title: impl Into<String>, icon: Option<ContextMenuIcon>) -> Self {
        self.header = Some(ContextMenuHeader {
            title: title.into(),
            icon,
        });
        self
    }

    /// Appends an actionable item row.
    #[inline]
    pub fn item(mut self, tag: u64, label: impl Into<String>) -> Self {
        self.items.push(ContextMenuItem::item(tag, label));
        self
    }

    /// Appends an actionable item row with an icon.
    #[inline]
    pub fn item_with_icon(
        mut self,
        tag: u64,
        icon: ContextMenuIcon,
        label: impl Into<String>,
    ) -> Self {
        self.items
            .push(ContextMenuItem::item_with_icon(tag, icon, label));
        self
    }

    /// Appends a destructive actionable item row.
    #[inline]
    pub fn destructive_item(mut self, tag: u64, label: impl Into<String>) -> Self {
        self.items.push(ContextMenuItem::destructive(tag, label));
        self
    }

    /// Appends a destructive actionable item row with an icon.
    #[inline]
    pub fn destructive_item_with_icon(
        mut self,
        tag: u64,
        icon: ContextMenuIcon,
        label: impl Into<String>,
    ) -> Self {
        self.items
            .push(ContextMenuItem::destructive_with_icon(tag, icon, label));
        self
    }

    /// Appends a horizontal separator line.
    #[inline]
    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::separator());
        self
    }

    /// Builds the context menu into the `UiTree` and returns its final bounding rectangle.
    /// All items are tagged with their respective `tag` numerical value on `UiLayer::Popup`.
    pub fn build(&self, tree: &mut UiTree, parent_id: WidgetId) -> Rect {
        let menu_w = self.width.unwrap_or(self.style.min_width);

        let header_h = if self.header.is_some() { 32.0 } else { 0.0 };
        let mut content_h = 8.0 + header_h; // 4.0 top + 4.0 bottom padding + optional header

        for item in &self.items {
            if item.is_separator {
                content_h += 7.0; // 1px line + 6px vertical margin
            } else {
                content_h += self.style.row_height;
            }
        }

        // Viewport bounds clamping
        let vp = self
            .viewport_bounds
            .filter(|r| r.width > 0.0 && r.height > 0.0)
            .or_else(|| {
                tree.root()
                    .and_then(|r| tree.get(r))
                    .map(|n| n.computed_rect)
                    .filter(|r| r.width > 0.0 && r.height > 0.0)
            })
            .unwrap_or(Rect::new(0.0, 0.0, 1920.0, 1080.0));

        let mut menu_x = self.click_pos.x;
        let mut menu_y = self.click_pos.y;

        if menu_x + menu_w > vp.right() - 4.0 {
            menu_x = (vp.right() - menu_w - 4.0).max(vp.x + 4.0);
        } else {
            menu_x = menu_x.max(vp.x + 4.0);
        }

        if menu_y + content_h > vp.bottom() - 4.0 {
            menu_y = (vp.bottom() - content_h - 4.0).max(vp.y + 4.0);
        } else {
            menu_y = menu_y.max(vp.y + 4.0);
        }

        let card_rect = Rect::new(menu_x, menu_y, menu_w, content_h);

        // 1. Popup Card Container
        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ContextMenuCard");
            node.set_role(WidgetRole::DropdownPopup);
            node.set_layer(UiLayer::Popup);
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(self.style.bg_color)
                .border(1.0, self.style.border_color)
                .border_radius(self.style.border_radius)
                .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.75));
        }
        let _ = tree.add_child(parent_id, card_id);

        let mut cur_y = menu_y + 4.0;

        // 2. Optional Header
        if let Some(ref hdr) = self.header {
            let hdr_rect = Rect::new(menu_x + 6.0, cur_y, menu_w - 12.0, 24.0);
            let hdr_id = tree.create_node();
            if let Some(node) = tree.get_mut(hdr_id) {
                node.set_name("ContextMenuHeader");
                node.set_layer(UiLayer::Popup);
                node.computed_rect = hdr_rect;
                node.interactive = false;
            }
            let _ = tree.add_child(card_id, hdr_id);

            let mut text_x = hdr_rect.x;
            if let Some(icon) = hdr.icon {
                let icon_rect = Rect::new(text_x, hdr_rect.y + 3.0, 16.0, 16.0);
                let ic_id = tree.create_node();
                if let Some(node) = tree.get_mut(ic_id) {
                    node.set_name("ContextMenuHeaderIcon");
                    node.set_layer(UiLayer::Popup);
                    node.computed_rect = icon_rect;
                    node.interactive = false;
                    match icon {
                        ContextMenuIcon::Text(glyph) => {
                            node.set_text(glyph);
                            node.font_size = 11.0;
                            node.line_height = 16.0;
                            node.text_align = TextAlign::Center;
                        }
                        ContextMenuIcon::Texture(uv) => {
                            node.texture_uv = Some(uv);
                        }
                    }
                }
                let _ = tree.add_child(hdr_id, ic_id);
                text_x += 22.0;
            }

            let text_rect = Rect::new(text_x, hdr_rect.y, hdr_rect.right() - text_x, 24.0);
            let txt_id = tree.create_node();
            if let Some(node) = tree.get_mut(txt_id) {
                node.set_name("ContextMenuHeaderTitle");
                node.set_layer(UiLayer::Popup);
                node.set_text(&hdr.title);
                node.font_size = 11.5;
                node.line_height = 24.0;
                node.text_color = Color::rgba(0.70, 0.74, 0.82, 1.0);
                node.text_align = TextAlign::Left;
                node.computed_rect = text_rect;
                node.interactive = false;
            }
            let _ = tree.add_child(hdr_id, txt_id);

            cur_y += 24.0 + 3.0;

            // Separator under header
            let sep_rect = Rect::new(menu_x + 6.0, cur_y, menu_w - 12.0, 1.0);
            let sep_id = tree.create_node();
            if let Some(node) = tree.get_mut(sep_id) {
                node.set_name("ContextMenuHeaderSeparator");
                node.set_layer(UiLayer::Popup);
                node.computed_rect = sep_rect;
                node.interactive = false;
                node.style = Style::new().background(self.style.separator_color);
            }
            let _ = tree.add_child(card_id, sep_id);

            cur_y += 4.0;
        }

        // 3. Actionable Items & Separators
        for item in &self.items {
            if item.is_separator {
                cur_y += 3.0;
                let sep_rect = Rect::new(menu_x + 6.0, cur_y, menu_w - 12.0, 1.0);
                let sep_id = tree.create_node();
                if let Some(node) = tree.get_mut(sep_id) {
                    node.set_name("ContextMenuSeparator");
                    node.set_layer(UiLayer::Popup);
                    node.computed_rect = sep_rect;
                    node.interactive = false;
                    node.style = Style::new().background(self.style.separator_color);
                }
                let _ = tree.add_child(card_id, sep_id);
                cur_y += 4.0;
                continue;
            }

            let item_rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, self.style.row_height);
            let is_hovered = item.is_enabled && item_rect.contains_point(self.cursor_pos);

            let (bg, text_col) = if is_hovered {
                if item.is_destructive {
                    (
                        self.style.destructive_hover_bg,
                        self.style.destructive_hover_text,
                    )
                } else {
                    (self.style.hover_bg, Color::WHITE)
                }
            } else if item.is_destructive {
                (Color::TRANSPARENT, self.style.destructive_text)
            } else if !item.is_enabled {
                (Color::TRANSPARENT, Color::rgba(0.50, 0.52, 0.58, 0.70))
            } else {
                (Color::TRANSPARENT, self.style.text_color)
            };

            let item_id = tree.create_node();
            if let Some(node) = tree.get_mut(item_id) {
                node.set_name("ContextMenuItem");
                node.set_role(WidgetRole::DropdownItem);
                node.set_layer(UiLayer::Popup);
                node.set_tag(item.tag);
                node.computed_rect = item_rect;
                node.style = Style::new().background(bg).border_radius(3.0);
            }
            let _ = tree.add_child(card_id, item_id);

            let mut text_x = item_rect.x + 6.0;

            if let Some(icon) = item.icon {
                let icon_rect = Rect::new(text_x, item_rect.y, 16.0, item_rect.height);
                let ic_id = tree.create_node();
                if let Some(node) = tree.get_mut(ic_id) {
                    node.set_name("ContextMenuIcon");
                    node.set_layer(UiLayer::Popup);
                    node.set_tag(item.tag);
                    node.computed_rect = icon_rect;
                    node.interactive = false;
                    match icon {
                        ContextMenuIcon::Text(glyph) => {
                            node.set_text(glyph);
                            node.font_size = 11.0;
                            node.line_height = item_rect.height;
                            node.text_align = TextAlign::Center;
                            node.text_color = text_col;
                        }
                        ContextMenuIcon::Texture(uv) => {
                            node.texture_uv = Some(uv);
                        }
                    }
                }
                let _ = tree.add_child(item_id, ic_id);
                text_x += 22.0;
            }

            let text_rect = Rect::new(
                text_x,
                item_rect.y,
                item_rect.right() - text_x - 6.0,
                item_rect.height,
            );
            let lbl_id = tree.create_node();
            if let Some(node) = tree.get_mut(lbl_id) {
                node.set_name("ContextMenuLabel");
                node.set_layer(UiLayer::Popup);
                node.set_tag(item.tag);
                node.set_text(&item.label);
                node.font_size = self.style.font_size;
                node.line_height = item_rect.height;
                node.text_color = text_col;
                node.text_align = TextAlign::Left;
                node.computed_rect = text_rect;
                node.interactive = false;
            }
            let _ = tree.add_child(item_id, lbl_id);

            if let Some(ref sc) = item.shortcut {
                let sc_rect = Rect::new(
                    item_rect.right() - 60.0,
                    item_rect.y,
                    54.0,
                    item_rect.height,
                );
                let sc_id = tree.create_node();
                if let Some(node) = tree.get_mut(sc_id) {
                    node.set_name("ContextMenuShortcut");
                    node.set_layer(UiLayer::Popup);
                    node.set_tag(item.tag);
                    node.set_text(sc);
                    node.font_size = 10.0;
                    node.line_height = item_rect.height;
                    node.text_color = Color::rgba(0.55, 0.58, 0.65, 0.85);
                    node.text_align = TextAlign::Right;
                    node.computed_rect = sc_rect;
                    node.interactive = false;
                }
                let _ = tree.add_child(item_id, sc_id);
            }

            cur_y += self.style.row_height;
        }

        card_rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_build_and_tagging() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        }
        let _ = tree.set_root(root);

        let menu = ContextMenuBuilder::new(Point::new(100.0, 100.0))
            .item(10, "First Item")
            .destructive_item(20, "Delete Item")
            .separator()
            .item_with_icon(30, ContextMenuIcon::Text("👁"), "Visible Item");

        let rect = menu.build(&mut tree, root);
        assert_eq!(rect.x, 100.0);
        assert_eq!(rect.y, 100.0);
        assert!(rect.width >= 170.0);

        // Hit test tagged items
        let hit_10 = tree.hit_test_target(Point::new(120.0, 110.0));
        assert!(hit_10.is_some());
        let info = hit_10.unwrap();
        assert_eq!(info.tag, 10);
        assert_eq!(info.role, WidgetRole::DropdownItem);
        assert_eq!(info.layer, UiLayer::Popup);

        let hit_20 = tree.hit_test_target(Point::new(120.0, 134.0));
        assert!(hit_20.is_some());
        assert_eq!(hit_20.unwrap().tag, 20);

        let hit_30 = tree.hit_test_target(Point::new(120.0, 164.0));
        assert!(hit_30.is_some());
        assert_eq!(hit_30.unwrap().tag, 30);
    }

    #[test]
    fn test_context_menu_viewport_clamping() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        let menu = ContextMenuBuilder::new(Point::new(1900.0, 1070.0))
            .viewport_bounds(Rect::new(0.0, 0.0, 1920.0, 1080.0))
            .item(1, "Clamped Item");

        let rect = menu.build(&mut tree, root);
        assert!(rect.right() <= 1920.0);
        assert!(rect.bottom() <= 1080.0);
    }
}