// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Category Filter Chips Bar for Iris UI Asset Browser.
//!
//! Renders categorized asset filter badges with dynamic count indicators
//! and synchronized selection state.
//!

use super::types::{AssetsPanelParams, encode_chip_tag};
use crate::assets::types::{AssetCategory, AssetSource};
use irisui::prelude::*;

/// Declaratively emits the category filter chips bar into the active [`UiScope`].
///
/// Features dynamic item counters, category accent borders, and 64-bit semantic tags
/// (`encode_chip_tag`) for zero-allocation $O(1)$ hit-testing.
pub fn build_category_chips_scope(
    scope: &mut UiScope<'_>,
    chips_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    let categories = [
        (AssetCategory::All, "All Assets"),
        (AssetCategory::Models3D, "3D Meshes"),
        (AssetCategory::Textures2D, "Textures"),
        (AssetCategory::Shaders, "Shaders"),
        (AssetCategory::Scenes, "Scenes"),
        (AssetCategory::Materials, "Materials"),
        (AssetCategory::Audio, "Audio"),
    ];

    scope.container_named(
        "AssetsCategoryChipsRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::FlexStart)
            .width(chips_rect.width)
            .height(chips_rect.height)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .gap(6.0),
        |row| {
            for (idx, (cat, label)) in categories.into_iter().enumerate() {
                let count = params
                    .cached_items
                    .iter()
                    .filter(|i| {
                        if !params.show_engine_content && i.source == AssetSource::Engine {
                            return false;
                        }
                        if params.is_2d_mode && i.is_3d {
                            return false;
                        }
                        if !params.is_2d_mode && !i.is_3d && i.category == AssetCategory::Scenes {
                            return false;
                        }
                        cat == AssetCategory::All || i.category == cat
                    })
                    .count();

                let chip_text = format!("{} ({})", label, count);
                let tag = encode_chip_tag(idx as u8);
                let is_selected = params.active_category == cat;
                let cat_color = super::cards::resolve_category_color(cat);
                let border_color = if is_selected {
                    cat_color
                } else {
                    Color::rgba(0.16, 0.18, 0.24, 0.40)
                };

                let bg_color = if is_selected {
                    Color::rgba(0.12, 0.16, 0.22, 0.95)
                } else {
                    Color::rgba(0.08, 0.09, 0.11, 0.60)
                };

                let text_color = if is_selected {
                    Color::WHITE
                } else {
                    Color::rgba(0.65, 0.69, 0.78, 1.0)
                };

                let chip_w = (chip_text.len() as f32 * 6.8 + 16.0).max(54.0);

                let mut chip_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(chip_w)
                    .height(22.0)
                    .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
                    .border_radius(4.0)
                    .border(1.0, border_color)
                    .background(bg_color);

                if !is_selected {
                    chip_style = chip_style
                        .hover_background(Color::rgba(0.10, 0.12, 0.16, 0.80))
                        .hover_border(1.0, Color::rgba(0.28, 0.32, 0.42, 0.70));
                }

                row.container_tagged(
                    "CategoryChip",
                    chip_style,
                    WidgetRole::Button,
                    tag,
                    |chip| {
                        let txt_id = chip.label_styled_passive(
                            "CategoryChipText",
                            &chip_text,
                            11.0,
                            text_color,
                            TextAlign::Center,
                            Style::new().width(chip_w - 16.0),
                        );
                        if !is_selected {
                            chip.set_hover_text_color(txt_id, Color::rgba(0.90, 0.93, 0.98, 1.0));
                        }
                    },
                );
            }
        },
    );
}

/// Builds the category filter chips row with live item counters into the Iris tree.
pub fn build_category_chips(
    tree: &mut UiTree,
    parent_id: WidgetId,
    chips_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    let mut scope = UiScope::new(tree, parent_id);
    build_category_chips_scope(&mut scope, chips_rect, params);
}

#[cfg(test)]
mod tests {
    use crate::assets::types::AssetCategory;

    #[test]
    fn test_category_chips_order_matches_canonical_all() {
        let expected_order = [
            AssetCategory::All,
            AssetCategory::Models3D,
            AssetCategory::Textures2D,
            AssetCategory::Shaders,
            AssetCategory::Scenes,
            AssetCategory::Materials,
            AssetCategory::Audio,
        ];

        for (idx, &expected_cat) in expected_order.iter().enumerate() {
            assert_eq!(
                AssetCategory::ALL[idx],
                expected_cat,
                "AssetCategory::ALL at index {} must match UI chips order exactly",
                idx
            );
        }
    }
}