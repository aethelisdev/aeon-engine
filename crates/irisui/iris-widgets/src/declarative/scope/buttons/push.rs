// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Standard Push Buttons
//!
//! Provides interactive standard push buttons and compact button variants on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits an interactive push button returning immediate [`WidgetResponse`].
    ///
    /// Generates a deterministic persistent tag from `label` to support cross-frame
    /// interaction tracking, hover highlighting, and hit testing without manual identifier management.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    pub fn button(&mut self, label: impl Into<String>) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.button_tagged(visible, tag)
    }

    /// Emits an interactive push button with a custom semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and layout styling internally.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_tagged(&mut self, label: impl Into<String>, tag: u64) -> WidgetResponse {
        self.button_named_tagged("Button", label, tag)
    }

    /// Emits an interactive push button with a custom [`Style`] and semantic tag.
    ///
    /// Automatically applies subtle hover brightening to the background and border
    /// while fully respecting custom sizing, padding, and layout properties from `style`.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `style` - Custom layout and visual appearance style.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_styled_tagged(
        &mut self,
        label: impl Into<String>,
        style: Style,
        tag: u64,
    ) -> WidgetResponse {
        self.button_named_styled_tagged("Button", label, style, tag)
    }

    /// Emits an interactive push button with a descriptive node name, custom [`Style`], and semantic tag.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the UI node for debugging and tree inspection.
    /// * `label` - Button caption text.
    /// * `style` - Custom layout and visual appearance style.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_named_styled_tagged(
        &mut self,
        name: &'static str,
        label: impl Into<String>,
        style: Style,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        let pressed = self.is_pressed(node_id);

        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered || pressed {
                Color::WHITE
            } else {
                Color::rgba(0.0, 0.88, 1.0, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);
            let mut resolved_style = style;
            if resolved_style.hover_background.is_none() {
                let bg = resolved_style.background_color;
                if bg.a > 0.01 {
                    resolved_style.hover_background = Some(Color::rgba(
                        (bg.r * 1.30).min(1.0),
                        (bg.g * 1.30).min(1.0),
                        (bg.b * 1.30).min(1.0),
                        bg.a,
                    ));
                }
            }
            if resolved_style.hover_border.is_none() {
                let bc = resolved_style.border.color;
                if bc.a > 0.01 {
                    let mut b = resolved_style.border;
                    b.color = Color::rgba(
                        (bc.r * 1.35).min(1.0),
                        (bc.g * 1.35).min(1.0),
                        (bc.b * 1.35).min(1.0),
                        bc.a,
                    );
                    resolved_style.hover_border = Some(b);
                }
            }
            if pressed {
                if let Some(hbg) = resolved_style.hover_background {
                    resolved_style.background_color =
                        Color::rgba(hbg.r * 0.82, hbg.g * 0.82, hbg.b * 0.82, hbg.a);
                }
                if let Some(hb) = resolved_style.hover_border {
                    resolved_style.border = hb;
                }
            } else if hovered {
                if let Some(hbg) = resolved_style.hover_background {
                    resolved_style.background_color = hbg;
                }
                if let Some(hb) = resolved_style.hover_border {
                    resolved_style.border = hb;
                }
            }
            node.set_style(resolved_style);
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, false);
        if pressed {
            resp = resp.with_pressed(true);
        }
        resp
    }

    /// Emits an interactive push button with a descriptive node name and a custom semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and layout styling internally.
    /// Uses content-based snug width by default.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the UI node for debugging and tree inspection.
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_named_tagged(
        &mut self,
        name: &'static str,
        label: impl Into<String>,
        tag: u64,
    ) -> WidgetResponse {
        let default_style = Style::new()
            .height(26.0)
            .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
            .background(Color::rgba(0.0, 0.30, 0.42, 0.60))
            .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.60))
            .border_radius(4.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.button_named_styled_tagged(name, label, default_style, tag)
    }

    /// Emits an interactive push button with an explicit width constraint and semantic tag.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `width` - Explicit button width in logical points.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn button_with_width_tagged(
        &mut self,
        label: impl Into<String>,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let mut style = Style::new()
            .height(26.0)
            .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
            .background(Color::rgba(0.0, 0.30, 0.42, 0.60))
            .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.60))
            .border_radius(4.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        style.width = Some(width);
        self.button_named_styled_tagged("Button", label, style, tag)
    }

    /// Emits a compact, small interactive push button (height: 20px, font: 9.5px).
    ///
    /// Designed for dense tables, inspector cards, and action rows.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    pub fn small_button(&mut self, label: impl Into<String>) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.small_button_tagged(visible, tag)
    }

    /// Emits a compact, small interactive push button with a custom semantic tag.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn small_button_tagged(&mut self, label: impl Into<String>, tag: u64) -> WidgetResponse {
        let small_style = Style::new()
            .height(20.0)
            .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
            .background(Color::rgba(0.12, 0.16, 0.22, 0.85))
            .border(1.0, Color::rgba(0.24, 0.28, 0.36, 0.80))
            .border_radius(3.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.button_named_styled_tagged("SmallButton", label, small_style, tag)
    }
}