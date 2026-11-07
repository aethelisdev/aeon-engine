// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Labeled Input Field Widget
//!
//! Provides a standardized form input block on [`UiScope`] containing an uppercase field label,
//! an interactive text box with focus border highlight, and an optional trailing button (e.g., browse picker).
//! Adheres strictly to the centralized [`iris_core::Theme`] token system with zero hardcoded colors.

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetRole};

/// Descriptor configuring an optional trailing action button attached to a [`LabeledInputConfig`].
#[derive(Debug, Clone, Copy)]
pub struct LabeledInputButton {
    /// GPU SDF atlas icon coordinates (`[u_min, v_min, u_max, layer]`).
    pub atlas_icon: [f32; 4],
    /// Persistent 64-bit semantic tag for click detection.
    pub tag: u64,
    /// Optional custom icon tint override.
    pub icon_tint: Option<Color>,
}

impl LabeledInputButton {
    /// Creates a new trailing button descriptor with icon and semantic tag.
    #[inline]
    #[must_use]
    pub const fn new(atlas_icon: [f32; 4], tag: u64) -> Self {
        Self {
            atlas_icon,
            tag,
            icon_tint: None,
        }
    }

    /// Sets a custom color tint for the trailing button icon.
    #[inline]
    #[must_use]
    pub const fn icon_tint(mut self, tint: Color) -> Self {
        self.icon_tint = Some(tint);
        self
    }
}

/// Configuration options for emitting a labeled form input block.
#[derive(Debug, Clone)]
pub struct LabeledInputConfig<'a> {
    /// Field label text displayed above the input box (e.g., "PROJECT NAME").
    pub label: &'a str,
    /// Current textual value or placeholder displayed inside the input box.
    pub value: &'a str,
    /// Persistent 64-bit semantic tag for the input box.
    pub tag: u64,
    /// Whether the text input field is currently active / focused.
    pub focused: bool,
    /// Total width of the input field block in logical pixels (defaults to `420.0`).
    pub width: f32,
    /// Height of the input box in logical pixels (defaults to `36.0`).
    pub height: f32,
    /// Optional trailing action button attached adjacent to the input box.
    pub trailing_button: Option<LabeledInputButton>,
}

impl<'a> LabeledInputConfig<'a> {
    /// Creates a new labeled input configuration with the specified label and current text value.
    #[inline]
    #[must_use]
    pub const fn new(label: &'a str, value: &'a str) -> Self {
        Self {
            label,
            value,
            tag: 0,
            focused: false,
            width: 420.0,
            height: 36.0,
            trailing_button: None,
        }
    }

    /// Sets the persistent 64-bit semantic tag for click/focus detection on the input box.
    #[inline]
    #[must_use]
    pub const fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Sets whether the input box is currently focused.
    #[inline]
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Overrides the total block width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Overrides the input box height in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Attaches an adjacent trailing action button (e.g., folder browse picker).
    #[inline]
    #[must_use]
    pub const fn trailing_button(mut self, btn: LabeledInputButton) -> Self {
        self.trailing_button = Some(btn);
        self
    }
}

