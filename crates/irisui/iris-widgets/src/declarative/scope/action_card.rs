// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Action Card & List Item Widget
//!
//! Provides a standardized, hardware-accelerated interactive list item card on [`UiScope`],
//! combining lead icons, title/subtitle typography stacks, trailing action buttons,
//! and persistent semantic tag routing in a single declarative expression.
//! Adheres strictly to the centralized [`iris_core::Theme`] token system with zero hardcoded colors.

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetRole};

/// Descriptor configuring the trailing action button on an [`ActionCardConfig`].
#[derive(Debug, Clone, Copy)]
pub struct ActionCardButton<'a> {
    /// Button text label.
    pub label: &'a str,
    /// Optional GPU SDF atlas icon coordinates.
    pub atlas_icon: Option<[f32; 4]>,
    /// Persistent 64-bit semantic tag for click detection.
    pub tag: u64,
    /// Optional custom background color override.
    pub bg_color: Option<Color>,
}

/// Configuration options for emitting an interactive action card list item.
#[derive(Debug, Clone)]
pub struct ActionCardConfig<'a> {
    /// Primary title text displayed on the card.
    pub title: &'a str,
    /// Optional secondary detail or path subtitle text.
    pub subtitle: Option<String>,
    /// Optional GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    pub atlas_icon: Option<[f32; 4]>,
    /// Optional icon color tint (defaults to `theme.accent`).
    pub icon_tint: Option<Color>,
    /// Size of the lead icon in logical pixels (defaults to `20.0`).
    pub icon_size: f32,
    /// Optional 64-bit semantic tag for the outer card container.
    pub tag: Option<u64>,
    /// Optional trailing action button descriptor.
    pub action_button: Option<ActionCardButton<'a>>,
    /// Fixed card height in logical pixels (defaults to `56.0`).
    pub height: f32,
}

impl<'a> ActionCardConfig<'a> {
    /// Creates a new action card configuration with the specified primary title.
    #[inline]
    #[must_use]
    pub const fn new(title: &'a str) -> Self {
        Self {
            title,
            subtitle: None,
            atlas_icon: None,
            icon_tint: None,
            icon_size: 20.0,
            tag: None,
            action_button: None,
            height: 56.0,
        }
    }

    /// Sets the secondary detail or path subtitle text.
    #[inline]
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Sets the GPU SDF atlas vector icon coordinates for the lead icon.
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

    /// Sets the size of the lead icon quad in logical pixels.
    #[inline]
    #[must_use]
    pub const fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// Sets the persistent 64-bit semantic tag for the outer card container.
    #[inline]
    #[must_use]
    pub const fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Configures a trailing action button with label, atlas icon, and semantic tag.
    #[inline]
    #[must_use]
    pub const fn action_button(mut self, label: &'a str, atlas_icon: [f32; 4], tag: u64) -> Self {
        self.action_button = Some(ActionCardButton {
            label,
            atlas_icon: Some(atlas_icon),
            tag,
            bg_color: None,
        });
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
    /// Emits a standardized action card list item in a single pure declarative expression.
    ///
    /// Assembles lead icon, title/subtitle column, and trailing action button without intermediate variables.
    pub fn action_card(
        &mut self,
        node_name: impl Into<String>,
        config: ActionCardConfig<'_>,
    ) -> WidgetResponse {
        let theme = self.theme();
        let card_tag = config.tag.unwrap_or(0);
        let icon_tint = config.icon_tint.unwrap_or(theme.accent);

        let card_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .height(config.height)
            .padding_insets(Insets::new(0.0, 16.0, 0.0, 16.0))
            .background(theme.card_bg)
            .border(1.0, theme.card_border)
            .border_radius(theme.corner_radius_lg)
            .hover_background(theme.button_hover_bg);

        self.container_tagged(
            node_name,
            card_style,
            WidgetRole::Button,
            card_tag,
            |card| {
                // Left: Icon + Text Group
                card.container_named(
                    "CardInfoGroup",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .gap(14.0),
                    |info| {
                        if let Some(uv) = config.atlas_icon {
                            info.icon_named("CardLeadIcon", uv, icon_tint, config.icon_size);
                        }

                        info.container_named(
                            "CardTextGroup",
                            Style::new().flex_col().gap(2.0).width(400.0),
                            |text_col| {
                                text_col.label_styled_passive(
                                    "CardTitle",
                                    config.title,
                                    13.0,
                                    theme.text_primary,
                                    TextAlign::Left,
                                    Style::new().width(400.0).height(18.0),
                                );

                                if let Some(ref subtitle) = config.subtitle {
                                    text_col.label_styled_passive(
                                        "CardSubtitle",
                                        subtitle,
                                        10.5,
                                        theme.text_muted,
                                        TextAlign::Left,
                                        Style::new().width(400.0).height(16.0),
                                    );
                                }
                            },
                        );
                    },
                );

                // Right: Trailing Action Button
                if let Some(btn) = config.action_button {
                    let btn_bg = btn.bg_color.unwrap_or(theme.accent);
                    let btn_style = Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .gap(6.0)
                        .padding_insets(Insets::new(0.0, 14.0, 0.0, 14.0))
                        .height(28.0)
                        .background(btn_bg)
                        .hover_background(theme.accent_hover)
                        .border_radius(theme.corner_radius_sm);

                    card.container_tagged(
                        "CardActionButton",
                        btn_style,
                        WidgetRole::Button,
                        btn.tag,
                        |btn_scope| {
                            if let Some(uv) = btn.atlas_icon {
                                btn_scope.icon_named("ActionButtonIcon", uv, theme.panel_bg, 14.0);
                            }
                            btn_scope.label_styled_passive(
                                "ActionButtonLabel",
                                btn.label,
                                11.0,
                                theme.panel_bg,
                                TextAlign::Left,
                                Style::new().width(40.0).height(16.0),
                            );
                        },
                    );
                }
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
    fn test_action_card_renders_declaratively() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(&mut tree, root);

        let response = scope.action_card(
            "TestCard",
            ActionCardConfig::new("Sample Project")
                .subtitle("/path/to/project")
                .atlas_icon([0.1, 0.2, 0.3, 0.4])
                .tag(0x1000)
                .action_button("Open", [0.5, 0.6, 0.7, 0.8], 0x2000),
        );

        assert_eq!(tree.get(response.id).map(|n| n.tag), Some(0x1000));

        let has_card = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("TestCard"));
        let has_title = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("Sample Project"));
        let has_subtitle = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("/path/to/project"));
        let has_button = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("CardActionButton"));

        assert!(has_card, "TestCard node must exist in tree");
        assert!(has_title, "CardTitle must contain 'Sample Project'");
        assert!(has_subtitle, "CardSubtitle must contain path");
        assert!(has_button, "CardActionButton must exist");
    }
}