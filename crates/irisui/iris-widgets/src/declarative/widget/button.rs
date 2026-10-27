// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Push Button Builder
//!
//! Provides the configurable [`Button`] builder struct implementing [`Widget`].
//! Replaces legacy `button_tagged`, `button_styled_tagged`, `button_named_styled_tagged`,
//! `small_button_tagged`, and `button_with_width_tagged` with a single unified builder.

use super::Widget;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{
    AlignItems, Border, Color, CornerRadii, Insets, JustifyContent, Style, TextAlign, UiLayer,
    WidgetCursor, WidgetRole,
};

/// Configurable builder for interactive push buttons.
///
/// Supports visual overrides, semantic tags, explicit debug names, sizing constraints,
/// and compact small button styling via fluent chained method calls.
#[derive(Debug, Clone)]
pub struct Button {
    name: &'static str,
    label: String,
    tag: Option<u64>,
    layer: Option<UiLayer>,
    style: Option<Style>,
    fill: Option<Color>,
    text_color: Option<Color>,
    border_color: Option<Color>,
    corner_radius: Option<f32>,
    width: Option<f32>,
    min_width: Option<f32>,
    height: Option<f32>,
    padding: Option<Insets>,
    is_small: bool,
    is_active: bool,
    is_flex: bool,
    icon_uv: Option<[f32; 4]>,
    icon_size: Option<f32>,
    tooltip: Option<String>,
}

impl Button {
    /// Creates a new push button builder with the specified label caption.
    ///
    /// # Arguments
    /// * `label` - Button caption text (can include `###id` suffix for ID derivation).
    #[inline]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            name: "Button",
            label: label.into(),
            tag: None,
            layer: None,
            style: None,
            fill: None,
            text_color: None,
            border_color: None,
            corner_radius: None,
            width: None,
            min_width: None,
            height: None,
            padding: None,
            is_small: false,
            is_active: false,
            is_flex: false,
            icon_uv: None,
            icon_size: None,
            tooltip: None,
        }
    }

    /// Configures this button as an active/selected toggle variant.
    #[inline]
    #[must_use]
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Configures this button to expand and fill available flex space (`flex_grow: 1.0`).
    #[inline]
    #[must_use]
    pub fn flex(mut self, flex: bool) -> Self {
        self.is_flex = flex;
        self
    }

    /// Sets the descriptive debug name of the node in the UI tree.
    #[inline]
    #[must_use]
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction dispatching.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Explicitly overrides the destination compositing layer (e.g. [`UiLayer::Modal`], [`UiLayer::Popup`]).
    #[inline]
    #[must_use]
    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = Some(layer);
        self
    }

    /// Sets the custom background fill color of the button.
    #[inline]
    #[must_use]
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the button text color.
    #[inline]
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Sets the button border color.
    #[inline]
    #[must_use]
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Sets the button corner rounding radius.
    #[inline]
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Sets the button corner rounding radius (alias for [`Self::corner_radius`]).
    #[inline]
    #[must_use]
    pub fn border_radius(self, radius: f32) -> Self {
        self.corner_radius(radius)
    }

    /// Sets a fixed button width constraint.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a minimum button width constraint.
    #[inline]
    #[must_use]
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Sets a fixed button height constraint.
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets both width and height constraints simultaneously.
    ///
    /// # Arguments
    /// * `width` - Fixed horizontal dimension in physical pixels.
    /// * `height` - Fixed vertical dimension in physical pixels.
    #[inline]
    #[must_use]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Sets custom internal padding for the button content.
    #[inline]
    #[must_use]
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Configures this button as a compact small button variant.
    #[inline]
    #[must_use]
    pub fn small(mut self) -> Self {
        self.is_small = true;
        self
    }

    /// Attaches vector icon UV coordinates and display dimensions.
    #[inline]
    #[must_use]
    pub fn icon(mut self, uv: [f32; 4], size: f32) -> Self {
        self.icon_uv = Some(uv);
        self.icon_size = Some(size);
        self
    }

    /// Overrides the baseline layout style of the button.
    #[inline]
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
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

impl Widget for Button {
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
        let pressed = scope.is_pressed(node_id);

