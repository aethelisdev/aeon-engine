// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Checkbox Builder
//!
//! Provides the configurable [`Checkbox`] builder struct implementing [`Widget`].
//! Replaces legacy `checkbox_tagged` and `checkbox_val_tagged` with a unified builder.

use super::Widget;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole,
};

/// Configurable builder for interactive checkboxes.
#[derive(Debug, Clone)]
pub struct Checkbox {
    name: &'static str,
    label: String,
    checked: bool,
    tag: Option<u64>,
}

impl Checkbox {
    /// Creates a new checkbox builder with caption and current boolean value.
    #[inline]
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            name: "Checkbox",
            label: label.into(),
            checked,
            tag: None,
        }
    }

    /// Sets the descriptive debug name of the node in the UI tree.
    #[inline]
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }
}

impl Widget for Checkbox {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> WidgetResponse {
        let (visible_text, _) = split_label_id(&self.label);
        let resolved_tag = self.tag.unwrap_or_else(|| scope.tag_for(&self.label));

        let node_id = scope.tree.create_node();
        let _ = scope.tree.add_child(scope.parent, node_id);
        if let Some(node) = scope.tree.get_mut(node_id) {
            node.set_name(self.name);
            node.tag = resolved_tag;
        }

        let (clicked, hovered, _) = scope.check_interaction(node_id);
        let pressed = scope.is_pressed(node_id);
        let theme = scope.theme();
        let is_effective_checked = self.checked ^ clicked;

        if let Some(node) = scope.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Checkbox;
            node.cursor = Some(WidgetCursor::Pointer);

            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .gap(6.0)
                .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
                .border_radius(theme.corner_radius_md);
            style.hover_background = Some(Color::rgba(0.25, 0.28, 0.35, 0.30));
            if pressed {
                style.background_color = Color::rgba(0.20, 0.22, 0.28, 0.50);
            } else if hovered && let Some(hbg) = style.hover_background {
                style.background_color = hbg;
            }
            node.set_style(style);
        }

        // 1. Box indicator node
        let box_id = scope.tree.create_node();
        let _ = scope.tree.add_child(node_id, box_id);
        if let Some(node) = scope.tree.get_mut(box_id) {
            node.interactive = false;
            node.role = WidgetRole::Default;
            node.set_name("CheckboxIndicator");

            let box_bg = if is_effective_checked {
                theme.button_active_bg
            } else if hovered {
                theme.button_hover_bg
            } else {
                theme.button_idle_bg
            };

            let border_color = if is_effective_checked {
                theme.accent
            } else if hovered {
                theme.button_hover_border
            } else {
                theme.button_border
            };

            node.set_style(
                Style::new()
                    .width(16.0)
                    .height(16.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .background(box_bg)
                    .border(1.0, border_color)
                    .border_radius(theme.corner_radius_sm),
            );

            if is_effective_checked {
                node.set_text("✓");
                node.font_size = 11.0;
                node.line_height = 14.0;
                node.text_align = TextAlign::Center;
                node.text_color = theme.text_primary;
            } else {
                node.set_text("");
            }
        }

        // 2. Label text node
        if !visible_text.is_empty() {
            let label_id = scope.tree.create_node();
            let _ = scope.tree.add_child(node_id, label_id);
            if let Some(node) = scope.tree.get_mut(label_id) {
                node.interactive = false;
                node.role = WidgetRole::Default;
                node.set_name("CheckboxLabel");
                node.set_text(visible_text.to_string());
                node.font_size = 12.0;
                node.line_height = 16.0;
                node.text_color = if hovered {
                    theme.text_primary
                } else {
                    theme.text_secondary
                };
            }
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, pressed);
        if let Some(node) = scope.tree.get(node_id) {
            resp.rect = node.computed_rect;
        }
        resp
    }
}