// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Hierarchical Tree View & Tree Row Widgets
//!
//! Provides standardized, GPU SDF-rendered tree row components for scene hierarchies,
//! directory browsers, and nested outline trees.
//!
//! Handles depth indentation, hierarchical branch connector lines (vertical stem and horizontal
//! branch arms), expand/collapse foldout chevrons, atlas texture or vector glyph icons,
//! selection capsules, and trailing action reservation.

use iris_core::color::Color;
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for a hierarchical tree row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeRowStyle {
    /// Background color in normal/idle state.
    pub bg_idle: Color,
    /// Background color when hovered by cursor.
    pub bg_hover: Color,
    /// Background color when selected.
    pub bg_selected: Color,
    /// Border color in normal/idle state.
    pub border_idle: Color,
    /// Border color when hovered by cursor.
    pub border_hover: Color,
    /// Border color when selected.
    pub border_selected: Color,
    /// Border width in logical pixels.
    pub border_width: f32,
    /// Corner rounding radius in logical pixels.
    pub border_radius: f32,
    /// Text label color in normal/idle state.
    pub text_color_idle: Color,
    /// Text label color when hovered by cursor.
    pub text_color_hover: Color,
    /// Text label color when selected.
    pub text_color_selected: Color,
    /// Foldout arrow color in normal/idle state.
    pub foldout_color_idle: Color,
    /// Foldout arrow color when selected.
    pub foldout_color_selected: Color,
    /// Hierarchy connector line color.
    pub line_color: Color,
    /// Hierarchy connector line thickness in logical pixels.
    pub line_width: f32,
    /// Horizontal indent stride per depth level in logical pixels.
    pub indent_stride: f32,
    /// Initial left margin offset before depth indenting in logical pixels.
    pub indent_offset: f32,
    /// Font size in logical points for the row label.
    pub font_size: f32,
    /// Font size in logical points for the foldout chevron arrow.
    pub foldout_font_size: f32,
}

impl TreeRowStyle {
    /// Creates a visual style tuned for scene hierarchy panels.
    ///
    /// Uses petrol blue selection capsules, vibrant cyan borders, and crisp blue branch lines.
    pub fn hierarchy_default() -> Self {
        Self {
            bg_idle: Color::TRANSPARENT,
            bg_hover: Color::rgba(0.10, 0.14, 0.20, 0.60),
            bg_selected: Color::rgba(0.02, 0.22, 0.32, 0.95),
            border_idle: Color::TRANSPARENT,
            border_hover: Color::rgba(0.18, 0.24, 0.35, 0.50),
            border_selected: Color::rgba(0.0, 0.88, 1.0, 0.95),
            border_width: 1.5,
            border_radius: 6.0,
            text_color_idle: Color::rgba(0.88, 0.91, 0.98, 1.0),
            text_color_hover: Color::WHITE,
            text_color_selected: Color::rgba(0.0, 0.95, 1.0, 1.0),
            foldout_color_idle: Color::rgba(0.65, 0.68, 0.78, 1.0),
            foldout_color_selected: Color::rgba(0.0, 0.95, 1.0, 1.0),
            line_color: Color::rgba(0.20, 0.55, 0.90, 0.85),
            line_width: 1.2,
            indent_stride: 14.0,
            indent_offset: 6.0,
            font_size: 11.5,
            foldout_font_size: 9.0,
        }
    }

    /// Creates a visual style tuned for directory/folder tree sidebars.
    ///
    /// Uses compact border radii, subtle slate hover highlights, and warm amber folder tints.
    pub fn folder_default() -> Self {
        Self {
            bg_idle: Color::TRANSPARENT,
            bg_hover: Color::rgba(0.14, 0.16, 0.22, 0.70),
            bg_selected: Color::rgba(0.08, 0.22, 0.32, 0.90),
            border_idle: Color::TRANSPARENT,
            border_hover: Color::TRANSPARENT,
            border_selected: Color::rgba(0.0, 0.85, 1.0, 0.80),
            border_width: 1.0,
            border_radius: 4.0,
            text_color_idle: Color::rgba(0.75, 0.78, 0.85, 1.0),
            text_color_hover: Color::rgba(0.90, 0.92, 0.96, 1.0),
            text_color_selected: Color::WHITE,
            foldout_color_idle: Color::rgba(0.60, 0.65, 0.75, 1.0),
            foldout_color_selected: Color::rgba(0.0, 0.90, 1.0, 1.0),
            line_color: Color::rgba(0.20, 0.55, 0.90, 0.85),
            line_width: 1.2,
            indent_stride: 14.0,
            indent_offset: 4.0,
            font_size: 11.5,
            foldout_font_size: 11.0,
        }
    }
}

