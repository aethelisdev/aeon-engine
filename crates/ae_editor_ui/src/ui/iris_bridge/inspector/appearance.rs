// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Appearance and Color Palette Declarative Inspector Card
//!
//! Renders object color swatch, live HEX text input, and quick-select palette swatches
//! using purely declarative [`UiScope`] containers, flexbox layout, and O(1) semantic tags.

use crate::ui::types::EngineUiAction;
use irisui::prelude::*;

/// Semantic tag for the interactive object color swatch button.
pub const TAG_APPEARANCE_COLOR_SWATCH: u64 = 0xDA00_0007;

/// Semantic tag for the hexadecimal color text input box.
pub const TAG_APPEARANCE_COLOR_HEX: u64 = 0xDA00_0008;

/// Base numeric domain for color palette swatch buttons.
pub const TAG_APPEARANCE_PALETTE_BASE: u64 = 0xDF20_0000;

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
    TAG_APPEARANCE_COLOR_SWATCH
}

/// Returns the 64-bit semantic tag for the HEX text input box.
#[inline]
#[must_use]
pub const fn appearance_hex_input_tag() -> u64 {
    TAG_APPEARANCE_COLOR_HEX
}

/// Computes the 64-bit semantic tag for a palette swatch pill at a given global index.
#[inline]
#[must_use]
pub const fn appearance_palette_swatch_tag(idx: usize) -> u64 {
    TAG_APPEARANCE_PALETTE_BASE + idx as u64
}

/// Extracts the current [`ae_core::ecs::Color`] of an entity, or returns the canonical fallback.
#[inline]
#[must_use]
pub fn entity_color(world: &hecs::World, entity: hecs::Entity) -> ae_core::ecs::Color {
    world
        .get::<&ae_core::ecs::Color>(entity)
        .map(|c| *c)
        .unwrap_or(ae_core::ecs::Color {
            r: 0.60,
            g: 0.75,
            b: 0.95,
            a: 1.0,
        })
}

/// Converts an [`ae_core::ecs::Color`] into an Iris UI [`Color`].
#[inline]
#[must_use]
pub const fn to_iris_color(c: ae_core::ecs::Color) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}

