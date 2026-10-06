// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Appearance and Color Palette Declarative Inspector Card
//!
//! Renders object color swatch, live HEX text input, and quick-select palette swatches
//! using purely declarative [`UiScope`] containers, flexbox layout, and O(1) semantic tags.
//!

use super::registry::ComponentRenderContext;
use irisui::prelude::*;

const APPEARANCE_SWATCH_BASE: u64 = 0xAA00_0000;
const APPEARANCE_HEX_TAG: u64 = 0xAA00_FF01;
const APPEARANCE_COLOR_SWATCH_TAG: u64 = 0xAA00_FF02;
const APPEARANCE_ADD_PALETTE_TAG: u64 = 0xAA00_FF03;
const APPEARANCE_CLEAR_PALETTE_TAG: u64 = 0xAA00_FF04;

/// Default 7-color palette swatches displayed in the Appearance inspector card.
pub const DEFAULT_PALETTE: [Color; 7] = [
    Color::rgba(1.0, 1.0, 1.0, 1.0),
    Color::rgba(0.55, 0.58, 0.64, 1.0),
    Color::rgba(0.10, 0.10, 0.12, 1.0),
    Color::rgba(0.95, 0.22, 0.22, 1.0),
    Color::rgba(0.15, 0.88, 0.35, 1.0),
    Color::rgba(0.20, 0.45, 0.98, 1.0),
    Color::rgba(0.98, 0.90, 0.15, 1.0),
];

/// Returns the 64-bit semantic tag for the primary object color swatch button.
#[inline]
pub fn appearance_color_swatch_tag() -> u64 {
    APPEARANCE_COLOR_SWATCH_TAG
}

/// Resolves whether a semantic tag corresponds to the primary object color swatch button.
#[inline]
pub fn resolve_appearance_color_swatch_tag(tag: u64) -> bool {
    tag == APPEARANCE_COLOR_SWATCH_TAG
}

/// Returns the 64-bit semantic tag for the HEX text input box.
#[inline]
pub fn appearance_hex_input_tag() -> u64 {
    APPEARANCE_HEX_TAG
}

/// Resolves whether a semantic tag corresponds to the HEX text input box.
#[inline]
pub fn resolve_appearance_hex_input_tag(tag: u64) -> bool {
    tag == APPEARANCE_HEX_TAG
}

/// Returns the 64-bit semantic tag for the 'Add to Palette' (+) button.
#[inline]
pub fn appearance_add_palette_tag() -> u64 {
    APPEARANCE_ADD_PALETTE_TAG
}

/// Resolves whether a semantic tag corresponds to the 'Add to Palette' (+) button.
#[inline]
pub fn resolve_appearance_add_palette_tag(tag: u64) -> bool {
    tag == APPEARANCE_ADD_PALETTE_TAG
}

/// Returns the 64-bit semantic tag for the 'Clear Palette' (🗑) button.
#[inline]
pub fn appearance_clear_palette_tag() -> u64 {
    APPEARANCE_CLEAR_PALETTE_TAG
}

/// Resolves whether a semantic tag corresponds to the 'Clear Palette' (🗑) button.
#[inline]
pub fn resolve_appearance_clear_palette_tag(tag: u64) -> bool {
    tag == APPEARANCE_CLEAR_PALETTE_TAG
}

/// Computes the 64-bit semantic tag for a palette swatch pill at a given global index.
#[inline]
pub fn appearance_palette_swatch_tag(idx: usize) -> u64 {
    APPEARANCE_SWATCH_BASE | ((idx as u64) & 0x00FF_FFFF)
}

/// Resolves whether a semantic tag corresponds to a palette swatch pill, returning its index.
#[inline]
pub fn resolve_appearance_palette_swatch_tag(tag: u64) -> Option<usize> {
    if (tag & 0xFF00_0000) == APPEARANCE_SWATCH_BASE {
        let idx = (tag & 0x00FF_FFFF) as usize;
        if idx <= 1024 {
            return Some(idx);
        }
    }
    None
}

