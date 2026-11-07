// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Navigation Item Widget
//!
//! Provides a standardized, hardware-accelerated navigation list item primitive on [`UiScope`],
//! combining GPU atlas vector icons (or unicode text fallback), active indicator styling,
//! hover background transitions, and semantic tag click routing into a single declarative expression.
//! Adheres strictly to the centralized [`iris_core::Theme`] token system with zero hardcoded color literals.

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, Insets, Style, TextAlign, WidgetRole};

/// Configuration options for emitting a declarative navigation rail item.
///
/// All color fields are optional; when omitted, the item automatically adopts
/// the active [`iris_core::Theme`] palette tokens for  dark/light theme switching.
#[derive(Debug, Clone, Copy)]
pub struct NavItemConfig<'a> {
    /// Text label displayed on the item.
    pub label: &'a str,
    /// Optional GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    pub atlas_icon: Option<[f32; 4]>,
    /// Optional fallback unicode icon text.
    pub icon_text: Option<&'a str>,
    /// Whether this navigation item is currently selected.
    pub active: bool,
    /// Optional accent color applied to icon and text when selected (defaults to `theme.accent`).
    pub active_color: Option<Color>,
    /// Optional background color applied when selected (defaults to `theme.button_active_bg`).
    pub active_bg: Option<Color>,
    /// Optional hover background color (defaults to `theme.button_hover_bg`).
    pub hover_bg: Option<Color>,
    /// Optional idle text/icon color override (defaults to `theme.text_secondary`).
    pub idle_color: Option<Color>,
    /// Optional semantic 64-bit tag for persistent event routing.
    pub tag: Option<u64>,
    /// Row container height in logical pixels (defaults to `36.0`).
    pub height: f32,
}

impl<'a> NavItemConfig<'a> {
    /// Creates a new default navigation item configuration with the specified text label.
    #[inline]
    #[must_use]
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            atlas_icon: None,
            icon_text: None,
            active: false,
            active_color: None,
            active_bg: None,
            hover_bg: None,
            idle_color: None,
            tag: None,
            height: 36.0,
        }
    }

    /// Sets the GPU SDF atlas vector icon coordinates for this item.
    #[inline]
    #[must_use]
    pub const fn atlas_icon(mut self, uv: [f32; 4]) -> Self {
        self.atlas_icon = Some(uv);
        self
    }

    /// Sets the fallback unicode text icon for this item.
    #[inline]
    #[must_use]
    pub const fn icon_text(mut self, icon: &'a str) -> Self {
        self.icon_text = Some(icon);
        self
    }

    /// Sets whether this item is currently selected and highlighted.
    #[inline]
    #[must_use]
    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Overrides the accent color applied when this item is selected.
    #[inline]
    #[must_use]
    pub const fn active_color(mut self, color: Color) -> Self {
        self.active_color = Some(color);
        self
    }

    /// Overrides the background color applied when this item is selected.
    #[inline]
    #[must_use]
    pub const fn active_bg(mut self, color: Color) -> Self {
        self.active_bg = Some(color);
        self
    }

    /// Overrides the hover background color for this item.
    #[inline]
    #[must_use]
    pub const fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = Some(color);
        self
    }

    /// Overrides the idle foreground color for this item.
    #[inline]
    #[must_use]
    pub const fn idle_color(mut self, color: Color) -> Self {
        self.idle_color = Some(color);
        self
    }

    /// Sets the persistent 64-bit semantic tag for this item.
    #[inline]
    #[must_use]
    pub const fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Overrides the default height (in logical pixels) for this item.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl UiScope<'_> {
    /// Emits a standardized navigation rail item in a single pure declarative expression.
    ///
    /// Dynamically pulls active contrast colors, hover background transitions, corner radii,
    /// and label typography from the centralized theme without hardcoded literal color values.
    pub fn nav_item(
        &mut self,
        node_name: impl Into<String>,
        config: NavItemConfig<'_>,
    ) -> WidgetResponse {
        let tag = config.tag.unwrap_or(0);
        let theme = self.theme();

        let active_bg = config.active_bg.unwrap_or(theme.button_active_bg);
        let hover_bg = config.hover_bg.unwrap_or(theme.button_hover_bg);
        let active_tint = config.active_color.unwrap_or(theme.accent);
        let idle_tint = config.idle_color.unwrap_or(theme.text_muted);
        let label_color = if config.active {
            theme.text_primary
        } else {
            config.idle_color.unwrap_or(theme.text_secondary)
        };

        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(config.height)
            .padding_insets(Insets::new(0.0, 12.0, 0.0, 12.0))
            .gap(10.0)
            .border_radius(theme.corner_radius_md)
            .background(if config.active {
                active_bg
            } else {
                Color::TRANSPARENT
            })
            .hover_background(hover_bg);

        let icon_tint = if config.active {
            active_tint
        } else {
            idle_tint
        };

        self.container_tagged(node_name, row_style, WidgetRole::Button, tag, |btn| {
            if let Some(uv) = config.atlas_icon {
                btn.icon_named("NavIcon", uv, icon_tint, 16.0);
            } else if let Some(icon_str) = config.icon_text {
                btn.label_styled_passive(
                    "NavIconText",
                    icon_str,
                    13.0,
                    icon_tint,
                    TextAlign::Center,
                    Style::new().width(16.0).height(16.0),
                );
            }

            btn.label_styled_passive(
                "NavLabel",
                config.label,
                12.0,
                label_color,
                TextAlign::Left,
                Style::new().flex_grow(1.0).height(18.0),
            );
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_nav_item_renders_declaratively_with_atlas_icon() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(&mut tree, root);

        let response = scope.nav_item(
            "TestNavItem",
            NavItemConfig::new("Overview")
                .atlas_icon([0.1, 0.2, 0.3, 0.4])
                .active(true)
                .tag(0x1234),
        );

        assert_eq!(tree.get(response.id).map(|n| n.tag), Some(0x1234));

        let has_item = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("TestNavItem"));
        let has_label = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("Overview"));
        let has_icon = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NavIcon"));

        assert!(has_item, "TestNavItem node must exist in tree");
        assert!(has_label, "NavLabel must contain 'Overview'");
        assert!(has_icon, "NavIcon node must exist with atlas icon");
    }
}