// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading Multi-Level Dropdown Menu Widget
//!
//! Provides a standardized, hardware-accelerated cascading and nested popup menu builder
//! for Iris UI applications and tools.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual icon representation for a menu item.
/// Can either be a Unicode text/emoji glyph or a normalized texture atlas sub-rectangle `[u_min, v_min, u_max, v_max]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CascadingMenuIcon {
    /// Unicode text or emoji glyph (e.g. `"➕"`, `"📁"`).
    Text(&'static str),
    /// GPU texture atlas UV quad `[u_min, v_min, u_max, v_max]`.
    Texture([f32; 4]),
}

/// A single item within a cascading menu tree.
/// Can represent an actionable leaf item, a branching submenu parent, or a visual separator.
#[derive(Debug, Clone)]
pub struct CascadingMenuItem {
    /// Human-readable label displayed on the menu row.
    pub label: String,
    /// Optional visual icon displayed to the left of the label.
    pub icon: Option<CascadingMenuIcon>,
    /// Optional keyboard shortcut text displayed right-aligned (e.g. `"Ctrl+N"`).
    pub shortcut: Option<String>,
    /// Unique identifier / action payload tag assigned to this item (`node.tag`).
    pub tag: u64,
    /// Optional nested submenu items branching off this row.
    pub submenu: Option<Vec<CascadingMenuItem>>,
    /// Whether this item is interactive and selectable.
    pub is_enabled: bool,
    /// Whether this item renders as a horizontal separator line rather than an interactive row.
    pub is_separator: bool,
}

impl CascadingMenuItem {
    /// Constructs an actionable leaf menu item.
    #[inline]
    pub fn item(label: impl Into<String>, tag: u64) -> Self {
        Self {
            label: label.into(),
            icon: None,
            shortcut: None,
            tag,
            submenu: None,
            is_enabled: true,
            is_separator: false,
        }
    }

    /// Constructs an actionable leaf menu item with an icon.
    #[inline]
    pub fn item_with_icon(label: impl Into<String>, icon: CascadingMenuIcon, tag: u64) -> Self {
        Self {
            label: label.into(),
            icon: Some(icon),
            shortcut: None,
            tag,
            submenu: None,
            is_enabled: true,
            is_separator: false,
        }
    }

    /// Constructs a branching submenu category item with an icon and child items.
    #[inline]
    pub fn branch(
        label: impl Into<String>,
        icon: Option<CascadingMenuIcon>,
        tag: u64,
        submenu: Vec<CascadingMenuItem>,
    ) -> Self {
        Self {
            label: label.into(),
            icon,
            shortcut: None,
            tag,
            submenu: Some(submenu),
            is_enabled: true,
            is_separator: false,
        }
    }

    /// Constructs a visual horizontal separator line.
    #[inline]
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            icon: None,
            shortcut: None,
            tag: 0,
            submenu: None,
            is_enabled: false,
            is_separator: true,
        }
    }

    /// Attaches a right-aligned keyboard shortcut string to this item.
    #[inline]
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Sets the interactive enabled state of this item.
    #[inline]
    pub fn enabled(mut self, is_enabled: bool) -> Self {
        self.is_enabled = is_enabled;
        self
    }

    /// Returns `true` if this item branches into a nested child submenu.
    #[inline]
    pub fn has_submenu(&self) -> bool {
        self.submenu.as_ref().is_some_and(|sub| !sub.is_empty())
    }
}

