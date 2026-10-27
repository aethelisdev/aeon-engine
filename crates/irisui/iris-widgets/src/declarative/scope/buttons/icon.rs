// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Icon Buttons & Buttons With Leading Icons
//!
//! Provides buttons featuring hardware atlas icons and icon-only triggers on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits an interactive push button with explicit width, custom icon tint, and semantic tag.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `icon_tint` - Idle tint color for the icon.
    /// * `label` - Button caption text.
    /// * `width` - Optional fixed width constraint in logical points.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn button_with_icon_styled_tagged(
        &mut self,
        icon_uv: [f32; 4],
        icon_tint: Color,
        label: impl Into<String>,
        width: Option<f32>,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(6.0)
                .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0))
                .background(if hovered {
                    Color::rgba(0.0, 0.45, 0.60, 0.85)
                } else {
                    Color::rgba(0.12, 0.14, 0.18, 0.90)
                })
                .hover_background(Color::rgba(0.0, 0.45, 0.60, 0.85))
                .border(
                    1.0,
                    if hovered {
                        Color::rgba(0.0, 0.90, 1.0, 0.95)
                    } else {
                        Color::rgba(0.20, 0.24, 0.30, 0.85)
                    },
                )
                .hover_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
                .border_radius(4.0)
                .height(24.0);
            let btn_w = width.unwrap_or_else(|| {
                let text_w = (label_str.chars().count() as f32 * 6.5).ceil().max(20.0);
                36.0 + text_w
            });
            style.width = Some(btn_w);
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(if hovered { Color::WHITE } else { icon_tint });
            node.hover_texture_tint = Some(Color::WHITE);
            node.set_style(Style::new().width(14.0).height(14.0));
        }
        let _ = self.tree.add_child(btn_id, icon_id);

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.interactive = false;
            node.set_text(label_str);
            node.font_size = 10.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.90, 0.92, 0.96, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);
        }
        let _ = self.tree.add_child(btn_id, text_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits an interactive push button with a leading icon quad and text label.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `label` - Button caption text.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn button_with_icon_tagged(
        &mut self,
        icon_uv: [f32; 4],
        label: impl Into<String>,
        tag: u64,
    ) -> WidgetResponse {
        self.button_with_icon_styled_tagged(
            icon_uv,
            Color::rgba(0.0, 0.85, 1.0, 0.95),
            label,
            None,
            tag,
        )
    }

    /// Emits an interactive push button with a leading icon quad and text label using a generated tag.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `label` - Button caption text.
    pub fn button_with_icon(
        &mut self,
        icon_uv: [f32; 4],
        label: impl Into<String>,
    ) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.button_with_icon_tagged(icon_uv, visible, tag)
    }

    /// Emits a compact, small interactive button with a leading icon quad and text label (height: 20px).
    ///
    /// Specifically tailored for action buttons in table rows, tool strips, and header badges.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `label` - Button caption text.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn small_button_with_icon_tagged(
        &mut self,
        icon_uv: [f32; 4],
        label: impl Into<String>,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("SmallIconButton");
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            let style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(3.0)
                .padding_insets(Insets::new(1.0, 6.0, 1.0, 6.0))
                .height(20.0)
                .background(if hovered {
                    Color::rgba(0.0, 0.45, 0.60, 0.85)
                } else {
                    Color::rgba(0.12, 0.16, 0.22, 0.85)
                })
                .hover_background(Color::rgba(0.0, 0.45, 0.60, 0.85))
                .border(
                    1.0,
                    if hovered {
                        Color::rgba(0.0, 0.90, 1.0, 0.95)
                    } else {
                        Color::rgba(0.24, 0.28, 0.36, 0.80)
                    },
                )
                .hover_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
                .border_radius(3.0);
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            });
            node.hover_texture_tint = Some(Color::WHITE);
            node.set_style(Style::new().width(10.0).height(10.0));
        }
        let _ = self.tree.add_child(btn_id, icon_id);

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.interactive = false;
            node.set_text(label_str);
            node.font_size = 9.5;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.85, 0.92, 1.0, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);
        }
        let _ = self.tree.add_child(btn_id, text_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits a compact, small interactive button with a leading icon and text using a generated tag.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `label` - Button caption text.
    pub fn small_button_with_icon(
        &mut self,
        icon_uv: [f32; 4],
        label: impl Into<String>,
    ) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.small_button_with_icon_tagged(icon_uv, visible, tag)
    }

    /// Emits a standalone interactive icon button (square, no text label).
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the icon.
    /// * `size` - Width and height in logical pixels.
    pub fn icon_button(&mut self, icon_uv: [f32; 4], size: f32) -> WidgetResponse {
        let tag_label = format!("IconBtn_{:.1}_{:.1}", icon_uv[0], icon_uv[1]);
        let tag = self.tag_for(&tag_label);
        self.icon_button_tagged(icon_uv, size, tag)
    }

    /// Emits a standalone interactive icon button with a custom semantic tag.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the icon.
    /// * `size` - Width and height in logical pixels.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn icon_button_tagged(&mut self, icon_uv: [f32; 4], size: f32, tag: u64) -> WidgetResponse {
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("IconButton");
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            let style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(size)
                .height(size)
                .background(if hovered {
                    Color::rgba(0.20, 0.24, 0.32, 0.90)
                } else {
                    Color::TRANSPARENT
                })
                .hover_background(Color::rgba(0.20, 0.24, 0.32, 0.90))
                .border_radius(3.0);
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.80, 0.85, 0.92, 0.90)
            });
            node.hover_texture_tint = Some(Color::WHITE);
            node.set_style(Style::new().width(size * 0.70).height(size * 0.70));
        }
        let _ = self.tree.add_child(btn_id, icon_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }
}