impl Default for TreeRowStyle {
    fn default() -> Self {
        Self::hierarchy_default()
    }
}

/// Optional visual icon specification for a tree row item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TreeRowIcon<'a> {
    /// Texture-backed atlas icon quad.
    Texture {
        /// UV coordinate bounding box `[u_min, v_min, u_max, v_max]` or atlas layer indicator.
        uv: [f32; 4],
        /// Multiplicative tint color.
        tint: Color,
        /// Width and height dimension in logical pixels.
        size: f32,
    },
    /// Text glyph icon.
    Text {
        /// UTF-8 glyph or short string representing the icon.
        text: &'a str,
        /// Glyphic font color.
        color: Color,
        /// Font size in logical points.
        size: f32,
    },
}

/// Structural layout frame and interactive child targets returned by [`TreeRowBuilder`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeRowFrame {
    /// Identifier of the outer row container widget.
    pub row_id: WidgetId,
    /// Bounding rectangle of the entire row.
    pub row_rect: Rect,
    /// Bounding rectangle of the foldout chevron, if the node has children.
    pub foldout_rect: Option<Rect>,
    /// Bounding rectangle of the row icon, if an icon was provided.
    pub icon_rect: Option<Rect>,
    /// Bounding rectangle allocated for the primary text label.
    pub label_rect: Rect,
}

/// Fluent builder for constructing standardized, hardware-accelerated tree view rows.
pub struct TreeRowBuilder<'a> {
    rect: Rect,
    name: Option<String>,
    depth: usize,
    has_children: bool,
    is_expanded: bool,
    is_selected: bool,
    is_hovered: bool,
    draw_connector_lines: bool,
    icon: Option<TreeRowIcon<'a>>,
    label: Option<String>,
    label_color: Option<Color>,
    label_align: TextAlign,
    foldout_glyph_expanded: &'a str,
    foldout_glyph_collapsed: &'a str,
    trailing_reserve_width: f32,
    style: TreeRowStyle,
}