/// Visual styling configuration for cascading popup menus.
#[derive(Debug, Clone)]
pub struct CascadingMenuStyle {
    /// Background color of the popup menu container.
    pub background: Color,
    /// Border thickness in pixels.
    pub border_width: f32,
    /// Border stroke color.
    pub border_color: Color,
    /// Corner radius in pixels.
    pub border_radius: f32,
    /// Drop shadow offset Y in pixels.
    pub shadow_y: f32,
    /// Drop shadow blur radius in pixels.
    pub shadow_blur: f32,
    /// Drop shadow color.
    pub shadow_color: Color,
    /// Background color of an idle, unhovered item.
    pub item_idle_bg: Color,
    /// Background color of a hovered item under the pointer.
    pub item_hover_bg: Color,
    /// Background color of an item whose submenu branch is currently active.
    pub item_branch_active_bg: Color,
    /// Text color of an idle, enabled item.
    pub text_idle_color: Color,
    /// Text color of a hovered item.
    pub text_hover_color: Color,
    /// Text color of a disabled item.
    pub text_disabled_color: Color,
    /// Text color for keyboard shortcuts.
    pub shortcut_color: Color,
    /// Line color for horizontal separators.
    pub separator_color: Color,
    /// Color for submenu chevron indicator (`▸`).
    pub arrow_color: Color,
    /// Font size in pixels for item labels.
    pub font_size: f32,
    /// Height of each individual item row in pixels.
    pub row_height: f32,
    /// Height of a horizontal separator line in pixels.
    pub separator_height: f32,
    /// Horizontal padding in pixels between container edge and row contents.
    pub item_padding_x: f32,
    /// Default width in pixels for popup cards.
    pub menu_width: f32,
}

impl Default for CascadingMenuStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.086, 0.090, 0.106, 0.98),
            border_width: 1.0,
            border_color: Color::rgba(0.173, 0.180, 0.208, 0.90),
            border_radius: 5.0,
            shadow_y: 6.0,
            shadow_blur: 18.0,
            shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.70),
            item_idle_bg: Color::TRANSPARENT,
            item_hover_bg: Color::rgba(0.18, 0.20, 0.26, 0.98),
            item_branch_active_bg: Color::rgba(0.157, 0.165, 0.188, 0.98),
            text_idle_color: Color::rgba(0.886, 0.894, 0.918, 1.0),
            text_hover_color: Color::WHITE,
            text_disabled_color: Color::rgba(0.45, 0.47, 0.52, 1.0),
            shortcut_color: Color::rgba(0.55, 0.58, 0.65, 1.0),
            separator_color: Color::rgba(0.18, 0.19, 0.22, 1.0),
            arrow_color: Color::rgba(0.60, 0.63, 0.70, 1.0),
            font_size: 11.5,
            row_height: 22.0,
            separator_height: 6.0,
            item_padding_x: 6.0,
            menu_width: 185.0,
        }
    }
}

/// Output layout and node frame resulting from building a cascading menu into a [`UiTree`].
#[derive(Debug, Clone)]
pub struct CascadingMenuFrame {
    /// Generational node identifier for the root popup container.
    pub root_popup_id: WidgetId,
    /// Absolute bounding rectangle of the root popup card.
    pub root_rect: Rect,
    /// Bounding rectangles of all currently rendered popup cards (root + active submenus).
    pub rendered_popup_rects: Vec<Rect>,
    /// Pairs of `(tag, widget_id)` for every rendered interactive item row.
    pub item_ids: Vec<(u64, WidgetId)>,
}

/// Internal rendering context grouping recursive parameters to ensure clean function signatures.
struct CascadingRenderContext<'a, 'b> {
    tree: &'a mut UiTree,
    rendered_rects: &'b mut Vec<Rect>,
    item_nodes: &'b mut Vec<(u64, WidgetId)>,
}

/// Fluent builder for constructing multi-level cascading popup menus in [`UiTree`].
/// Supports unlimited cascading depth (Level 1 -> Level 2 -> Level 3...), automatic screen boundary
/// flipping / clamping, and zero-allocation $O(1)$ hit-test event routing via [`WidgetRole::DropdownItem`].
#[derive(Debug, Clone)]
pub struct CascadingMenuBuilder<'a> {
    anchor_rect: Rect,
    items: &'a [CascadingMenuItem],
    active_path: &'a [u64],
    cursor_pos: Point,
    viewport_bounds: Option<Rect>,
    style: CascadingMenuStyle,
    menu_name: &'static str,
    open_upward: bool,
    custom_width: Option<f32>,
}

