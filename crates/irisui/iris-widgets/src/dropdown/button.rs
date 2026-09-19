// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standardized combobox pill button trigger widget.
//!
//! Provides a compact interactive dropdown button displaying the currently selected
//! value and an expandable directional arrow indicator (`▼` / `▲`).
//! Emits semantic `WidgetRole::Button` and child label nodes with zero-allocation hit targets.

use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Visual theme and metrics configuration for combobox pill buttons.
#[derive(Debug, Clone)]
pub struct ComboboxButtonStyle {
    /// Background color in the idle / unhovered state.
    pub bg_idle: Color,
    /// Border stroke color in the idle state.
    pub border_idle: Color,
    /// Text label and arrow color in the idle state.
    pub text_idle: Color,
    /// Background color when hovered under the mouse cursor.
    pub bg_hover: Color,
    /// Border stroke color when hovered.
    pub border_hover: Color,
    /// Text label and arrow color when hovered.
    pub text_hover: Color,
    /// Background color when the dropdown popup is actively open.
    pub bg_open: Color,
    /// Border stroke color when the dropdown popup is actively open.
    pub border_open: Color,
    /// Text label and arrow color when open.
    pub text_open: Color,
    /// Stroke thickness in logical pixels.
    pub border_width: f32,
    /// Exterior corner radius in logical pixels.
    pub corner_radius: f32,
    /// Font size in logical pixels.
    pub font_size: f32,
    /// Character symbol indicating an open / expanded menu state.
    pub arrow_open: &'static str,
    /// Character symbol indicating a closed / collapsed menu state.
    pub arrow_closed: &'static str,
}

impl Default for ComboboxButtonStyle {
    fn default() -> Self {
        Self {
            bg_idle: Color::rgba(0.157, 0.165, 0.188, 0.98),
            border_idle: Color::rgba(0.212, 0.220, 0.259, 0.85),
            text_idle: Color::rgba(0.886, 0.894, 0.918, 1.0),
            bg_hover: Color::rgba(0.200, 0.208, 0.235, 1.0),
            border_hover: Color::rgba(0.271, 0.282, 0.329, 0.95),
            text_hover: Color::WHITE,
            bg_open: Color::rgba(0.118, 0.125, 0.145, 1.0),
            border_open: Color::rgba(0.353, 0.376, 0.439, 0.95),
            text_open: Color::WHITE,
            border_width: 1.0,
            corner_radius: 5.0,
            font_size: 10.5,
            arrow_open: "▲",
            arrow_closed: "▼",
        }
    }
}

/// Output layout frame resulting from instantiating a combobox pill button in the UI tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboboxButtonFrame {
    /// Generational node identifier for the interactive button container.
    pub button_id: WidgetId,
    /// Absolute computed screen-space rectangle of the button.
    pub button_rect: Rect,
    /// Node identifier for the interior label and arrow text.
    pub text_id: WidgetId,
}

/// Fluent builder for constructing standardized combobox pill buttons.
///
/// Automatically creates a root container with [`WidgetRole::Button`] and an interior
/// text label with `interactive = false`, ensuring mouse clicks reliably target
/// the button container while rendering rich active rings and state transitions.
#[derive(Debug, Clone)]
pub struct ComboboxButtonBuilder<'a> {
    rect: Rect,
    selected_text: &'a str,
    is_open: bool,
    is_hovered: bool,
    style: ComboboxButtonStyle,
    name: String,
    tag: Option<u64>,
}

