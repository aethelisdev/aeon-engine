// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Appearance and Color Palette Declarative Inspector Card
//!
//! Renders object color swatch, live HEX text input, and quick-select palette swatches
//! using purely declarative [`UiScope`] containers, flexbox layout, and O(1) semantic tags.

use super::registry::ComponentRenderContext;
use super::types::InspectorAction;
use irisui::prelude::*;

use super::tags::{
    TAG_INSPECTOR_ADD_PALETTE, TAG_INSPECTOR_CLEAR_PALETTE, TAG_INSPECTOR_COLOR_HEX,
    TAG_INSPECTOR_COLOR_SWATCH, encode_palette_swatch_tag, resolve_palette_swatch_tag,
};

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
#[must_use]
pub const fn appearance_color_swatch_tag() -> u64 {
    TAG_INSPECTOR_COLOR_SWATCH
}

/// Resolves whether a semantic tag corresponds to the primary object color swatch button.
#[inline]
#[must_use]
pub const fn resolve_appearance_color_swatch_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_COLOR_SWATCH
}

/// Returns the 64-bit semantic tag for the HEX text input box.
#[inline]
#[must_use]
pub const fn appearance_hex_input_tag() -> u64 {
    TAG_INSPECTOR_COLOR_HEX
}

/// Resolves whether a semantic tag corresponds to the HEX text input box.
#[inline]
#[must_use]
pub const fn resolve_appearance_hex_input_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_COLOR_HEX
}

/// Returns the 64-bit semantic tag for the 'Add to Palette' (+) button.
#[inline]
#[must_use]
pub const fn appearance_add_palette_tag() -> u64 {
    TAG_INSPECTOR_ADD_PALETTE
}

/// Resolves whether a semantic tag corresponds to the 'Add to Palette' (+) button.
#[inline]
#[must_use]
pub const fn resolve_appearance_add_palette_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_ADD_PALETTE
}

/// Returns the 64-bit semantic tag for the 'Clear Palette' (🗑) button.
#[inline]
#[must_use]
pub const fn appearance_clear_palette_tag() -> u64 {
    TAG_INSPECTOR_CLEAR_PALETTE
}

/// Resolves whether a semantic tag corresponds to the 'Clear Palette' (🗑) button.
#[inline]
#[must_use]
pub const fn resolve_appearance_clear_palette_tag(tag: u64) -> bool {
    tag == TAG_INSPECTOR_CLEAR_PALETTE
}

/// Computes the 64-bit semantic tag for a palette swatch pill at a given global index.
#[inline]
#[must_use]
pub const fn appearance_palette_swatch_tag(idx: usize) -> u64 {
    encode_palette_swatch_tag(idx)
}

/// Resolves whether a semantic tag corresponds to a palette swatch pill, returning its index.
#[inline]
#[must_use]
pub const fn resolve_appearance_palette_swatch_tag(tag: u64) -> Option<usize> {
    resolve_palette_swatch_tag(tag)
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

    let entity = ctx.entity;
    let is_color_picker_open = ctx.params.is_color_picker_open;
    let inspector_color_hex = ctx.params.inspector_color_hex;
    let active_hex_buffer = ctx.params.active_hex_buffer;
    let blink_caret = ctx.params.blink_caret;
    let saved_swatches = ctx.params.saved_swatches;
    let mut actions = ctx.actions.take();

    scope.card_named("AppearanceCard", "🎨 Appearance", |card| {
        render_object_color_row(
            card,
            obj_color,
            is_color_picker_open,
            inspector_color_hex,
            active_hex_buffer,
            blink_caret,
            actions.as_deref_mut(),
        );

        render_palette_action_row(card, obj_color, actions.as_deref_mut());

        render_palette_swatches(card, entity, saved_swatches, actions.as_deref_mut());
    });

    ctx.actions = actions;
}