/// Builds the `🎨 Appearance` card declaratively in the `UiTree` and returns the computed height.
///
/// Fully adheres to 100% declarative [`UiScope`] architecture without raw arena allocation
/// or manual pixel coordinate manipulations.
pub fn build_appearance_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    let obj_color = ctx
        .world
        .get::<&ae_core::ecs::Color>(ctx.entity)
        .map(|c| Color::rgba(c.r, c.g, c.b, c.a))
        .unwrap_or(Color::rgba(0.60, 0.75, 0.95, 1.0));

    scope.card_named("AppearanceCard", "🎨 Appearance", |card| {
        render_object_color_row(
            card,
            obj_color,
            ctx.params.is_color_picker_open,
            ctx.params.inspector_color_hex,
            ctx.params.active_hex_buffer,
            ctx.params.blink_caret,
            ctx.params.cursor_pos,
        );

        render_palette_action_row(card, ctx.params.cursor_pos);

        render_palette_swatches(card, ctx.params.saved_swatches, ctx.params.cursor_pos);
    });
}

/// Renders Row 1: Object Color swatch pill and live HEX text input field.
fn render_object_color_row(
    scope: &mut UiScope<'_>,
    obj_color: Color,
    is_color_picker_open: bool,
    color_hex: &str,
    active_hex_buffer: Option<&str>,
    blink_caret: bool,
    _cursor_pos: Point,
) -> (WidgetId, WidgetId) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(22.0)
        .gap(8.0);

    let mut captured_swatch_id = WidgetId::default();
    let mut captured_hex_id = WidgetId::default();

    scope.container_named("ObjectColorRow", row_style, |row| {
        row.label_styled_passive(
            "ObjectColorLabel",
            "Object Color:",
            11.0,
            Color::rgba(0.620, 0.635, 0.678, 1.0),
            TextAlign::Left,
            Style::new().width(75.0).height(22.0),
        );

        let swatch_tag = appearance_color_swatch_tag();
        let is_swatch_hovered = row.is_tag_hovered(swatch_tag);
        let swatch_border = if is_swatch_hovered || is_color_picker_open {
            Color::rgba(1.0, 1.0, 1.0, 0.95)
        } else {
            Color::rgba(0.85, 0.88, 0.95, 0.70)
        };
        let swatch_style = Style::new()
            .width(38.0)
            .height(18.0)
            .background(obj_color)
            .border(1.0, swatch_border)
            .border_radius(5.0);
        captured_swatch_id = row.empty_box_tagged(
            "ColorSwatchBox",
            swatch_style,
            WidgetRole::Button,
            swatch_tag,
        );

        row.label_styled_passive(
            "HexPrefixLabel",
            "Hex:",
            11.0,
            Color::rgba(0.620, 0.635, 0.678, 1.0),
            TextAlign::Left,
            Style::new().width(28.0).height(22.0),
        );

        let hex_tag = appearance_hex_input_tag();
        let is_hex_focused = active_hex_buffer.is_some();
        let is_hex_hovered = row.is_tag_hovered(hex_tag);
        let (hex_bg, hex_border) = if is_hex_focused {
            (
                Color::rgba(0.180, 0.190, 0.220, 1.0),
                Color::rgba(0.85, 0.88, 0.98, 0.95),
            )
        } else if is_hex_hovered {
            (
                Color::rgba(0.180, 0.190, 0.220, 1.0),
                Color::rgba(0.35, 0.38, 0.45, 0.95),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
            )
        };

        let mut edit_buf = String::new();
        let hex_display_str: &str = if let Some(buf) = active_hex_buffer {
            if blink_caret {
                edit_buf.push_str(buf);
                edit_buf.push('|');
                &edit_buf
            } else {
                buf
            }
        } else {
            color_hex
        };

        let hex_box_style = Style::new()
            .width(64.0)
            .height(22.0)
            .background(hex_bg)
            .border(1.0, hex_border)
            .border_radius(5.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);

        captured_hex_id = row.container_tagged(
            "HexInputBox",
            hex_box_style,
            WidgetRole::TextInput,
            hex_tag,
            |hex_scope| {
                let hex_text_col = if is_hex_focused {
                    Color::WHITE
                } else {
                    Color::rgba(0.886, 0.894, 0.918, 1.0)
                };
                hex_scope.label_styled_passive(
                    "HexInputText",
                    hex_display_str,
                    10.5,
                    hex_text_col,
                    TextAlign::Center,
                    Style::new().flex_grow(1.0).height(22.0),
                );
            },
        );
    });

    (captured_swatch_id, captured_hex_id)
}