/// Converts an Iris UI [`Color`] into an [`ae_core::ecs::Color`].
#[inline]
#[must_use]
pub const fn to_ecs_color(c: Color) -> ae_core::ecs::Color {
    ae_core::ecs::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

/// Renders the `🎨 Appearance` card declaratively and maps any committed color change to [`EngineUiAction::ModifyColor`].
pub fn render_appearance_card(
    scope: &mut UiScope<'_>,
    current_col: ae_core::ecs::Color,
    entity: hecs::Entity,
    hex_str: &str,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<EngineUiAction> {
    scope
        .color_palette_card(
            ColorPaletteCardConfig::new(
                "AppearanceCard",
                "🎨 Appearance",
                to_iris_color(current_col),
                hex_str,
                &DEFAULT_PALETTE,
            )
            .with_tags(appearance_color_swatch_tag(), appearance_hex_input_tag()),
            saved_swatches,
            is_color_picker_open,
        )
        .map(|new_col| EngineUiAction::ModifyColor(entity, current_col, to_ecs_color(new_col)))
}

/// Renders the declarative `🎨 Appearance` card by delegating to [`render_appearance_card`].
///
/// Standardizes card rendering alongside other Inspector standalone cards (`draw_transform_card`, `draw_ui_transform_card`).
pub fn draw_appearance_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
    hex_str: &str,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<EngineUiAction> {
    render_appearance_card(
        scope,
        entity_color(world, entity),
        entity,
        hex_str,
        saved_swatches,
        is_color_picker_open,
    )
}

/// Renders an in-place declarative property row using [`UiScope::prop_color_rgba`]
/// directly bound to an [`ae_core::ecs::Color`] component without heap allocations.
pub fn draw_color_property(
    scope: &mut UiScope<'_>,
    label: &str,
    color: &mut ae_core::ecs::Color,
) -> PropertyColorResponse {
    scope.prop_color_rgba(
        label,
        (&mut color.r, &mut color.g, &mut color.b, &mut color.a),
    )
}

/// Builds the `🎨 Appearance` card declaratively in the `UiTree` and returns
/// any committed color mutation directly as [`EngineUiAction::ModifyColor`].
///
/// Fully adheres to 100% declarative [`UiScope`] architecture without intermediate
/// `let` plumbing or manual pixel coordinate manipulations.
pub fn build_appearance_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
    hex_str: &str,
    saved_swatches: &mut Vec<[f32; 4]>,
    is_color_picker_open: &mut bool,
) -> Option<EngineUiAction> {
    draw_appearance_card(
        scope,
        world,
        entity,
        hex_str,
        saved_swatches,
        is_color_picker_open,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appearance_semantic_tags() {
        assert_eq!(appearance_color_swatch_tag(), TAG_APPEARANCE_COLOR_SWATCH);
        assert_eq!(appearance_hex_input_tag(), TAG_APPEARANCE_COLOR_HEX);
        for idx in [0, 1, 6, 7, 15] {
            assert_eq!(
                appearance_palette_swatch_tag(idx),
                TAG_APPEARANCE_PALETTE_BASE + idx as u64
            );
        }
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

        let mut saved_swatches = vec![[1.0, 0.5, 0.0, 1.0], [0.0, 1.0, 1.0, 1.0]];
        let mut is_color_picker_open = false;

        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        scope.container_rect(
            "TestContainer",
            Style::new().width(280.0).height(300.0),
            Rect::new(10.0, 20.0, 280.0, 300.0),
            |card_scope| {
                build_appearance_card(
                    card_scope,
                    &world,
                    entity,
                    "#3366cc",
                    &mut saved_swatches,
                    &mut is_color_picker_open,
                );
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
            .any(|(_, n)| n.name.as_deref() == Some("AddPaletteButton"));
        assert!(has_add, "Add palette button must be present in tree");

        let has_clear = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ClearPaletteButton"));
        assert!(has_clear, "Clear palette button must be present in tree");
    }

    #[test]
    fn test_appearance_card_palette_direct_mutations() {
        let mut tree = UiTree::new();
        let mut saved_swatches = Vec::new();

        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        scope.card_named("AppearanceCard", "🎨 Appearance", |card| {
            let resp = card.prop_color_palette(
                "Add to Palette:",
                Color::rgba(0.25, 0.50, 0.75, 1.0),
                &DEFAULT_PALETTE,
                &mut saved_swatches,
            );
            assert_eq!(resp.selected_color, None);
        });

        assert_eq!(saved_swatches.len(), 0);
    }

    #[test]
    fn test_appearance_card_hex_commit_produces_modify_color() {
        let mut tree = UiTree::new();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        },));

        let hex_tag = appearance_hex_input_tag();
        tree.event_state
            .start_text_edit(None, hex_tag, "#ff5500", 7, None);
        tree.event_state.frame_key_pressed = true;
        tree.event_state.frame_key = Some(KeyCode::Enter);

        let mut saved_swatches = Vec::new();
        let mut is_color_picker_open = false;

        let action = {
            let mut scope = UiScope::new(&mut tree, WidgetId::default());
            build_appearance_card(
                &mut scope,
                &world,
                entity,
                "#1a334d",
                &mut saved_swatches,
                &mut is_color_picker_open,
            )
        };

        assert!(matches!(
            action,
            Some(EngineUiAction::ModifyColor(e, old, new))
                if e == entity
                    && (old.r - 0.1).abs() < 1e-4
                    && (new.r - 1.0).abs() < 1e-4
        ));
    }

    #[test]
    fn test_draw_color_property_standalone() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut scope = UiScope::new(&mut tree, root);
        let mut color = ae_core::ecs::Color {
            r: 0.5,
            g: 0.6,
            b: 0.7,
            a: 1.0,
        };

        draw_color_property(&mut scope, "Object Color:", &mut color);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ColorSwatchPill")),
            "ColorSwatchPill node must be created"
        );
    }

    #[test]
    fn test_draw_appearance_card_standalone() {
        let mut tree = UiTree::new();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Color {
            r: 0.2,
            g: 0.4,
            b: 0.8,
            a: 1.0,
        },));

        let mut saved_swatches = vec![[1.0, 0.5, 0.0, 1.0]];
        let mut is_color_picker_open = false;

        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        draw_appearance_card(
            &mut scope,
            &world,
            entity,
            "#3366cc",
            &mut saved_swatches,
            &mut is_color_picker_open,
        );

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("AppearanceCard")),
            "AppearanceCard container must be rendered"
        );
    }
}