// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hardware-accelerated GPU SDF floating popup menu widget for combobox choices.
//!
//! Provides customizable styling, icons, right-alignment, and semantic item tagging.

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for a combobox popup menu and its interactive items.
#[derive(Debug, Clone)]
pub struct ComboboxPopupStyle {
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
    /// Background color of an unselected, idle item.
    pub item_idle_bg: Color,
    /// Background color of a hovered item under the pointer cursor.
    pub item_hover_bg: Color,
    /// Background color of the currently selected active item.
    pub item_selected_bg: Color,
    /// Text color of an unselected, idle item.
    pub text_idle_color: Color,
    /// Text color of a hovered item.
    pub text_hover_color: Color,
    /// Text color of the selected active item.
    pub text_selected_color: Color,
    /// Font size in pixels for item labels.
    pub font_size: f32,
    /// Height of each individual dropdown row in pixels.
    pub row_height: f32,
    /// Horizontal padding in pixels between item edge and text.
    pub item_padding_x: f32,
}

impl Default for ComboboxPopupStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.08, 0.09, 0.13, 0.98),
            border_width: 1.0,
            border_color: Color::rgba(0.0, 0.85, 1.0, 0.85),
            border_radius: 6.0,
            shadow_y: 6.0,
            shadow_blur: 18.0,
            shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.85),
            item_idle_bg: Color::TRANSPARENT,
            item_hover_bg: Color::rgba(0.24, 0.27, 0.37, 0.95),
            item_selected_bg: Color::rgba(0.0, 0.35, 0.45, 0.80),
            text_idle_color: Color::rgba(0.85, 0.88, 0.95, 1.0),
            text_hover_color: Color::WHITE,
            text_selected_color: Color::rgba(0.0, 0.90, 1.0, 1.0),
            font_size: 11.5,
            row_height: 24.0,
            item_padding_x: 8.0,
        }
    }
}

/// Output frame produced when building a combobox popup menu into a [`UiTree`].
///
/// Contains the generated node keys and the computed bounding rectangle.
#[derive(Debug, Clone)]
pub struct ComboboxPopupFrame {
    /// Generational node identifier for the popup container quad.
    pub popup_id: WidgetId,
    /// Absolute computed screen-space rectangle of the popup menu.
    pub popup_rect: Rect,
    /// Pairs of `(item_index, widget_id)` for each populated dropdown option.
    pub item_ids: Vec<(usize, WidgetId)>,
}

/// Fluent builder for constructing standardized combobox dropdown popup menus into a [`UiTree`].
///
/// Creates nodes automatically tagged with their item indices (`node.tag = index as u64`)
/// and assigned [`WidgetRole::DropdownItem`] on [`UiLayer::Popup`], enabling zero-allocation
/// retained-mode event dispatching without requiring the host application to track manual `Rect` lists.
#[derive(Debug, Clone)]
pub struct ComboboxPopupBuilder<'a> {
    trigger_rect: Rect,
    items: Vec<&'a str>,
    item_icons: Vec<Option<&'a str>>,
    selected_index: Option<usize>,
    cursor_pos: Point,
    style: ComboboxPopupStyle,
    menu_name: &'static str,
    custom_width: Option<f32>,
    min_width: Option<f32>,
    align_right: bool,
}

impl<'a> ComboboxPopupBuilder<'a> {
    /// Creates a new combobox popup builder anchored to the specified trigger button rectangle.
    #[inline]
    pub fn new(trigger_rect: Rect) -> Self {
        Self {
            trigger_rect,
            items: Vec::new(),
            item_icons: Vec::new(),
            selected_index: None,
            cursor_pos: Point::new(-1000.0, -1000.0),
            style: ComboboxPopupStyle::default(),
            menu_name: "ComboboxDropdownPopup",
            custom_width: None,
            min_width: None,
            align_right: false,
        }
    }

    /// Sets the list of text option labels displayed by the dropdown menu.
    #[inline]
    pub fn items(mut self, items: &[&'a str]) -> Self {
        self.items = items.to_vec();
        self.item_icons = Vec::new();
        self
    }

    /// Sets the list of text option labels paired with optional icon strings.
    ///
    /// When an icon string is provided, a dedicated icon node is generated to the left
    /// of the text label.
    #[inline]
    pub fn items_with_icons(mut self, items: &[(&'a str, Option<&'a str>)]) -> Self {
        self.items = items.iter().map(|(l, _)| *l).collect();
        self.item_icons = items.iter().map(|(_, i)| *i).collect();
        self
    }

    /// Sets an explicit width in pixels for the dropdown menu popup.
    ///
    /// When set, this overrides the default behavior of matching the trigger button width.
    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.custom_width = Some(width);
        self
    }

    /// Sets a minimum width in pixels for the dropdown menu popup.
    ///
    /// Ensures the popup does not shrink below this width even if the trigger button is narrower.
    #[inline]
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Aligns the popup menu's right edge with the trigger button's right edge.
    ///
    /// Useful for dropdown buttons anchored to the right side of a panel or screen boundary.
    #[inline]
    pub fn align_right(mut self, align_right: bool) -> Self {
        self.align_right = align_right;
        self
    }

