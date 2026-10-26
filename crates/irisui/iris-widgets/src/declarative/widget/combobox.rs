// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Declarative ComboBox & Dropdown Widget (`ComboBox`, `ComboBoxResponse`)
//!
//! Provides the fluent [`ComboBox`] widget builder implementing [`Widget`] that autonomously
//! manages open/closed popup state via [`iris_core::UiTree::active_combobox_tag`] and renders
//! options directly on [`iris_core::UiLayer::Popup`], returning a rich [`ComboBoxResponse`]
//! without requiring manual engine-side popup flags or external state bags.
//!

use std::ops::Deref;

use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Rect, Style, TextAlign, WidgetCursor,
    WidgetResponse, WidgetRole,
};

use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::Widget;

/// Comprehensive interaction response returned by evaluating an autonomous [`ComboBox`] widget.
///
/// Implements [`Deref<Target = WidgetResponse>`] for transparent access to generic
/// interaction flags (`clicked()`, `hovered()`) while providing direct access to selected option state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ComboBoxResponse {
    /// Underlying generic widget response for the combobox trigger button.
    pub response: WidgetResponse,
    /// Whether an option was clicked and selected during this frame.
    pub changed: bool,
    /// 0-indexed option that was clicked and selected during this frame, if any.
    pub selected_index: Option<usize>,
    /// Whether the combobox popup menu is currently open.
    pub is_open: bool,
}

impl Deref for ComboBoxResponse {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl ComboBoxResponse {
    /// Returns whether an option was selected during this frame.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }

    /// Returns the 0-indexed option selected during this frame, if any.
    #[inline]
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Returns whether the combobox popup menu is currently open.
    #[inline]
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Custom closure type for rendering the trailing dropdown indicator icon.
pub type ComboBoxIndicatorFn<'a> = Box<dyn Fn(&mut UiScope<'_>, bool) + 'a>;

/// Autonomous declarative dropdown ComboBox builder implementing [`Widget`].
///
/// Manages open/close toggle state autonomously via [`iris_core::UiTree::active_combobox_tag`],
/// renders elevated options on [`iris_core::UiLayer::Popup`], and emits a rich [`ComboBoxResponse`].
pub struct ComboBox<'a> {
    label: Option<&'a str>,
    selected_text: &'a str,
    options: &'a [&'a str],
    tag: Option<u64>,
    width: Option<f32>,
    label_width: Option<f32>,
    indicator: Option<ComboBoxIndicatorFn<'a>>,
}

impl<'a> ComboBox<'a> {
    /// Creates a new [`ComboBox`] builder with currently selected text and option labels.
    #[inline]
    pub fn new(selected_text: &'a str, options: &'a [&'a str]) -> Self {
        Self {
            label: None,
            selected_text,
            options,
            tag: None,
            width: None,
            label_width: None,
            indicator: None,
        }
    }

    /// Attaches an optional descriptive label aligned to the left of the combo pill.
    #[inline]
    #[must_use]
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets an explicit persistent 64-bit semantic tag for hit-testing and state tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets the fixed width of the interactive combobox trigger button pill.
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the fixed column width for the left-aligned label.
    #[inline]
    #[must_use]
    pub fn label_width(mut self, label_width: f32) -> Self {
        self.label_width = Some(label_width);
        self
    }

    /// Attaches a custom indicator renderer closure.
    ///
    /// Receives the combo pill scope and current `is_open` boolean state.
    /// Allows callers (such as game engines) to supply custom texture atlas icons
    /// without hardcoding external texture coordinates or asset paths into `iris-widgets`.
    #[inline]
    #[must_use]
    pub fn indicator<F: Fn(&mut UiScope<'_>, bool) + 'a>(mut self, f: F) -> Self {
        self.indicator = Some(Box::new(f));
        self
    }
}

impl<'a> Widget for ComboBox<'a> {
    type Response = ComboBoxResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response {
        let tag = self.tag.unwrap_or_else(|| {
            let src = self.label.unwrap_or(self.selected_text);
            scope.tag_for(src)
        });

        let was_active = scope.tree.active_combobox_tag == Some(tag);
        let mut is_active = was_active;

        let combo_w = self.width.unwrap_or(90.0);
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(22.0)
            .gap(6.0);

        let row_id = scope.tree.create_node();
        if let Some(node) = scope.tree.get_mut(row_id) {
            node.set_name("ComboboxRow");
            node.set_style(row_style);
        }
        let _ = scope.tree.add_child(scope.parent, row_id);

        let mut row_scope = scope.child_scope(row_id);

        // Optional Left Label
        if let Some(lbl) = self.label {
            let (visible_label, _) = crate::declarative::types::split_label_id(lbl);
            let lbl_w = self.label_width.unwrap_or(65.0);
            row_scope.label_styled_passive(
                "ComboLabel",
                visible_label,
                11.0,
                row_scope.theme().text_muted,
                TextAlign::Left,
                Style::new().width(lbl_w).height(22.0),
            );
        }

        // Interactive Combo Trigger Pill
        let pill_id = row_scope.tree.create_node();
        let _ = row_scope.tree.add_child(row_scope.parent, pill_id);

        if let Some(node) = row_scope.tree.get_mut(pill_id) {
            node.set_name("ComboPill");
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.tag = tag;
            node.cursor = Some(WidgetCursor::Pointer);
        }

        let (clicked, hovered, _) = row_scope.check_interaction(pill_id);

        if clicked {
            if was_active {
                row_scope.tree.active_combobox_tag = None;
                is_active = false;
            } else {
                row_scope.tree.active_combobox_tag = Some(tag);
                is_active = true;
            }
        }

        let theme = row_scope.theme();
        let (bg, border) = if is_active {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.0, 0.80, 1.0, 0.95),
            )
        } else if hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };

