// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toggle Pills & Filter Pills
//!
//! Provides mode selection pills, segmented toggle tabs, and severity filter pills on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits an interactive pill toggle button for mode or option selection.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `is_active` - Whether this option is currently selected.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn toggle_pill_tagged(&mut self, label: &str, is_active: bool, tag: u64) -> WidgetResponse {
        self.emit_toggle_pill(label, is_active, tag, false)
    }

    /// Emits an interactive toggle pill button that expands to fill available flex space (`flex_grow: 1.0`).
    ///
    /// Ideal for segmented mode selector button groups where items evenly divide the row width.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `is_active` - Whether this option is currently selected.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn toggle_pill_flex_tagged(
        &mut self,
        label: &str,
        is_active: bool,
        tag: u64,
    ) -> WidgetResponse {
        self.emit_toggle_pill(label, is_active, tag, true)
    }

    fn emit_toggle_pill(
        &mut self,
        label: &str,
        is_active: bool,
        tag: u64,
        is_flex: bool,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;
            let is_effective_active = is_active ^ clicked;
            let (bg, border, text_color) = if is_effective_active {
                (
                    Color::rgba(0.06, 0.22, 0.32, 0.75),
                    Color::rgba(0.14, 0.65, 0.95, 0.65),
                    Color::rgba(0.25, 0.85, 1.0, 1.0),
                )
            } else if hovered {
                (
                    Color::rgba(0.18, 0.21, 0.28, 0.85),
                    Color::rgba(0.35, 0.40, 0.50, 0.65),
                    Color::WHITE,
                )
            } else {
                (
                    Color::rgba(0.11, 0.13, 0.17, 0.70),
                    Color::rgba(0.22, 0.25, 0.32, 0.50),
                    Color::rgba(0.65, 0.70, 0.78, 1.0),
                )
            };
            node.text_color = text_color;
            let mut style = Style::new()
                .height(24.0)
                .background(bg)
                .border(1.0, border)
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);
            if !is_effective_active {
                style = style
                    .hover_background(Color::rgba(0.18, 0.21, 0.28, 0.85))
                    .hover_border(1.0, Color::rgba(0.35, 0.40, 0.50, 0.65));
                node.hover_text_color = Some(Color::WHITE);
            }
            if is_flex {
                style = style
                    .flex_grow(1.0)
                    .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0));
            } else {
                style = style.padding_insets(Insets::new(2.0, 10.0, 2.0, 10.0));
            }
            node.set_style(style);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a severity-colored filter pill tab for toolbar strips.
    ///
    /// Unlike solid toggle pills, uses an elevated dark slate background (`rgba(0.12, 0.16, 0.24, 0.95)`)
    /// and a severity-specific accent outline border (`accent_color`) when active, preventing visual overload.
    ///
    /// # Arguments
    /// * `label` - Filter caption text (e.g. `"Errors (1)"`).
    /// * `is_active` - Whether this filter level is currently selected.
    /// * `accent_color` - Severity accent color applied to the outline border when active.
    /// * `width` - Explicit button width in physical pixels.
    /// * `tag` - Semantic identifier inspected during interaction event dispatching.
    pub fn filter_pill_tagged(
        &mut self,
        label: &str,
        is_active: bool,
        accent_color: Color,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;

            let is_effective_active = is_active || clicked;
            let (bg, border_c, border_w, text_color) = if is_effective_active {
                let bg_c = Color::rgba(
                    (accent_color.r * 0.45).clamp(0.08, 0.50),
                    (accent_color.g * 0.45).clamp(0.08, 0.50),
                    (accent_color.b * 0.45).clamp(0.08, 0.50),
                    0.52,
                );
                (bg_c, accent_color, 1.5, Color::WHITE)
            } else if hovered {
                let bg_c = Color::rgba(
                    (accent_color.r * 0.28).clamp(0.05, 0.35),
                    (accent_color.g * 0.28).clamp(0.05, 0.35),
                    (accent_color.b * 0.28).clamp(0.05, 0.35),
                    0.32,
                );
                (
                    bg_c,
                    Color::rgba(accent_color.r, accent_color.g, accent_color.b, 0.75),
                    1.0,
                    Color::WHITE,
                )
            } else {
                let bg_c = Color::rgba(
                    (accent_color.r * 0.18).clamp(0.03, 0.22),
                    (accent_color.g * 0.18).clamp(0.03, 0.22),
                    (accent_color.b * 0.18).clamp(0.03, 0.22),
                    0.22,
                );
                let txt_c = Color::rgba(
                    (0.70 + accent_color.r * 0.30).min(1.0),
                    (0.70 + accent_color.g * 0.30).min(1.0),
                    (0.70 + accent_color.b * 0.30).min(1.0),
                    1.0,
                );
                (
                    bg_c,
                    Color::rgba(accent_color.r, accent_color.g, accent_color.b, 0.42),
                    1.0,
                    txt_c,
                )
            };

            node.text_color = text_color;
            let mut style = Style::new()
                .width(width)
                .height(24.0)
                .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                .background(bg)
                .border(border_w, border_c)
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);
            if !is_effective_active {
                let hover_bg_c = Color::rgba(
                    (accent_color.r * 0.28).clamp(0.05, 0.35),
                    (accent_color.g * 0.28).clamp(0.05, 0.35),
                    (accent_color.b * 0.28).clamp(0.05, 0.35),
                    0.32,
                );
                let hover_border_c =
                    Color::rgba(accent_color.r, accent_color.g, accent_color.b, 0.75);
                style = style
                    .hover_background(hover_bg_c)
                    .hover_border(1.0, hover_border_c);
                node.hover_text_color = Some(Color::WHITE);
            }
            node.set_style(style);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }
}