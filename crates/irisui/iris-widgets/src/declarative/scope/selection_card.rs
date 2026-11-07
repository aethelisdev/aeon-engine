// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Selection Card & Option Tile Widget
//!
//! Provides a standardized, selectable tile card on [`UiScope`] designed for wizard setup choices,
//! template pickers, and dimension mode selection. Supports active selection border highlighting,
//! lead icons, title, multi-line descriptions, and optional status badges without imperative plumbing.
//! Adheres strictly to the centralized [`iris_core::Theme`] token system with zero hardcoded colors.

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetRole};

/// Configuration options for emitting an interactive selectable tile card.
#[derive(Debug, Clone)]
pub struct SelectionCardConfig<'a> {
    /// Primary title text displayed on the card header.
    pub title: &'a str,
    /// Detailed description or feature bullet summary text.
    pub description: &'a str,
    /// Optional GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    pub atlas_icon: Option<[f32; 4]>,
    /// Optional color tint applied across the lead icon quad.
    pub icon_tint: Option<Color>,
    /// Optional badge caption label (e.g., "BETA", "EXPERIMENTAL").
    pub badge: Option<&'a str>,
    /// Optional badge accent color (defaults to `theme.warning`).
    pub badge_color: Option<Color>,
    /// Whether this card is currently selected / active.
    pub selected: bool,
    /// Persistent 64-bit semantic tag for click detection.
    pub tag: u64,
    /// Card fixed width in logical pixels (defaults to `204.0`).
    pub width: f32,
    /// Card fixed height in logical pixels (defaults to `84.0`).
    pub height: f32,
}

impl<'a> SelectionCardConfig<'a> {
    /// Creates a new selection card configuration with the specified title and description.
    #[inline]
    #[must_use]
    pub const fn new(title: &'a str, description: &'a str) -> Self {
        Self {
            title,
            description,
            atlas_icon: None,
            icon_tint: None,
            badge: None,
            badge_color: None,
            selected: false,
            tag: 0,
            width: 204.0,
            height: 84.0,
        }
    }

    /// Sets the GPU SDF atlas vector icon coordinates for the lead card icon.
    #[inline]
    #[must_use]
    pub const fn atlas_icon(mut self, uv: [f32; 4]) -> Self {
        self.atlas_icon = Some(uv);
        self
    }

    /// Sets the color tint applied across the lead icon quad.
    #[inline]
    #[must_use]
    pub const fn icon_tint(mut self, tint: Color) -> Self {
        self.icon_tint = Some(tint);
        self
    }

    /// Attaches an informational or warning badge to the top-right corner of the card.
    #[inline]
    #[must_use]
    pub const fn badge(mut self, label: &'a str) -> Self {
        self.badge = Some(label);
        self
    }

    /// Sets the background accent color for the attached badge.
    #[inline]
    #[must_use]
    pub const fn badge_color(mut self, color: Color) -> Self {
        self.badge_color = Some(color);
        self
    }

    /// Sets whether the card is rendered in the active/selected state.
    #[inline]
    #[must_use]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the persistent 64-bit semantic tag for click detection.
    #[inline]
    #[must_use]
    pub const fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Overrides the card width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Overrides the card height in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl UiScope<'_> {
    /// Emits a standardized selection card tile in a single pure declarative expression.
    pub fn selection_card(
        &mut self,
        node_name: impl Into<String>,
        config: SelectionCardConfig<'_>,
    ) -> WidgetResponse {
        let theme = self.theme();
        let border_color = if config.selected {
            theme.accent
        } else {
            theme.card_border
        };
        let border_width = if config.selected { 1.5 } else { 1.0 };
        let icon_tint = config.icon_tint.unwrap_or(if config.selected {
            theme.accent
        } else {
            theme.text_secondary
        });
        let badge_color = config.badge_color.unwrap_or(theme.warning);

        let card_style = Style::new()
            .flex_col()
            .width(config.width)
            .height(config.height)
            .padding_insets(Insets::new(12.0, 14.0, 12.0, 14.0))
            .justify_content(JustifyContent::SpaceBetween)
            .background(theme.card_bg)
            .border(border_width, border_color)
            .border_radius(theme.corner_radius_md)
            .hover_background(theme.button_hover_bg);

        self.container_tagged(
            node_name,
            card_style,
            WidgetRole::Button,
            config.tag,
            |card| {
                // Header row: icon + title (left) and optional badge (right)
                card.container_named(
                    "SelectionCardHeader",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .width(config.width - 28.0),
                    |header| {
                        header.container_named(
                            "SelectionCardLead",
                            Style::new()
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .gap(6.0),
                            |lead| {
                                if let Some(uv) = config.atlas_icon {
                                    lead.icon_named("CardIcon", uv, icon_tint, 16.0);
                                }
                                lead.label_styled_passive(
                                    "CardTitle",
                                    config.title,
                                    11.5,
                                    if config.selected {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    TextAlign::Left,
                                    Style::new().width(88.0).height(18.0),
                                );
                            },
                        );

                        if let Some(badge_text) = config.badge {
                            header.badge(badge_text, badge_color);
                        }
                    },
                );

                // Description text
                card.label_styled_passive(
                    "CardDescription",
                    config.description,
                    9.5,
                    theme.text_muted,
                    TextAlign::Left,
                    Style::new().width(config.width - 28.0).height(28.0),
                );
            },
        )
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_selection_card_creation_and_attributes() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(&mut tree, root);

        let resp = scope.selection_card(
            "TestModeCard",
            SelectionCardConfig::new("3D Engine", "Full spatial pipeline")
                .selected(true)
                .badge("PRO")
                .tag(0x1234),
        );

        assert_eq!(
            tree.get(resp.id).map(|n| n.tag),
            Some(0x1234),
            "Tag must match configured value"
        );
        let has_header = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("SelectionCardHeader"));
        let has_desc = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("CardDescription"));
        assert!(has_header, "Card header must exist in UI tree");
        assert!(has_desc, "Card description must exist in UI tree");
    }
}