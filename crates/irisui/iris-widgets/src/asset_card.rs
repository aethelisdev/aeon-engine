// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Asset Card Widget & Styles
//!
//! Provides standardized, GPU SDF-rendered asset thumbnail cards for content browsers,
//! project drawers, and media libraries.
//!
//! Handles category pill badges, VRAM/memory status indicators, texture thumbnails,
//! canonical vector icons, truncated titles, metadata labels, and selection/hover states.

use iris_core::color::Color;
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::node::{WidgetCursor, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual styling configuration for an asset browser grid card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetCardStyle {
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
    /// Border width in normal/idle or hovered state in logical pixels.
    pub border_width_idle: f32,
    /// Border width when selected in logical pixels.
    pub border_width_selected: f32,
    /// Corner rounding radius of the outer card capsule in logical pixels.
    pub border_radius: f32,
    /// Title text color in normal/idle state.
    pub title_color_idle: Color,
    /// Title text color when hovered by cursor.
    pub title_color_hover: Color,
    /// Title text color when selected.
    pub title_color_selected: Color,
    /// Font size in logical points for the title label.
    pub title_font_size: f32,
    /// Text color for the bottom metadata/size label.
    pub meta_color: Color,
    /// Font size in logical points for the metadata label.
    pub meta_font_size: f32,
    /// Background color of the central thumbnail/preview box.
    pub preview_box_bg: Color,
    /// Border color of the central thumbnail/preview box.
    pub preview_box_border: Color,
    /// Corner rounding radius of the central thumbnail/preview box.
    pub preview_box_radius: f32,
    /// Dimension (width and height) of the central preview box in logical pixels.
    pub preview_box_size: f32,
    /// Font size in logical points for the category badge.
    pub badge_font_size: f32,
    /// Corner rounding radius of the category badge capsule.
    pub badge_radius: f32,
    /// Alpha transparency multiplier for the category badge background fill.
    pub badge_bg_alpha: f32,
    /// Dimension (width and height) of the status indicator dot in logical pixels.
    pub status_dot_size: f32,
    /// Default color of the status indicator dot (e.g. VRAM resident).
    pub status_dot_color: Color,
}

impl Default for AssetCardStyle {
    fn default() -> Self {
        Self {
            bg_idle: Color::rgba(0.07, 0.08, 0.10, 0.95),
            bg_hover: Color::rgba(0.11, 0.12, 0.16, 0.95),
            bg_selected: Color::rgba(0.10, 0.13, 0.18, 0.98),
            border_idle: Color::rgba(0.16, 0.18, 0.23, 0.70),
            border_hover: Color::rgba(0.0, 0.90, 1.0, 0.85),
            border_selected: Color::rgba(0.0, 0.90, 1.0, 1.0),
            border_width_idle: 1.0,
            border_width_selected: 1.5,
            border_radius: 6.0,
            title_color_idle: Color::rgba(0.80, 0.83, 0.90, 1.0),
            title_color_hover: Color::rgba(0.92, 0.94, 0.98, 1.0),
            title_color_selected: Color::WHITE,
            title_font_size: 11.0,
            meta_color: Color::rgba(0.50, 0.54, 0.64, 1.0),
            meta_font_size: 9.5,
            preview_box_bg: Color::rgba(0.04, 0.05, 0.07, 0.95),
            preview_box_border: Color::rgba(0.16, 0.18, 0.24, 0.60),
            preview_box_radius: 4.0,
            preview_box_size: 54.0,
            badge_font_size: 9.0,
            badge_radius: 3.0,
            badge_bg_alpha: 0.16,
            status_dot_size: 7.0,
            status_dot_color: Color::rgba(0.0, 0.90, 1.0, 1.0),
        }
    }
}

/// Preview content specification for an asset card's central display area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssetCardPreview<'a> {
    /// Texture-backed real rendered thumbnail quad.
    Texture {
        /// UV coordinate bounding box `[u_min, v_min, u_max, v_max]` or atlas layer indicator.
        uv: [f32; 4],
        /// Multiplicative tint color.
        tint: Color,
    },
    /// Canonical vector category icon quad centered within the preview box.
    VectorIcon {
        /// UV coordinate bounding box `[u_min, v_min, u_max, v_max]`.
        uv: [f32; 4],
        /// Multiplicative tint color.
        tint: Color,
        /// Width and height of the icon quad in logical pixels.
        size: f32,
    },
    /// Text glyph icon.
    TextGlyph {
        /// Short string or glyph icon.
        text: &'a str,
        /// Glyphic font color.
        color: Color,
        /// Font size in logical points.
        size: f32,
    },
}

