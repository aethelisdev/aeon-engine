// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Toolbar Button Builder
//!
//! Provides the configurable [`ToolbarButton`] builder struct implementing [`Widget`].
//! Replaces legacy `toolbar_button_tagged` with a unified builder.
//!

use super::Widget;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
};

/// Configurable builder for toolbar strip buttons.
#[derive(Debug, Clone)]
pub struct ToolbarButton {
    name: &'static str,
    label: String,
    width: Option<f32>,
    tag: Option<u64>,
    is_active: bool,
}

impl ToolbarButton {
    /// Creates a new toolbar button builder with the given caption.
    #[inline]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            name: "ToolbarButton",
            label: label.into(),
            width: None,
            tag: None,
            is_active: false,
        }
    }

    /// Sets the descriptive debug name of the node in the UI tree.
    #[inline]
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets a fixed width constraint for the toolbar button.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets whether this toolbar button represents an active/toggled mode.
    #[inline]
    #[must_use]
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }
}

impl Widget for ToolbarButton {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> WidgetResponse {
        let (visible_text, _) = split_label_id(&self.label);
        let resolved_tag = self.tag.unwrap_or_else(|| scope.tag_for(&self.label));

        let node_id = scope.tree.create_node();
        if let Some(node) = scope.tree.get_mut(node_id) {
            node.set_name(self.name);
            node.tag = resolved_tag;
        }
        let _ = scope.tree.add_child(scope.parent, node_id);
        let (clicked, hovered, _) = scope.check_interaction(node_id);

        if let Some(node) = scope.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(visible_text.to_string());
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered || self.is_active {
                Color::WHITE
            } else {
                Color::rgba(0.90, 0.93, 0.98, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);

            let mut style = Style::new()
                .height(24.0)
                .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                .background(if self.is_active {
                    Color::rgba(0.0, 0.60, 0.90, 0.90)
                } else if hovered {
                    Color::rgba(0.24, 0.29, 0.39, 1.0)
                } else {
                    Color::rgba(0.15, 0.18, 0.25, 0.95)
                })
                .hover_background(Color::rgba(0.24, 0.29, 0.39, 1.0))
                .border(
                    1.0,
                    if hovered {
                        Color::rgba(0.0, 0.85, 1.0, 0.85)
                    } else {
                        Color::rgba(0.30, 0.36, 0.48, 0.75)
                    },
                )
                .hover_border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            if let Some(w) = self.width {
                style = style.width(w);
            }

            node.set_style(style);
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, false);
        if let Some(node) = scope.tree.get(node_id) {
            resp.rect = node.computed_rect;
        }
        resp
    }
}