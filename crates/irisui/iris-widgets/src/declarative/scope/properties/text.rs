// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Text & String Property Rows
//!
//! Provides single-line string and text property editors on [`UiScope`].
//!

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::{WidgetResponse, split_label_id};
use iris_core::{AlignItems, Color, Insets, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole};

/// Configuration options for text and string property rows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PropertyTextOptions<'a> {
    /// Placeholder hint displayed when the value string is empty.
    pub placeholder: &'a str,
    /// Whether the text input field is currently active in an editing session.
    pub is_editing: bool,
    /// Whether the cursor caret is visible in the current animation frame.
    pub blink_caret: bool,
    /// Whether all text within the input field is selected with a highlight capsule.
    pub is_all_selected: bool,
    /// Optional fixed column width in physical pixels for the left-aligned property label.
    pub label_width: Option<f32>,
    /// Optional explicit semantic tag override. If `None`, computed via `tag_for(label)`.
    pub custom_tag: Option<u64>,
}

impl<'a> PropertyTextOptions<'a> {
    /// Creates a default text property options descriptor.
    pub const fn new() -> Self {
        Self {
            placeholder: "",
            is_editing: false,
            blink_caret: false,
            is_all_selected: false,
            label_width: None,
            custom_tag: None,
        }
    }

    /// Sets the placeholder hint displayed when the value string is empty.
    pub const fn with_placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Sets whether the text input field is currently in an active editing session.
    pub const fn with_editing(mut self, is_editing: bool, blink_caret: bool) -> Self {
        self.is_editing = is_editing;
        self.blink_caret = blink_caret;
        self
    }

    /// Sets whether all text within the input field is currently selected with a highlight capsule.
    pub const fn with_all_selected(mut self, is_all_selected: bool) -> Self {
        self.is_all_selected = is_all_selected;
        self
    }

    /// Sets the fixed column width in physical pixels for the left-aligned property label.
    pub const fn with_label_width(mut self, label_width: f32) -> Self {
        self.label_width = Some(label_width);
        self
    }

    /// Sets a custom semantic tag for interactive hit-testing and event routing.
    pub const fn with_custom_tag(mut self, tag: u64) -> Self {
        self.custom_tag = Some(tag);
        self
    }
}

/// Detailed interaction response returned from a property text row.
#[derive(Debug, Clone, Copy, Default)]
pub struct PropertyTextResponse {
    /// Generic interaction state (clicked, hovered, changed).
    pub response: WidgetResponse,
    /// Allocated widget identifier for the text input box.
    pub box_id: WidgetId,
    /// Allocated widget identifier for the text label node.
    pub text_id: WidgetId,
}

impl std::ops::Deref for PropertyTextResponse {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<'a> UiScope<'a> {
    /// Emits a standardized text input property row with label and interactive input pill.
    pub fn property_text(&mut self, label: &str, value: &str) -> PropertyTextResponse {
        self.property_text_with_options(label, value, PropertyTextOptions::default())
    }