        let pill_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .width(combo_w)
            .height(22.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);

        if let Some(node) = row_scope.tree.get_mut(pill_id) {
            node.set_style(pill_style);
        }

        {
            let mut pill_scope = row_scope.child_scope(pill_id);
            pill_scope.label_styled_passive(
                "ComboText",
                self.selected_text,
                10.5,
                if is_active {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
                TextAlign::Left,
                Style::new().flex_grow(1.0),
            );

            if let Some(ref ind) = self.indicator {
                ind(&mut pill_scope, is_active);
            } else {
                let chevron_tint = if is_active {
                    theme.text_primary
                } else {
                    theme.text_secondary
                };

                let chevron = if is_active { "▴" } else { "▾" };
                pill_scope.label_styled_passive(
                    "ComboChevron",
                    chevron,
                    8.0,
                    chevron_tint,
                    TextAlign::Right,
                    Style::new().width(10.0),
                );
            }
        }

        let mut changed = false;
        let mut selected_index = None;

        // Render elevated floating dropdown menu on UiLayer::Popup when active
        if is_active {
            let anchor_rect = row_scope
                .tree
                .tag_rect(tag)
                .or_else(|| {
                    row_scope
                        .tree
                        .get(pill_id)
                        .map(|n| n.computed_rect)
                        .filter(|r| r.width > 0.0)
                })
                .or_else(|| {
                    row_scope
                        .tree
                        .event_state
                        .press_origin
                        .or(row_scope.tree.event_state.last_pointer_pos)
                        .map(|pt| Rect::new(pt.x - 20.0, pt.y - 11.0, combo_w, 22.0))
                })
                .unwrap_or(Rect::new(0.0, 0.0, combo_w, 22.0));

            let popup_w = combo_w.max(100.0);

            let root_id = row_scope.tree.root().unwrap_or(row_scope.parent);
            let mut root_scope = row_scope.child_scope(root_id);

            let popup_resp =
                root_scope.popup_anchored_named("ComboBoxPopup", anchor_rect, popup_w, |menu| {
                    for (idx, &item_text) in self.options.iter().enumerate() {
                        let item_tag =
                            crate::declarative::types::combine_seeds(tag, idx as u64 + 1);
                        let item_resp = menu.dropdown_item(item_tag, "", item_text, None, true);
                        if item_resp.clicked() {
                            selected_index = Some(idx);
                            changed = true;
                        }
                    }
                });

            let popup_rect = popup_resp.response.rect;

            // Outside click dismissal: close combobox if clicked outside
            if changed {
                row_scope.tree.active_combobox_tag = None;
                is_active = false;
            } else if !clicked
                && let Some(click_pos) = row_scope
                    .tree
                    .event_state
                    .frame_clicked_pos
                    .or(row_scope.tree.event_state.press_origin)
                && !popup_rect.contains_point(click_pos)
                && !anchor_rect.contains_point(click_pos)
            {
                row_scope.tree.active_combobox_tag = None;
                is_active = false;
            }
        }

        ComboBoxResponse {
            response: WidgetResponse::new(pill_id, clicked, hovered, changed),
            changed,
            selected_index,
            is_open: is_active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Point, Rect, UiTree};

    #[test]
    fn test_declarative_combobox_click_to_open() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut scope = UiScope::new(&mut tree, root);

        let options = ["Box", "Sphere", "Capsule"];
        let resp = scope.add(ComboBox::new("Box", &options).label("Shape"));
        assert!(!resp.is_open());
        assert!(!resp.changed());
        assert_eq!(resp.selected_index(), None);
    }

    #[test]
    fn test_declarative_combobox_anchored_to_pill_tag() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let tag = 0xABCD_1234;
        let pill_rect = Rect::new(320.0, 200.0, 140.0, 22.0);
        tree.set_tag_rect(tag, pill_rect);
        tree.active_combobox_tag = Some(tag);

        let mut scope = UiScope::new(&mut tree, root);
        let options = ["Dynamic", "Kinematic", "Static"];
        let resp = scope.add(ComboBox::new("Dynamic", &options).tag(tag).width(140.0));
        assert!(resp.is_open());

        // Find the ComboBoxPopup node in the tree
        let popup_node = tree
            .iter()
            .map(|(_, n)| n)
            .find(|n| n.name.as_deref() == Some("ComboBoxPopup"));
        assert!(popup_node.is_some(), "ComboBoxPopup must exist in tree");
        let popup = popup_node.expect("popup node");
        assert_eq!(popup.computed_rect.x, 320.0);
        assert_eq!(popup.computed_rect.y, 224.0); // 200.0 + 22.0 + 2.0
        assert!(popup.computed_rect.x != 60.0);
        assert!(popup.computed_rect.y != 30.0);
    }

    #[test]
    fn test_declarative_combobox_outside_click_dismissal() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let tag = 0x5555_7777;
        let pill_rect = Rect::new(400.0, 300.0, 100.0, 22.0);
        tree.set_tag_rect(tag, pill_rect);
        tree.active_combobox_tag = Some(tag);
        // Simulate clicking outside on the viewport at (50.0, 50.0)
        tree.event_state.frame_clicked_pos = Some(Point::new(50.0, 50.0));

        let mut scope = UiScope::new(&mut tree, root);
        let options = ["Perspective", "Orthographic"];
        let resp = scope.add(ComboBox::new("Perspective", &options).tag(tag).width(100.0));

        assert!(!resp.is_open(), "ComboBox must close on outside click");
        assert_eq!(
            tree.active_combobox_tag, None,
            "active_combobox_tag must be cleared"
        );
    }
}