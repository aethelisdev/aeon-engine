// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Generic Card & Collapsible Section UI Widget (`iris-widgets::card`)
//!
//! Provides a standardized, hardware-accelerated container component for inspector
//! sections, tool property cards, and structural grouping blocks. Encapsulates
//! styling, header icon management (both unicode glyphs and GPU texture atlas quads),
//! title typography, and optional header action controls (e.g. deletion buttons).
//!

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for card and section containers.
#[derive(Debug, Clone)]
pub struct CardStyle {
    /// Background fill color of the card container.
    pub background: Color,
    /// Outer border stroke thickness in logical pixels.
    pub border_width: f32,
    /// Outer border stroke color.
    pub border_color: Color,
    /// Uniform corner rounding radius.
    pub border_radius: f32,
    /// Inner padding margin between card border and inner content.
    pub padding: f32,
    /// Vertical height allocated for the header title bar.
    pub header_height: f32,
    /// Font size for the header title text.
    pub header_font_size: f32,
    /// Fixed dimension for square header action buttons (e.g. deletion trash button).
    pub action_btn_size: f32,
    /// Idle background fill color for header deletion button.
    pub delete_btn_idle_bg: Color,
    /// Idle border stroke color for header deletion button.
    pub delete_btn_idle_border: Color,
    /// Idle glyph color for header deletion button.
    pub delete_btn_idle_text: Color,
    /// Hovered background fill color for header deletion button.
    pub delete_btn_hover_bg: Color,
    /// Hovered border stroke color for header deletion button.
    pub delete_btn_hover_border: Color,
    /// Hovered glyph color for header deletion button.
    pub delete_btn_hover_text: Color,
}

impl Default for CardStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.090, 0.094, 0.110, 0.98),
            border_width: 1.0,
            border_color: Color::rgba(0.133, 0.141, 0.165, 0.85),
            border_radius: 6.0,
            padding: 8.0,
            header_height: 20.0,
            header_font_size: 11.5,
            action_btn_size: 18.0,
            delete_btn_idle_bg: Color::rgba(0.157, 0.165, 0.188, 0.98),
            delete_btn_idle_border: Color::rgba(0.212, 0.220, 0.259, 0.85),
            delete_btn_idle_text: Color::rgba(0.54, 0.56, 0.60, 1.0),
            delete_btn_hover_bg: Color::rgba(0.35, 0.10, 0.10, 0.95),
            delete_btn_hover_border: Color::rgba(0.70, 0.18, 0.18, 0.85),
            delete_btn_hover_text: Color::rgba(1.0, 0.40, 0.40, 1.0),
        }
    }
}

/// Icon representation for card headers.
#[derive(Debug, Clone)]
pub enum CardIcon {
    /// No header icon.
    None,
    /// Unicode character or text glyph prepended to the title string.
    Text(String),
    /// Dedicated hardware texture atlas quad with normalized UV coordinates and tint color.
    TextureAtlas {
        /// Normalized texture coordinates `[u_min, v_min, u_max, layer]`.
        uv: [f32; 4],
        /// Vertex tint color applied to the icon quad.
        tint: Color,
    },
}

/// Computed geometry and allocated widget identifiers returned from card construction.
#[derive(Debug, Clone)]
pub struct CardFrame {
    /// Allocated widget ID of the main card outer container.
    pub card_id: WidgetId,
    /// Allocated widget ID of the header text node.
    pub header_id: WidgetId,
    /// Optional bounding rectangle of the top-right deletion action button for hit testing.
    pub delete_btn_rect: Option<Rect>,
    /// Whether the cursor is currently hovering over the deletion action button.
    pub is_delete_hovered: bool,
}

/// Fluent builder for constructing standardized UI cards and property sections.
pub struct CardBuilder<'a> {
    tree: &'a mut UiTree,
    parent_id: WidgetId,
    name: String,
    rect: Rect,
    title: String,
    title_color: Color,
    icon: CardIcon,
    has_delete_action: bool,
    cursor_pos: Point,
    style: CardStyle,
}