impl<'a> TreeRowBuilder<'a> {
    /// Initializes a new tree row builder with the target bounding box.
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            name: None,
            depth: 0,
            has_children: false,
            is_expanded: false,
            is_selected: false,
            is_hovered: false,
            draw_connector_lines: false,
            icon: None,
            label: None,
            label_color: None,
            label_align: TextAlign::Left,
            foldout_glyph_expanded: "▼",
            foldout_glyph_collapsed: "▶",
            trailing_reserve_width: 0.0,
            style: TreeRowStyle::default(),
        }
    }

    /// Sets the debug/semantic name of the row container node.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the nesting depth level of this row (0 = root level).
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Sets whether this row represents a branch with child items.
    pub fn has_children(mut self, has_children: bool) -> Self {
        self.has_children = has_children;
        self
    }

    /// Sets whether the node is currently expanded in the tree view.
    pub fn is_expanded(mut self, is_expanded: bool) -> Self {
        self.is_expanded = is_expanded;
        self
    }

    /// Sets whether the row is currently selected.
    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Sets whether the row is currently hovered by the pointer.
    pub fn is_hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    /// Sets whether to render hierarchical vertical and horizontal SDF connector lines.
    pub fn draw_connector_lines(mut self, draw: bool) -> Self {
        self.draw_connector_lines = draw;
        self
    }

    /// Assigns an optional leading icon (texture quad or text glyph).
    pub fn icon(mut self, icon: Option<TreeRowIcon<'a>>) -> Self {
        self.icon = icon;
        self
    }

    /// Sets the textual label of the tree row item.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets an explicit text color, overriding the default style progression.
    pub fn label_color(mut self, color: Option<Color>) -> Self {
        self.label_color = color;
        self
    }

    /// Sets horizontal alignment for the primary label.
    pub fn label_align(mut self, align: TextAlign) -> Self {
        self.label_align = align;
        self
    }

    /// Configures custom glyph strings for the expanded and collapsed states.
    pub fn foldout_glyphs(mut self, expanded: &'a str, collapsed: &'a str) -> Self {
        self.foldout_glyph_expanded = expanded;
        self.foldout_glyph_collapsed = collapsed;
        self
    }

    /// Reserves horizontal width at the right edge of the row for trailing actions (e.g. eye toggle button).
    pub fn trailing_reserve_width(mut self, width: f32) -> Self {
        self.trailing_reserve_width = width;
        self
    }

    /// Applies a custom visual styling configuration.
    pub fn style(mut self, style: TreeRowStyle) -> Self {
        self.style = style;
        self
    }

    /// Compiles the tree row into the target `UiTree` under `parent_id` and returns layout targets.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> TreeRowFrame {
        // 1. Container Capsule
        let row_id = tree.create_node();
        let (bg, border, border_w) = if self.is_selected {
            (
                self.style.bg_selected,
                self.style.border_selected,
                self.style.border_width,
            )
        } else if self.is_hovered {
            (self.style.bg_hover, self.style.border_hover, 1.0)
        } else {
            (self.style.bg_idle, self.style.border_idle, 0.0)
        };

        if let Some(node) = tree.get_mut(row_id) {
            let name_str = self.name.as_deref().unwrap_or("TreeRow");
            node.set_name(name_str);
            node.computed_rect = self.rect;
            node.style = Style::new()
                .background(bg)
                .border(border_w, border)
                .border_radius(self.style.border_radius);
        }
        let _ = tree.add_child(parent_id, row_id);

        // 2. Hierarchy Tree Connector Lines
        if self.draw_connector_lines && self.depth > 0 {
            for d in 0..self.depth {
                let stem_x = self.rect.x
                    + self.style.indent_offset
                    + 2.5
                    + (d as f32 * self.style.indent_stride);
                let is_last_level = d == self.depth - 1;

                // Vertical branch line
                let v_h = if is_last_level {
                    self.rect.height * 0.5
                } else {
                    self.rect.height
                };
                let v_id = tree.create_node();
                if let Some(node) = tree.get_mut(v_id) {
                    node.set_name("TreeLineVertical");
                    node.computed_rect = Rect::new(stem_x, self.rect.y, self.style.line_width, v_h);
                    node.style = Style::new().background(self.style.line_color);
                }
                let _ = tree.add_child(row_id, v_id);

                // Horizontal branch arm into icon
                if is_last_level {
                    let h_id = tree.create_node();
                    if let Some(node) = tree.get_mut(h_id) {
                        node.set_name("TreeLineHorizontal");
                        node.computed_rect = Rect::new(
                            stem_x,
                            self.rect.y + self.rect.height * 0.5 - self.style.line_width * 0.5,
                            9.0,
                            self.style.line_width,
                        );
                        node.style = Style::new().background(self.style.line_color);
                    }
                    let _ = tree.add_child(row_id, h_id);
                }
            }
        }

        // 3. Foldout Chevron Arrow
        let indent = self.depth as f32 * self.style.indent_stride;
        let prefix_x = self.rect.x + self.style.indent_offset + indent;

        let foldout_w = 12.0;
        let (foldout_rect, icon_x) = if self.has_children {
            let f_rect = Rect::new(prefix_x, self.rect.y, foldout_w, self.rect.height);
            let fold_id = tree.create_node();
            if let Some(node) = tree.get_mut(fold_id) {
                node.set_name("TreeFoldoutArrow");
                node.set_text(if self.is_expanded {
                    self.foldout_glyph_expanded
                } else {
                    self.foldout_glyph_collapsed
                });
                node.font_size = self.style.foldout_font_size;
                node.line_height = self.rect.height;
                node.text_align = TextAlign::Center;
                node.text_color = if self.is_selected {
                    self.style.foldout_color_selected
                } else {
                    self.style.foldout_color_idle
                };
                node.computed_rect = f_rect;
            }
            let _ = tree.add_child(row_id, fold_id);
            (Some(f_rect), prefix_x + foldout_w + 2.0)
        } else {
            (None, prefix_x + 2.0)
        };

        // 4. Row Icon
        let (icon_rect, next_x) = if let Some(icon) = self.icon {
            match icon {
                TreeRowIcon::Texture { uv, tint, size } => {
                    let icon_y = self.rect.y + (self.rect.height - size) * 0.5;
                    let i_rect = Rect::new(icon_x, icon_y, size, size);
                    let icon_id = tree.create_node();
                    if let Some(node) = tree.get_mut(icon_id) {
                        node.set_name("TreeRowIconTexture");
                        node.computed_rect = i_rect;
                        node.set_texture_uv(uv);
                        node.set_texture_tint(tint);
                    }
                    let _ = tree.add_child(row_id, icon_id);
                    (Some(i_rect), icon_x + size + 6.0)
                }
                TreeRowIcon::Text { text, color, size } => {
                    let i_rect = Rect::new(icon_x, self.rect.y, size, self.rect.height);
                    let icon_id = tree.create_node();
                    if let Some(node) = tree.get_mut(icon_id) {
                        node.set_name("TreeRowIconText");
                        node.computed_rect = i_rect;
                        node.set_text(text);
                        node.font_size = size;
                        node.line_height = self.rect.height;
                        node.text_color = color;
                    }
                    let _ = tree.add_child(row_id, icon_id);
                    (Some(i_rect), icon_x + size + 6.0)
                }
            }
        } else {
            (None, icon_x)
        };

        // 5. Primary Row Label
        let label_color = self.label_color.unwrap_or(if self.is_selected {
            self.style.text_color_selected
        } else if self.is_hovered {
            self.style.text_color_hover
        } else {
            self.style.text_color_idle
        });

        let label_rect = match self.label_align {
            TextAlign::Center => {
                let start_x = self.rect.x + 28.0;
                let available_w =
                    (self.rect.right() - self.trailing_reserve_width - start_x).max(20.0);
                Rect::new(start_x, self.rect.y, available_w, self.rect.height)
            }
            _ => {
                let available_w =
                    (self.rect.right() - self.trailing_reserve_width - next_x).max(20.0);
                Rect::new(next_x, self.rect.y, available_w, self.rect.height)
            }
        };

        if let Some(text) = self.label {
            let label_id = tree.create_node();
            if let Some(node) = tree.get_mut(label_id) {
                node.set_name("TreeRowLabel");
                node.set_text(text);
                node.font_size = self.style.font_size;
                node.line_height = self.rect.height;
                node.text_align = self.label_align;
                node.text_color = label_color;
                node.computed_rect = label_rect;
            }
            let _ = tree.add_child(row_id, label_id);
        }

        TreeRowFrame {
            row_id,
            row_rect: self.rect,
            foldout_rect,
            icon_rect,
            label_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_row_builder_basic_hierarchy() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node required");

        let row_rect = Rect::new(10.0, 50.0, 260.0, 24.0);
        let frame = TreeRowBuilder::new(row_rect)
            .name("EntityNode_1")
            .depth(2)
            .has_children(true)
            .is_expanded(true)
            .is_selected(true)
            .draw_connector_lines(true)
            .icon(Some(TreeRowIcon::Text {
                text: "Point Light",
                color: Color::WHITE,
                size: 14.0,
            }))
            .label("Main Camera")
            .trailing_reserve_width(24.0)
            .style(TreeRowStyle::hierarchy_default())
            .build(&mut tree, root);

        assert_eq!(frame.row_rect, row_rect);
        assert!(frame.foldout_rect.is_some());
        assert!(frame.icon_rect.is_some());
        assert!(frame.label_rect.width > 50.0);

        let row_node = tree.get(frame.row_id).expect("Row node must exist");
        assert_eq!(row_node.computed_rect, row_rect);
        // depth 2 with connector lines:
        // level 0: 1 vertical line
        // level 1: 1 vertical line + 1 horizontal line
        // + 1 foldout + 1 icon + 1 label = 6 children
        assert_eq!(row_node.children.len(), 6);
    }

    #[test]
    fn test_tree_row_builder_folder_tree() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node required");

        let row_rect = Rect::new(0.0, 0.0, 180.0, 22.0);
        let frame = TreeRowBuilder::new(row_rect)
            .name("Folder_assets")
            .depth(0)
            .has_children(false)
            .is_expanded(false)
            .is_selected(false)
            .draw_connector_lines(false)
            .icon(Some(TreeRowIcon::Texture {
                uv: [0.0, 0.0, 1.0, 6.0],
                tint: Color::WHITE,
                size: 16.0,
            }))
            .label("assets")
            .style(TreeRowStyle::folder_default())
            .build(&mut tree, root);

        assert_eq!(frame.row_rect, row_rect);
        assert!(frame.foldout_rect.is_none());
        assert!(frame.icon_rect.is_some());
        let row_node = tree.get(frame.row_id).expect("Folder row must exist");
        assert_eq!(row_node.children.len(), 2); // 1 icon + 1 label
    }
}