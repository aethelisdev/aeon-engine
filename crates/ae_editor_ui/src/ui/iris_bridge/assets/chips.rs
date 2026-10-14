// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Category Filter Chips Bar for Iris UI Asset Browser.
//!
//! Renders categorized asset filter badges with dynamic count indicators
//! and synchronized selection state using pure declarative method chaining.
//!

use super::types::{AssetsPanelAction, AssetsPanelParams, encode_chip_tag};
use crate::assets::types::{AssetCategory, AssetSource};
use irisui::prelude::*;

/// Canonical category order and display labels for the Asset Browser filter chips bar.
pub const CATEGORIES: [(AssetCategory, &str); 7] = [
    (AssetCategory::All, "All Assets"),
    (AssetCategory::Models3D, "3D Meshes"),
    (AssetCategory::Textures2D, "Textures"),
    (AssetCategory::Shaders, "Shaders"),
    (AssetCategory::Scenes, "Scenes"),
    (AssetCategory::Materials, "Materials"),
    (AssetCategory::Audio, "Audio"),
];

/// Declaratively emits the category filter chips bar into the active [`UiScope`].
///
/// Features dynamic item counters, category accent borders, and 64-bit semantic tags
/// (`encode_chip_tag`) for zero-allocation $O(1)$ hit-testing.
/// Clicking a chip appends [`AssetsPanelAction::SelectCategory`] to the action stream.
pub fn build_category_chips_scope(
    scope: &mut UiScope<'_>,
    chips_rect: Rect,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
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
            for (idx, &(cat, label)) in CATEGORIES.iter().enumerate() {
                category_chip(
                    row,
                    label,
                    count_category_items(params, cat),
                    cat,
                    active_category(row, params.active_category) == cat,
                    encode_chip_tag(idx as u8),
                )
                .on_click(|| {
                    actions.push(AssetsPanelAction::SelectCategory(cat));
                });
            }
        },
    );
}

/// Resolves the currently active or clicked category for instant visual feedback.
#[inline]
fn active_category(scope: &UiScope<'_>, default_cat: AssetCategory) -> AssetCategory {
    scope
        .tree()
        .event_state
        .frame_clicked_tag
        .and_then(|clicked_tag| {
            CATEGORIES
                .iter()
                .enumerate()
                .find(|(idx, _)| encode_chip_tag(*idx as u8) == clicked_tag)
                .map(|(_, (cat, _))| *cat)
        })
        .unwrap_or(default_cat)
}

/// Returns the number of cached items matching the specified category and engine visibility filters.
fn count_category_items(params: &AssetsPanelParams<'_>, cat: AssetCategory) -> usize {
    params
        .cached_items
        .iter()
        .filter(|item| {
            (params.show_engine_content || item.source != AssetSource::Engine)
                && (!params.is_2d_mode || !item.is_3d)
                && (params.is_2d_mode || item.is_3d || item.category != AssetCategory::Scenes)
                && (cat == AssetCategory::All || item.category == cat)
        })
        .count()
}

/// Constructs the declarative flexbox style for a category filter chip.
#[inline]
fn chip_style(is_selected: bool, color: Color, width: f32) -> Style {
    Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(width)
        .height(22.0)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .border_radius(4.0)
        .border(
            1.0,
            if is_selected {
                color
            } else {
                Color::rgba(0.16, 0.18, 0.24, 0.40)
            },
        )
        .background(if is_selected {
            Color::rgba(0.12, 0.16, 0.22, 0.95)
        } else {
            Color::rgba(0.08, 0.09, 0.11, 0.60)
        })
        .hover_background(Color::rgba(0.10, 0.12, 0.16, 0.80))
        .hover_border(1.0, Color::rgba(0.28, 0.32, 0.42, 0.70))
}

/// Computes the fixed layout width for a category filter chip based on label and item count.
#[inline]
fn chip_width(label: &str, count: usize) -> f32 {
    (((label.len() + 3 + count_digits(count)) as f32) * 6.8 + 16.0).max(54.0)
}

/// Computes the decimal digit count of an item counter using hardware-accelerated integer logarithm.
#[inline]
const fn count_digits(count: usize) -> usize {
    match count.checked_ilog10() {
        Some(exp) => exp as usize + 1,
        None => 1,
    }
}

/// Emits an individual category filter chip with live item counter and accent styling.
fn category_chip(
    scope: &mut UiScope<'_>,
    label: &str,
    count: usize,
    cat: AssetCategory,
    is_selected: bool,
    tag: u64,
) -> WidgetResponse {
    scope.container_tagged_resp(
        "CategoryChip",
        chip_style(
            is_selected,
            super::cards::resolve_category_color(cat),
            chip_width(label, count),
        ),
        WidgetRole::Button,
        tag,
        |chip| {
            chip.label_styled_hover_passive(
                "CategoryChipText",
                format!("{label} ({count})"),
                11.0,
                (
                    if is_selected {
                        Color::WHITE
                    } else {
                        Color::rgba(0.65, 0.69, 0.78, 1.0)
                    },
                    if is_selected {
                        Color::WHITE
                    } else {
                        Color::rgba(0.90, 0.93, 0.98, 1.0)
                    },
                ),
                TextAlign::Center,
                Style::new().width(chip_width(label, count) - 16.0),
            );
        },
    )
}

/// Builds the category filter chips row with live item counters into the Iris tree.
pub fn build_category_chips(
    tree: &mut UiTree,
    parent_id: WidgetId,
    chips_rect: Rect,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    build_category_chips_scope(
        &mut UiScope::new(tree, parent_id),
        chips_rect,
        params,
        actions,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::AssetCategory;

    #[test]
    fn test_category_chips_order_matches_canonical_all() {
        for (idx, &(cat, _)) in CATEGORIES.iter().enumerate() {
            assert_eq!(
                AssetCategory::ALL[idx],
                cat,
                "AssetCategory::ALL at index {idx} must match UI chips order exactly",
            );
        }
    }

    #[test]
    fn test_count_digits_invariants() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(9), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(99), 2);
        assert_eq!(count_digits(100), 3);
        assert_eq!(count_digits(999), 3);
        assert_eq!(count_digits(1000), 4);
        assert_eq!(count_digits(9999), 4);
        assert_eq!(count_digits(10000), 5);
        assert_eq!(count_digits(99999), 5);
        assert_eq!(count_digits(100_000), 6);
        assert_eq!(count_digits(1_000_000), 7);
    }

    #[test]
    fn test_chip_width_matches_text_formatting() {
        for label in ["All Assets", "3D Meshes", "Textures", "Shaders"] {
            for count in [0, 5, 42, 100, 1500] {
                assert_eq!(
                    chip_width(label, count),
                    (format!("{label} ({count})").len() as f32 * 6.8 + 16.0).max(54.0),
                    "chip_width must match format! string length calculation"
                );
            }
        }
    }
}