impl<'a> ComboboxButtonBuilder<'a> {
    /// Creates a new combobox button builder positioned at the specified bounding rectangle.
    #[inline]
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            selected_text: "",
            is_open: false,
            is_hovered: false,
            style: ComboboxButtonStyle::default(),
            name: "ComboboxPillButton".to_string(),
            tag: None,
        }
    }

    /// Sets the currently active choice label displayed on the pill.
    #[inline]
    pub fn selected_text(mut self, text: &'a str) -> Self {
        self.selected_text = text;
        self
    }

    /// Sets whether the associated dropdown popup menu is currently open.
    ///
    /// When `true`, applies the active open background, border, and the upwards arrow (`▲`).
    #[inline]
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Explicitly sets the mouse hover highlight state.
    #[inline]
    pub fn is_hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    /// Automatically evaluates the hover state by checking if `cursor_pos` is within the button bounds.
    #[inline]
    pub fn cursor_pos(mut self, cursor_pos: Point) -> Self {
        self.is_hovered = self.rect.contains_point(cursor_pos);
        self
    }

    /// Overrides the visual styling theme for the combobox button.
    #[inline]
    pub fn style(mut self, style: ComboboxButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Assigns a semantic identifier tag to the button node (`node.tag`).
    #[inline]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets a descriptive debug name for the container node.
    #[inline]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Instantiates the combobox button and its interior text in the target [`UiTree`].
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> ComboboxButtonFrame {
        let (bg, border, text_color) = if self.is_open {
            (
                self.style.bg_open,
                self.style.border_open,
                self.style.text_open,
            )
        } else if self.is_hovered {
            (
                self.style.bg_hover,
                self.style.border_hover,
                self.style.text_hover,
            )
        } else {
            (
                self.style.bg_idle,
                self.style.border_idle,
                self.style.text_idle,
            )
        };

        // 1. Button Container Quad
        let button_id = tree.create_node();
        if let Some(node) = tree.get_mut(button_id) {
            node.set_name(self.name);
            node.set_role(WidgetRole::Button);
            node.computed_rect = self.rect;
            if let Some(tag_val) = self.tag {
                node.set_tag(tag_val);
            }
            node.style = Style::new()
                .background(bg)
                .border(self.style.border_width, border)
                .border_radius(self.style.corner_radius);
        }
        let _ = tree.add_child(parent_id, button_id);

        // 2. Interior Label and Directional Arrow
        let arrow = if self.is_open {
            self.style.arrow_open
        } else {
            self.style.arrow_closed
        };
        let label_text = if self.selected_text.is_empty() {
            arrow.to_string()
        } else {
            format!("{}  {}", self.selected_text, arrow)
        };

        let text_id = tree.create_node();
        if let Some(node) = tree.get_mut(text_id) {
            node.set_name("ComboboxButtonText");
            node.set_role(WidgetRole::Default);
            node.interactive = false;
            node.computed_rect = self.rect;
            node.set_text(label_text);
            node.font_size = self.style.font_size;
            node.line_height = self.rect.height;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
        }
        let _ = tree.add_child(button_id, text_id);

        ComboboxButtonFrame {
            button_id,
            button_rect: self.rect,
            text_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combobox_button_builder_idle_and_open() {
        let mut tree = UiTree::new();
        let root = tree.create_node();

        let rect = Rect::new(100.0, 50.0, 88.0, 22.0);

        // Idle state
        let frame_idle = ComboboxButtonBuilder::new(rect)
            .selected_text("Capsule")
            .is_open(false)
            .tag(42)
            .build(&mut tree, root);

        assert_eq!(frame_idle.button_rect, rect);
        let btn_node = tree.get(frame_idle.button_id).unwrap();
        assert_eq!(btn_node.role, WidgetRole::Button);
        assert_eq!(btn_node.tag, 42);

        let txt_node = tree.get(frame_idle.text_id).unwrap();
        assert_eq!(txt_node.text.as_deref(), Some("Capsule  ▼"));
        assert!(!txt_node.interactive);

        // Open state
        let frame_open = ComboboxButtonBuilder::new(rect)
            .selected_text("Sphere")
            .is_open(true)
            .build(&mut tree, root);

        let txt_open_node = tree.get(frame_open.text_id).unwrap();
        assert_eq!(txt_open_node.text.as_deref(), Some("Sphere  ▲"));
    }
}