        let theme = scope.theme();

        let mut base_style = self.style.unwrap_or_else(|| {
            if self.is_small {
                Style::new()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(20.0)
                    .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                    .border_radius(theme.corner_radius_sm)
                    .background(theme.button_idle_bg)
                    .border(1.0, theme.button_border)
            } else {
                Style::new()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(28.0)
                    .padding_insets(Insets::new(4.0, 10.0, 4.0, 10.0))
                    .border_radius(theme.corner_radius_md)
                    .background(Color::rgba(0.14, 0.16, 0.20, 0.90))
                    .border(1.0, Color::rgba(0.24, 0.26, 0.32, 0.60))
            }
        });

        if let Some(fill) = self.fill {
            base_style.background_color = fill;
        }
        if let Some(border) = self.border_color {
            base_style.border.color = border;
        }
        if let Some(radius) = self.corner_radius {
            base_style.corner_radii = CornerRadii::all(radius);
        }
        if let Some(w) = self.width {
            base_style.width = Some(w);
        }
        if let Some(min_w) = self.min_width {
            base_style.min_width = Some(min_w);
        }
        if let Some(h) = self.height {
            base_style.height = Some(h);
        }
        if let Some(p) = self.padding {
            base_style.padding = p;
        } else {
            if let Some(w) = self.width {
                let horiz = base_style.padding.left + base_style.padding.right;
                if horiz > w {
                    let side = (w * 0.5 - 1.0).max(0.0);
                    base_style.padding.left = side;
                    base_style.padding.right = side;
                }
            }
            if let Some(h) = self.height {
                let vert = base_style.padding.top + base_style.padding.bottom;
                if vert > h {
                    let side = (h * 0.5 - 1.0).max(0.0);
                    base_style.padding.top = side;
                    base_style.padding.bottom = side;
                }
            }
        }
        if self.is_flex {
            base_style.flex_grow = 1.0;
        }
        if self.is_active {
            base_style.background_color = theme.button_active_bg;
            base_style.border.color = theme.accent;
        }

        if let Some(node) = scope.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            if let Some(l) = self.layer {
                node.layer = l;
            }
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(visible_text.to_string());
            node.font_size = if self.is_small { 10.0 } else { 11.0 };
            node.line_height = if self.is_small { 12.0 } else { 14.0 };
            node.text_align = TextAlign::Center;

            let default_text = if self.is_active {
                theme.text_primary
            } else if self.is_small {
                theme.text_secondary
            } else if hovered || pressed {
                theme.text_primary
            } else {
                theme.text_secondary
            };

            node.text_color = self.text_color.unwrap_or(default_text);
            node.hover_text_color = Some(Color::WHITE);

            if base_style.hover_background.is_none() {
                let bg = base_style.background_color;
                if bg.a > 0.01 {
                    base_style.hover_background = Some(Color::rgba(
                        (bg.r * 1.30).min(1.0),
                        (bg.g * 1.30).min(1.0),
                        (bg.b * 1.30).min(1.0),
                        bg.a,
                    ));
                }
            }
            if base_style.hover_border.is_none() {
                let bc = base_style.border.color;
                if bc.a > 0.01 {
                    base_style.hover_border = Some(Border::uniform(
                        base_style.border.width.top,
                        Color::rgba(
                            (bc.r * 1.40).min(1.0),
                            (bc.g * 1.40).min(1.0),
                            (bc.b * 1.40).min(1.0),
                            bc.a,
                        ),
                    ));
                }
            }

            if let Some(uv) = self.icon_uv {
                node.texture_uv = Some(uv);
                node.texture_tint = Some(Color::WHITE);
                if let Some(size) = self.icon_size {
                    base_style.width = Some(size);
                    base_style.height = Some(size);
                }
            }

            node.style = base_style;
        }

        if let Some(tip) = self.tooltip {
            scope.tooltip_for(node_id, move |ui| {
                ui.tooltip_label(tip);
            });
        }

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, false);
        if pressed {
            resp = resp.with_pressed(true);
        }
        if let Some(node) = scope.tree.get(node_id) {
            resp.rect = node.computed_rect;
        }
        resp
    }
}