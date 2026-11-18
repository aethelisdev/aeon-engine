// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Content / Asset Browser Panel Autonomous Builder
//!
//! Autonomous builder coordinating asset filtering, folder navigation, thumbnail metrics,
//! search input live buffer synchronization with [`EventState`], and drag payload management.

use super::panel::build_assets_panel;
use super::types::{
    ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT, AssetsPanelAction, AssetsPanelParams,
    AssetsPanelState,
};
use crate::ui::iris_bridge::panel_views::DeferredPanelNotifier;
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;
use irisui::widgets::controls::{FocusState, FocusTarget};

impl AssetsPanelState {
    /// Synchronizes search query text when unfocused and tags `PanelId::Assets` for redraw
    /// when external asset browser state has changed since the previous frame.
    pub(crate) fn sync_dirty_invalidation(
        &mut self,
        asset_browser: &crate::assets::types::AssetBrowserState,
        is_2d_mode: bool,
        is_search_focused: bool,
        notifier: &mut irisui::dock::UiNotifier<crate::ui::panel_layout::PanelId>,
    ) {
        if !is_search_focused {
            self.interactions
                .search_input
                .set_text(&asset_browser.search_query);
        }
        if self.filtered_cache.is_dirty_with(|key| {
            key.matches(
                asset_browser,
                self.interactions.search_input.text().trim(),
                is_2d_mode,
            )
        }) || !self.actions.is_empty()
        {
            notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
        }
    }

