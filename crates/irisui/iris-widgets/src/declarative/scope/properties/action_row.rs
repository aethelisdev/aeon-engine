// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Property Action Rows
//!
//! Provides read-only and optional reference property rows paired with a trailing
//! interactive action button (`prop_action_row`, `prop_optional_action_row`) on [`UiScope`].
//! Eliminates imperative `let` variables and manual row/button construction in inspector views.

use std::borrow::Cow;

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, JustifyContent, Style, TextAlign, WidgetCursor, WidgetRole};

/// Configuration descriptor for a read-only or optional reference property row
/// paired with a trailing action button.
///
/// When `active_text` is `Some(text)`, renders a horizontal row with the left-aligned
/// primary value caption and a right-aligned compact action button (`action_label`).
/// When `active_text` is `None`, renders a single muted fallback caption (`empty_text`).
#[derive(Debug, Clone)]
pub struct PropertyActionRow<'a> {
    /// Text displayed on the left when an active target or value is present.
    pub active_text: Option<Cow<'a, str>>,
    /// Fallback text displayed in muted style when `active_text` is `None`.
    pub empty_text: &'a str,
    /// Caption displayed inside the trailing action button when `active_text` is `Some`.
    pub action_label: &'a str,
    /// Optional static prefix text placed before the active label.
    pub prefix: &'a str,
    /// Prefix string prepended when formatting a fallback numeric identifier. Defaults to `""`.
    pub id_prefix: &'a str,
    /// Optional custom text color for the active value label. Resolves to `theme.text_primary` when `None`.
    pub text_color: Option<Color>,
    /// Optional custom text color for the prefix label. Resolves to `theme.text_muted` when `None`.
    pub prefix_color: Option<Color>,
    /// Debug node name assigned to the horizontal container row.
    pub row_name: &'a str,
    /// Debug node name assigned to the prefix label when prefix is non-empty.
    pub prefix_name: &'a str,
    /// Debug node name assigned to the primary left label.
    pub label_name: &'a str,
    /// Debug node name assigned to the trailing action button container.
    pub button_name: &'a str,
    /// Debug node name assigned to the action button text node.
    pub button_text_name: &'a str,
    /// Debug node name assigned to the fallback empty-state label node.
    pub empty_label_name: &'a str,
    /// Width of the trailing action button in logical pixels.
    pub button_width: f32,
    /// Height of the outer horizontal row in logical pixels.
    pub row_height: f32,
    /// Height of the trailing action button and labels in logical pixels.
    pub button_height: f32,
}

impl<'a> PropertyActionRow<'a> {
    /// Constructs an always-active [`PropertyActionRow`] displaying `label_text` and a trailing `action_label` button.
    #[inline]
    #[must_use]
    pub fn new(label_text: impl Into<Cow<'a, str>>, action_label: &'a str) -> Self {
        Self {
            active_text: Some(label_text.into()),
            empty_text: "",
            action_label,
            prefix: "",
            id_prefix: "",
            text_color: None,
            prefix_color: None,
            row_name: "PropertyActionRow",
            prefix_name: "PropertyActionPrefix",
            label_name: "PropertyActionLabel",
            button_name: "PropertyActionButton",
            button_text_name: "PropertyActionButtonText",
            empty_label_name: "PropertyEmptyLabel",
            button_width: 80.0,
            row_height: 22.0,
            button_height: 20.0,
        }
    }

    /// Constructs an optional [`PropertyActionRow`] that renders `active_text` with a trailing `action_label`
    /// button when `Some`, or a muted `empty_text` fallback label when `None`.
    #[inline]
    #[must_use]
    pub fn optional(
        active_text: Option<Cow<'a, str>>,
        empty_text: &'a str,
        action_label: &'a str,
    ) -> Self {
        Self {
            active_text,
            empty_text,
            action_label,
            prefix: "",
            id_prefix: "",
            text_color: None,
            prefix_color: None,
            row_name: "PropertyActionRow",
            prefix_name: "PropertyActionPrefix",
            label_name: "PropertyActionLabel",
            button_name: "PropertyActionButton",
            button_text_name: "PropertyActionButtonText",
            empty_label_name: "PropertyEmptyLabel",
            button_width: 80.0,
            row_height: 22.0,
            button_height: 20.0,
        }
    }

    /// Constructs an optional [`PropertyActionRow`] from a borrowed string slice option.
    #[inline]
    #[must_use]
    pub fn optional_str(
        active_text: Option<&'a str>,
        empty_text: &'a str,
        action_label: &'a str,
    ) -> Self {
        Self::optional(active_text.map(Cow::Borrowed), empty_text, action_label)
    }

    /// Constructs an optional [`PropertyActionRow`] from an owned [`String`] option.
    #[inline]
    #[must_use]
    pub fn optional_string(
        active_text: Option<String>,
        empty_text: &'a str,
        action_label: &'a str,
    ) -> Self {
        Self::optional(active_text.map(Cow::Owned), empty_text, action_label)
    }