impl UiScope<'_> {
    /// Emits a standardized labeled form input block in a single declarative expression.
    pub fn labeled_input(
        &mut self,
        node_name: impl Into<String>,
        config: LabeledInputConfig<'_>,
    ) -> WidgetResponse {
        let theme = self.theme();
        let border_color = if config.focused {
            theme.accent
        } else {
            theme.card_border
        };
        let border_width = if config.focused { 1.5 } else { 1.0 };
        let box_width = if config.trailing_button.is_some() {
            config.width - 48.0
        } else {
            config.width
        };

        self.container_named(node_name, Style::new().flex_col().gap(6.0), |col| {
            col.label_styled_passive(
                "FieldLabel",
                config.label,
                10.0,
                theme.text_muted,
                TextAlign::Left,
                Style::new().width(config.width).height(16.0),
            );

            col.container_named(
                "InputRow",
                Style::new()
                    .flex_row()
                    .gap(8.0)
                    .align_items(AlignItems::Center),
                |row| {
                    row.container_tagged(
                        "InputBox",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .width(box_width)
                            .height(config.height)
                            .padding_insets(Insets::new(0.0, 12.0, 0.0, 12.0))
                            .background(theme.card_bg)
                            .border_radius(theme.corner_radius_sm)
                            .border(border_width, border_color),
                        WidgetRole::TextInput,
                        config.tag,
                        |box_scope| {
                            box_scope.label_styled_passive(
                                "InputValue",
                                config.value,
                                11.5,
                                if config.focused {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                },
                                TextAlign::Left,
                                Style::new().flex_grow(1.0).height(18.0),
                            );
                        },
                    );

                    if let Some(btn) = config.trailing_button {
                        let icon_tint = btn.icon_tint.unwrap_or(theme.warning);
                        row.container_tagged(
                            "TrailingButton",
                            Style::new()
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::Center)
                                .width(40.0)
                                .height(config.height)
                                .background(theme.button_idle_bg)
                                .hover_background(theme.button_hover_bg)
                                .border_radius(theme.corner_radius_sm)
                                .border(1.0, theme.card_border),
                            WidgetRole::Button,
                            btn.tag,
                            |btn_scope| {
                                btn_scope.icon_named(
                                    "TrailingIcon",
                                    btn.atlas_icon,
                                    icon_tint,
                                    18.0,
                                );
                            },
                        );
                    }
                },
            );
        })
        .response
    }

    /// Emits a standardized labeled text input field using Iris's native editable text box.
    ///
    /// Manages caret positioning, selection highlighting, and full keyboard interaction
    /// (Ctrl+Z, Ctrl+X, Ctrl+Y, Ctrl+A, clipboard, caret movement) via [`UiScope::text_input_with_width_tagged`].
    pub fn labeled_text_input(
        &mut self,
        node_name: impl Into<String>,
        label: &str,
        text: &mut String,
        tag: u64,
        width: f32,
    ) -> WidgetResponse {
        self.labeled_text_input_with_button(node_name, label, text, tag, width, None)
    }

    /// Emits a standardized labeled text input field with an optional trailing action button.
    pub fn labeled_text_input_with_button(
        &mut self,
        node_name: impl Into<String>,
        label: &str,
        text: &mut String,
        tag: u64,
        width: f32,
        trailing_button: Option<LabeledInputButton>,
    ) -> WidgetResponse {
        let theme = self.theme();
        let box_width = if trailing_button.is_some() {
            width - 48.0
        } else {
            width
        };

        self.container_named(node_name, Style::new().flex_col().gap(6.0), |col| {
            col.label_styled_passive(
                "FieldLabel",
                label,
                10.0,
                theme.text_muted,
                TextAlign::Left,
                Style::new().width(width).height(16.0),
            );

            col.container_named(
                "InputRow",
                Style::new()
                    .flex_row()
                    .gap(8.0)
                    .align_items(AlignItems::Center),
                |row| {
                    row.text_input_with_width_tagged("", tag, box_width, text);

                    if let Some(btn) = trailing_button {
                        let icon_tint = btn.icon_tint.unwrap_or(theme.warning);
                        row.container_tagged(
                            "TrailingButton",
                            Style::new()
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::Center)
                                .width(40.0)
                                .height(26.0)
                                .background(theme.button_idle_bg)
                                .hover_background(theme.button_hover_bg)
                                .border_radius(theme.corner_radius_sm)
                                .border(1.0, theme.card_border),
                            WidgetRole::Button,
                            btn.tag,
                            |btn_scope| {
                                btn_scope.icon_named(
                                    "TrailingIcon",
                                    btn.atlas_icon,
                                    icon_tint,
                                    18.0,
                                );
                            },
                        );
                    }
                },
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
    fn test_labeled_input_creation() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut scope = UiScope::new(&mut tree, root);

        let resp = scope.labeled_input(
            "NameField",
            LabeledInputConfig::new("PROJECT NAME", "MyGame")
                .focused(true)
                .tag(0x100),
        );

        assert_eq!(
            tree.get(resp.id).map(|n| n.name.as_deref()),
            Some(Some("NameField"))
        );
        let has_label = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("FieldLabel"));
        let has_box = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("InputBox"));
        assert!(has_label, "Field label must exist");
        assert!(has_box, "Input box must exist");
    }
}