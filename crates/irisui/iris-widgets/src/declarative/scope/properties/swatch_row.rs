// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Color Swatch & Live Hex Property Row
//!
//! Provides [`UiScope::prop_color_swatch_row`] and [`UiScope::prop_color_swatch_row_with_config`]
//! for immediate two-way bound color swatch preview and native Iris UI text input editing with
//! autonomous caret rendering, selection, clipboard, undo/redo, and color validation.

use iris_core::response::WidgetResponse;
use iris_core::{AlignItems, Color, Style, TextAlign};

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::input_box::InputBoxProps;
use crate::declarative::widget::color_swatch::ColorSwatch;

/// Fluent configuration builder for rendering a declarative color swatch and live hex property row.
///
/// Encapsulates sizing dimensions, colors, string labels, and persistent semantic interaction tags
/// without exposing intermediate imperative variable bindings in consumer UI code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyColorSwatchRowConfig<'a> {
    /// Human-readable label displayed on the far left.
    pub label: &'a str,
    /// Concrete RGBA color to preview inside the swatch pill.
    pub color: Color,
    /// Current hex representation string (e.g. `"#6699cc"`).
    pub hex: &'a str,
    /// Fixed width in physical pixels allocated for the left title label.
    pub label_width: f32,
    /// Fixed width in physical pixels allocated for the `"Hex:"` separator label.
    pub hex_label_width: f32,
    /// Fixed width in physical pixels allocated for the hex text box pill.
    pub hex_box_width: f32,
    /// Physical width of the interactive color swatch pill.
    pub swatch_width: f32,
    /// Physical height of the interactive color swatch pill.
    pub swatch_height: f32,
    /// Overall layout row height in physical pixels.
    pub row_height: f32,
    /// Optional persistent 64-bit hit-testing tag assigned to the color swatch button.
    pub swatch_tag: Option<u64>,
    /// Optional persistent 64-bit hit-testing tag assigned to the hex display input box.
    pub hex_tag: Option<u64>,
}

impl<'a> PropertyColorSwatchRowConfig<'a> {
    /// Constructs a new [`PropertyColorSwatchRowConfig`] with standard industry defaults.
    ///
    /// # Arguments
    /// * `label` - Human-readable description label (e.g. `"Object Color:"`).
    /// * `color` - Color rendered inside the preview swatch pill.
    /// * `hex` - Hex formatted string preview.
    #[inline]
    #[must_use]
    pub const fn new(label: &'a str, color: Color, hex: &'a str) -> Self {
        Self {
            label,
            color,
            hex,
            label_width: 75.0,
            hex_label_width: 28.0,
            hex_box_width: 68.0,
            swatch_width: 48.0,
            swatch_height: 18.0,
            row_height: 22.0,
            swatch_tag: None,
            hex_tag: None,
        }
    }

    /// Overrides the persistent hit-testing semantic tags for swatch and hex boxes.
    ///
    /// # Arguments
    /// * `swatch_tag` - 64-bit semantic tag for the swatch button.
    /// * `hex_tag` - 64-bit semantic tag for the hex display box.
    #[inline]
    #[must_use]
    pub const fn with_tags(mut self, swatch_tag: u64, hex_tag: u64) -> Self {
        self.swatch_tag = Some(swatch_tag);
        self.hex_tag = Some(hex_tag);
        self
    }

    /// Overrides the physical width constraint for the primary left title label.
    #[inline]
    #[must_use]
    pub const fn with_label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    /// Overrides the physical dimensions of the interactive color swatch pill.
    #[inline]
    #[must_use]
    pub const fn with_swatch_size(mut self, width: f32, height: f32) -> Self {
        self.swatch_width = width;
        self.swatch_height = height;
        self
    }
}