    /// Emits a standardized text input property row with custom configuration options.
    pub fn property_text_with_options(
        &mut self,
        label: &str,
        value: &str,
        options: PropertyTextOptions<'_>,
    ) -> PropertyTextResponse {
        let tag = options.custom_tag.unwrap_or_else(|| self.tag_for(label));
        let mut text_response = PropertyTextResponse::default();

        let row_id = self.property_row_styled(label, options.label_width, |slot| {
            let (visible_label, _) = split_label_id(label);
            let sanitized_label = visible_label.trim_end_matches(':').trim();
            let box_name = format!("{}InputBox", sanitized_label);
            let text_name = format!("{}InputText", sanitized_label);

            let is_hovered = slot.hovered_tag == Some(tag);

            let (bg, border_col) = if options.is_editing {
                (
                    Color::rgba(0.086, 0.090, 0.106, 1.0),
                    Color::rgba(0.353, 0.376, 0.439, 0.95),
                )
            } else if is_hovered {
                (
                    Color::rgba(0.125, 0.133, 0.149, 0.98),
                    Color::rgba(0.271, 0.282, 0.329, 0.95),
                )
            } else {
                (
                    Color::rgba(0.106, 0.110, 0.125, 0.98),
                    Color::rgba(0.173, 0.180, 0.208, 0.85),
                )
            };

            let box_style = Style::new()
                .flex_grow(1.0)
                .height(24.0)
                .background(bg)
                .border(1.0, border_col)
                .border_radius(5.0)
                .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
                .flex_row()
                .align_items(AlignItems::Center);

            let display_text = if options.is_editing {
                if options.is_all_selected {
                    value.to_string()
                } else if options.blink_caret {
                    format!("{}|", value)
                } else {
                    value.to_string()
                }
            } else if value.is_empty() {
                options.placeholder.to_string()
            } else {
                value.to_string()
            };

            let text_color = if options.is_editing {
                Color::WHITE
            } else if value.is_empty() {
                Color::rgba(0.50, 0.53, 0.60, 1.0)
            } else {
                Color::rgba(0.886, 0.894, 0.918, 1.0)
            };

            let mut captured_text_id = WidgetId::default();

            let box_id = slot.container_tagged(
                box_name,
                box_style,
                WidgetRole::TextInput,
                tag,
                |box_scope| {
                    if options.is_editing && options.is_all_selected && !value.is_empty() {
                        let approx_char_w = 6.8;
                        let text_w = (value.chars().count() as f32 * approx_char_w).max(12.0);
                        let sel_x = 6.0;
                        let sel_w = text_w + 4.0;
                        let sel_name = format!("{}_Sel", sanitized_label);
                        let sel_style = Style::new()
                            .position_absolute()
                            .left(sel_x)
                            .top(3.5)
                            .width(sel_w)
                            .height(17.0)
                            .background(Color::rgba(0.14, 0.46, 0.88, 0.95))
                            .border_radius(3.0);
                        box_scope.empty_box_passive_named(sel_name, sel_style);
                    }

                    captured_text_id = box_scope.label_styled_passive(
                        text_name,
                        display_text,
                        11.5,
                        text_color,
                        TextAlign::Left,
                        Style::new().flex_grow(1.0),
                    );
                },
            );

            if let Some(node) = slot.tree.get_mut(box_id) {
                node.cursor = Some(WidgetCursor::Text);
            }

            let (clicked, hovered, _) = slot.check_interaction(box_id);

            text_response = PropertyTextResponse {
                response: WidgetResponse::new(box_id, clicked, hovered, false),
                box_id,
                text_id: captured_text_id,
            };
        });

        if text_response.response.id == WidgetId::default() {
            text_response.response.id = row_id;
        }
        text_response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Rect, UiTree};

    #[test]
    fn test_property_text_structure_and_layout() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.property_text_with_options(
            "🏷 Name:",
            "TestEntity",
            PropertyTextOptions::new().with_label_width(60.0),
        );

        assert!(resp.box_id != WidgetId::default());
        assert!(resp.text_id != WidgetId::default());

        let box_node = scope.tree().get(resp.box_id).expect("Box node must exist");
        assert_eq!(box_node.role, WidgetRole::TextInput);
        assert_eq!(box_node.cursor, Some(WidgetCursor::Text));

        let text_node = scope
            .tree()
            .get(resp.text_id)
            .expect("Text node must exist");
        assert_eq!(text_node.text.as_deref(), Some("TestEntity"));

        // Layout subtree and verify geometry computation
        crate::declarative::layout_subtree(
            scope.tree_mut(),
            root,
            Rect::new(0.0, 0.0, 300.0, 50.0),
        );

        let box_rect = scope.tree().get(resp.box_id).unwrap().computed_rect;
        assert!(box_rect.width > 100.0, "Box width must flex-expand");
        assert_eq!(box_rect.height, 24.0);
    }

    #[test]
    fn test_property_text_editing_caret() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.property_text_with_options(
            "Name",
            "Hero",
            PropertyTextOptions::new().with_editing(true, true),
        );

        let text_node = scope
            .tree()
            .get(resp.text_id)
            .expect("Text node must exist");
        assert_eq!(text_node.text.as_deref(), Some("Hero|"));
    }

    #[test]
    fn test_property_text_all_selected_capsule() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.property_text_with_options(
            "🏷 Name:",
            "Dynamic Cube",
            PropertyTextOptions::new()
                .with_editing(true, true)
                .with_all_selected(true),
        );

        let text_node = scope
            .tree()
            .get(resp.text_id)
            .expect("Text node must exist");
        assert_eq!(text_node.text.as_deref(), Some("Dynamic Cube"));

        let box_node = scope.tree().get(resp.box_id).expect("Box node must exist");
        assert_eq!(box_node.children.len(), 2);
        let sel_id = box_node.children[0];
        let sel_node = scope
            .tree()
            .get(sel_id)
            .expect("Selection capsule node must exist");
        assert_eq!(sel_node.name.as_deref(), Some("🏷 Name_Sel"));
        assert_eq!(
            sel_node.style.background_color,
            Color::rgba(0.14, 0.46, 0.88, 0.95)
        );

        crate::declarative::layout_subtree(
            scope.tree_mut(),
            root,
            Rect::new(0.0, 0.0, 300.0, 50.0),
        );
        let sel_node_after = scope
            .tree()
            .get(sel_id)
            .expect("Selection capsule node must exist after layout");
        assert!(
            sel_node_after.computed_rect.width < 100.0,
            "Capsule width {} must only cover text length",
            sel_node_after.computed_rect.width
        );
        assert!(sel_node_after.computed_rect.width >= 50.0);
    }
}