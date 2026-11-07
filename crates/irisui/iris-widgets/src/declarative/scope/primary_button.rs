// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Primary Action Button Widget
//!
//! Provides a high-prominence primary action button on [`UiScope`] featuring lead icons,
//! prominent accent borders, hover transitions, and an optional bottom status feedback banner.
//! Adheres strictly to the centralized [`iris_core::Theme`] token system with zero hardcoded colors.

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, JustifyContent, Style, TextAlign, WidgetRole};

/// Configuration options for emitting a primary hero action button.
#[derive(Debug, Clone)]
pub struct PrimaryButtonConfig<'a> {
    /// Button text caption.
    pub label: &'a str,
    /// Optional GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    pub atlas_icon: Option<[f32; 4]>,
    /// Persistent 64-bit semantic tag for click detection.
    pub tag: u64,
    /// Total button width in logical pixels (defaults to `420.0`).
    pub width: f32,
    /// Total button height in logical pixels (defaults to `42.0`).
    pub height: f32,
    /// Optional inline status message text rendered below the button.
    pub status_message: Option<&'a str>,
    /// Optional status message text color (defaults to `theme.warning`).
    pub status_color: Option<Color>,
}

impl<'a> PrimaryButtonConfig<'a> {
    /// Creates a new primary button configuration with the specified text label.
    #[inline]
    #[must_use]
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            atlas_icon: None,
            tag: 0,
            width: 420.0,
            height: 42.0,
            status_message: None,
            status_color: None,
        }
    }

    /// Sets the GPU SDF atlas vector icon coordinates for the lead button icon.
    #[inline]
    #[must_use]
    pub const fn atlas_icon(mut self, uv: [f32; 4]) -> Self {
        self.atlas_icon = Some(uv);
        self
    }

    /// Sets the persistent 64-bit semantic tag for click detection.
    #[inline]
    #[must_use]
    pub const fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Overrides the button width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Overrides the button height in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Attaches an inline status feedback message rendered below the button.
    #[inline]
    #[must_use]
    pub const fn status_message(mut self, msg: Option<&'a str>) -> Self {
        self.status_message = msg;
        self
    }

    /// Sets a custom text color for the inline status feedback message.
    #[inline]
    #[must_use]
    pub const fn status_color(mut self, color: Color) -> Self {
        self.status_color = Some(color);
        self
    }
}

impl UiScope<'_> {
    /// Emits a prominent primary action button block in a single declarative expression.
    pub fn primary_button(
        &mut self,
        node_name: impl Into<String>,
        config: PrimaryButtonConfig<'_>,
    ) -> WidgetResponse {
        let theme = self.theme();
        let status_color = config.status_color.unwrap_or(theme.warning);

        self.container_named(node_name, Style::new().flex_col().gap(10.0), |col| {
            col.container_tagged(
                "PrimaryButtonInteractive",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(config.width)
                    .height(config.height)
                    .gap(8.0)
                    .background(theme.button_idle_bg)
                    .hover_background(theme.button_hover_bg)
                    .border_radius(theme.corner_radius_md)
                    .border(1.5, theme.accent),
                WidgetRole::Button,
                config.tag,
                |btn| {
                    if let Some(uv) = config.atlas_icon {
                        btn.icon_named("PrimaryButtonIcon", uv, theme.accent, 16.0);
                    }
                    btn.label_styled_passive(
                        "PrimaryButtonText",
                        config.label,
                        13.0,
                        theme.text_primary,
                        TextAlign::Center,
                        Style::new().width(180.0).height(20.0),
                    );
                },
            );

            if let Some(status_text) = config.status_message {
                col.label_styled_passive(
                    "PrimaryButtonStatus",
                    status_text,
                    11.0,
                    status_color,
                    TextAlign::Left,
                    Style::new().width(config.width).height(18.0),
                );
            }
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_primary_button_creation() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(&mut tree, root);

        let resp = scope.primary_button(
            "CreateBtn",
            PrimaryButtonConfig::new("Create Project")
                .tag(0x200)
                .status_message(Some("Ready")),
        );

        assert_eq!(
            tree.get(resp.id).map(|n| n.name.as_deref()),
            Some(Some("CreateBtn"))
        );
        let has_text = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("PrimaryButtonText"));
        let has_status = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("PrimaryButtonStatus"));
        assert!(has_text, "Primary button label must exist");
        assert!(has_status, "Primary button status must exist");
    }
}