    /// Constructs a reference target action row displaying `prefix` with `target_name` (or formatted `target_id`)
    /// and a trailing `action_label` button when present, or `empty_text` when absent.
    #[inline]
    #[must_use]
    pub fn reference_target(
        prefix: &'a str,
        target_name: Option<impl Into<Cow<'a, str>>>,
        fallback_id: Option<u64>,
        id_prefix: &'a str,
        empty_text: &'a str,
        action_label: &'a str,
    ) -> Self {
        let active_text = match (target_name, fallback_id) {
            (Some(name), _) => Some(name.into()),
            (None, Some(id)) => {
                let mut buf = [0u8; 32];
                let formatted = format_id_with_prefix_stack(id_prefix, id, &mut buf);
                Some(Cow::Owned(formatted.to_string()))
            }
            (None, None) => None,
        };

        Self {
            active_text,
            empty_text,
            action_label,
            prefix,
            id_prefix,
            text_color: None,
            prefix_color: None,
            row_name: "PropertyActionRow",
            prefix_name: "PropertyActionPrefix",
            label_name: "PropertyActionLabel",
            button_name: "PropertyActionButton",
            button_text_name: "PropertyActionButtonText",
            empty_label_name: "PropertyEmptyLabel",
            button_width: 80.0,
            row_height: 22.0,
            button_height: 20.0,
        }
    }

    /// Overrides the text color for the primary active label.
    #[inline]
    #[must_use]
    pub const fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Overrides the text color for the prefix label.
    #[inline]
    #[must_use]
    pub const fn with_prefix_color(mut self, color: Color) -> Self {
        self.prefix_color = Some(color);
        self
    }

    /// Sets an optional static prefix label displayed before `active_text`.
    #[inline]
    #[must_use]
    pub const fn with_prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    /// Overrides the debug node names for the row, value label, button container, button caption, and empty fallback label.
    #[inline]
    #[must_use]
    pub const fn with_names(
        mut self,
        row_name: &'a str,
        label_name: &'a str,
        button_name: &'a str,
        button_text_name: &'a str,
        empty_label_name: &'a str,
    ) -> Self {
        self.row_name = row_name;
        self.label_name = label_name;
        self.button_name = button_name;
        self.button_text_name = button_text_name;
        self.empty_label_name = empty_label_name;
        self
    }

    /// Overrides the debug node name for the prefix label.
    #[inline]
    #[must_use]
    pub const fn with_prefix_name(mut self, prefix_name: &'a str) -> Self {
        self.prefix_name = prefix_name;
        self
    }