/// Renders Row 2: Add to palette (+) and clear custom palette (🗑) action buttons.
fn render_palette_action_row(scope: &mut UiScope<'_>, _cursor_pos: Point) -> (WidgetId, WidgetId) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(22.0)
        .gap(6.0);

    let mut captured_add_id = WidgetId::default();
    let mut captured_clr_id = WidgetId::default();

    scope.container_named("AddPaletteRow", row_style, |row| {
        row.label_styled_passive(
            "AddPaletteLabel",
            "Add to Palette:",
            11.0,
            Color::rgba(0.620, 0.635, 0.678, 1.0),
            TextAlign::Left,
            Style::new().width(88.0).height(22.0),
        );

        let btn_size = 18.0;

        let add_tag = appearance_add_palette_tag();
        let is_add_hovered = row.is_tag_hovered(add_tag);
        let (add_bg, add_border, add_text_col) = if is_add_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.82, 0.84, 0.88, 1.0),
            )
        };
        let add_style = Style::new()
            .width(btn_size)
            .height(btn_size)
            .background(add_bg)
            .border(1.0, add_border)
            .border_radius(5.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        captured_add_id = row.container_tagged(
            "AddPaletteBtn",
            add_style,
            WidgetRole::Button,
            add_tag,
            |b| {
                b.label_styled_passive(
                    "AddPaletteText",
                    "+",
                    13.0,
                    add_text_col,
                    TextAlign::Center,
                    Style::new().height(btn_size),
                );
            },
        );

        let clr_tag = appearance_clear_palette_tag();
        let is_clr_hovered = row.is_tag_hovered(clr_tag);
        let (clr_bg, clr_border, clr_text_col) = if is_clr_hovered {
            (
                Color::rgba(0.35, 0.10, 0.10, 0.95),
                Color::rgba(0.70, 0.18, 0.18, 0.85),
                Color::rgba(1.0, 0.40, 0.40, 1.0),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.70, 0.73, 0.80, 0.90),
            )
        };
        let clr_style = Style::new()
            .width(btn_size)
            .height(btn_size)
            .background(clr_bg)
            .border(1.0, clr_border)
            .border_radius(5.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        captured_clr_id = row.container_tagged(
            "ClearPaletteBtn",
            clr_style,
            WidgetRole::Button,
            clr_tag,
            |b| {
                b.label_styled_passive(
                    "ClearPaletteText",
                    "🗑",
                    10.5,
                    clr_text_col,
                    TextAlign::Center,
                    Style::new().height(btn_size),
                );
            },
        );
    });

    (captured_add_id, captured_clr_id)
}

