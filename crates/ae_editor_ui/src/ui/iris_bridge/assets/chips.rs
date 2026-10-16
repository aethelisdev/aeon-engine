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
                row.add(
                    Chip::new(label)
                        .count(count_category_items(params, cat))
                        .selected(active_category(row, params.active_category) == cat)
                        .tag(encode_chip_tag(idx as u8))
                        .accent_color(super::cards::resolve_category_color(cat)),
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
    fn test_category_chips_renders_in_tree() {
        use std::collections::HashMap;
        use std::path::Path;

        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let mut actions = Vec::new();
        let focus_state = FocusState::new();
        let current_folder = Path::new("assets");
        let thumbnail_layers = HashMap::new();

        let params = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 400.0),
            screen_size: (1280.0, 720.0),
            current_folder,
            search_query: "",
            active_category: AssetCategory::All,
            view_mode: crate::assets::types::AssetViewMode::Grid,
            selected_asset: None,
            cached_items: &[],
            filtered_items: &[],
            subfolders: &[],
            is_2d_mode: false,
            show_engine_content: false,
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            scroll_y: 0.0,
            tree_scroll_y: 0.0,
            cursor_pos: Point::new(10.0, 10.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            thumbnail_layers: &thumbnail_layers,
            focus_state: &focus_state,
            cursor_byte_idx: 0,
        };

        build_category_chips(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 800.0, 32.0),
            &params,
            &mut actions,
        );

        let row_id = tree
            .find_node_by_name("AssetsCategoryChipsRow")
            .expect("AssetsCategoryChipsRow must be emitted");
        let row_node = tree.get(row_id).expect("Row node must exist");
        assert_eq!(
            row_node.children.len(),
            CATEGORIES.len(),
            "Must emit exactly one Chip container per category"
        );
    }
}