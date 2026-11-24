// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Primitives Inspector Cards
//!
//! Provides declarative inspector cards for core UI Designer widgets:
//! - `UiElement`: Primary 2D screen transform placeholder
//! - `UiPanel`: Background panel with border width and corner radius
//! - `UiText`: Text label with font size and alignment dropdown
//! - `UiProgressBar`: Numeric min/max/value scrubbers with visual bar preview
//! - `UiButton`: Interactive button with enable/disable checkbox
//! - `UiImage`: Sprite image display with slice mode label

use ae_ui::UiTextAlignment;
use irisui::prelude::*;

/// Maps [`UiTextAlignment`] enum variant to combo dropdown selection index.
#[inline]
const fn alignment_to_index(alignment: UiTextAlignment) -> usize {
    match alignment {
        UiTextAlignment::Left => 0,
        UiTextAlignment::Center => 1,
        UiTextAlignment::Right => 2,
    }
}

/// Maps combo dropdown selection index back to [`UiTextAlignment`].
#[inline]
const fn index_to_alignment(index: usize) -> UiTextAlignment {
    match index {
        0 => UiTextAlignment::Left,
        1 => UiTextAlignment::Center,
        _ => UiTextAlignment::Right,
    }
}

/// Computes the normalized [0.0, 1.0] fraction for progress bar rendering.
#[inline]
fn progress_bar_fraction(min: f32, max: f32, value: f32) -> f32 {
    if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Returns the static slice mode description label without runtime heap allocations.
#[inline]
const fn slice_mode_text(slice_mode: ae_plugin_api::UiSliceMode) -> &'static str {
    match slice_mode {
        ae_plugin_api::UiSliceMode::Stretch => "Slice Mode: Stretch",
        ae_plugin_api::UiSliceMode::Fit => "Slice Mode: Fit",
        ae_plugin_api::UiSliceMode::NineSlice => "Slice Mode: NineSlice",
        ae_plugin_api::UiSliceMode::Tile => "Slice Mode: Tile",
    }
}

/// Renders the declarative 2D Screen UI Element (RectTransform) component card.
///
/// Handled as the primary 2D Screen Transform card at the top of the Inspector
/// when [`ae_plugin_api::UiElement`] is present on [`ae_ecs::Entity`].
#[inline]
pub fn draw_ui_element_card(
    _scope: &mut UiScope<'_>,
    _world: &ae_ecs::World,
    _entity: ae_ecs::Entity,
) {
    // Handled as the primary 2D Screen Transform card at the top of the Inspector
}

/// Renders the declarative UI Background Panel component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Modifies border width and corner radius in-place via [`UiScope::prop_f32`] with zero intermediate variable plumbing.
pub fn draw_ui_panel_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .mutate::<ae_plugin_api::UiPanel, _>(entity, |p| {
            scope.card_panel("UiPanelCard", |card| {
                card.card_header_desc(UiScope::comp_card_desc(
                    "UiPanel",
                    "UI Background Panel",
                    "🔲",
                    Color::rgba(0.20, 0.85, 1.0, 1.0),
                ));
                card.prop_f32("Border Width:", &mut p.border_width, 0.0..=50.0, 0.5);
                card.prop_f32("Corner Radius:", &mut p.corner_radius, 0.0..=100.0, 0.5);
            });
        })
        .ok();
}

/// Renders the declarative UI Text Label component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Modifies text string, font size, and text alignment in-place with zero intermediate variable plumbing.
pub fn draw_ui_text_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .mutate::<ae_plugin_api::UiText, _>(entity, |t| {
            scope.card_panel("UiTextCard", |card| {
                card.card_header_desc(UiScope::comp_card_desc(
                    "UiText",
                    "UI Text Label",
                    "🔤",
                    Color::rgba(0.20, 0.85, 1.0, 1.0),
                ));
                card.prop_text("Text", &mut t.text);
                card.prop_f32("Font Size:", &mut t.font_size, 6.0..=128.0, 0.5);
                card.prop_combobox_mapped_styled(
                    "Align:",
                    alignment_to_index(t.alignment),
                    &["Left", "Center", "Right"],
                    ComboRowLayout::new(52.0, 80.0),
                    &mut t.alignment,
                    index_to_alignment,
                );
            });
        })
        .ok();
}

