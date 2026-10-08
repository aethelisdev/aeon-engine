// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Detailed Multi-Column Table List View.
//!
//! Renders structured table rows with category badges, canonical vector icons,
//! size metrics, memory residency tags, and direct action triggers via 100%
//! declarative [`UiScope`] widgets.
//!
//! Zero per-frame heap allocations: table rows are rendered directly without
//! copying metadata into parallel target collections.
//!

use super::components::{ASSET_LIST_ROW_HEIGHT, asset_list_row};
use super::types::AssetsPanelParams;
use irisui::prelude::*;

/// Height of an individual table row in pixels.
pub const LIST_ROW_HEIGHT: f32 = ASSET_LIST_ROW_HEIGHT;

/// Height of the table header row in pixels.
pub const LIST_HEADER_HEIGHT: f32 = 26.0;

/// Constructs the multi-column table rows into the Iris `UiTree`.
pub fn build_asset_list_table(
    tree: &mut UiTree,
    parent_id: WidgetId,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    let mut scope = UiScope::new(tree, parent_id);
    build_asset_list_table_scope(&mut scope, vp_rect, params);
}

/// Constructs the multi-column table rows directly via a declarative [`UiScope`].
pub fn build_asset_list_table_scope(
    scope: &mut UiScope<'_>,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
) {
    let total_w = vp_rect.width - 16.0;
    let col_name_w = (total_w * 0.40).max(140.0);
    let col_cat_w = 64.0;
    let col_size_w = 80.0;
    let col_status_w = 72.0;
    let col_act_w = (total_w - col_name_w - col_cat_w - col_size_w - col_status_w).max(80.0);

    // 1. Table Header Row
    scope.container_named(
        "ListTableHeader",
        Style::new()
            .flex_row()
            .background(Color::rgba(0.09, 0.10, 0.14, 0.95))
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60))
            .height(LIST_HEADER_HEIGHT)
            .width(total_w)
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .margin_insets(Insets::new(4.0, 8.0, 4.0, 8.0)),
        |hdr| {
            hdr.label_styled_passive(
                "ColNameHeader",
                "Name",
                11.0,
                Color::rgba(0.60, 0.65, 0.75, 1.0),
                TextAlign::Left,
                Style::new().width(col_name_w - 8.0),
            );
            hdr.label_styled_passive(
                "ColCatHeader",
                "Category",
                11.0,
                Color::rgba(0.60, 0.65, 0.75, 1.0),
                TextAlign::Left,
                Style::new().width(col_cat_w),
            );
            hdr.label_styled_passive(
                "ColSizeHeader",
                "Size / Metric",
                11.0,
                Color::rgba(0.60, 0.65, 0.75, 1.0),
                TextAlign::Left,
                Style::new().width(col_size_w),
            );
            hdr.label_styled_passive(
                "ColStatusHeader",
                "Status",
                11.0,
                Color::rgba(0.60, 0.65, 0.75, 1.0),
                TextAlign::Left,
                Style::new().width(col_status_w),
            );
            hdr.label_styled_passive(
                "ColActionsHeader",
                "Actions",
                11.0,
                Color::rgba(0.60, 0.65, 0.75, 1.0),
                TextAlign::Left,
                Style::new().width(col_act_w),
            );
        },
    );

    // 2. Table Data Rows via iris-widgets VirtualList
    let available_table_h = (vp_rect.height - (LIST_HEADER_HEIGHT + 8.0)).max(10.0);
    let vlist = VirtualList::new(params.filtered_items.len(), LIST_ROW_HEIGHT + 2.0);
    let slice = vlist.compute_slice(available_table_h, params.scroll_y);

    scope.container_named(
        "AssetListRowsContainer",
        Style::new()
            .flex_col()
            .width(total_w)
            .clip_children(true)
            .margin_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .gap(2.0)
            .scroll_offset_y(-(params.scroll_y % (LIST_ROW_HEIGHT + 2.0))),
        |rows_scope| {
            for (offset, item) in params.filtered_items[slice.start_idx..slice.end_idx]
                .iter()
                .enumerate()
            {
                let row_idx = slice.start_idx + offset;
                let is_selected = params.selected_asset == Some(&item.path);

                // Zero heap allocation: no item clones, no target list pushes
                asset_list_row(rows_scope, row_idx as u32, item, is_selected);
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
            scroll_y: 0.0,
            tree_scroll_y: 0.0,
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            subfolders: &subfolders,
            thumbnail_layers: &HashMap::new(),
        };

        build_asset_list_table(
            &mut tree,
            root_id,
            Rect::new(0.0, 0.0, 600.0, 400.0),
            &params,
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