    /// Builds the Content / Asset Browser panel directly using typed assets state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        focus_manager: &mut FocusManager,
        focus_state: &mut FocusState<FocusTarget>,
        notifier: &mut DeferredPanelNotifier<'_>,
        (cursor_pos, blink_caret): (Point, bool),
    ) {
        if scope.is_collapsed() {
            return;
        }

        if scope.popups_dismissed() {
            self.context_menu = None;
        }

        scope.auto_sync_search_input(
            ASSETS_TAG_SEARCH_INPUT,
            Some(ASSETS_TAG_SEARCH_CLEAR),
            &mut self.interactions.search_input,
            focus_manager,
            focus_state,
        );

        self.filtered_cache.sync_with(
            |key| {
                key.matches(
                    params.panel_data.asset_browser,
                    self.interactions.search_input.text().trim(),
                    params.context.is_2d_mode,
                )
            },
            || {
                super::types::AssetsFilterKey::capture(
                    params.panel_data.asset_browser,
                    self.interactions.search_input.text().trim(),
                    params.context.is_2d_mode,
                )
            },
            || {
                params.panel_data.asset_browser.filter_items(
                    self.interactions.search_input.text(),
                    params.context.is_2d_mode,
                )
            },
        );
        self.pending_build_actions.clear();

        build_assets_panel(
            scope,
            &AssetsPanelParams {
                panel_rect: scope.parent_rect(),
                screen_size: (scope.screen_bounds().width, scope.screen_bounds().height),
                current_folder: &params.panel_data.asset_browser.current_folder,
                search_query: self.interactions.search_input.text(),
                active_category: params.panel_data.asset_browser.active_category,
                view_mode: params.panel_data.asset_browser.view_mode,
                selected_asset: params.panel_data.asset_browser.selected_asset.as_deref(),
                cached_items: &params.panel_data.asset_browser.cached_items,
                filtered_items: self.filtered_cache.get(),
                is_2d_mode: params.context.is_2d_mode,
                show_engine_content: params.panel_data.asset_browser.show_engine_content,
                sidebar_width: params.panel_data.asset_browser.sidebar_width,
                sidebar_collapsed: params.panel_data.asset_browser.sidebar_collapsed,
                cursor_pos: scope.resolved_cursor_pos(cursor_pos),
                blink_caret,
                active_context_menu: self.context_menu.as_ref(),
                active_preview_modal: self.preview_modal.as_ref(),
                subfolders: &params.panel_data.asset_browser.subfolders,
                thumbnail_layers: &self.thumbnail_layers,
                focus_state: &*focus_state,
                cursor_byte_idx: self.interactions.search_input.cursor_byte_idx,
            },
            &mut self.tree_state,
            &mut self.pending_build_actions,
        );

        self.pending_build_actions.extend(self.action_bus.drain());
        std::mem::take(&mut self.pending_build_actions)
            .into_iter()
            .for_each(|action| {
                self.handle_panel_action(scope, focus_manager, focus_state, notifier, action)
            });
    }

    fn handle_panel_action(
        &mut self,
        scope: &mut UiScope<'_>,
        focus_manager: &mut FocusManager,
        focus_state: &mut FocusState<FocusTarget>,
        notifier: &mut DeferredPanelNotifier<'_>,
        action: AssetsPanelAction,
    ) {
        match action {
            AssetsPanelAction::NavigateFolder(path) => {
                self.tree_state.select_single(path.clone());
                std::iter::successors(path.parent(), |p| p.parent()).for_each(|parent| {
                    _ = self.tree_state.set_expanded(parent.to_path_buf(), true);
                });
                self.actions.push(AssetsPanelAction::NavigateFolder(path));
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
            AssetsPanelAction::SelectAsset(opt) => {
                self.actions.push(AssetsPanelAction::SelectAsset(opt));
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
            AssetsPanelAction::FocusSearch => {
                scope.assign_focus_with(ASSETS_TAG_SEARCH_INPUT, focus_manager, focus_state);
            }
            AssetsPanelAction::ClearSearch => {
                self.interactions.search_input.clear();
                scope.clear_focus_with(focus_manager, focus_state);
                self.actions.push(AssetsPanelAction::ClearSearch);
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
            AssetsPanelAction::OpenDelete(path) => {
                self.dialog_host.open_confirm_danger(
                    "Delete Asset",
                    format!(
                        "Are you sure you want to permanently delete '{}'?",
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("selected item")
                    ),
                    AssetsPanelAction::ConfirmDelete(path),
                );
                notifier.tag_all();
            }
            AssetsPanelAction::OpenRename(path, name, is_folder) => {
                self.dialog_host.open_input(
                    format!("Rename {}", if is_folder { "Folder" } else { "Asset" }),
                    "New Name:",
                    name,
                    move |new_name| AssetsPanelAction::ConfirmRename(path.clone(), new_name),
                );
                notifier.tag_all();
            }
            AssetsPanelAction::OpenCreateSubfolder(parent) => {
                self.dialog_host.open_input(
                    "New Folder",
                    "Folder Name:",
                    "New Folder",
                    move |name| AssetsPanelAction::ConfirmNewFolder(parent.clone(), name),
                );
                notifier.tag_all();
            }
            AssetsPanelAction::ConfirmDelete(path) => {
                drop(crate::assets::file_ops::delete_asset_or_folder(&path));
                self.filtered_cache.invalidate();
                self.actions.push(AssetsPanelAction::SelectAsset(None));
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
            AssetsPanelAction::ConfirmRename(path, new_name) => {
                drop(crate::assets::file_ops::rename_asset_or_folder(
                    &path, &new_name,
                ));
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
            AssetsPanelAction::ConfirmNewFolder(parent, name) => {
                drop(crate::assets::file_ops::create_subfolder(&parent, &name));
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
            AssetsPanelAction::CloseInspectModal => {
                self.preview_modal = None;
                notifier.tag_all();
            }
            other => {
                self.actions.push(other);
                notifier.tag_redraw(crate::ui::panel_layout::PanelId::Assets);
            }
        }
    }

    /// Uploads dynamic 64x64 thumbnail previews for uncached asset browser items into `surface`'s
    /// managed [`irisui::TextureArrayAtlas`].
    pub fn ensure_thumbnails(
        &mut self,
        surface: &irisui::IrisSurfaceHost,
        queue: &wgpu::Queue,
        items: &[crate::assets::types::AssetItem],
    ) {
        surface
            .texture_atlas
            .as_ref()
            .into_iter()
            .for_each(|atlas| {
                for item in items {
                    if self.next_thumbnail_layer >= atlas.layer_count() {
                        break;
                    }
                    if self.thumbnail_layers.contains_key(&item.path) {
                        continue;
                    }
                    crate::assets::thumbnails::generate_thumbnail_rgba_64(
                        &item.path,
                        item.category,
                    )
                    .into_iter()
                    .for_each(|rgba| {
                        if atlas.upload_rgba_layer(queue, self.next_thumbnail_layer, &rgba) {
                            self.thumbnail_layers
                                .insert(item.path.clone(), self.next_thumbnail_layer);
                            self.next_thumbnail_layer += 1;
                        }
                    });
                }
            });
    }
}