/// Renders the declarative UI Progress Bar component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Modifies min, max, and current value scrubbers with visual bar rendering and zero intermediate variable plumbing.
pub fn draw_ui_progress_bar_card(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    entity: ae_ecs::Entity,
) {
    world
        .mutate::<ae_plugin_api::UiProgressBar, _>(entity, |pb| {
            scope.card_panel("UiProgressBarCard", |card| {
                card.card_header_desc(UiScope::comp_card_desc(
                    "UiProgressBar",
                    "UI Progress Bar",
                    "📊",
                    Color::rgba(0.20, 0.85, 1.0, 1.0),
                ));
                card.prop_f32("Min:", &mut pb.min, -100_000.0..=100_000.0, 1.0);
                card.prop_f32("Max:", &mut pb.max, -100_000.0..=100_000.0, 1.0);
                card.prop_f32("Value:", &mut pb.value, pb.min..=pb.max.max(pb.min), 1.0);
                card.progress_bar_named(
                    "MiniBarTrack",
                    "MiniBarFill",
                    240.0,
                    10.0,
                    progress_bar_fraction(pb.min, pb.max, pb.value),
                    card.theme().accent,
                );
            });
        })
        .ok();
}

/// Renders the declarative Interactive Button component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Modifies button enabled state in-place with zero intermediate variable plumbing.
pub fn draw_ui_button_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    world
        .mutate::<ae_plugin_api::UiButton, _>(entity, |b| {
            scope.card_panel("UiButtonCard", |card| {
                card.card_header_desc(UiScope::comp_card_desc(
                    "UiButton",
                    "Interactive Button",
                    "🔘",
                    Color::rgba(0.40, 0.75, 1.0, 1.0),
                ));
                card.prop_checkbox("Enabled", &mut b.is_enabled);
            });
        })
        .ok();
}