impl<'a> CascadingMenuBuilder<'a> {
    /// Creates a new cascading menu builder anchored to a trigger rectangle.
    #[inline]
    pub fn new(anchor_rect: Rect, items: &'a [CascadingMenuItem], active_path: &'a [u64]) -> Self {
        Self {
            anchor_rect,
            items,
            active_path,
            cursor_pos: Point::new(-1.0, -1.0),
            viewport_bounds: None,
            style: CascadingMenuStyle::default(),
            menu_name: "CascadingMenuPopup",
            open_upward: false,
            custom_width: None,
        }
    }

    /// Sets the pointer cursor position for computing hover states.
    #[inline]
    pub fn cursor_pos(mut self, cursor_pos: Point) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Sets the screen viewport bounds used for boundary clamping and directional flipping.
    #[inline]
    pub fn viewport_bounds(mut self, viewport_bounds: Rect) -> Self {
        self.viewport_bounds = Some(viewport_bounds);
        self
    }

    /// Overrides the visual styling theme.
    #[inline]
    pub fn style(mut self, style: CascadingMenuStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets an explicit width for the popup cards.
    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.custom_width = Some(width);
        self
    }

    /// Configures the root menu to expand upwards rather than downwards.
    #[inline]
    pub fn open_upward(mut self, open_upward: bool) -> Self {
        self.open_upward = open_upward;
        self
    }

    /// Sets a debug name for the root popup node.
    #[inline]
    pub fn name(mut self, name: &'static str) -> Self {
        self.menu_name = name;
        self
    }

    /// Builds the cascading menu hierarchy into the specified [`UiTree`].
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> Option<CascadingMenuFrame> {
        if self.items.is_empty() {
            return None;
        }

        let menu_w = self.custom_width.unwrap_or(self.style.menu_width);
        let root_h = self.calculate_menu_height(self.items);

        // Compute Root Menu Position
        let mut root_x = self.anchor_rect.x;
        let mut root_y = if self.open_upward {
            self.anchor_rect.y - root_h - 2.0
        } else {
            self.anchor_rect.bottom() + 2.0
        };

        // Clamp root position inside viewport if provided
        if let Some(vp) = self.viewport_bounds {
            if root_x + menu_w > vp.right() {
                root_x = (vp.right() - menu_w - 4.0).max(vp.x + 4.0);
            }
            if root_y + root_h > vp.bottom() {
                root_y = (vp.bottom() - root_h - 4.0).max(vp.y + 4.0);
            }
            if root_y < vp.y {
                root_y = vp.y + 4.0;
            }
        }

        let root_rect = Rect::new(root_x, root_y, menu_w, root_h);
        let mut rendered_popup_rects = Vec::new();
        let mut item_nodes = Vec::new();

        let root_popup_id = self.render_menu_card(tree, parent_id, self.menu_name, root_rect);
        rendered_popup_rects.push(root_rect);

        // Render recursive levels according to active_path
        let mut ctx = CascadingRenderContext {
            tree,
            rendered_rects: &mut rendered_popup_rects,
            item_nodes: &mut item_nodes,
        };

        self.render_items_recursive(&mut ctx, root_popup_id, self.items, root_rect, 0);

        Some(CascadingMenuFrame {
            root_popup_id,
            root_rect,
            rendered_popup_rects,
            item_ids: item_nodes,
        })
    }

    fn calculate_menu_height(&self, items: &[CascadingMenuItem]) -> f32 {
        let mut total_h = 8.0; // Top and bottom padding
        for item in items {
            if item.is_separator {
                total_h += self.style.separator_height;
            } else {
                total_h += self.style.row_height;
            }
        }
        total_h
    }

