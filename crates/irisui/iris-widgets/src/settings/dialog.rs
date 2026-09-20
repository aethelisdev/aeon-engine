// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Tabbed Preference Dialog Window Builder
//!
//! Assembles a complete modal window layout containing a draggable titlebar with close action,
//! a left vertical category tab strip, and a clipped content viewport for settings tabs.

use super::types::{TabbedDialogFrame, TabbedDialogStyle, TabbedDialogTab};
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::node::{UiLayer, WidgetCursor, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Fluent builder for assembling full tabbed preferences modal dialogs.
pub struct TabbedDialogBuilder<'a> {
    rect: Rect,
    title: String,
    sidebar_width: f32,
    active_tab: u8,
    tabs: &'a [TabbedDialogTab<'a>],
    cursor_pos: Option<Point>,
    style: TabbedDialogStyle,
}

impl<'a> TabbedDialogBuilder<'a> {
    /// Creates a new tabbed dialog builder with target dimensions and title.
    pub fn new(rect: Rect, title: impl Into<String>, tabs: &'a [TabbedDialogTab<'a>]) -> Self {
        Self {
            rect,
            title: title.into(),
            sidebar_width: 160.0,
            active_tab: 0,
            tabs,
            cursor_pos: None,
            style: TabbedDialogStyle::default(),
        }
    }

    /// Sets the width of the left navigation sidebar in logical pixels.
    #[inline]
    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width;
        self
    }

    /// Sets the identifier of the currently active sidebar tab.
    #[inline]
    pub fn active_tab(mut self, tab: u8) -> Self {
        self.active_tab = tab;
        self
    }

    /// Provides cursor coordinates for calculating hover highlights.
    #[inline]
    pub fn cursor_pos(mut self, pos: Option<Point>) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Overrides the visual styling of the tabbed dialog.
    #[inline]
    pub fn style(mut self, style: TabbedDialogStyle) -> Self {
        self.style = style;
        self
    }

    /// Compiles the tabbed dialog hierarchy into the provided [`UiTree`] and returns target frames.
    pub fn build(self, tree: &mut UiTree) -> TabbedDialogFrame {
        let left = self.rect.x;
        let top = self.rect.y;
        let width = self.rect.width;
        let height = self.rect.height;

        let titlebar_rect = Rect::new(left, top, width - 36.0, self.style.titlebar_height);
        let close_btn_rect = Rect::new(
            left + width - 30.0,
            top + (self.style.titlebar_height - 22.0) * 0.5,
            22.0,
            22.0,
        );
        let content_rect = Rect::new(
            left + self.sidebar_width,
            top + self.style.titlebar_height,
            width - self.sidebar_width,
            height - self.style.titlebar_height,
        );

        // 1. Glassmorphic SDF Main Card
        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("PreferencesCard");
            node.role = WidgetRole::ModalWindow;
            node.layer = UiLayer::Modal;
            node.computed_rect = self.rect;
            node.style = Style::new()
                .background(self.style.card_bg)
                .border(self.style.card_border_width, self.style.card_border_color)
                .border_radius(self.style.card_border_radius)
                .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
        }

        // 2. Custom Titlebar Header (Draggable)
        let titlebar_id = tree.create_node();
        if let Some(node) = tree.get_mut(titlebar_id) {
            node.set_name("PreferencesTitlebar");
            node.computed_rect = Rect::new(left, top, width, self.style.titlebar_height);
            node.style = Style::new()
                .background(self.style.titlebar_bg)
                .border(1.0, self.style.titlebar_border_color)
                .border_radius(self.style.card_border_radius);
        }
        let _ = tree.add_child(card_id, titlebar_id);

        // Titlebar Text
        let title_id = tree.create_node();
        if let Some(node) = tree.get_mut(title_id) {
            node.set_name("PreferencesTitle");
            node.set_text(&self.title);
            node.font_size = 13.0;
            node.line_height = self.style.titlebar_height;
            node.text_color = self.style.title_color;
            node.computed_rect = Rect::new(left + 14.0, top, 200.0, self.style.titlebar_height);
        }
        let _ = tree.add_child(titlebar_id, title_id);

        // Titlebar Close Button '✖'
        let is_close_hovered = self
            .cursor_pos
            .is_some_and(|p| close_btn_rect.contains_point(p));
        let close_id = tree.create_node();
        if let Some(node) = tree.get_mut(close_id) {
            node.set_name("PreferencesCloseButton");
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.computed_rect = close_btn_rect;
            let bg = if is_close_hovered {
                self.style.close_btn_bg_hover
            } else {
                self.style.close_btn_bg_idle
            };
            node.style = Style::new().background(bg).border_radius(4.0);
        }
        let _ = tree.add_child(titlebar_id, close_id);

        let close_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(close_txt_id) {
            node.set_name("CloseText");
            node.set_text("✖");
            node.font_size = 11.0;
            node.line_height = close_btn_rect.height;
            node.text_align = TextAlign::Center;
            node.text_color = if is_close_hovered {
                Color::WHITE
            } else {
                self.style.close_btn_text_color
            };
            node.computed_rect = close_btn_rect;
        }
        let _ = tree.add_child(close_id, close_txt_id);

        // 3. Left Sidebar Navigation Container
        let sidebar_rect = Rect::new(
            left,
            top + self.style.titlebar_height,
            self.sidebar_width,
            height - self.style.titlebar_height,
        );
        let sidebar_id = tree.create_node();
        if let Some(node) = tree.get_mut(sidebar_id) {
            node.set_name("PreferencesSidebar");
            node.computed_rect = sidebar_rect;
            node.style = Style::new().background(self.style.sidebar_bg);
        }
        let _ = tree.add_child(card_id, sidebar_id);

        // Sidebar Vertical Divider
        let divider_id = tree.create_node();
        if let Some(node) = tree.get_mut(divider_id) {
            node.set_name("SidebarDivider");
            node.computed_rect = Rect::new(
                left + self.sidebar_width - 1.0,
                top + self.style.titlebar_height,
                1.0,
                sidebar_rect.height,
            );
            node.style = Style::new().background(self.style.sidebar_divider_color);
        }
        let _ = tree.add_child(card_id, divider_id);

        // 4. Sidebar Tabs
        let tab_h = 34.0;
        let mut tab_y = top + self.style.titlebar_height + 8.0;
        let mut tab_rects = Vec::with_capacity(self.tabs.len());

        for tab in self.tabs {
            let is_selected = self.active_tab == tab.id;
            let tab_rect = Rect::new(left + 6.0, tab_y, self.sidebar_width - 12.0, tab_h);
            let is_hovered = self.cursor_pos.is_some_and(|p| tab_rect.contains_point(p));

            let tab_node_id = tree.create_node();
            if let Some(node) = tree.get_mut(tab_node_id) {
                node.set_name("SidebarTab");
                node.interactive = true;
                node.role = WidgetRole::DockTab;
                node.cursor = Some(WidgetCursor::Pointer);
                node.computed_rect = tab_rect;
                let bg = if is_selected {
                    self.style.tab_bg_selected
                } else if is_hovered {
                    self.style.tab_bg_hover
                } else {
                    self.style.tab_bg_idle
                };
                node.style = Style::new().background(bg).border_radius(4.0);
            }
            let _ = tree.add_child(sidebar_id, tab_node_id);

            // Left Cyan Indicator Line for selected tab
            if is_selected {
                let ind_id = tree.create_node();
                if let Some(node) = tree.get_mut(ind_id) {
                    node.set_name("TabIndicator");
                    node.computed_rect =
                        Rect::new(tab_rect.x, tab_rect.y + 4.0, 3.5, tab_rect.height - 8.0);
                    node.style = Style::new()
                        .background(self.style.tab_indicator_color)
                        .border_radius(1.5);
                }
                let _ = tree.add_child(tab_node_id, ind_id);
            }

            // Tab Label
            let lbl_id = tree.create_node();
            if let Some(node) = tree.get_mut(lbl_id) {
                node.set_name("TabLabel");
                node.set_text(tab.label);
                node.font_size = 12.5;
                node.line_height = tab_h;
                node.text_color = if is_selected {
                    self.style.tab_text_selected
                } else if is_hovered {
                    self.style.tab_text_hover
                } else {
                    self.style.tab_text_idle
                };
                node.computed_rect =
                    Rect::new(tab_rect.x + 14.0, tab_rect.y, tab_rect.width - 20.0, tab_h);
            }
            let _ = tree.add_child(tab_node_id, lbl_id);

            tab_rects.push((tab.id, tab_rect));
            tab_y += tab_h + 2.0;
        }

        // 5. Right Content Area Container
        let content_id = tree.create_node();
        if let Some(node) = tree.get_mut(content_id) {
            node.set_name("PreferencesContentArea");
            node.computed_rect = content_rect;
            node.style = Style::new().clip_children(true);
        }
        let _ = tree.add_child(card_id, content_id);

        TabbedDialogFrame {
            card_id,
            titlebar_id,
            titlebar_rect,
            close_btn_rect,
            content_id,
            content_rect,
            tab_rects,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabbed_dialog_builder_hierarchy() {
        let mut tree = UiTree::new();
        let tabs = [
            TabbedDialogTab::new("General", 0),
            TabbedDialogTab::new("Graphics", 1),
        ];
        let frame = TabbedDialogBuilder::new(
            Rect::new(100.0, 100.0, 800.0, 600.0),
            "Settings Test",
            &tabs,
        )
        .active_tab(1)
        .build(&mut tree);

        assert_eq!(frame.tab_rects.len(), 2);
        assert_eq!(frame.tab_rects[0].0, 0);
        assert_eq!(frame.tab_rects[1].0, 1);
        assert_eq!(frame.content_rect.x, 100.0 + 160.0);
        assert_eq!(frame.titlebar_rect.width, 800.0 - 36.0);

        let card_node = tree.get(frame.card_id).expect("card node exists");
        assert_eq!(card_node.role, WidgetRole::ModalWindow);
        assert_eq!(card_node.layer, UiLayer::Modal);
    }
}