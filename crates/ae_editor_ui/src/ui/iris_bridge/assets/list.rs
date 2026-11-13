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

use super::components::{ASSET_LIST_ROW_HEIGHT, asset_list_row};
use super::types::{ASSETS_TAG_CONTENT_VIEWPORT, AssetsPanelAction, AssetsPanelParams};
use irisui::prelude::*;

/// Height of an individual table row in pixels.
pub const LIST_ROW_HEIGHT: f32 = ASSET_LIST_ROW_HEIGHT;

/// Height of the table header row in pixels.
pub const LIST_HEADER_HEIGHT: f32 = 26.0;

/// Constructs the multi-column table rows directly via a declarative [`UiScope`].
pub fn build_asset_list_table_scope(
    scope: &mut UiScope<'_>,
    vp_rect: Rect,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    let total_w = vp_rect.width - 16.0;
    let col_name_w = (total_w * 0.40).max(140.0);
    let col_cat_w = 64.0;
    let col_size_w = 80.0;
    let col_status_w = 72.0;
    let col_act_w = (total_w - col_name_w - col_cat_w - col_size_w - col_status_w).max(80.0);

    // 1. Table Header Row
    let th_bg = scope.theme().card_header_bg;
    let th_border = scope.theme().divider;
    scope.container_named(
        "ListTableHeader",
        Style::new()
            .flex_row()
            .background(th_bg)
            .border_radius(4.0)
            .border(1.0, th_border)
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

    // 2. Table Data Rows via declarative VirtualList
    let available_table_h = (vp_rect.height - (LIST_HEADER_HEIGHT + 8.0)).max(10.0);
    let scroll_y = scope.scroll_offset(ASSETS_TAG_CONTENT_VIEWPORT);

    scope
        .virtual_list(params.filtered_items.len(), LIST_ROW_HEIGHT + 2.0)
        .name("AssetListRowsContainer")
        .tag(ASSETS_TAG_CONTENT_VIEWPORT)
        .width(total_w)
        .margin_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .gap(2.0)
        .visible_height(available_table_h)
        .scroll_y(scroll_y)
        .render_item(|rows_scope, idx| {
            let item = &params.filtered_items[idx];
            let item_idx = idx as u32;
            let item_tag = super::types::encode_item_tag(item_idx);
            let is_clicked_this_frame = rows_scope.is_tag_clicked(item_tag);
            let is_pressed_this_frame = rows_scope.is_tag_pressed(item_tag);
            let is_selected = is_clicked_this_frame
                || is_pressed_this_frame
                || params.selected_asset == Some(&item.path);

            asset_list_row(rows_scope, item_idx, item, is_selected, actions).context_menu(
                rows_scope,
                |menu| {
                    super::context_menu::render_asset_context_menu_items(menu, item, actions);
                },
            );
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
    use irisui::widgets::controls::FocusState;
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
            focus_state: &FocusState::new(),
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