/// Comprehensive interaction response returned by evaluating a declarative color swatch row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyColorSwatchRowResponse {
    /// Interaction response for the primary color swatch pill.
    pub swatch_response: WidgetResponse,
    /// Interaction response for the native hex text input box.
    pub hex_response: WidgetResponse,
    /// Current open status of the elevated color picker popup after evaluating this frame.
    pub is_open: bool,
    /// Whether the color swatch was clicked and toggled state during this frame.
    pub toggled: bool,
    /// Validated color committed via Enter key or edit session completion (if any).
    pub hex_committed_color: Option<Color>,
    /// Validated color updated live during active typing if matching a valid hex string (if any).
    pub hex_changed_color: Option<Color>,
    /// Color rendered inside the preview swatch pill.
    pub swatch_color: Color,
}

impl PropertyColorSwatchRowResponse {
    /// Returns the color committed via hex input (e.g. Enter key pressed), if any.
    #[inline]
    #[must_use]
    pub const fn committed_color(&self) -> Option<Color> {
        self.hex_committed_color
    }

    /// Evaluates a following declarative color palette on `scope`, combining hex text commitment
    /// and palette swatch selection, and maps the resulting color via `f`.
    ///
    /// Enables 100% declarative property chaining without intermediate `let` variables.
    ///
    /// # Arguments
    /// * `scope` - Declarative UI scope to render the color palette into.
    /// * `label` - Human-readable label for the palette section.
    /// * `default_swatches` - Slice of canonical default colors.
    /// * `saved_swatches` - Mutable accumulation vector of user-saved custom swatches.
    /// * `f` - Mapper closure transforming the resolved [`Color`] into domain action `T`.
    pub fn then_palette_map<T, F: FnOnce(Color) -> T>(
        self,
        scope: &mut UiScope<'_>,
        label: &str,
        default_swatches: &[Color],
        saved_swatches: &mut Vec<[f32; 4]>,
        f: F,
    ) -> Option<T> {
        let palette_res =
            scope.prop_color_palette(label, self.swatch_color, default_swatches, saved_swatches);
        self.hex_committed_color
            .or(palette_res.selected_color)
            .map(f)
    }
    /// Invokes `f(is_open)` if the color swatch was clicked and toggled during this frame.
    #[inline]
    pub fn on_toggle<F: FnOnce(bool)>(self, f: F) -> Self {
        if self.toggled {
            f(self.is_open);
        }
        self
    }

    /// Invokes `f()` if the hex text input box was clicked during this frame.
    #[inline]
    pub fn on_hex_click<F: FnOnce()>(self, f: F) -> Self {
        if self.hex_response.clicked() {
            f();
        }
        self
    }

    /// Invokes `f(color)` if a valid hex color was committed (e.g. Enter key pressed).
    #[inline]
    pub fn on_hex_commit<F: FnOnce(Color)>(self, f: F) -> Self {
        if let Some(col) = self.hex_committed_color {
            f(col);
        }
        self
    }