/// Renders Row 1: Object Color swatch pill and live HEX text input field.
fn render_object_color_row(
    scope: &mut UiScope<'_>,
    obj_color: Color,
    is_color_picker_open: bool,
    color_hex: &str,
    active_hex_buffer: Option<&str>,
    blink_caret: bool,
    mut actions: Option<&mut Vec<InspectorAction>>,
) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(22.0)
        .gap(8.0);

    scope.container_named("ObjectColorRow", row_style, |row| {
        row.label_styled_passive(
            "ObjectColorLabel",
            "Object Color:",
            11.0,
            row.theme().text_muted,
            TextAlign::Left,
            Style::new().width(75.0).height(22.0),
        );

        if row
            .add(
                ColorSwatch::new(obj_color)
                    .active(is_color_picker_open)
                    .tag(appearance_color_swatch_tag()),
            )
            .clicked()
            && let Some(ref mut act) = actions
        {
            act.push(InspectorAction::ToggleColorPicker);
        }

        row.label_styled_passive(
            "HexPrefixLabel",
            "Hex:",
            11.0,
            row.theme().text_muted,
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

        if row
            .container_tagged_resp(
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
            )
            .clicked()
            && let Some(ref mut act) = actions
        {
            act.push(InspectorAction::FocusHexInput);
        }
    });
}

/// Renders Row 2: Add to palette (+) and clear custom palette (🗑) action buttons.
fn render_palette_action_row(
    scope: &mut UiScope<'_>,
    obj_color: Color,
    mut actions: Option<&mut Vec<InspectorAction>>,
) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(22.0)
        .gap(6.0);

    scope.container_named("AddPaletteRow", row_style, |row| {
        row.label_styled_passive(
            "AddPaletteLabel",
            "Add to Palette:",
            11.0,
            row.theme().text_muted,
            TextAlign::Left,
            Style::new().width(88.0).height(22.0),
        );

        if row
            .add(
                Button::new("+")
                    .size(18.0, 18.0)
                    .padding(Insets::ZERO)
                    .tag(appearance_add_palette_tag()),
            )
            .clicked()
            && let Some(ref mut act) = actions
        {
            act.push(InspectorAction::AddColorToPalette(obj_color));
        }

        if row
            .add(
                Button::new("🗑")
                    .size(18.0, 18.0)
                    .padding(Insets::ZERO)
                    .tag(appearance_clear_palette_tag()),
            )
            .clicked()
            && let Some(ref mut act) = actions
        {
            act.push(InspectorAction::ClearCustomPalette);
        }
    });
}

/// Renders Row 3+: Palette swatches inside a multi-line wrapping flex container.
fn render_palette_swatches(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    saved_swatches: &[[f32; 4]],
    mut actions: Option<&mut Vec<InspectorAction>>,
) {
    let swatch_size = 16.0;
    let swatch_gap = 6.0;

    let container_style = Style::new()
        .flex_row()
        .flex_wrap(FlexWrap::Wrap)
        .align_items(AlignItems::Center)
        .gap(swatch_gap);

    scope.container_named("PaletteSwatches", container_style, |wrap_scope| {
        for (idx, &col) in DEFAULT_PALETTE.iter().enumerate() {
            let tag = appearance_palette_swatch_tag(idx);
            if wrap_scope
                .add(
                    ColorSwatch::new(col)
                        .size(swatch_size, swatch_size)
                        .corner_radius(3.0)
                        .tag(tag),
                )
                .clicked()
                && let Some(ref mut act) = actions
            {
                act.push(InspectorAction::SetObjectColor(entity, col));
            }
        }

        for (custom_idx, &s) in saved_swatches.iter().enumerate() {
            let global_idx = 7 + custom_idx;
            let col = Color::rgba(s[0], s[1], s[2], s[3]);
            let tag = appearance_palette_swatch_tag(global_idx);
            if wrap_scope
                .add(
                    ColorSwatch::new(col)
                        .size(swatch_size, swatch_size)
                        .corner_radius(3.0)
                        .tag(tag),
                )
                .clicked()
                && let Some(ref mut act) = actions
            {
                act.push(InspectorAction::SetObjectColor(entity, col));
            }
        }
    });
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
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 280.0);

        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        scope.container_rect(
            "TestContainer",
            Style::new().width(280.0).height(300.0),
            Rect::new(10.0, 20.0, 280.0, 300.0),
            |card_scope| {
                build_appearance_card(card_scope, &mut ctx);
            },
        );

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