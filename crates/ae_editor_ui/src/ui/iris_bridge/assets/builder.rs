// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Content / Asset Browser Panel Autonomous Builder
//!
//! Autonomous builder coordinating asset filtering, folder navigation, thumbnail metrics,
//! search input live buffer synchronization with [`EventState`], and drag payload management.
//!

use super::panel::build_assets_panel;
use super::types::{
    ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT, AssetsPanelAction, AssetsPanelParams,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Content / Asset Browser panel if active in either docked or floating mode.
    ///
    /// Manages the full lifecycle of the Content Browser:
    /// 1. Resolves physical panel bounds directly from the unified [`UiScope`] docking hierarchy.
    /// 2. Filters asset cache by category, 2D/3D mode, and live text queries.
    /// 3. Assembles directory tree view, asset thumbnail cards, and context menus in unified coordinates.
    /// 4. Dispatches folder navigation, selection, and search actions.
    pub(crate) fn build_assets_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            return;
        }

        if let Some(buf) = scope
            .tree()
            .event_state
            .text_edit_buffer(ASSETS_TAG_SEARCH_INPUT)
        {
            self.assets.interactions.search_input.set_text(buf);
            if let Some(cur) = scope
                .tree()
                .event_state
                .text_edit_cursor(ASSETS_TAG_SEARCH_INPUT)
            {
                self.assets.interactions.search_input.cursor_byte_idx = cur;
            }
        } else if scope.tree().event_state.frame_clicked_tag == Some(ASSETS_TAG_SEARCH_CLEAR) {
            self.assets.interactions.search_input.clear();
            self.clear_focus();
            scope.tree_mut().event_state.clear_text_edit();
        }

        let query_text = self.assets.interactions.search_input.text().trim();
        let is_filter_dirty = !self.assets.is_filter_initialized
            || self.assets.current_folder != params.panel_data.asset_browser.current_folder
            || self.assets.last_search_query != query_text
            || self.assets.last_active_category != params.panel_data.asset_browser.active_category
            || self.assets.last_is_2d_mode != params.context.is_2d_mode
            || self.assets.last_show_engine_content
                != params.panel_data.asset_browser.show_engine_content
            || self.assets.last_cached_items_count
                != params.panel_data.asset_browser.cached_items.len();

        if is_filter_dirty {
            let is_root_folder =
                params.panel_data.asset_browser.current_folder == std::path::Path::new("assets");
            let query_lower = query_text.to_ascii_lowercase();

            self.assets.filtered_items_cache = params
                .panel_data
                .asset_browser
                .cached_items
                .iter()
                .filter(|item| {
                    if !params.panel_data.asset_browser.show_engine_content
                        && item.source == crate::assets::types::AssetSource::Engine
                    {
                        return false;
                    }
                    if params.context.is_2d_mode && item.is_3d {
                        return false;
                    }
                    if !params.context.is_2d_mode
                        && !item.is_3d
                        && item.category == crate::assets::types::AssetCategory::Scenes
                    {
                        return false;
                    }
                    if !is_root_folder
                        && !item
                            .path
                            .starts_with(&params.panel_data.asset_browser.current_folder)
                    {
                        return false;
                    }
                    if params.panel_data.asset_browser.active_category
                        != crate::assets::types::AssetCategory::All
                        && item.category != params.panel_data.asset_browser.active_category
                    {
                        return false;
                    }
                    if !query_lower.is_empty()
                        && !item.name.to_ascii_lowercase().contains(&query_lower)
                        && !item
                            .relative_path
                            .to_ascii_lowercase()
                            .contains(&query_lower)
                    {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();

            self.assets.current_folder = params.panel_data.asset_browser.current_folder.clone();
            self.assets.last_search_query = query_text.to_string();
            self.assets.last_active_category = params.panel_data.asset_browser.active_category;
            self.assets.last_view_mode = params.panel_data.asset_browser.view_mode;
            self.assets.last_sidebar_collapsed = params.panel_data.asset_browser.sidebar_collapsed;
            self.assets.last_is_2d_mode = params.context.is_2d_mode;
            self.assets.last_show_engine_content =
                params.panel_data.asset_browser.show_engine_content;
            self.assets.last_cached_items_count =
                params.panel_data.asset_browser.cached_items.len();
            self.assets.is_filter_initialized = true;
        }

        self.assets.selected_asset = params.panel_data.asset_browser.selected_asset.clone();

        let cursor_pos = self.cursor_pos();
        let blink_caret = (self.start_time.elapsed().as_millis() / 500).is_multiple_of(2);
        let screen_size = (self.screen_width, self.screen_height);
        let focus_state = &self.focus_state;

        let assets_params = AssetsPanelParams {
            panel_rect,
            screen_size,
            current_folder: &self.assets.current_folder,
            search_query: self.assets.interactions.search_input.text(),
            active_category: params.panel_data.asset_browser.active_category,
            view_mode: params.panel_data.asset_browser.view_mode,
            selected_asset: params.panel_data.asset_browser.selected_asset.as_deref(),
            cached_items: &params.panel_data.asset_browser.cached_items,
            filtered_items: &self.assets.filtered_items_cache,
            is_2d_mode: params.context.is_2d_mode,
            show_engine_content: params.panel_data.asset_browser.show_engine_content,
            sidebar_width: params.panel_data.asset_browser.sidebar_width,
            sidebar_collapsed: params.panel_data.asset_browser.sidebar_collapsed,
            cursor_pos,
            blink_caret,
            active_context_menu: self.assets.context_menu.as_ref(),
            active_preview_modal: self.assets.preview_modal.as_ref(),
            subfolders: &params.panel_data.asset_browser.subfolders,
            thumbnail_layers: &self.assets.thumbnail_layers,
            focus_state,
            cursor_byte_idx: self.assets.interactions.search_input.cursor_byte_idx,
        };

        let mut actions = Vec::new();
        build_assets_panel(
            scope,
            &assets_params,
            &mut self.assets.tree_state,
            &mut actions,
        );

        for action in actions {
            match action {
                AssetsPanelAction::NavigateFolder(ref path) => {
                    self.assets.tree_state.select_single(path.clone());
                    let mut curr = path.as_path();
                    while let Some(parent) = curr.parent() {
                        self.assets
                            .tree_state
                            .set_expanded(parent.to_path_buf(), true);
                        curr = parent;
                    }
                    self.assets.current_folder = path.clone();
                    self.assets.actions.push(action);
                    self.notifier.tag_all();
                }
                AssetsPanelAction::SelectAsset(ref opt) => {
                    self.assets.selected_asset = opt.clone();
                    self.assets.actions.push(action);
                    self.notifier.tag_all();
                }
                AssetsPanelAction::FocusSearch => {
                    self.set_focus_tag(ASSETS_TAG_SEARCH_INPUT);
                }
                AssetsPanelAction::ClearSearch => {
                    self.assets.interactions.search_input.clear();
                    self.clear_focus();
                    scope.tree_mut().event_state.clear_text_edit();
                    self.assets.actions.push(action);
                    self.notifier.tag_all();
                }
                other => {
                    self.assets.actions.push(other);
                    self.notifier.tag_all();
                }
            }
        }
    }
}