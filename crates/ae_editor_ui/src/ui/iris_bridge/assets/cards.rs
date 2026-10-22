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
use super::types::{ASSETS_TAG_CONTENT_VIEWPORT, AssetsPanelAction, AssetsPanelParams};
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
    actions: &mut Vec<AssetsPanelAction>,
) {
    build_asset_grid_cards_scope(&mut UiScope::new(tree, parent_id), vp_rect, params, actions);
}

/// Constructs the responsive grid cards directly via a declarative [`UiScope`].
///
/// Uses 100% declarative UI scope widgets with zero imperative node allocations.
/// Handles autonomous scrolling via [`WidgetRole::ScrollAreaViewport`], asset selection
/// via single click, and instantiation into the active scene via double-click.
pub fn build_asset_grid_cards_scope(
    scope: &mut UiScope<'_>,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    if params.filtered_items.is_empty() {
        asset_empty_notice(scope, vp_rect, params.search_query);
        return;
    }

    let scroll_y = scope.tree().scroll_offset(ASSETS_TAG_CONTENT_VIEWPORT);

    scope
        .container_tagged_resp(
            "AssetGridContainer",
            Style::new()
                .flex_col()
                .width(vp_rect.width)
                .height(vp_rect.height)
                .scroll_offset_y(scroll_y)
                .clip_children(true),
            WidgetRole::ScrollAreaViewport,
            ASSETS_TAG_CONTENT_VIEWPORT,
            |grid_scope| {
                for (item_idx, card_rect) in asset_responsive_grid().compute_visible_cells(
                    vp_rect,
                    params.filtered_items.len(),
                    scroll_y,
                ) {
                    let card_pos =
                        Point::new(card_rect.x - vp_rect.x, card_rect.y - vp_rect.y + scroll_y);
                    render_grid_card(
                        grid_scope,
                        item_idx as u32,
                        &params.filtered_items[item_idx],
                        card_pos,
                        params,
                        actions,
                    );
                }
            },
        )
        .content_size(
            scope,
            Size::new(
                vp_rect.width,
                asset_responsive_grid()
                    .compute_content_height(params.filtered_items.len(), vp_rect.width),
            ),
        )
        .context_menu(scope, |menu| {
            super::context_menu::render_folder_context_menu_items(
                menu,
                params.current_folder,
                actions,
            );
        });
}

/// Constructs the canonical responsive layout grid for asset browser cards.
#[inline]
fn asset_responsive_grid() -> ResponsiveGrid {
    ResponsiveGrid::new(CARD_WIDTH, CARD_HEIGHT)
        .spacing(CARD_SPACING, CARD_SPACING)
        .padding(10.0, 10.0)
}

/// Renders an individual asset grid card with its context menu into the active scope.
fn render_grid_card(
    scope: &mut UiScope<'_>,
    item_idx: u32,
    item: &crate::assets::types::AssetItem,
    card_pos: Point,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    asset_grid_card(
        scope,
        item_idx,
        item,
        params.thumbnail_layers.get(&item.path).copied(),
        params.selected_asset == Some(&item.path),
        card_pos,
        actions,
    )
    .context_menu(scope, |menu| {
        super::context_menu::render_asset_context_menu_items(menu, item, actions);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
    use irisui::widgets::controls::{FocusState, FocusTarget};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn test_assets_params<'a>(
        items: &'a [AssetItem],
        layers: &'a HashMap<PathBuf, u32>,
        focus: &'a FocusState<FocusTarget>,
    ) -> AssetsPanelParams<'a> {
        AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder: std::path::Path::new("assets"),
            search_query: "nonexistent",
            active_category: AssetCategory::All,
            view_mode: AssetViewMode::Grid,
            selected_asset: None,
            cached_items: items,
            filtered_items: items,
            is_2d_mode: false,
            show_engine_content: false,
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &[],
            thumbnail_layers: layers,
            focus_state: focus,
            cursor_byte_idx: 0,
        }
    }

    fn test_asset_item(name: &str, file_size_bytes: u64, is_loaded: bool) -> AssetItem {
        AssetItem {
            name: name.to_string(),
            path: PathBuf::from(format!("assets/models/{name}")),
            relative_path: format!("models/{name}"),
            category: AssetCategory::Models3D,
            source: AssetSource::Project,
            file_size_bytes,
            metadata_badge: format!("{} KB", file_size_bytes / 1024),
            is_loaded_in_memory: is_loaded,
            model_handle: None,
            texture_handle: None,
            shader_handle: None,
            is_3d: true,
        }
    }

    #[test]
    fn test_cards_grid_and_empty_notice_declarative_build() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let layers = HashMap::new();
        let focus = FocusState::new();
        let mut actions = Vec::new();

        // 1. Empty items case -> triggers asset_empty_notice
        build_asset_grid_cards(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 600.0, 400.0),
            &test_assets_params(&[], &layers, &focus),
            &mut actions,
        );

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("EmptyAssetsNotice")),
            "Empty notice container should be emitted when items are empty"
        );

        // 2. Populated items case -> triggers asset_grid_card placed side-by-side
        let items = [
            test_asset_item("hero.gltf", 4096, true),
            test_asset_item("sword.gltf", 2048, false),
        ];
        build_asset_grid_cards(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 600.0, 400.0),
            &test_assets_params(&items, &layers, &focus),
            &mut actions,
        );

        let grid_cards: Vec<_> = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("AssetGridCard"))
            .collect();
        assert_eq!(grid_cards.len(), 2, "Both cards must be emitted");
        assert!(
            grid_cards[1].1.style.inset_left > grid_cards[0].1.style.inset_left,
            "Second grid card must be placed to the right of first card (side-by-side)"
        );
    }
}