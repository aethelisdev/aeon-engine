// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Interactive Card Grid View.
//!
//! Renders responsive wrapping grid cards with category badges, memory status indicators,
//! canonical vector icons, truncated names, and file size metadata via 100% declarative
//! [`UiScope`] widgets.
//!
//! Zero per-frame heap allocations: card items and paths are referenced directly without
//! copying metadata into parallel target collections.
//!

use super::components::{
    ASSET_CARD_GAP, ASSET_CARD_HEIGHT, ASSET_CARD_WIDTH, asset_empty_notice, asset_grid_card,
};
use super::types::AssetsPanelParams;
use irisui::prelude::*;

/// Standard width of an asset grid card in logical pixels.
pub const CARD_WIDTH: f32 = ASSET_CARD_WIDTH;

/// Standard height of an asset grid card in logical pixels.
pub const CARD_HEIGHT: f32 = ASSET_CARD_HEIGHT;

/// Horizontal and vertical spacing between adjacent grid cards.
pub const CARD_SPACING: f32 = ASSET_CARD_GAP;

/// Re-export canonical category color resolver for module consumers.
pub use super::components::resolve_category_color;
/// Re-export canonical category icon resolver for module consumers.
pub use super::components::resolve_category_icon;

/// Constructs the responsive grid cards into the Iris `UiTree`.
pub fn build_asset_grid_cards(
    tree: &mut UiTree,
    parent_id: WidgetId,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    let mut scope = UiScope::new(tree, parent_id);
    build_asset_grid_cards_scope(&mut scope, vp_rect, params);
}

/// Constructs the responsive grid cards directly via a declarative [`UiScope`].
///
/// Uses 100% declarative UI scope widgets with zero imperative node allocations.
pub fn build_asset_grid_cards_scope(
    scope: &mut UiScope<'_>,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    if params.filtered_items.is_empty() {
        asset_empty_notice(scope, vp_rect, params.search_query);
        return;
    }

    let items = params.filtered_items;
    let grid = ResponsiveGrid::new(CARD_WIDTH, CARD_HEIGHT)
        .spacing(CARD_SPACING, CARD_SPACING)
        .padding(10.0, 10.0);

    let visible_cells = grid.compute_visible_cells(vp_rect, items.len(), params.scroll_y);

    scope.container_named(
        "AssetGridContainer",
        Style::new()
            .flex_col()
            .width(vp_rect.width)
            .height(vp_rect.height)
            .clip_children(true),
        |grid_scope| {
            for (item_idx, card_rect) in visible_cells {
                let item = &items[item_idx];
                let layer = params.thumbnail_layers.get(&item.path).copied();
                let is_selected = params.selected_asset == Some(&item.path);
                let rel_pos = Point::new(card_rect.x - vp_rect.x, card_rect.y - vp_rect.y);

                asset_grid_card(
                    grid_scope,
                    item_idx as u32,
                    item,
                    layer,
                    is_selected,
                    params.hovered_tag,
                    rel_pos,
                );
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_cards_grid_and_empty_notice_declarative_build() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let current_folder = PathBuf::from("assets");
        let subfolders = Vec::new();

        // 1. Empty items case -> triggers asset_empty_notice
        let empty_items: Vec<AssetItem> = Vec::new();
        let params_empty = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder: &current_folder,
            search_query: "nonexistent",
            active_category: AssetCategory::All,
            view_mode: AssetViewMode::Grid,
            selected_asset: None,
            cached_items: &empty_items,
            filtered_items: &empty_items,
            is_2d_mode: false,
            show_engine_content: false,
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            scroll_y: 0.0,
            tree_scroll_y: 0.0,
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &subfolders,
            hovered_tag: None,
            thumbnail_layers: &HashMap::new(),
        };

        build_asset_grid_cards(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 600.0, 400.0),
            &params_empty,
        );

        let found_empty_notice = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("EmptyAssetsNotice"));
        assert!(
            found_empty_notice,
            "Empty notice container should be emitted when items are empty"
        );

        // 2. Populated items case -> triggers asset_grid_card placed side-by-side
        let item1 = AssetItem {
            name: "hero.gltf".to_string(),
            path: PathBuf::from("assets/models/hero.gltf"),
            relative_path: "models/hero.gltf".to_string(),
            category: AssetCategory::Models3D,
            source: AssetSource::Project,
            file_size_bytes: 4096,
            metadata_badge: "4.0 KB".to_string(),
            is_loaded_in_memory: true,
            model_handle: None,
            texture_handle: None,
            shader_handle: None,
            is_3d: true,
        };
        let item2 = AssetItem {
            name: "sword.gltf".to_string(),
            path: PathBuf::from("assets/models/sword.gltf"),
            relative_path: "models/sword.gltf".to_string(),
            category: AssetCategory::Models3D,
            source: AssetSource::Project,
            file_size_bytes: 2048,
            metadata_badge: "2.0 KB".to_string(),
            is_loaded_in_memory: false,
            model_handle: None,
            texture_handle: None,
            shader_handle: None,
            is_3d: true,
        };
        let items = vec![item1, item2];
        let params_populated = AssetsPanelParams {
            filtered_items: &items,
            cached_items: &items,
            ..params_empty
        };

        build_asset_grid_cards(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 600.0, 400.0),
            &params_populated,
        );

        let grid_cards: Vec<_> = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("AssetGridCard"))
            .collect();
        assert_eq!(grid_cards.len(), 2, "Both cards must be emitted");

        // Verify side-by-side placement (card 1 is to the right of card 0)
        let left0 = grid_cards[0].1.style.inset_left;
        let left1 = grid_cards[1].1.style.inset_left;
        assert!(
            left1 > left0,
            "Second grid card must be placed to the right of first card (side-by-side): left0={:?}, left1={:?}",
            left0,
            left1
        );
    }
}