/// Renders Row 3+: Palette swatches inside a multi-line wrapping flex container.
fn render_palette_swatches(
    scope: &mut UiScope<'_>,
    saved_swatches: &[[f32; 4]],
    _cursor_pos: Point,
) -> Vec<(usize, WidgetId, Color)> {
    let swatch_size = 16.0;
    let swatch_gap = 6.0;

    let container_style = Style::new()
        .flex_row()
        .flex_wrap(FlexWrap::Wrap)
        .align_items(AlignItems::Center)
        .gap(swatch_gap);

    let mut swatch_nodes = Vec::with_capacity(7 + saved_swatches.len());

    scope.container_named("PaletteSwatches", container_style, |wrap_scope| {
        for (idx, &col) in DEFAULT_PALETTE.iter().enumerate() {
            let tag = appearance_palette_swatch_tag(idx);
            let is_hovered = wrap_scope.is_tag_hovered(tag);
            let border_col = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.212, 0.220, 0.259, 0.85)
            };
            let sw_style = Style::new()
                .width(swatch_size)
                .height(swatch_size)
                .background(col)
                .border(1.0, border_col)
                .border_radius(3.0);
            let sw_id =
                wrap_scope.empty_box_tagged("PaletteSwatch", sw_style, WidgetRole::Button, tag);
            swatch_nodes.push((idx, sw_id, col));
        }

        for (custom_idx, &s) in saved_swatches.iter().enumerate() {
            let global_idx = 7 + custom_idx;
            let col = Color::rgba(s[0], s[1], s[2], s[3]);
            let tag = appearance_palette_swatch_tag(global_idx);
            let is_hovered = wrap_scope.is_tag_hovered(tag);
            let border_col = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.35, 0.38, 0.45, 0.85)
            };
            let sw_style = Style::new()
                .width(swatch_size)
                .height(swatch_size)
                .background(col)
                .border(1.0, border_col)
                .border_radius(3.0);
            let sw_id =
                wrap_scope.empty_box_tagged("PaletteSwatch", sw_style, WidgetRole::Button, tag);
            swatch_nodes.push((global_idx, sw_id, col));
        }
    });

    swatch_nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appearance_semantic_tags() {
        let swatch_tag = appearance_color_swatch_tag();
        assert!(resolve_appearance_color_swatch_tag(swatch_tag));
        assert!(!resolve_appearance_color_swatch_tag(swatch_tag + 1));

        let hex_tag = appearance_hex_input_tag();
        assert!(resolve_appearance_hex_input_tag(hex_tag));
        assert!(!resolve_appearance_hex_input_tag(hex_tag + 1));

        let add_tag = appearance_add_palette_tag();
        assert!(resolve_appearance_add_palette_tag(add_tag));
        assert!(!resolve_appearance_add_palette_tag(add_tag + 1));

        let clr_tag = appearance_clear_palette_tag();
        assert!(resolve_appearance_clear_palette_tag(clr_tag));
        assert!(!resolve_appearance_clear_palette_tag(clr_tag + 1));

        for idx in [0, 1, 6, 7, 15] {
            let tag = appearance_palette_swatch_tag(idx);
            assert_eq!(resolve_appearance_palette_swatch_tag(tag), Some(idx));
        }
        assert_eq!(resolve_appearance_palette_swatch_tag(0), None);
    }

    #[test]
    fn test_declarative_appearance_card_structure_and_layout() {
        let mut tree = UiTree::new();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Color {
            r: 0.2,
            g: 0.4,
            b: 0.8,
            a: 1.0,
        },));

        let euler = [0.0, 0.0, 0.0];
        let saved_swatches = vec![[1.0, 0.5, 0.0, 1.0], [0.0, 1.0, 1.0, 1.0]];

        let params = crate::ui::iris_bridge::inspector::InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 900.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#3366cc",
            saved_swatches: &saved_swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
            hovered_tag: None,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 280.0);

        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        let root = scope.container_named(
            "TestContainer",
            Style::new().width(280.0).height(300.0),
            |card_scope| {
                build_appearance_card(card_scope, &mut ctx);
            },
        );
        layout_subtree(&mut tree, root, Rect::new(10.0, 20.0, 280.0, 300.0));

        let has_swatch = tree
            .iter()
            .any(|(_, n)| n.tag == appearance_color_swatch_tag());
        assert!(has_swatch, "Color swatch button must be tagged");

        let has_hex = tree
            .iter()
            .any(|(_, n)| n.tag == appearance_hex_input_tag());
        assert!(has_hex, "Hex input box must be tagged");

        let has_add = tree
            .iter()
            .any(|(_, n)| n.tag == appearance_add_palette_tag());
        assert!(has_add, "Add palette button must be tagged");

        let has_clr = tree
            .iter()
            .any(|(_, n)| n.tag == appearance_clear_palette_tag());
        assert!(has_clr, "Clear palette button must be tagged");

        // 7 default swatches + 2 saved swatches = 9
        let swatches_count = (0..9)
            .filter(|&idx| {
                tree.iter()
                    .any(|(_, n)| n.tag == appearance_palette_swatch_tag(idx))
            })
            .count();
        assert_eq!(
            swatches_count, 9,
            "All 9 palette swatches must be tagged in the tree"
        );
    }
}