/// Visual category badge specification placed in the top-left of an asset card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetCardBadge<'a> {
    /// Short textual badge label (e.g. `MODELS`, `TEXTURES`, `SHADERS`).
    pub text: &'a str,
    /// Accent color used for badge text and semi-transparent background fill.
    pub color: Color,
}

impl<'a> AssetCardBadge<'a> {
    /// Creates a new asset category badge with the specified label and theme color.
    #[inline]
    pub fn new(text: &'a str, color: Color) -> Self {
        Self { text, color }
    }
}

/// Structural layout frame returned by [`AssetCardBuilder`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetCardFrame {
    /// Identifier of the outer card container widget.
    pub card_id: WidgetId,
    /// Bounding rectangle of the entire card.
    pub card_rect: Rect,
    /// Bounding rectangle allocated for the central preview box.
    pub preview_rect: Rect,
}

/// Fluent builder for constructing standardized, hardware-accelerated asset browser cards.
pub struct AssetCardBuilder<'a> {
    rect: Rect,
    name: Option<String>,
    badge: Option<AssetCardBadge<'a>>,
    status_dot: Option<Color>,
    preview: Option<AssetCardPreview<'a>>,
    title: Option<String>,
    metadata: Option<String>,
    is_selected: bool,
    is_hovered: bool,
    hover_border_color: Option<Color>,
    style: AssetCardStyle,
}

