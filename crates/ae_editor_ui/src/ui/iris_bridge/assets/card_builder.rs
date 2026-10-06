// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated Asset Card Widget & Builder
//!
//! Provides standardized, GPU SDF-rendered asset thumbnail cards for content browsers,
//! project drawers, and media libraries within the Aeon Engine editor UI.
//!
//! Handles category pill badges, VRAM/memory status indicators, texture thumbnails,
//! canonical vector icons, truncated titles, metadata labels, and selection/hover states.
//!

use irisui::prelude::{
    AlignItems, Color, Insets, JustifyContent, Rect, Style, TextAlign, UiScope, UiTree, WidgetId,
    WidgetRole,
};

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
    #[must_use]
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
    name: Option<&'static str>,
    badge: Option<AssetCardBadge<'a>>,
    status_dot: Option<Color>,
    preview: Option<AssetCardPreview<'a>>,
    title: Option<&'a str>,
    metadata: Option<&'a str>,
    is_selected: bool,
    is_hovered: bool,
    hover_border_color: Option<Color>,
    style: AssetCardStyle,
    tag: Option<u64>,
    parent_rect: Option<Rect>,
}

impl<'a> AssetCardBuilder<'a> {
    /// Initializes a new asset card builder with the target bounding box.
    #[inline]
    #[must_use]
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
            tag: None,
            parent_rect: None,
        }
    }

    /// Assigns the parent viewport rectangle to calculate relative offsets for absolute positioning.
    #[inline]
    #[must_use]
    pub fn parent_rect(mut self, parent_rect: Rect) -> Self {
        self.parent_rect = Some(parent_rect);
        self
    }

    /// Sets the 64-bit semantic tag for zero-allocation hit-testing.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets the semantic/debug name of the card container node.
    #[inline]
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    /// Assigns an optional category pill badge in the top-left corner.
    #[inline]
    #[must_use]
    pub fn badge(mut self, badge: Option<AssetCardBadge<'a>>) -> Self {
        self.badge = badge;
        self
    }

    /// Configures whether to display a status dot (e.g. VRAM resident indicator) in the top-right corner.
    #[inline]
    #[must_use]
    pub fn status_dot(mut self, color: Option<Color>) -> Self {
        self.status_dot = color;
        self
    }

    /// Assigns the central preview content (texture thumbnail, vector icon, or text glyph).
    #[inline]
    #[must_use]
    pub fn preview(mut self, preview: Option<AssetCardPreview<'a>>) -> Self {
        self.preview = preview;
        self
    }

    /// Sets the primary title text of the card (e.g. filename or asset name).
    #[inline]
    #[must_use]
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Sets the bottom metadata label text (e.g. file size "2.4 MB").
    #[inline]
    #[must_use]
    pub fn metadata(mut self, metadata: &'a str) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets whether the card is currently selected.
    #[inline]
    #[must_use]
    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    /// Sets whether the card is currently hovered by the pointer.
    #[inline]
    #[must_use]
    pub fn is_hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    /// Overrides the border color when hovered (e.g. to match category accent color).
    #[inline]
    #[must_use]
    pub fn hover_border_color(mut self, color: Option<Color>) -> Self {
        self.hover_border_color = color;
        self
    }

    /// Applies a custom visual styling configuration.
    #[inline]
    #[must_use]
    pub fn style(mut self, style: AssetCardStyle) -> Self {
        self.style = style;
        self
    }

    /// Compiles the asset card into the target `UiTree` under `parent_id` via a declarative scope.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> AssetCardFrame {
        let mut scope = UiScope::new(tree, parent_id);
        self.build_scope(&mut scope)
    }

    /// Compiles the asset card directly into an active declarative [`UiScope`].
    ///
    /// Uses 100% declarative UI scope primitives with zero imperative node allocations.
    pub fn build_scope(self, scope: &mut UiScope<'_>) -> AssetCardFrame {
        let card_w = self.rect.width;
        let card_h = self.rect.height;
        let p_rect = self
            .parent_rect
            .or_else(|| scope.tree().get(scope.parent()).map(|p| p.computed_rect))
            .unwrap_or(Rect::ZERO);
        let rel_x = self.rect.x - p_rect.x;
        let rel_y = self.rect.y - p_rect.y;

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

        let card_style = Style::new()
            .position_absolute()
            .left(rel_x)
            .top(rel_y)
            .width(card_w)
            .height(card_h)
            .flex_col()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(6.0, 6.0, 6.0, 6.0))
            .gap(3.0)
            .background(bg_color)
            .border_radius(self.style.border_radius)
            .border(border_width, border_color)
            .clip_children(true);

        let name_str = self.name.unwrap_or("AssetCard");

        let box_size = self.style.preview_box_size;
        let box_rel_x = (self.rect.width - box_size) * 0.5;
        let box_x = self.rect.x + box_rel_x;
        let box_y = self.rect.y + 26.0;
        let preview_rect = Rect::new(box_x, box_y, box_size, box_size);

        let badge = self.badge;
        let status_dot = self.status_dot;
        let preview = self.preview;
        let title = self.title;
        let metadata = self.metadata;
        let style = self.style;
        let is_selected = self.is_selected;
        let is_hovered = self.is_hovered;

        let card_body = |card_scope: &mut UiScope<'_>| {
            // 1. Top Header Row (flex_row, justify_content(SpaceBetween))
            let has_header = badge.is_some() || status_dot.is_some();
            if has_header {
                card_scope.container_named(
                    "CardHeaderRow",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .width(card_w - 12.0)
                        .height(16.0),
                    |hdr_scope| {
                        if let Some(b) = badge {
                            hdr_scope.container_named(
                                "CategoryBadge",
                                Style::new()
                                    .flex_row()
                                    .align_items(AlignItems::Center)
                                    .justify_content(JustifyContent::Center)
                                    .padding_insets(Insets::new(1.0, 4.0, 1.0, 4.0))
                                    .background(Color::rgba(
                                        b.color.r,
                                        b.color.g,
                                        b.color.b,
                                        style.badge_bg_alpha,
                                    ))
                                    .border_radius(style.badge_radius),
                                |b_scope| {
                                    b_scope.label_styled_passive(
                                        "BadgeText",
                                        b.text,
                                        style.badge_font_size,
                                        b.color,
                                        TextAlign::Center,
                                        Style::new(),
                                    );
                                },
                            );
                        } else {
                            hdr_scope.empty_box_passive(Style::new().width(1.0).height(1.0));
                        }

                        if let Some(dot_col) = status_dot {
                            let dot_size = style.status_dot_size;
                            hdr_scope.empty_box_passive_named(
                                "StatusIndicatorDot",
                                Style::new()
                                    .width(dot_size)
                                    .height(dot_size)
                                    .background(dot_col)
                                    .border_radius(dot_size * 0.5),
                            );
                        }
                    },
                );
            }

            // 2. Middle Thumbnail / Preview Box (Centered flex container)
            card_scope.container_named(
                "ThumbnailBox",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(box_size)
                    .height(box_size)
                    .background(style.preview_box_bg)
                    .border_radius(style.preview_box_radius)
                    .border(1.0, style.preview_box_border),
                |prev_scope| {
                    if let Some(p) = preview {
                        match p {
                            AssetCardPreview::Texture { uv, tint } => {
                                prev_scope.icon_named("CardRealThumbnail", uv, tint, box_size);
                            }
                            AssetCardPreview::VectorIcon { uv, tint, size } => {
                                prev_scope.icon_named("CardVectorIcon", uv, tint, size);
                            }
                            AssetCardPreview::TextGlyph { text, color, size } => {
                                prev_scope.label_styled_passive(
                                    "CardGlyphIcon",
                                    text,
                                    size,
                                    color,
                                    TextAlign::Center,
                                    Style::new(),
                                );
                            }
                        }
                    }
                },
            );

            // 3. Truncated Asset Name Label (Centered)
            let title_color = if is_selected {
                style.title_color_selected
            } else if is_hovered {
                style.title_color_hover
            } else {
                style.title_color_idle
            };

            if let Some(t) = title {
                card_scope.label_styled_passive(
                    "CardAssetName",
                    t,
                    style.title_font_size,
                    title_color,
                    TextAlign::Center,
                    Style::new().width(card_w - 8.0).height(16.0),
                );
            }

            // 4. Metadata Badge / Size Label (Centered)
            if let Some(m) = metadata {
                card_scope.label_styled_passive(
                    "CardMetadata",
                    m,
                    style.meta_font_size,
                    style.meta_color,
                    TextAlign::Center,
                    Style::new().width(card_w - 8.0).height(14.0),
                );
            }
        };

        let card_id = if let Some(tag) = self.tag {
            scope.container_tagged(name_str, card_style, WidgetRole::Button, tag, card_body)
        } else {
            scope.container_named(name_str, card_style, card_body)
        };

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
        assert_eq!(card_node.children.len(), 4); // CardHeaderRow, ThumbnailBox, title, metadata
        let header_node = tree.get(card_node.children[0]).expect("Header exists");
        assert_eq!(header_node.children.len(), 2); // badge, dot
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
        assert_eq!(card_node.children.len(), 2); // ThumbnailBox + title
    }
}