/// Renders the declarative UI Sprite Image component card into [`UiScope`] if present on [`ae_ecs::Entity`].
///
/// Displays the active sprite slice mode with zero intermediate variable plumbing.
pub fn draw_ui_image_card(scope: &mut UiScope<'_>, world: &ae_ecs::World, entity: ae_ecs::Entity) {
    scope.card_panel_desc_when_ok(
        "UiImageCard",
        UiScope::comp_card_desc(
            "UiImage",
            "UI Sprite Image",
            "🖼️",
            Color::rgba(0.20, 0.85, 1.0, 1.0),
        ),
        world.inspect::<ae_plugin_api::UiImage, _>(entity, |img| img.slice_mode),
        |card, slice_mode| {
            card.label_desc_colored(
                "UiImageMode",
                slice_mode_text(slice_mode),
                card.theme().text_primary,
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_panel_render_and_in_place_mutation() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiPanel {
            border_width: 2.0,
            corner_radius: 4.0,
            ..Default::default()
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_ui_panel_card(&mut scope, &world, entity);
        }

        // Verify direct in-place mutation
        {
            let p = world.get_mut::<ae_plugin_api::UiPanel>(entity).unwrap();
            p.border_width = 5.0;
            p.corner_radius = 8.0;
        }
        world
            .inspect::<ae_plugin_api::UiPanel, _>(entity, |panel| {
                assert_eq!(panel.border_width, 5.0);
                assert_eq!(panel.corner_radius, 8.0);
            })
            .unwrap();
    }

    #[test]
    fn test_draw_ui_panel_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiPanel::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_panel_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiPanelCard")),
            "UiPanelCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "UiPanel must emit 2 numeric drag pills (Border Width, Corner Radius)"
        );
    }

    #[test]
    fn test_draw_ui_text_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiText::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_text_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiTextCard")),
            "UiTextCard container must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| n.role == WidgetRole::TextInput),
            "TextInput must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ComboPill")),
            "ComboPill must be rendered for Alignment"
        );
    }

    #[test]
    fn test_draw_ui_progress_bar_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiProgressBar {
            value: 50.0,
            min: 0.0,
            max: 100.0,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_progress_bar_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiProgressBarCard")),
            "UiProgressBarCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "UiProgressBar must emit 3 numeric pills (Min, Max, Value)"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("MiniBarTrack")),
            "MiniBarTrack must be rendered"
        );
    }

    #[test]
    fn test_ui_progress_bar_inverted_min_max_does_not_panic() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiProgressBar {
            value: 50.0,
            min: 141.0,
            max: 100.0,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_progress_bar_card(&mut scope, &world, entity);
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiProgressBarCard"))
        );
    }

    #[test]
    fn test_ui_button_toggle_mutation() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiButton {
            is_enabled: true,
            ..Default::default()
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        {
            let mut scope = UiScope::new(&mut tree, root);
            draw_ui_button_card(&mut scope, &world, entity);
        }

        {
            let b = world.get_mut::<ae_plugin_api::UiButton>(entity).unwrap();
            b.is_enabled = false;
        }
        assert!(
            !world
                .inspect::<ae_plugin_api::UiButton, _>(entity, |b| b.is_enabled)
                .unwrap()
        );
    }

    #[test]
    fn test_draw_ui_button_card_standalone() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiButton::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_button_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiButtonCard")),
            "UiButtonCard container must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| n.role == WidgetRole::Checkbox),
            "UiButton checkbox must be rendered"
        );
    }

    #[test]
    fn test_draw_ui_image_card_standalone() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = ae_ecs::World::new();
        let entity = world.spawn((ae_plugin_api::UiImage {
            slice_mode: ae_ui::UiSliceMode::NineSlice,
            ..Default::default()
        },));

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_image_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiImageCard")),
            "UiImageCard container must be rendered"
        );
        let has_mode = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("UiImageMode")
                && n.text.as_deref().is_some_and(|t| t.contains("NineSlice"))
        });
        assert!(has_mode, "UiImageMode label must display NineSlice");
    }

    #[test]
    fn test_text_alignment_and_slice_mode_mappings() {
        assert_eq!(alignment_to_index(UiTextAlignment::Left), 0);
        assert_eq!(alignment_to_index(UiTextAlignment::Center), 1);
        assert_eq!(alignment_to_index(UiTextAlignment::Right), 2);

        assert_eq!(index_to_alignment(0), UiTextAlignment::Left);
        assert_eq!(index_to_alignment(1), UiTextAlignment::Center);
        assert_eq!(index_to_alignment(2), UiTextAlignment::Right);
        assert_eq!(index_to_alignment(99), UiTextAlignment::Right);

        assert_eq!(
            slice_mode_text(ae_plugin_api::UiSliceMode::Stretch),
            "Slice Mode: Stretch"
        );
        assert_eq!(
            slice_mode_text(ae_plugin_api::UiSliceMode::Fit),
            "Slice Mode: Fit"
        );
        assert_eq!(
            slice_mode_text(ae_plugin_api::UiSliceMode::NineSlice),
            "Slice Mode: NineSlice"
        );
        assert_eq!(
            slice_mode_text(ae_plugin_api::UiSliceMode::Tile),
            "Slice Mode: Tile"
        );

        assert_eq!(progress_bar_fraction(0.0, 100.0, 50.0), 0.5);
        assert_eq!(progress_bar_fraction(0.0, 100.0, -10.0), 0.0);
        assert_eq!(progress_bar_fraction(0.0, 100.0, 150.0), 1.0);
        assert_eq!(progress_bar_fraction(100.0, 50.0, 75.0), 0.0);
    }
}