impl<'a> AssetCardBuilder<'a> {
    /// Initializes a new asset card builder with the target bounding box.
    #[inline]
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            name: None,
            badge: None,
            status_dot: None,
            preview: None,
            title: None,
            metadata: None,
            is_selected: false,
            is_hovered: false,
            hover_border_color: None,
            style: AssetCardStyle::default(),
        }
    }

    /// Sets the semantic/debug name of the card container node.
    #[inline]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Assigns an optional category pill badge in the top-left corner.
    #[inline]
    pub fn badge(mut self, badge: Option<AssetCardBadge<'a>>) -> Self {
        self.badge = badge;
        self
    }

    /// Configures whether to display a status dot (e.g. VRAM resident indicator) in the top-right corner.
    #[inline]
    pub fn status_dot(mut self, color: Option<Color>) -> Self {
        self.status_dot = color;
        self
    }

    /// Assigns the central preview content (texture thumbnail, vector icon, or text glyph).
    #[inline]
    pub fn preview(mut self, preview: Option<AssetCardPreview<'a>>) -> Self {
        self.preview = preview;
        self
    }

    /// Sets the primary title text of the card (e.g. filename or asset name).
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the bottom metadata label text (e.g. file size "2.4 MB").
    #[inline]
    pub fn metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// Sets whether the card is currently selected.
    #[inline]
    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Sets whether the card is currently hovered by the pointer.
    #[inline]
    pub fn is_hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    /// Overrides the border color when hovered (e.g. to match category accent color).
    #[inline]
    pub fn hover_border_color(mut self, color: Option<Color>) -> Self {
        self.hover_border_color = color;
        self
    }

    /// Applies a custom visual styling configuration.
    #[inline]
    pub fn style(mut self, style: AssetCardStyle) -> Self {
        self.style = style;
        self
    }

    /// Compiles the asset card into the target `UiTree` under `parent_id` and returns layout targets.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> AssetCardFrame {
        // 1. Card Outer Container
        let card_id = tree.create_node();
        let bg_color = if self.is_selected {
            self.style.bg_selected
        } else if self.is_hovered {
            self.style.bg_hover
        } else {
            self.style.bg_idle
        };

        let border_color = if self.is_selected {
            self.style.border_selected
        } else if self.is_hovered {
            self.hover_border_color.unwrap_or(self.style.border_hover)
        } else {
            self.style.border_idle
        };

        let border_width = if self.is_selected {
            self.style.border_width_selected
        } else {
            self.style.border_width_idle
        };

        if let Some(node) = tree.get_mut(card_id) {
            let name_str = self.name.as_deref().unwrap_or("AssetCard");
            node.set_name(name_str);
            node.computed_rect = self.rect;
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.style = Style::new()
                .background(bg_color)
                .border_radius(self.style.border_radius)
                .border(border_width, border_color)
                .clip_children(true);
        }
        let _ = tree.add_child(parent_id, card_id);

        // 2. Category Pill Badge (Top Left)
        if let Some(badge) = self.badge {
            let badge_rect = Rect::new(self.rect.x + 6.0, self.rect.y + 6.0, 38.0, 16.0);
            let badge_id = tree.create_node();
            if let Some(node) = tree.get_mut(badge_id) {
                node.set_name("CategoryBadge");
                node.set_text(badge.text);
                node.font_size = self.style.badge_font_size;
                node.line_height = 16.0;
                node.text_align = TextAlign::Center;
                node.text_color = badge.color;
                node.computed_rect = badge_rect;
                node.style = Style::new()
                    .background(Color::rgba(
                        badge.color.r,
                        badge.color.g,
                        badge.color.b,
                        self.style.badge_bg_alpha,
                    ))
                    .border_radius(self.style.badge_radius);
            }
            let _ = tree.add_child(card_id, badge_id);
        }

        // 3. Status Indicator Dot (Top Right)
        if let Some(dot_col) = self.status_dot {
            let dot_size = self.style.status_dot_size;
            let dot_rect = Rect::new(
                self.rect.right() - dot_size - 7.0,
                self.rect.y + 8.0,
                dot_size,
                dot_size,
            );
            let dot_id = tree.create_node();
            if let Some(node) = tree.get_mut(dot_id) {
                node.set_name("StatusIndicatorDot");
                node.computed_rect = dot_rect;
                node.style = Style::new()
                    .background(dot_col)
                    .border_radius(dot_size * 0.5);
            }
            let _ = tree.add_child(card_id, dot_id);
        }

        // 4. Center Thumbnail / Preview Box
        let box_size = self.style.preview_box_size;
        let box_x = self.rect.x + (self.rect.width - box_size) * 0.5;
        let box_y = self.rect.y + 26.0;
        let preview_rect = Rect::new(box_x, box_y, box_size, box_size);

        let preview_box_id = tree.create_node();
        if let Some(node) = tree.get_mut(preview_box_id) {
            node.set_name("ThumbnailBox");
            node.computed_rect = preview_rect;
            node.style = Style::new()
                .background(self.style.preview_box_bg)
                .border_radius(self.style.preview_box_radius)
                .border(1.0, self.style.preview_box_border);
        }
        let _ = tree.add_child(card_id, preview_box_id);

        if let Some(preview) = self.preview {
            match preview {
                AssetCardPreview::Texture { uv, tint } => {
                    let thumb_id = tree.create_node();
                    if let Some(node) = tree.get_mut(thumb_id) {
                        node.set_name("CardRealThumbnail");
                        node.computed_rect = preview_rect;
                        node.set_texture_uv(uv);
                        node.set_texture_tint(tint);
                        node.style = Style::new().border_radius(self.style.preview_box_radius);
                    }
                    let _ = tree.add_child(preview_box_id, thumb_id);
                }
                AssetCardPreview::VectorIcon { uv, tint, size } => {
                    let icon_x = box_x + (box_size - size) * 0.5;
                    let icon_y = box_y + (box_size - size) * 0.5;
                    let icon_rect = Rect::new(icon_x, icon_y, size, size);
                    let icon_id = tree.create_node();
                    if let Some(node) = tree.get_mut(icon_id) {
                        node.set_name("CardVectorIcon");
                        node.computed_rect = icon_rect;
                        node.set_texture_uv(uv);
                        node.set_texture_tint(tint);
                    }
                    let _ = tree.add_child(preview_box_id, icon_id);
                }
                AssetCardPreview::TextGlyph { text, color, size } => {
                    let glyph_id = tree.create_node();
                    if let Some(node) = tree.get_mut(glyph_id) {
                        node.set_name("CardGlyphIcon");
                        node.computed_rect = preview_rect;
                        node.set_text(text);
                        node.font_size = size;
                        node.line_height = box_size;
                        node.text_align = TextAlign::Center;
                        node.text_color = color;
                    }
                    let _ = tree.add_child(preview_box_id, glyph_id);
                }
            }
        }

        // 5. Truncated Asset Name Label
        let title_color = if self.is_selected {
            self.style.title_color_selected
        } else if self.is_hovered {
            self.style.title_color_hover
        } else {
            self.style.title_color_idle
        };

        if let Some(title) = self.title {
            let name_rect = Rect::new(
                self.rect.x + 4.0,
                self.rect.y + 84.0,
                self.rect.width - 8.0,
                16.0,
            );
            let name_id = tree.create_node();
            if let Some(node) = tree.get_mut(name_id) {
                node.set_name("CardAssetName");
                node.set_text(title);
                node.font_size = self.style.title_font_size;
                node.line_height = 16.0;
                node.text_align = TextAlign::Center;
                node.text_color = title_color;
                node.computed_rect = name_rect;
            }
            let _ = tree.add_child(card_id, name_id);
        }

        // 6. Metadata Badge / Size Label
        if let Some(meta) = self.metadata {
            let meta_rect = Rect::new(
                self.rect.x + 4.0,
                self.rect.y + 102.0,
                self.rect.width - 8.0,
                14.0,
            );
            let meta_id = tree.create_node();
            if let Some(node) = tree.get_mut(meta_id) {
                node.set_name("CardMetadata");
                node.set_text(meta);
                node.font_size = self.style.meta_font_size;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = self.style.meta_color;
                node.computed_rect = meta_rect;
            }
            let _ = tree.add_child(card_id, meta_id);
        }

        AssetCardFrame {
            card_id,
            card_rect: self.rect,
            preview_rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_card_builder_with_texture() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let card_rect = Rect::new(10.0, 10.0, 115.0, 125.0);
        let frame = AssetCardBuilder::new(card_rect)
            .name("Card_model")
            .badge(Some(AssetCardBadge::new("3D", Color::CYAN)))
            .status_dot(Some(Color::GREEN))
            .preview(Some(AssetCardPreview::Texture {
                uv: [0.0, 0.0, 1.0, 2.0],
                tint: Color::WHITE,
            }))
            .title("Character.glb")
            .metadata("1.2 MB")
            .is_selected(true)
            .build(&mut tree, root);

        assert_eq!(frame.card_rect, card_rect);
        let card_node = tree.get(frame.card_id).expect("Card node must exist");
        assert_eq!(card_node.children.len(), 5); // badge, dot, preview box, title, metadata
    }

    #[test]
    fn test_asset_card_builder_with_vector_icon() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let card_rect = Rect::new(0.0, 0.0, 100.0, 120.0);
        let frame = AssetCardBuilder::new(card_rect)
            .preview(Some(AssetCardPreview::VectorIcon {
                uv: [0.0, 0.0, 1.0, 1.0],
                tint: Color::YELLOW,
                size: 28.0,
            }))
            .title("Shader.wgsl")
            .is_hovered(true)
            .build(&mut tree, root);

        assert_eq!(frame.card_rect, card_rect);
        let card_node = tree.get(frame.card_id).expect("Card node must exist");
        assert_eq!(card_node.children.len(), 2); // preview box + title
    }
}