    fn render_menu_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        name: &str,
        rect: Rect,
    ) -> WidgetId {
        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name(name);
            node.set_role(WidgetRole::DropdownPopup);
            node.layer = UiLayer::Popup;
            node.computed_rect = rect;
            node.style = Style::new()
                .background(self.style.background)
                .border(self.style.border_width, self.style.border_color)
                .border_radius(self.style.border_radius)
                .box_shadow(
                    0.0,
                    self.style.shadow_y,
                    self.style.shadow_blur,
                    self.style.shadow_color,
                );
        }
        let _ = tree.add_child(parent_id, card_id);
        card_id
    }

    fn render_items_recursive(
        &self,
        ctx: &mut CascadingRenderContext<'_, '_>,
        card_id: WidgetId,
        items: &[CascadingMenuItem],
        card_rect: Rect,
        level_index: usize,
    ) {
        let menu_w = card_rect.width;
        let mut cur_y = card_rect.y + 4.0;
        let target_active_tag = self.active_path.get(level_index).copied();

        for item in items {
            if item.is_separator {
                let sep_rect = Rect::new(card_rect.x + 6.0, cur_y + 2.0, menu_w - 12.0, 1.0);
                let sep_id = ctx.tree.create_node();
                if let Some(node) = ctx.tree.get_mut(sep_id) {
                    node.set_name("MenuSeparator");
                    node.layer = UiLayer::Popup;
                    node.computed_rect = sep_rect;
                    node.style = Style::new().background(self.style.separator_color);
                }
                let _ = ctx.tree.add_child(card_id, sep_id);
                cur_y += self.style.separator_height;
                continue;
            }

            let item_rect = Rect::new(
                card_rect.x + self.style.item_padding_x,
                cur_y,
                menu_w - (self.style.item_padding_x * 2.0),
                self.style.row_height,
            );

            let is_hovered = item.is_enabled && item_rect.contains_point(self.cursor_pos);
            let is_branch_active = target_active_tag == Some(item.tag);

            let bg = if !item.is_enabled {
                self.style.item_idle_bg
            } else if is_branch_active {
                self.style.item_branch_active_bg
            } else if is_hovered {
                self.style.item_hover_bg
            } else {
                self.style.item_idle_bg
            };

            let text_color = if !item.is_enabled {
                self.style.text_disabled_color
            } else if is_hovered || is_branch_active {
                self.style.text_hover_color
            } else {
                self.style.text_idle_color
            };

            let row_id = ctx.tree.create_node();
            if let Some(node) = ctx.tree.get_mut(row_id) {
                node.set_name(format!("MenuItem_{}", item.label));
                node.set_role(WidgetRole::DropdownItem);
                node.layer = UiLayer::Popup;
                node.tag = item.tag;
                node.computed_rect = item_rect;
                node.style = Style::new().background(bg).border_radius(3.0);
            }
            let _ = ctx.tree.add_child(card_id, row_id);
            ctx.item_nodes.push((item.tag, row_id));

            // 1. Icon (if present)
            let mut text_x = item_rect.x + 6.0;
            if let Some(icon) = item.icon {
                let ic_rect = Rect::new(
                    text_x,
                    cur_y + (self.style.row_height - 14.0) * 0.5,
                    14.0,
                    14.0,
                );
                let ic_id = ctx.tree.create_node();
                if let Some(node) = ctx.tree.get_mut(ic_id) {
                    node.set_layer(UiLayer::Popup);
                    node.set_role(WidgetRole::DropdownIcon);
                    node.set_tag(item.tag);
                    node.interactive = false;
                    node.computed_rect = ic_rect;
                    match icon {
                        CascadingMenuIcon::Text(glyph) => {
                            node.set_name("MenuIconText");
                            node.set_text(glyph);
                            node.font_size = 11.0;
                            node.line_height = self.style.row_height;
                            node.text_align = TextAlign::Center;
                            node.text_color = text_color;
                        }
                        CascadingMenuIcon::Texture(uv) => {
                            node.set_name("MenuIconTexture");
                            node.texture_uv = Some(uv);
                            node.texture_tint = Some(Color::WHITE);
                        }
                    }
                }
                let _ = ctx.tree.add_child(row_id, ic_id);
                text_x += 20.0;
            }

            // 2. Text Label
            let label_w = item_rect.right()
                - text_x
                - if item.has_submenu() || item.shortcut.is_some() {
                    36.0
                } else {
                    4.0
                };
            let lbl_rect = Rect::new(text_x, cur_y, label_w.max(10.0), self.style.row_height);
            let lbl_id = ctx.tree.create_node();
            if let Some(node) = ctx.tree.get_mut(lbl_id) {
                node.set_name("MenuItemLabel");
                node.set_layer(UiLayer::Popup);
                node.set_role(WidgetRole::DropdownLabel);
                node.set_tag(item.tag);
                node.interactive = false;
                node.computed_rect = lbl_rect;
                node.set_text(&item.label);
                node.font_size = self.style.font_size;
                node.line_height = self.style.row_height;
                node.text_color = text_color;
            }
            let _ = ctx.tree.add_child(row_id, lbl_id);

            // 3. Right side: Shortcut or Submenu Chevron
            if item.has_submenu() {
                let arrow_rect =
                    Rect::new(item_rect.right() - 16.0, cur_y, 12.0, self.style.row_height);
                let arrow_id = ctx.tree.create_node();
                if let Some(node) = ctx.tree.get_mut(arrow_id) {
                    node.set_name("MenuSubmenuChevron");
                    node.set_layer(UiLayer::Popup);
                    node.set_role(WidgetRole::DropdownShortcut);
                    node.set_tag(item.tag);
                    node.interactive = false;
                    node.computed_rect = arrow_rect;
                    node.set_text("▸");
                    node.font_size = 10.0;
                    node.line_height = self.style.row_height;
                    node.text_align = TextAlign::Right;
                    node.text_color = if is_hovered || is_branch_active {
                        self.style.text_hover_color
                    } else {
                        self.style.arrow_color
                    };
                }
                let _ = ctx.tree.add_child(row_id, arrow_id);
            } else if let Some(shortcut) = &item.shortcut {
                let sc_rect =
                    Rect::new(item_rect.right() - 55.0, cur_y, 50.0, self.style.row_height);
                let sc_id = ctx.tree.create_node();
                if let Some(node) = ctx.tree.get_mut(sc_id) {
                    node.set_name("MenuItemShortcut");
                    node.set_layer(UiLayer::Popup);
                    node.set_role(WidgetRole::DropdownShortcut);
                    node.set_tag(item.tag);
                    node.interactive = false;
                    node.computed_rect = sc_rect;
                    node.set_text(shortcut);
                    node.font_size = 10.0;
                    node.line_height = self.style.row_height;
                    node.text_align = TextAlign::Right;
                    node.text_color = self.style.shortcut_color;
                }
                let _ = ctx.tree.add_child(row_id, sc_id);
            }

            // 4. If this item has an active submenu branch, render the nested card
            if is_branch_active && let Some(sub_items) = &item.submenu {
                let sub_w = self.custom_width.unwrap_or(self.style.menu_width);
                let sub_h = self.calculate_menu_height(sub_items);

                // Placement: Open to the right by default, or flip to the left if overflowing viewport
                let mut sub_x = card_rect.right() + 2.0;
                let mut sub_y = item_rect.y - 4.0;

                if let Some(vp) = self.viewport_bounds {
                    if sub_x + sub_w > vp.right() {
                        sub_x = card_rect.x - sub_w - 2.0;
                    }
                    if sub_y + sub_h > vp.bottom() {
                        sub_y = (vp.bottom() - sub_h - 4.0).max(vp.y + 4.0);
                    }
                    if sub_y < vp.y {
                        sub_y = vp.y + 4.0;
                    }
                }

                let sub_rect = Rect::new(sub_x, sub_y, sub_w, sub_h);
                let sub_card_id = self.render_menu_card(
                    ctx.tree,
                    card_id,
                    &format!("{}_Submenu", item.label),
                    sub_rect,
                );
                ctx.rendered_rects.push(sub_rect);

                self.render_items_recursive(ctx, sub_card_id, sub_items, sub_rect, level_index + 1);
            }

            cur_y += self.style.row_height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascading_menu_build_and_multilevel_rendering() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        }

        let items = vec![
            CascadingMenuItem::branch(
                "3D Objects",
                None,
                100,
                vec![
                    CascadingMenuItem::item("Cube", 101),
                    CascadingMenuItem::item("Sphere", 102),
                    CascadingMenuItem::branch(
                        "Advanced",
                        None,
                        103,
                        vec![
                            CascadingMenuItem::item("Torus", 104),
                            CascadingMenuItem::item("Capsule", 105),
                        ],
                    ),
                ],
            ),
            CascadingMenuItem::separator(),
            CascadingMenuItem::item("Delete", 200),
        ];

        // 1. Level 1 only (no branch active)
        let frame_lvl1 = CascadingMenuBuilder::new(Rect::new(100.0, 50.0, 80.0, 24.0), &items, &[])
            .build(&mut tree, root)
            .expect("Level 1 menu must build");

        assert_eq!(frame_lvl1.rendered_popup_rects.len(), 1);
        assert_eq!(frame_lvl1.root_rect.x, 100.0);

        // 2. Level 2 active (3D Objects open)
        let frame_lvl2 =
            CascadingMenuBuilder::new(Rect::new(100.0, 50.0, 80.0, 24.0), &items, &[100])
                .build(&mut tree, root)
                .expect("Level 2 menu must build");

        assert_eq!(frame_lvl2.rendered_popup_rects.len(), 2);

        // 3. Level 3 active (3D Objects -> Advanced open)
        let frame_lvl3 =
            CascadingMenuBuilder::new(Rect::new(100.0, 50.0, 80.0, 24.0), &items, &[100, 103])
                .build(&mut tree, root)
                .expect("Level 3 menu must build");

        assert_eq!(frame_lvl3.rendered_popup_rects.len(), 3);

        // Verify hit-testing on Torus (tag 104)
        let lvl3_rect = frame_lvl3.rendered_popup_rects[2];
        let click_point = Point::new(lvl3_rect.x + 10.0, lvl3_rect.y + 10.0);
        let hit = tree.hit_test_target(click_point);
        assert!(hit.is_some());
        let hit_info = hit.unwrap();
        assert_eq!(hit_info.layer, UiLayer::Popup);
        assert_eq!(hit_info.role, WidgetRole::DropdownItem);
        assert_eq!(hit_info.tag, 104);
    }

    #[test]
    fn test_cascading_menu_viewport_boundary_flip() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        let items = vec![CascadingMenuItem::branch(
            "Nested",
            None,
            10,
            vec![CascadingMenuItem::item("Leaf", 11)],
        )];

        let viewport = Rect::new(0.0, 0.0, 300.0, 600.0);

        // Place anchor near right edge of viewport (250.0)
        let frame = CascadingMenuBuilder::new(Rect::new(200.0, 100.0, 80.0, 24.0), &items, &[10])
            .width(100.0)
            .viewport_bounds(viewport)
            .build(&mut tree, root)
            .expect("Menu should build");

        // Submenu should flip to the left because 200 + 100 + 100 > 300
        assert_eq!(frame.rendered_popup_rects.len(), 2);
        let root_rect = frame.rendered_popup_rects[0];
        let sub_rect = frame.rendered_popup_rects[1];
        assert!(
            sub_rect.x < root_rect.x,
            "Submenu must flip to the left when overflowing right viewport bound"
        );
    }
}