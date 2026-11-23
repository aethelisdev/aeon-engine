// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Detailed Tabular List View for Iris UI Asset Browser.
//!
//! Renders a multi-column virtualized table of assets via [`UiScope::data_table`]:
//! - Columns: Name, Category, Size/Metadata, Status (RAM/Disk), and Quick Actions (`+ Spawn`, `Inspect`).

use super::components::{
    ASSET_LIST_ROW_HEIGHT as LIST_ROW_HEIGHT, asset_empty_notice, asset_list_row,
};
use super::types::{
    ASSETS_TAG_CONTENT_VIEWPORT, ASSETS_TAG_SCROLLBAR_THUMB, ASSETS_TAG_SCROLLBAR_TRACK,
    AssetsPanelAction, encode_item_tag,
};
use ae_assets::types::AssetItem;
use irisui::prelude::*;
use std::path::Path;

/// Height of the table column header row in logical pixels.
pub const LIST_HEADER_HEIGHT: f32 = 26.0;

const ASSET_TABLE_COLUMNS: &[TableColumnSpec] = &[
    TableColumnSpec::proportional("ColNameHeader", "Asset Name", 0.40, 140.0, TextAlign::Left),
    TableColumnSpec::fixed("ColCatHeader", "Type", 64.0, TextAlign::Center),
    TableColumnSpec::fixed("ColSizeHeader", "Metadata", 80.0, TextAlign::Right),
    TableColumnSpec::fixed("ColStatusHeader", "Status", 72.0, TextAlign::Center),
    TableColumnSpec::remainder("ColActionsHeader", "Actions", 80.0, TextAlign::Left),
];

/// Builds the detailed tabular list view automatically sized to its parent via [`UiScope::data_table`].
pub fn build_asset_list_table_scope(
    scope: &mut UiScope<'_>,
    filtered_items: &[AssetItem],
    selected_asset: Option<&Path>,
    current_folder: &Path,
    search_query: &str,
    actions: &mut Vec<AssetsPanelAction>,
) {
    scope
        .data_table(
            filtered_items.len(),
            DataTableConfig::auto(
                "AssetsContentViewport",
                ASSETS_TAG_CONTENT_VIEWPORT,
                ASSET_TABLE_COLUMNS,
            )
            .with_section_names("ListTableHeader", "AssetListRowsContainer")
            .with_row_metrics(LIST_HEADER_HEIGHT, LIST_ROW_HEIGHT + 2.0, 2.0)
            .with_scrollbar_tags(ASSETS_TAG_SCROLLBAR_TRACK, ASSETS_TAG_SCROLLBAR_THUMB),
        )
        .empty_state(|vp| {
            asset_empty_notice(vp, search_query);
        })
        .render_row(|rows_scope, idx| {
            asset_list_row(
                rows_scope,
                idx as u32,
                &filtered_items[idx],
                rows_scope.is_tag_clicked(encode_item_tag(idx as u32))
                    || rows_scope.is_tag_pressed(encode_item_tag(idx as u32))
                    || selected_asset == Some(&filtered_items[idx].path),
                actions,
            )
            .context_menu(rows_scope, |menu| {
                super::context_menu::render_asset_context_menu_items(
                    menu,
                    &filtered_items[idx],
                    actions,
                );
            });
        })
        .context_menu(scope, |menu| {
            super::context_menu::render_folder_context_menu_items(menu, current_folder, actions);
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_assets::types::{AssetCategory, AssetItem, AssetSource};
    use std::path::PathBuf;

    #[test]
    fn test_list_table_declarative_build() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let current_folder = PathBuf::from("assets");

        let item = AssetItem {
            name: "ambient.wav".to_string(),
            path: PathBuf::from("assets/audio/ambient.wav"),
            relative_path: "audio/ambient.wav".to_string(),
            category: AssetCategory::Audio,
            source: AssetSource::Project,
            file_size_bytes: 8192,
            metadata_badge: "8.0 KB".to_string(),
            is_loaded_in_memory: false,
            model_handle: None,
            texture_handle: None,
            shader_handle: None,
            is_3d: false,
        };
        let items = vec![item];

        let mut actions = Vec::new();
        {
            let mut scope = UiScope::new(&mut tree, root_id);
            scope
                .panel("Host")
                .bounds(Rect::new(0.0, 0.0, 600.0, 400.0))
                .show(|s| {
                    build_asset_list_table_scope(
                        s,
                        &items,
                        None,
                        &current_folder,
                        "",
                        &mut actions,
                    );
                });
        }

        let found_header = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ListTableHeader"));
        let found_row = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("AssetListRow"));
        assert!(found_header, "ListTableHeader should be emitted");
        assert!(found_row, "AssetListRow should be emitted");
    }
}