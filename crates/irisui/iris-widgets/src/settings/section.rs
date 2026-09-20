// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collapsible Setting Section Card Builder
//!
//! Provides a styled collapsible group card container with an interactive header toggle,
//! chevron indicator, title, and optional description subtitle.

use super::types::{SettingSectionFrame, SettingSectionStyle};
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{WidgetCursor, WidgetRole};
use iris_core::style::Style;
use iris_core::tree::UiTree;

/// Fluent builder for assembling collapsible settings section cards.
pub struct SettingSectionBuilder {
    rect: Rect,
    title: String,
    is_collapsed: bool,
    description: Option<String>,
    cursor_pos: Option<Point>,
    style: SettingSectionStyle,
}

impl SettingSectionBuilder {
    /// Creates a new section builder with the given bounding rectangle and header title.
    #[inline]
    pub fn new(rect: Rect, title: impl Into<String>) -> Self {
        Self {
            rect,
            title: title.into(),
            is_collapsed: false,
            description: None,
            cursor_pos: None,
            style: SettingSectionStyle::default(),
        }
    }

    /// Sets whether this section is collapsed or expanded.
    #[inline]
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    /// Sets an optional descriptive subtitle explaining the section's contents.
    #[inline]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Provides cursor position for hover highlight calculation.
    #[inline]
    pub fn cursor_pos(mut self, pos: Option<Point>) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Overrides the visual styling of the section card.
    #[inline]
    pub fn style(mut self, style: SettingSectionStyle) -> Self {
        self.style = style;
        self
    }

    /// Compiles the section card into the provided [`UiTree`] and attaches it to `parent_id`.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> SettingSectionFrame {
        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("SettingSectionCard");
            node.computed_rect = self.rect;
            node.style = Style::new()
                .background(self.style.bg_color)
                .border(self.style.border_width, self.style.border_color)
                .border_radius(self.style.border_radius);
        }
        let _ = tree.add_child(parent_id, card_id);

        let header_rect = Rect::new(
            self.rect.x + 8.0,
            self.rect.y + 6.0,
            self.rect.width - 16.0,
            24.0,
        );
        let is_hdr_hovered = self
            .cursor_pos
            .is_some_and(|p| header_rect.contains_point(p));

        let title_id = tree.create_node();
        if let Some(node) = tree.get_mut(title_id) {
            node.set_name("SettingSectionTitle");
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            let arrow = if self.is_collapsed { "▸" } else { "▾" };
            node.set_text(format!("{}  {}", arrow, self.title));
            node.font_size = self.style.title_font_size;
            node.line_height = 24.0;
            node.text_color = if is_hdr_hovered {
                self.style.title_color_hover
            } else {
                self.style.title_color_idle
            };
            node.computed_rect = header_rect;
        }
        let _ = tree.add_child(card_id, title_id);

        let mut content_start_y = self.rect.y + 36.0;

        if let (false, Some(desc)) = (self.is_collapsed, self.description) {
            let desc_id = tree.create_node();
            if let Some(node) = tree.get_mut(desc_id) {
                node.set_name("SettingSectionDesc");
                node.set_text(&desc);
                node.font_size = self.style.description_font_size;
                node.line_height = 16.0;
                node.text_color = self.style.description_color;
                node.computed_rect = Rect::new(
                    self.rect.x + 10.0,
                    self.rect.y + 34.0,
                    self.rect.width - 20.0,
                    28.0,
                );
            }
            let _ = tree.add_child(card_id, desc_id);
            content_start_y = self.rect.y + 66.0;
        }

        SettingSectionFrame {
            card_id,
            header_rect,
            content_start_y,
            total_height: self.rect.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_section_builder_collapsed_and_expanded() {
        let mut tree = UiTree::new();
        let root_id = tree.create_node();

        let collapsed_frame =
            SettingSectionBuilder::new(Rect::new(10.0, 10.0, 300.0, 36.0), "Collapsed Section")
                .collapsed(true)
                .build(&mut tree, root_id);

        assert_eq!(collapsed_frame.content_start_y, 10.0 + 36.0);

        let expanded_frame =
            SettingSectionBuilder::new(Rect::new(10.0, 50.0, 300.0, 120.0), "Expanded Section")
                .collapsed(false)
                .description("Test description text")
                .build(&mut tree, root_id);

        assert_eq!(expanded_frame.content_start_y, 50.0 + 66.0);
    }
}