    /// Invokes `f(color)` if the hex text changed and represents a valid color.
    #[inline]
    pub fn on_hex_change<F: FnOnce(Color)>(self, f: F) -> Self {
        if let Some(col) = self.hex_changed_color {
            f(col);
        }
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative two-way bound color swatch and live hex property row.
    ///
    /// Evaluates swatch clicks, directly mutates `*is_open`, consumes the frame click to prevent
    /// double-dispatch toggling glitches, and renders an interactive Iris UI native text input
    /// for hexadecimal editing.
    ///
    /// # Arguments
    /// * `label` - Left title description label (e.g. `"Object Color:"`).
    /// * `color` - Bound concrete RGBA color previewed in the swatch pill.
    /// * `hex` - Hex code string previewed in the hex pill (e.g. `"#6699cc"`).
    /// * `is_open` - Mutable reference to the panel local state tracking popup visibility.
    pub fn prop_color_swatch_row(
        &mut self,
        label: &str,
        color: Color,
        hex: &str,
        is_open: &mut bool,
    ) -> PropertyColorSwatchRowResponse {
        self.prop_color_swatch_row_with_config(
            PropertyColorSwatchRowConfig::new(label, color, hex),
            is_open,
        )
    }

    /// Emits a declarative color swatch and hex property row configured via [`PropertyColorSwatchRowConfig`].
    ///
    /// # Arguments
    /// * `config` - Full layout, styling, and semantic tag configuration.
    /// * `is_open` - Mutable reference to the panel local state tracking popup visibility.
    pub fn prop_color_swatch_row_with_config(
        &mut self,
        config: PropertyColorSwatchRowConfig<'_>,
        is_open: &mut bool,
    ) -> PropertyColorSwatchRowResponse {
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(config.row_height)
            .gap(6.0);

        let mut swatch_resp = WidgetResponse::default();
        let mut hex_resp = WidgetResponse::default();
        let mut toggled = false;
        let mut hex_committed_color = None;
        let mut hex_changed_color = None;

        self.container_named("ObjectColorRow", row_style, |row| {
            row.label_styled_passive(
                "ObjectColorLabel",
                config.label,
                11.0,
                row.theme().text_muted,
                TextAlign::Left,
                Style::new()
                    .width(config.label_width)
                    .height(config.row_height),
            );

            let mut swatch_widget = ColorSwatch::new(config.color)
                .active(*is_open)
                .size(config.swatch_width, config.swatch_height);

            if let Some(tag) = config.swatch_tag {
                swatch_widget = swatch_widget.tag(tag);
            }

            swatch_resp = row.add(swatch_widget);
            if swatch_resp.clicked() {
                *is_open = !*is_open;
                toggled = true;
                // Crucial atomic consumption: clear confirmed clicks so subsequent layout passes
                // or dirty subtree rebuilds within the same frame cannot double-toggle the popup.
                row.tree.event_state.clear_frame_clicks();
            }

            row.label_styled_passive(
                "HexPrefixLabel",
                "Hex:",
                11.0,
                row.theme().text_muted,
                TextAlign::Left,
                Style::new()
                    .width(config.hex_label_width)
                    .height(config.row_height),
            );

            let hex_tag = config.hex_tag.unwrap_or_else(|| row.tag_for("HexInputBox"));
            let is_editing = row
                .tree
                .event_state
                .active_text_edit
                .as_ref()
                .is_some_and(|s| s.tag == hex_tag);

            let mut input_props = InputBoxProps::new(
                config.hex,
                "#RRGGBB",
                config.hex_box_width,
                is_editing,
                true,
            );
            input_props.font_size = 10.5;

            hex_resp = row.input_box_tagged(&input_props, hex_tag);

            let get_active_hex_text = |tree: &iris_core::UiTree,
                                       box_id: iris_core::WidgetId,
                                       tag: u64|
             -> Option<String> {
                tree.event_state
                    .text_edit_buffer(tag)
                    .map(str::to_owned)
                    .or_else(|| {
                        let children = tree.get(box_id).map(|n| n.children.clone())?;
                        for child_id in children {
                            if let Some(node) = tree.get(child_id)
                                && let Some(ref text) = node.text
                            {
                                return Some(text.clone());
                            }
                        }
                        None
                    })
            };

            if hex_resp.submitted() {
                let text_candidate = get_active_hex_text(row.tree, hex_resp.id, hex_tag);
                let text = text_candidate.as_deref().unwrap_or(config.hex);
                if let Some(col) = Color::try_from_hex(text) {
                    hex_committed_color = Some(col);
                }
            } else if hex_resp.changed() {
                let text_candidate = get_active_hex_text(row.tree, hex_resp.id, hex_tag);
                let text = text_candidate.as_deref().unwrap_or(config.hex);
                if let Some(col) = Color::try_from_hex(text) {
                    hex_changed_color = Some(col);
                }
            }
        });

        PropertyColorSwatchRowResponse {
            swatch_response: swatch_resp,
            hex_response: hex_resp,
            is_open: *is_open,
            toggled,
            hex_committed_color,
            hex_changed_color,
            swatch_color: config.color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::tree::UiTree;

    #[test]
    fn test_swatch_row_structure_and_click() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let mut scope = UiScope::new(&mut tree, root);

        let mut is_open = false;
        let swatch_tag = 0x8881;
        let hex_tag = 0x8882;

        let resp = scope.prop_color_swatch_row_with_config(
            PropertyColorSwatchRowConfig::new("Object Color:", Color::RED, "#ff0000")
                .with_tags(swatch_tag, hex_tag),
            &mut is_open,
        );

        assert!(!resp.is_open);
        assert!(!resp.toggled);
        assert!(!is_open);
        assert_eq!(resp.hex_committed_color, None);
    }

    #[test]
    fn test_swatch_row_toggle_consumption_and_clear_frame_clicks() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");

        let swatch_tag = 0x7771;
        let hex_tag = 0x7772;
        tree.event_state.frame_clicked_tag = Some(swatch_tag);

        let mut is_open = false;
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.prop_color_swatch_row_with_config(
                PropertyColorSwatchRowConfig::new("Object Color:", Color::BLUE, "#0000ff")
                    .with_tags(swatch_tag, hex_tag),
                &mut is_open,
            );

            assert!(resp.toggled);
            assert!(resp.is_open);
            assert!(is_open, "is_open must be directly toggled to true");
        }

        // Frame clicks must be cleared atomically by the swatch click handler
        assert_eq!(
            tree.event_state.frame_clicked_tag, None,
            "clear_frame_clicks must clear frame_clicked_tag to prevent second toggle"
        );

        // Immediate subsequent evaluation in the same frame cannot re-toggle
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp2 = scope.prop_color_swatch_row_with_config(
                PropertyColorSwatchRowConfig::new("Object Color:", Color::BLUE, "#0000ff")
                    .with_tags(swatch_tag, hex_tag),
                &mut is_open,
            );

            assert!(!resp2.toggled);
            assert!(resp2.is_open);
            assert!(is_open, "is_open must remain true and not toggle back");
        }
    }

    #[test]
    fn test_swatch_row_hex_commit_callback() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");

        let hex_tag = 0x9992;
        tree.event_state
            .start_text_edit(None, hex_tag, "#ceba00", 7, None);

        // Simulate enter key submission
        tree.event_state.frame_key_pressed = true;
        tree.event_state.frame_key = Some(iris_core::KeyCode::Enter);

        let mut is_open = false;
        let mut committed = None;

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .prop_color_swatch_row_with_config(
                    PropertyColorSwatchRowConfig::new("Object Color:", Color::RED, "#ff0000")
                        .with_tags(0x9991, hex_tag),
                    &mut is_open,
                )
                .on_hex_commit(|col| {
                    committed = Some(col);
                });
        }

        assert_eq!(committed, Color::try_from_hex("#ceba00"));
    }

    #[test]
    fn test_swatch_row_then_palette_map() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");

        let hex_tag = 0x9994;
        tree.event_state
            .start_text_edit(None, hex_tag, "#112233", 7, None);
        tree.event_state.frame_key_pressed = true;
        tree.event_state.frame_key = Some(iris_core::KeyCode::Enter);

        let mut is_open = false;
        let mut swatches = Vec::new();
        let defaults = [Color::RED, Color::GREEN, Color::BLUE];

        let action = {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .prop_color_swatch_row_with_config(
                    PropertyColorSwatchRowConfig::new("Object Color:", Color::BLACK, "#000000")
                        .with_tags(0x9993, hex_tag),
                    &mut is_open,
                )
                .then_palette_map(&mut scope, "Palette:", &defaults, &mut swatches, |col| {
                    format!(
                        "Committed: {:02x}{:02x}{:02x}",
                        (col.r * 255.0) as u8,
                        (col.g * 255.0) as u8,
                        (col.b * 255.0) as u8
                    )
                })
        };

        assert_eq!(action, Some("Committed: 112233".to_string()));
    }
}