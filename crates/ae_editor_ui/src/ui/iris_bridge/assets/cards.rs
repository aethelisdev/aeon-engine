// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Responsive Asset Grid Cards Renderer.
//!
//! Renders responsive grid cards via [`UiScope::virtual_grid`] with category badges,
//! VRAM status dots, texture thumbnails or vector atlas icons, and truncated titles.

pub use super::components::{
    ASSET_CARD_GAP as CARD_GAP, ASSET_CARD_HEIGHT as CARD_HEIGHT, ASSET_CARD_WIDTH as CARD_WIDTH,
    asset_empty_notice, asset_grid_card, resolve_category_color, resolve_category_icon,
};

use super::types::{
    ASSETS_TAG_CONTENT_VIEWPORT, ASSETS_TAG_SCROLLBAR_THUMB, ASSETS_TAG_SCROLLBAR_TRACK,
    AssetsPanelAction, encode_item_tag,
};
use crate::assets::types::AssetItem;
use irisui::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Builds the responsive asset grid automatically sized to its parent via [`UiScope::virtual_grid`].
pub fn build_asset_grid_cards_scope(
    scope: &mut UiScope<'_>,
    filtered_items: &[AssetItem],
    selected_asset: Option<&Path>,
    current_folder: &Path,
    search_query: &str,
    thumbnail_layers: &HashMap<PathBuf, u32>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    scope
        .virtual_grid(
            filtered_items.len(),
            VirtualGridConfig::auto("AssetGridContainer", ASSETS_TAG_CONTENT_VIEWPORT)
                .with_cell_size(CARD_WIDTH, CARD_HEIGHT, CARD_GAP)
                .with_scrollbar_tags(ASSETS_TAG_SCROLLBAR_TRACK, ASSETS_TAG_SCROLLBAR_THUMB),
        )
        .empty_state(|vp| {
            asset_empty_notice(vp, search_query);
        })
        .render_item(|grid_scope, item_idx, card_pos| {
            render_grid_card(
                grid_scope,
                item_idx as u32,
                &filtered_items[item_idx],
                card_pos,
                selected_asset,
                thumbnail_layers,
                actions,
            );
        })
        .context_menu(scope, |menu| {
            super::context_menu::render_folder_context_menu_items(menu, current_folder, actions);
        });
}

/// Renders an individual asset grid card using declarative [`asset_grid_card`].
fn render_grid_card(
    scope: &mut UiScope<'_>,
    item_idx: u32,
    item: &AssetItem,
    rel_pos: Point,
    selected_asset: Option<&Path>,
    thumbnail_layers: &HashMap<PathBuf, u32>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    asset_grid_card(
        scope,
        item_idx,
        item,
        thumbnail_layers.get(&item.path).copied(),
        scope.is_tag_clicked(encode_item_tag(item_idx))
            || scope.is_tag_pressed(encode_item_tag(item_idx))
            || selected_asset == Some(&item.path),
        rel_pos,
        actions,
    )
    .context_menu(scope, |menu| {
        super::context_menu::render_asset_context_menu_items(menu, item, actions);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::{AssetCategory, AssetItem, AssetSource};

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
        let mut actions = Vec::new();
        let folder = Path::new("assets");

        {
            let mut scope = UiScope::new(&mut tree, root_id);
            scope
                .panel("Host")
                .bounds(Rect::new(0.0, 0.0, 600.0, 400.0))
                .show(|s| {
                    build_asset_grid_cards_scope(s, &[], None, folder, "", &layers, &mut actions);
                });
        }

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("EmptyAssetsNotice")),
            "Empty notice container should be emitted when items are empty"
        );

        let items = [
            test_asset_item("hero.gltf", 4096, true),
            test_asset_item("sword.gltf", 2048, false),
        ];
        {
            let mut scope = UiScope::new(&mut tree, root_id);
            scope
                .panel("Host")
                .bounds(Rect::new(0.0, 0.0, 600.0, 400.0))
                .show(|s| {
                    build_asset_grid_cards_scope(
                        s,
                        &items,
                        None,
                        folder,
                        "",
                        &layers,
                        &mut actions,
                    );
                });
        }

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