    /// Overrides the width of the trailing action button in logical pixels.
    #[inline]
    #[must_use]
    pub const fn with_button_width(mut self, width: f32) -> Self {
        self.button_width = width;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a read-only property row displaying `label_text` on the left and a compact interactive
    /// `action_label` button on the right, returning the button's [`WidgetResponse`].
    pub fn prop_action_row(
        &mut self,
        label_text: impl Into<Cow<'a, str>>,
        action_label: &'a str,
    ) -> WidgetResponse {
        self.prop_optional_action_row(PropertyActionRow::new(label_text, action_label))
    }

    /// Emits a conditional property action row from a [`PropertyActionRow`] descriptor.
    ///
    /// When `cfg.active_text` is `Some(text)`, emits a horizontal row (`cfg.row_name`) containing
    /// the left-aligned value label (with optional `cfg.prefix`) and a right-aligned action button (`cfg.button_name`), returning
    /// the button's [`WidgetResponse`] for fluent `.on_click(...)` or `.map_click(...)` chaining.
    /// When `cfg.active_text` is `None`, emits a single muted passive label (`cfg.empty_label_name`)
    /// and returns an empty [`WidgetResponse`].
    pub fn prop_optional_action_row(&mut self, cfg: PropertyActionRow<'_>) -> WidgetResponse {
        let Some(active_text) = cfg.active_text else {
            self.label_styled_passive(
                cfg.empty_label_name,
                cfg.empty_text,
                11.0,
                self.theme().text_muted,
                TextAlign::Left,
                Style::new().height(cfg.button_height),
            );
            return WidgetResponse::default();
        };

        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .height(cfg.row_height);

        self.container_named(cfg.row_name, row_style, |row| {
            if !cfg.prefix.is_empty() {
                let prefix_color = cfg.prefix_color.unwrap_or(row.theme().text_muted);
                row.label_styled_passive(
                    cfg.prefix_name,
                    cfg.prefix,
                    11.0,
                    prefix_color,
                    TextAlign::Left,
                    Style::new().height(cfg.button_height),
                );
            }

            let text_color = cfg.text_color.unwrap_or(row.theme().text_primary);
            row.label_styled_passive(
                cfg.label_name,
                active_text.as_ref(),
                11.0,
                text_color,
                TextAlign::Left,
                Style::new().flex_grow(1.0).height(cfg.button_height),
            );

            let btn_tag = row.tag_for(cfg.action_label);
            let btn_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(cfg.button_width)
                .height(cfg.button_height)
                .background(row.theme().button_idle_bg)
                .hover_background(row.theme().button_hover_bg)
                .border(1.0, row.theme().button_border)
                .border_radius(row.theme().corner_radius_md);

            let btn_resp = row.container_tagged(
                cfg.button_name,
                btn_style,
                WidgetRole::Button,
                btn_tag,
                |btn| {
                    btn.label_styled_passive(
                        cfg.button_text_name,
                        cfg.action_label,
                        10.0,
                        btn.theme().text_secondary,
                        TextAlign::Center,
                        Style::new()
                            .width(cfg.button_width)
                            .height(cfg.button_height),
                    );
                },
            );
            if let Some(node) = row.tree.get_mut(btn_resp.response.id) {
                node.cursor = Some(WidgetCursor::Pointer);
            }
            btn_resp.response
        })
        .inner
    }
}

/// Formats an arbitrary numeric identifier with an optional prefix onto a stack buffer without heap allocations.
#[inline]
pub fn format_id_with_prefix_stack<'b>(prefix: &str, id: u64, buf: &'b mut [u8; 32]) -> &'b str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cursor, "{prefix}{id}");
    let pos = cursor.position() as usize;
    std::str::from_utf8(&buf[..pos]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{MouseButton, Point, Rect, UiEvent, UiTree};

    #[test]
    fn test_prop_optional_action_row_active_and_empty_states() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        // 1. Test None (empty fallback state)
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.configure_container("TestRoot", Style::new().flex_col(), false);
            let mapped = scope
                .prop_optional_action_row(
                    PropertyActionRow::optional(None, "Parent: None (Root Entity)", "❌ Unparent")
                        .with_names(
                            "ParentRow",
                            "ParentLbl",
                            "UnparentBtn",
                            "UnparentTxt",
                            "ParentRootLbl",
                        ),
                )
                .map_click(|| "unparented");
            assert_eq!(mapped, None);
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 40.0));
        }
        tree.assert_node_exists("ParentRootLbl");
        tree.assert_node_absent("UnparentBtn");

        // 2. Test Some(text) (active row with button and click mapping)
        let mut tree2 = UiTree::new();
        let root2 = tree2.create_root().expect("Root node must exist");
        {
            let mut scope = UiScope::new(&mut tree2, root2);
            scope.configure_container("TestRoot", Style::new().flex_col(), false);
            let mapped = scope
                .prop_optional_action_row(
                    PropertyActionRow::optional_str(
                        Some("Parent: RootSceneNode"),
                        "Parent: None (Root Entity)",
                        "❌ Unparent",
                    )
                    .with_names(
                        "ParentRow",
                        "ParentLbl",
                        "UnparentBtn",
                        "UnparentTxt",
                        "ParentRootLbl",
                    ),
                )
                .map_click(|| "unparented");
            assert_eq!(mapped, None);
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 40.0));
        }

        let btn_node = tree2.assert_node("UnparentBtn", WidgetRole::Button);
        assert_ne!(btn_node.tag, 0);
        assert_eq!(btn_node.computed_rect.width, 80.0);
        assert_eq!(btn_node.computed_rect.height, 20.0);

        let btn_rect = tree2.assert_node_rect("UnparentBtn");
        let click_pt = Point::new(btn_rect.x + 10.0, btn_rect.y + 10.0);
        let _ = tree2.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree2.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        let emitted = {
            let mut scope = UiScope::new(&mut tree2, root2);
            scope
                .prop_optional_action_row(
                    PropertyActionRow::optional_str(
                        Some("Parent: RootSceneNode"),
                        "Parent: None (Root Entity)",
                        "❌ Unparent",
                    )
                    .with_names(
                        "ParentRow",
                        "ParentLbl",
                        "UnparentBtn",
                        "UnparentTxt",
                        "ParentRootLbl",
                    ),
                )
                .map_click(|| "unparented")
        };
        assert_eq!(emitted, Some("unparented"));
    }

    #[test]
    fn test_prop_optional_action_row_reference_target() {
        let row_named = PropertyActionRow::reference_target(
            "Target: ",
            Some("MainCamera"),
            Some(42),
            "#",
            "Target: None",
            "Select",
        );
        assert_eq!(row_named.active_text.as_deref(), Some("MainCamera"));
        assert_eq!(row_named.prefix, "Target: ");

        let row_id_fallback = PropertyActionRow::reference_target(
            "Target: ",
            None::<&str>,
            Some(42),
            "#",
            "Target: None",
            "Select",
        );
        assert_eq!(row_id_fallback.active_text.as_deref(), Some("#42"));

        let row_empty = PropertyActionRow::reference_target(
            "Target: ",
            None::<&str>,
            None,
            "#",
            "Target: None",
            "Select",
        );
        assert_eq!(row_empty.active_text, None);
    }
}