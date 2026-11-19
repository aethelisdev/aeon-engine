// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Content / Asset Browser Panel Autonomous Builder
//!
//! Autonomous builder coordinating asset filtering, folder navigation, thumbnail metrics,
//! search input live buffer synchronization with [`EventState`], and drag payload management.

use super::panel::build_assets_panel;
use super::types::{
    ASSETS_TAG_PREVIEW_MODAL, ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT,
    AssetPreviewModalState, AssetsPanelAction, AssetsPanelParams, AssetsPanelState,
};
use crate::assets::types::{AssetCategory, AssetDragPayload, AssetItem};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::{ActionBus, DialogHost};

/// Execution context passed to [`AssetsPanelState::build_view`].
pub(crate) struct AssetsBuildContext<'a, 'p> {
    pub params: &'a mut OverlayUpdateParams<'p>,
    pub notifier: &'a mut DeferredNotifier<PanelId>,
    pub action_bus: &'a mut ActionBus<EngineUiAction>,
    pub dialog_host: &'a mut DialogHost<EngineUiAction>,
    pub cursor_pos: Point,
    pub blink_caret: bool,
}

impl AssetsPanelState {
    /// Builds the Content / Asset Browser panel directly using typed assets state and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        mut ctx: AssetsBuildContext<'_, '_>,
    ) {
        if scope.is_collapsed() {
            return;
        }
        if !scope.is_tag_focused(ASSETS_TAG_SEARCH_INPUT)
            && self.search_input.text() != ctx.params.panel_data.asset_browser.search_query
        {
            self.search_input
                .set_text(&ctx.params.panel_data.asset_browser.search_query);
        }
        scope.auto_sync_text_input(
            ASSETS_TAG_SEARCH_INPUT,
            Some(ASSETS_TAG_SEARCH_CLEAR),
            &mut self.search_input,
        );
        if self.search_input.text() != ctx.params.panel_data.asset_browser.search_query {
            ctx.params.panel_data.asset_browser.search_query = self.search_input.text().to_owned();
        }

        self.filtered_cache.sync_with(
            |key| {
                key.matches(
                    ctx.params.panel_data.asset_browser,
                    self.search_input.text().trim(),
                    ctx.params.context.is_2d_mode,
                )
            },
            || {
                super::types::AssetsFilterKey::capture(
                    ctx.params.panel_data.asset_browser,
                    self.search_input.text().trim(),
                    ctx.params.context.is_2d_mode,
                )
            },
            || {
                ctx.params
                    .panel_data
                    .asset_browser
                    .filter_items(self.search_input.text(), ctx.params.context.is_2d_mode)
            },
        );

        self.render_and_dispatch(scope, &mut ctx, Vec::new());
    }

    fn render_and_dispatch(
        &mut self,
        scope: &mut UiScope<'_>,
        ctx: &mut AssetsBuildContext<'_, '_>,
        mut actions: Vec<AssetsPanelAction>,
    ) {
        build_assets_panel(
            scope,
            &AssetsPanelParams {
                panel_rect: scope.parent_rect(),
                screen_size: (scope.screen_bounds().width, scope.screen_bounds().height),
                current_folder: &ctx.params.panel_data.asset_browser.current_folder,
                search_query: self.search_input.text(),
                active_category: ctx.params.panel_data.asset_browser.active_category,
                view_mode: ctx.params.panel_data.asset_browser.view_mode,
                selected_asset: ctx
                    .params
                    .panel_data
                    .asset_browser
                    .selected_asset
                    .as_deref(),
                cached_items: &ctx.params.panel_data.asset_browser.cached_items,
                filtered_items: self.filtered_cache.get(),
                is_2d_mode: ctx.params.context.is_2d_mode,
                show_engine_content: ctx.params.panel_data.asset_browser.show_engine_content,
                sidebar_width: ctx.params.panel_data.asset_browser.sidebar_width,
                sidebar_collapsed: ctx.params.panel_data.asset_browser.sidebar_collapsed,
                cursor_pos: scope.resolved_cursor_pos(ctx.cursor_pos),
                blink_caret: ctx.blink_caret,
                active_context_menu: None,
                active_preview_modal: self.preview_modal.as_ref(),
                subfolders: &ctx.params.panel_data.asset_browser.subfolders,
                thumbnail_layers: &self.thumbnail_layers,
                cursor_byte_idx: self.search_input.cursor_byte_idx,
            },
            &mut self.tree_state,
            &mut actions,
        );

        for action in actions {
            self.handle_panel_action(scope, ctx, action);
        }
    }

    fn handle_panel_action(
        &mut self,
        scope: &mut UiScope<'_>,
        ctx: &mut AssetsBuildContext<'_, '_>,
        action: AssetsPanelAction,
    ) {
        match action {
            AssetsPanelAction::NavigateFolder(path) => {
                self.tree_state.select_single(path.clone());
                std::iter::successors(path.parent(), |p| p.parent()).for_each(|parent| {
                    _ = self.tree_state.set_expanded(parent.to_path_buf(), true);
                });
                ctx.params.panel_data.asset_browser.current_folder = path;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::SelectAsset(opt) => {
                ctx.params.panel_data.asset_browser.selected_asset = opt;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::SelectCategory(cat) => {
                ctx.params.panel_data.asset_browser.active_category = cat;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::SetViewMode(mode) => {
                ctx.params.panel_data.asset_browser.view_mode = mode;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::ToggleSidebar => {
                ctx.params.panel_data.asset_browser.sidebar_collapsed =
                    !ctx.params.panel_data.asset_browser.sidebar_collapsed;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::SearchInput(query) => {
                self.search_input.set_text(&query);
                ctx.params.panel_data.asset_browser.search_query = query;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::FocusSearch => {
                scope.set_focus_tag(ASSETS_TAG_SEARCH_INPUT);
            }
            AssetsPanelAction::ClearSearch => {
                self.search_input.clear();
                ctx.params.panel_data.asset_browser.search_query.clear();
                scope.clear_focus();
                scope.clear_text_edit();
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::ToggleEngineContent => {
                ctx.params.panel_data.asset_browser.show_engine_content =
                    !ctx.params.panel_data.asset_browser.show_engine_content;
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::OpenImportDialog => {
                ctx.action_bus.emit(EngineUiAction::OpenModelDialog);
            }
            AssetsPanelAction::RevealFolder(path) => {
                drop(crate::assets::file_ops::open_in_file_explorer(&path));
            }
            AssetsPanelAction::CleanVram => {
                ctx.action_bus.emit(EngineUiAction::GarbageCollect);
            }
            AssetsPanelAction::CopyPath(path) => {
                _ = irisui::widgets::SystemClipboard::new().set_text(&path.to_string_lossy());
            }
            AssetsPanelAction::ImportDroppedFiles(paths) => {
                ctx.action_bus.emit(EngineUiAction::ImportDroppedFiles {
                    paths,
                    screen_pos: Some([
                        scope.resolved_cursor_pos(ctx.cursor_pos).x,
                        scope.resolved_cursor_pos(ctx.cursor_pos).y,
                    ]),
                });
            }
            AssetsPanelAction::SpawnAsset(path, cat) => match cat {
                AssetCategory::Models3D => {
                    ctx.action_bus
                        .emit(EngineUiAction::SpawnModelPathAt(path, [0.0, 0.0, 0.0]));
                }
                AssetCategory::Textures2D => {
                    ctx.action_bus
                        .emit(EngineUiAction::SpawnSpritePathAt(path, [0.0, 0.0, 0.0]));
                }
                AssetCategory::Scenes => {
                    ctx.action_bus.emit(EngineUiAction::LoadSceneFromPath(path));
                }
                _ => {}
            },
            AssetsPanelAction::InspectAsset(item) | AssetsPanelAction::OpenInspectModal(item) => {
                self.preview_modal = Some(AssetPreviewModalState {
                    item,
                    orbit_yaw: 0.6,
                    orbit_pitch: 0.35,
                    zoom_distance: 1.0,
                    show_wireframe: true,
                });
                scope.open_modal(ASSETS_TAG_PREVIEW_MODAL);
                ctx.notifier.tag_all();
            }
            AssetsPanelAction::CloseInspectModal => {
                self.preview_modal = None;
                scope.close_modal(ASSETS_TAG_PREVIEW_MODAL);
                ctx.notifier.tag_all();
            }
            AssetsPanelAction::OpenDelete(path) => {
                ctx.dialog_host.open_confirm_danger(
                    "Delete Asset",
                    format!(
                        "Are you sure you want to permanently delete '{}'?",
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("selected item")
                    ),
                    EngineUiAction::DeleteAssetOrFolder(path),
                );
                ctx.notifier.tag_all();
            }
            AssetsPanelAction::OpenRename(target, name, is_folder) => {
                ctx.dialog_host.open_input(
                    format!("Rename {}", if is_folder { "Folder" } else { "Asset" }),
                    "New Name:",
                    name,
                    move |new_name| EngineUiAction::RenameAssetOrFolder {
                        target: target.clone(),
                        new_name,
                    },
                );
                ctx.notifier.tag_all();
            }
            AssetsPanelAction::OpenCreateSubfolder(parent) => {
                ctx.dialog_host.open_input(
                    "New Folder",
                    "Folder Name:",
                    "New Folder",
                    move |name| EngineUiAction::CreateSubfolder {
                        parent: parent.clone(),
                        name,
                    },
                );
                ctx.notifier.tag_all();
            }
            AssetsPanelAction::ConfirmDelete(path) => {
                ctx.action_bus
                    .emit(EngineUiAction::DeleteAssetOrFolder(path));
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::ConfirmRename(target, new_name) => {
                ctx.action_bus
                    .emit(EngineUiAction::RenameAssetOrFolder { target, new_name });
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::ConfirmNewFolder(parent, name) => {
                ctx.action_bus
                    .emit(EngineUiAction::CreateSubfolder { parent, name });
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            AssetsPanelAction::StartAssetDrag(payload) => {
                payload
                    .downcast_ref::<AssetItem>()
                    .into_iter()
                    .for_each(|item| {
                        if ctx
                            .params
                            .panel_data
                            .asset_browser
                            .drag_payload
                            .as_ref()
                            .is_none_or(|p| p.path != item.path)
                        {
                            ctx.params.panel_data.asset_browser.drag_payload =
                                Some(AssetDragPayload {
                                    path: item.path.clone(),
                                    name: item.name.clone(),
                                    category: item.category,
                                    model_handle: item.model_handle,
                                    texture_handle: item.texture_handle,
                                });
                            ctx.notifier.tag_redraw(PanelId::Viewport);
                            ctx.notifier.tag_redraw(PanelId::Assets);
                        }
                    });
            }
            AssetsPanelAction::EndAssetDrag => {
                crate::assets::drag_drop::resolve_viewport_drop_action(
                    ctx.params.panel_data.asset_browser,
                    scope.resolved_cursor_pos(ctx.cursor_pos),
                    ctx.params.viewport.viewport_rect,
                    ctx.params.viewport.camera,
                    ctx.params.context.is_2d_mode,
                )
                .into_iter()
                .for_each(|act| ctx.action_bus.emit(act));
                ctx.notifier.tag_redraw(PanelId::Viewport);
                ctx.notifier.tag_redraw(PanelId::Assets);
            }
            _ => {}
        }
    }

    /// Uploads dynamic 64x64 thumbnail previews for uncached asset browser items into `surface`'s
    /// managed [`irisui::TextureArrayAtlas`].
    pub fn ensure_thumbnails(
        &mut self,
        surface: &irisui::IrisSurfaceHost,
        queue: &wgpu::Queue,
        items: &[AssetItem],
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