impl<'a> CardBuilder<'a> {
    /// Initializes a new [`CardBuilder`] attached to the specified parent widget.
    pub fn new(tree: &'a mut UiTree, parent_id: WidgetId) -> Self {
        Self {
            tree,
            parent_id,
            name: "Card".to_string(),
            rect: Rect::ZERO,
            title: String::new(),
            title_color: Color::rgba(0.886, 0.894, 0.918, 1.0),
            icon: CardIcon::None,
            has_delete_action: false,
            cursor_pos: Point::new(-1.0, -1.0),
            style: CardStyle::default(),
        }
    }

    /// Sets the semantic debug identifier name for the card container.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the full bounding box of the card in logical pixels.
    pub fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    /// Sets the human-readable header title text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the foreground color of the header title text.
    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = color;
        self
    }

    /// Sets a unicode text icon for the card header.
    pub fn icon_text(mut self, icon: impl Into<String>) -> Self {
        self.icon = CardIcon::Text(icon.into());
        self
    }

    /// Sets a hardware texture atlas icon with custom tint color.
    pub fn icon_atlas(mut self, uv: [f32; 4], tint: Color) -> Self {
        self.icon = CardIcon::TextureAtlas { uv, tint };
        self
    }

    /// Configures whether to display the top-right deletion action button.
    pub fn with_delete_action(mut self, enabled: bool) -> Self {
        self.has_delete_action = enabled;
        self
    }

    /// Provides current cursor coordinates for evaluating button hover highlights.
    pub fn cursor_pos(mut self, cursor_pos: Point) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Overrides default visual metrics with a custom [`CardStyle`].
    pub fn style(mut self, style: CardStyle) -> Self {
        self.style = style;
        self
    }

    /// Constructs the complete card widget hierarchy and returns its identifiers and geometry.
    pub fn build(self) -> CardFrame {
        let style = &self.style;
        let padding = style.padding;
        let btn_size = style.action_btn_size;

        // 1. Outer Card Container
        let card_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(card_id) {
            node.set_name(self.name);
            node.computed_rect = self.rect;
            node.style = Style::new()
                .background(style.background)
                .border(style.border_width, style.border_color)
                .border_radius(style.border_radius);
        }
        let _ = self.tree.add_child(self.parent_id, card_id);

        // 2. Header Icon & Title Text Setup
        let (icon_offset_x, text_content) = match self.icon {
            CardIcon::TextureAtlas { uv, tint } => {
                let icon_id = self.tree.create_node();
                if let Some(node) = self.tree.get_mut(icon_id) {
                    node.set_name("CardHeaderIcon");
                    node.computed_rect = Rect::new(
                        self.rect.x + padding,
                        self.rect.y + padding + (style.header_height - 14.0) * 0.5,
                        14.0,
                        14.0,
                    );
                    node.set_texture_uv(uv);
                    node.set_texture_tint(tint);
                }
                let _ = self.tree.add_child(card_id, icon_id);
                (18.0, self.title)
            }
            CardIcon::Text(glyph) => {
                let formatted = if glyph.is_empty() {
                    self.title
                } else {
                    format!("{} {}", glyph, self.title)
                };
                (0.0, formatted)
            }
            CardIcon::None => (0.0, self.title),
        };

        // 3. Optional Header Action: Deletion Trash Button
        let (delete_btn_rect, is_delete_hovered) = if self.has_delete_action {
            let del_rect = Rect::new(
                self.rect.right() - padding - btn_size,
                self.rect.y + padding,
                btn_size,
                btn_size,
            );
            let is_hovered = del_rect.contains_point(self.cursor_pos);

            let del_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(del_id) {
                node.set_name("CardDeleteBtn");
                node.set_role(WidgetRole::Button);
                node.computed_rect = del_rect;
                node.set_text("🗑");
                node.font_size = 10.5;
                node.line_height = btn_size;
                node.text_align = TextAlign::Center;

                let (bg, border, txt_col) = if is_hovered {
                    (
                        style.delete_btn_hover_bg,
                        style.delete_btn_hover_border,
                        style.delete_btn_hover_text,
                    )
                } else {
                    (
                        style.delete_btn_idle_bg,
                        style.delete_btn_idle_border,
                        style.delete_btn_idle_text,
                    )
                };
                node.style = Style::new()
                    .background(bg)
                    .border(1.0, border)
                    .border_radius(5.0);
                node.text_color = txt_col;
            }
            let _ = self.tree.add_child(card_id, del_id);

            (Some(del_rect), is_hovered)
        } else {
            (None, false)
        };

        // 4. Header Title Node
        let action_space = if self.has_delete_action {
            btn_size + 4.0
        } else {
            0.0
        };
        let title_w = (self.rect.width - padding * 2.0 - action_space - icon_offset_x).max(0.0);
        let hdr_rect = Rect::new(
            self.rect.x + padding + icon_offset_x,
            self.rect.y + padding,
            title_w,
            style.header_height,
        );

        let header_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(header_id) {
            node.set_name("CardHeaderTitle");
            node.set_text(text_content);
            node.font_size = style.header_font_size;
            node.line_height = style.header_height;
            node.text_color = self.title_color;
            node.computed_rect = hdr_rect;
        }
        let _ = self.tree.add_child(card_id, header_id);

        CardFrame {
            card_id,
            header_id,
            delete_btn_rect,
            is_delete_hovered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_builder_basic_hierarchy() {
        let mut tree = UiTree::new();
        let root = tree.create_node();

        let rect = Rect::new(10.0, 20.0, 200.0, 100.0);
        let frame = CardBuilder::new(&mut tree, root)
            .name("TestCard")
            .rect(rect)
            .title("Transform")
            .icon_text("📐")
            .with_delete_action(false)
            .build();

        let card_node = tree.get(frame.card_id).expect("Card node exists");
        assert_eq!(card_node.name.as_deref(), Some("TestCard"));
        assert_eq!(card_node.computed_rect, rect);

        let header_node = tree.get(frame.header_id).expect("Header node exists");
        assert_eq!(header_node.name.as_deref(), Some("CardHeaderTitle"));
        assert_eq!(header_node.text.as_deref(), Some("📐 Transform"));
        assert!(frame.delete_btn_rect.is_none());
        assert!(!frame.is_delete_hovered);
    }

    #[test]
    fn test_card_builder_with_delete_action_and_hover() {
        let mut tree = UiTree::new();
        let root = tree.create_node();

        let rect = Rect::new(10.0, 20.0, 200.0, 100.0);
        // Cursor placed on top right delete button area (x: 184..202, y: 28..46)
        let cursor_pos = Point::new(190.0, 32.0);

        let frame = CardBuilder::new(&mut tree, root)
            .name("RigidBodyCard")
            .rect(rect)
            .title("RigidBody")
            .icon_text("⚙")
            .with_delete_action(true)
            .cursor_pos(cursor_pos)
            .build();

        assert!(frame.delete_btn_rect.is_some());
        assert!(frame.is_delete_hovered);
        let del_rect = frame.delete_btn_rect.unwrap();
        assert!(del_rect.contains_point(cursor_pos));
    }

    #[test]
    fn test_card_builder_with_atlas_icon() {
        let mut tree = UiTree::new();
        let root = tree.create_node();

        let rect = Rect::new(0.0, 0.0, 150.0, 80.0);
        let uv = [0.1, 0.2, 0.3, 0.0];
        let tint = Color::rgba(0.2, 0.8, 1.0, 1.0);

        let frame = CardBuilder::new(&mut tree, root)
            .rect(rect)
            .title("Collider")
            .icon_atlas(uv, tint)
            .build();

        let header_node = tree.get(frame.header_id).expect("Header node exists");
        // Title directly matches without prepended text icon when atlas icon is used
        assert_eq!(header_node.text.as_deref(), Some("Collider"));
    }
}