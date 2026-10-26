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
    icon_uv: Option<[f32; 4]>,
    width: Option<f32>,
    height: Option<f32>,
    tag: Option<u64>,
    is_active: bool,
    is_open: bool,
    tooltip: Option<String>,
    background: Option<Color>,
    hover_background: Option<Color>,
    border_color: Option<Color>,
    hover_border_color: Option<Color>,
    text_color: Option<Color>,
    corner_radius: Option<f32>,
}

impl ToolbarButton {
    /// Creates a new toolbar button builder with the given caption.
    #[inline]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            name: "ToolbarButton",
            label: label.into(),
            icon_uv: None,
            width: None,
            height: None,
            tag: None,
            is_active: false,
            is_open: false,
            tooltip: None,
            background: None,
            hover_background: None,
            border_color: None,
            hover_border_color: None,
            text_color: None,
            corner_radius: None,
        }
    }

    /// Sets the descriptive debug name of the node in the UI tree.
    #[inline]
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets an optional leading texture icon from the canonical editor atlas.
    ///
    /// When specified, renders a 14×14 icon alongside the label text with standard 4px flex gap.
    #[inline]
    #[must_use]
    pub fn icon(mut self, uv: [f32; 4]) -> Self {
        self.icon_uv = Some(uv);
        self
    }

    /// Sets an explicit width constraint for the toolbar button.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets an explicit height constraint for the toolbar button.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
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

    /// Sets whether the associated dropdown popup for this button is currently expanded.
    #[inline]
    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Configures an explicit background fill color overriding the default theme token.
    #[inline]
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Configures an explicit hover background fill color overriding the default theme token.
    #[inline]
    #[must_use]
    pub fn hover_background(mut self, hover_bg: Color) -> Self {
        self.hover_background = Some(hover_bg);
        self
    }

    /// Configures an explicit border outline color overriding the default theme token.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, border: Color) -> Self {
        self.border_color = Some(border);
        self
    }

    /// Configures an explicit hover border outline color overriding the default theme token.
    #[inline]
    #[must_use]
    pub fn hover_border_color(mut self, hover_border: Color) -> Self {
        self.hover_border_color = Some(hover_border);
        self
    }

    /// Configures an explicit text caption color overriding the default theme token.
    #[inline]
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Configures an explicit corner radius overriding the default theme token.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Sets an optional tooltip text displayed on hover after the delay threshold.
    #[inline]
    #[must_use]
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Alias for [`Self::tooltip`] conforming to egui standard convention.
    #[inline]
    #[must_use]
    pub fn on_hover_text(self, text: impl Into<String>) -> Self {
        self.tooltip(text)
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
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
        }
        let _ = scope.tree.add_child(scope.parent, node_id);
        let (clicked, hovered, _) = scope.check_interaction(node_id);

        let theme = scope.theme();
        let is_highlighted = self.is_open || self.is_active || hovered;

        let bg = if let Some(custom_bg) = self.background {
            if hovered {
                self.hover_background.unwrap_or(theme.button_hover_bg)
            } else {
                custom_bg
            }
        } else if self.is_active {
            theme.button_active_bg
        } else if self.is_open || hovered {
            theme.button_hover_bg
        } else {
            theme.button_idle_bg
        };

        let border = if let Some(custom_border) = self.border_color {
            if hovered || self.is_open {
                self.hover_border_color.unwrap_or(theme.button_hover_border)
            } else {
                custom_border
            }
        } else if self.is_active || self.is_open || hovered {
            theme.button_hover_border
        } else {
            theme.button_border
        };

        let hover_bg = self.hover_background.unwrap_or(theme.button_hover_bg);
        let hover_border = self.hover_border_color.unwrap_or(theme.button_hover_border);

        let default_text_color = if is_highlighted {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let resolved_text_color = self.text_color.unwrap_or(default_text_color);

        let radius = self.corner_radius.unwrap_or(theme.corner_radius_md);
        let h = self.height.unwrap_or(24.0);

        let text_w = if visible_text.is_empty() {
            0.0
        } else {
            iris_text::measure_text_advance(visible_text, 11.0).ceil()
        };

        let resolved_width = self.width.or_else(|| {
            if self.icon_uv.is_some() {
                let inner_w = if text_w > 0.0 {
                    14.0 + 4.0 + text_w
                } else {
                    14.0
                };
                Some(inner_w + 12.0)
            } else {
                None
            }
        });

        if let Some(node) = scope.tree.get_mut(node_id) {
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(4.0)
                .height(h)
                .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
                .background(bg)
                .border(1.0, border)
                .border_radius(radius)
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));

            if !self.is_active && !self.is_open {
                style = style
                    .hover_background(hover_bg)
                    .hover_border(1.0, hover_border);
            }

            if let Some(w) = resolved_width {
                style = style.width(w);
            }

            node.set_style(style);

            if self.icon_uv.is_none() {
                node.set_text(visible_text.to_string());
                node.font_size = 11.0;
                node.line_height = h;
                node.text_align = TextAlign::Center;
                node.text_color = resolved_text_color;
                node.hover_text_color = Some(theme.text_primary);
            }
        }

        if let Some(uv) = self.icon_uv {
            let icon_id = scope.tree.create_node();
            if let Some(node) = scope.tree.get_mut(icon_id) {
                node.interactive = false;
                node.set_name("ToolbarButtonIcon");
                node.set_texture_uv(uv);
                node.set_texture_tint(resolved_text_color);
                node.hover_texture_tint = Some(theme.text_primary);
                node.set_style(Style::new().width(14.0).height(14.0));
            }
            let _ = scope.tree.add_child(node_id, icon_id);

            if !visible_text.is_empty() {
                let txt_id = scope.tree.create_node();
                if let Some(node) = scope.tree.get_mut(txt_id) {
                    node.interactive = false;
                    node.set_name("ToolbarButtonText");
                    node.set_text(visible_text.to_string());
                    node.font_size = 11.0;
                    node.line_height = h;
                    node.text_align = TextAlign::Center;
                    node.text_color = resolved_text_color;
                    node.hover_text_color = Some(theme.text_primary);
                    node.set_style(Style::new().width(text_w).height(h));
                }
                let _ = scope.tree.add_child(node_id, txt_id);
            }
        }

        if let Some(tip) = self.tooltip {
            scope.tooltip_for(node_id, move |ui| {
                ui.tooltip_label(tip);
            });
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, false);
        if let Some(node) = scope.tree.get(node_id) {
            resp.rect = node.computed_rect;
        }
        resp
    }
}