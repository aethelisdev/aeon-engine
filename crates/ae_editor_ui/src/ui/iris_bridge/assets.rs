// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Native Iris UI Asset / Content Browser Subsystem
//!
//! Provides a 100% GPU SDF-accelerated Asset Browser panel,
//! completely free of emojis, featuring breadcrumb navigation, live search,
//! canonical vector icons, folder tree sidebar, floating right-click context menus,
//! interactive quick asset preview modal, and responsive grid/table views.
//!

pub mod card_builder;
pub mod cards;
pub mod chips;
pub mod components;
pub mod context_menu;
pub mod drag_overlay;
pub mod events;
pub mod footer;
pub mod list;
pub mod panel;
pub mod preview;
#[cfg(test)]
mod tests;
pub mod toolbar;
pub mod tree;
pub mod types;

pub use card_builder::{
    AssetCardBadge, AssetCardBuilder, AssetCardFrame, AssetCardPreview, AssetCardStyle,
};
pub use chips::build_category_chips;
pub use components::{
    ASSET_CARD_GAP, ASSET_CARD_HEIGHT, ASSET_CARD_WIDTH, ASSET_LIST_ROW_HEIGHT,
    FOLDER_TREE_ROW_HEIGHT, asset_breadcrumb_bar, asset_empty_notice, asset_folder_tree_item,
    asset_grid_card, asset_list_row, resolve_category_color, resolve_category_icon,
};
pub use drag_overlay::build_asset_drag_overlays;
pub use events::{
    AssetClickTracker, AssetsEventContext, handle_assets_click, handle_assets_panel_event,
    handle_assets_right_click, handle_assets_scroll,
};
pub use panel::build_assets_panel;
pub use types::{
    ASSETS_TAG_CLEAN_VRAM, ASSETS_TAG_DOMAIN, ASSETS_TAG_ENGINE_CONTENT, ASSETS_TAG_IMPORT,
    ASSETS_TAG_PANEL_ROOT, ASSETS_TAG_SCROLLBAR_THUMB, ASSETS_TAG_SCROLLBAR_TRACK,
    ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT, ASSETS_TAG_TOGGLE_SIDEBAR,
    ASSETS_TAG_TREE_SCROLLBAR_THUMB, ASSETS_TAG_TREE_SCROLLBAR_TRACK, ASSETS_TAG_VIEW_GRID,
    ASSETS_TAG_VIEW_LIST, AssetItemAction, AssetPreviewModalState, AssetsContextMenuTarget,
    AssetsPanelAction, AssetsPanelLayoutMetrics, AssetsPanelParams, AssetsPanelState,
    encode_breadcrumb_tag, encode_chip_tag, encode_ctx_item_tag, encode_item_inspect_tag,
    encode_item_spawn_tag, encode_item_tag, encode_tree_chevron_tag, encode_tree_row_tag,
    is_assets_tag, parse_breadcrumb_tag, parse_chip_tag, parse_ctx_item_tag, parse_item_tag,
    parse_tree_tag,
};