    /// Sets the currently active or selected item index.
    #[inline]
    pub fn selected_index(mut self, selected_index: Option<usize>) -> Self {
        self.selected_index = selected_index;
        self
    }

    /// Sets the pointer cursor position used to compute visual hover highlights.
    #[inline]
    pub fn cursor_pos(mut self, cursor_pos: Point) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Overrides the visual styling theme for the dropdown menu.
    #[inline]
    pub fn style(mut self, style: ComboboxPopupStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets an optional debug name for the popup menu node.
    #[inline]
    pub fn name(mut self, name: &'static str) -> Self {
        self.menu_name = name;
        self
    }

    /// Builds the dropdown popup container and item nodes into the target [`UiTree`].
    ///
    /// Automatically tags each item node with its zero-based index (`node.tag = idx as u64`)
    /// and assigns [`WidgetRole::DropdownItem`] on [`UiLayer::Popup`].
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> ComboboxPopupFrame {
        let items_count = self.items.len();
        let popup_h = (items_count as f32) * self.style.row_height + 4.0;
        let mut popup_w = self.custom_width.unwrap_or(self.trigger_rect.width);
        if let Some(min_w) = self.min_width {
            popup_w = popup_w.max(min_w);
        }
        let popup_x = if self.align_right {
            self.trigger_rect.right() - popup_w
        } else {
            self.trigger_rect.x
        };
        let popup_rect = Rect::new(
            popup_x,
            self.trigger_rect.y + self.trigger_rect.height + 2.0,
            popup_w,
            popup_h,
        );

        let popup_id = tree.create_node();
        if let Some(node) = tree.get_mut(popup_id) {
            node.set_name(self.menu_name);
            node.set_role(WidgetRole::DropdownPopup);
            node.set_layer(UiLayer::Popup);
            node.computed_rect = popup_rect;
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
        let _ = tree.add_child(parent_id, popup_id);

        let mut item_ids = Vec::with_capacity(items_count);

        for (idx, label) in self.items.iter().enumerate() {
            let item_y = popup_rect.y + 2.0 + (idx as f32) * self.style.row_height;
            let item_rect = Rect::new(
                popup_rect.x + 2.0,
                item_y,
                popup_rect.width - 4.0,
                self.style.row_height - 2.0,
            );
            let is_hovered = item_rect.contains_point(self.cursor_pos);
            let is_selected = self.selected_index == Some(idx);

            let item_id = tree.create_node();
            if let Some(node) = tree.get_mut(item_id) {
                node.set_name("DropdownItem");
                node.set_role(WidgetRole::DropdownItem);
                node.set_layer(UiLayer::Popup);
                node.set_tag(idx as u64);
                node.computed_rect = item_rect;

                let bg = if is_selected {
                    self.style.item_selected_bg
                } else if is_hovered {
                    self.style.item_hover_bg
                } else {
                    self.style.item_idle_bg
                };
                node.style = Style::new().background(bg).border_radius(4.0);
            }
            let _ = tree.add_child(popup_id, item_id);

            let icon_opt = self.item_icons.get(idx).copied().flatten();
            if let Some(icon_str) = icon_opt {
                let icon_id = tree.create_node();
                if let Some(node) = tree.get_mut(icon_id) {
                    node.set_name("DropdownItemIcon");
                    node.set_role(WidgetRole::Default);
                    node.set_layer(UiLayer::Popup);
                    node.set_tag(idx as u64);
                    node.interactive = false;
                    node.set_text(icon_str);
                    node.font_size = 11.0;
                    node.line_height = self.style.row_height - 2.0;
                    node.text_align = TextAlign::Left;
                    node.text_color = if is_selected {
                        self.style.text_selected_color
                    } else if is_hovered {
                        self.style.text_hover_color
                    } else {
                        self.style.text_idle_color
                    };
                    node.computed_rect = Rect::new(
                        item_rect.x + self.style.item_padding_x,
                        item_rect.y,
                        18.0,
                        self.style.row_height - 2.0,
                    );
                }
                let _ = tree.add_child(item_id, icon_id);
            }

            let text_x = if icon_opt.is_some() {
                item_rect.x + self.style.item_padding_x + 22.0
            } else {
                item_rect.x + self.style.item_padding_x
            };
            let text_w = (item_rect.right() - self.style.item_padding_x - text_x).max(0.0);

            let lbl_id = tree.create_node();
            if let Some(node) = tree.get_mut(lbl_id) {
                node.set_name("DropdownItemText");
                node.set_role(WidgetRole::DropdownLabel);
                node.set_layer(UiLayer::Popup);
                node.set_tag(idx as u64);
                node.interactive = false;
                node.set_text(*label);
                node.font_size = self.style.font_size;
                node.line_height = self.style.row_height - 2.0;
                node.text_align = TextAlign::Left;
                node.text_color = if is_selected {
                    self.style.text_selected_color
                } else if is_hovered {
                    self.style.text_hover_color
                } else {
                    self.style.text_idle_color
                };
                node.computed_rect =
                    Rect::new(text_x, item_rect.y, text_w, self.style.row_height - 2.0);
            }
            let _ = tree.add_child(item_id, lbl_id);

            item_ids.push((idx, item_id));
        }

        ComboboxPopupFrame {
            popup_id,
            popup_rect,
            item_ids,
        }
    }
}