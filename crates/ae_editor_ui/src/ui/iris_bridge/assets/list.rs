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
    AssetsPanelAction, AssetsPanelParams, encode_item_tag,
};
use irisui::prelude::*;

/// Height of the table column header row in logical pixels.
pub const LIST_HEADER_HEIGHT: f32 = 26.0;

const ASSET_TABLE_COLUMNS: &[TableColumnSpec] = &[
    TableColumnSpec::proportional("ColNameHeader", "Asset Name", 0.40, 140.0, TextAlign::Left),
    TableColumnSpec::fixed("ColCatHeader", "Type", 64.0, TextAlign::Center),
    TableColumnSpec::fixed("ColSizeHeader", "Metadata", 80.0, TextAlign::Right),
    TableColumnSpec::fixed("ColStatusHeader", "Status", 72.0, TextAlign::Center),
    TableColumnSpec::remainder("ColActionsHeader", "Actions", 80.0, TextAlign::Left),
];

/// Builds the detailed tabular list view inside `vp_rect` via [`UiScope::data_table`].
pub fn build_asset_list_table_scope(
    scope: &mut UiScope<'_>,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    scope
        .data_table(
            params.filtered_items.len(),
            DataTableConfig::new(
                "AssetsContentViewport",
                ASSETS_TAG_CONTENT_VIEWPORT,
                vp_rect,
                ASSET_TABLE_COLUMNS,
            )
            .with_section_names("ListTableHeader", "AssetListRowsContainer")
            .with_row_metrics(LIST_HEADER_HEIGHT, LIST_ROW_HEIGHT + 2.0, 2.0)
            .with_scrollbar_tags(ASSETS_TAG_SCROLLBAR_TRACK, ASSETS_TAG_SCROLLBAR_THUMB),
        )
        .empty_state(|vp| {
            asset_empty_notice(vp, vp_rect, params.search_query);
        })
        .render_row(|rows_scope, idx| {
            asset_list_row(
                rows_scope,
                idx as u32,
                &params.filtered_items[idx],
                rows_scope.is_tag_clicked(encode_item_tag(idx as u32))
                    || rows_scope.is_tag_pressed(encode_item_tag(idx as u32))
                    || params.selected_asset == Some(&params.filtered_items[idx].path),
                actions,
            )
            .context_menu(rows_scope, |menu| {
                super::context_menu::render_asset_context_menu_items(
                    menu,
                    &params.filtered_items[idx],
                    actions,
                );
            });
        })
        .context_menu(scope, |menu| {
            super::context_menu::render_folder_context_menu_items(
                menu,
                params.current_folder,
                actions,
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_list_table_declarative_build() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let current_folder = PathBuf::from("assets");
        let subfolders = Vec::new();

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

        let params = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder: &current_folder,
            search_query: "",
            active_category: AssetCategory::All,
            view_mode: AssetViewMode::List,
            selected_asset: None,
            cached_items: &items,
            filtered_items: &items,
            is_2d_mode: false,
            show_engine_content: false,
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &subfolders,
            thumbnail_layers: &HashMap::new(),
            cursor_byte_idx: 0,
        };

        let mut actions = Vec::new();
        build_asset_list_table_scope(
            &mut UiScope::new(&mut tree, root_id),
            Rect::new(0.0, 0.0, 600.0, 400.0),
            &params,
            &mut actions,
        );

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