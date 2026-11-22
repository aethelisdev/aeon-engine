// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Main Panel Orchestrator for Iris UI.
//!
//! Ties together top breadcrumb and action navigation, category filter chips,
//! left folder tree sidebar, scrollable card/table views, and telemetry footer
//! using pure declarative Taffy flexbox layout without manual pixel math.

use super::cards::build_asset_grid_cards_scope;
use super::chips::build_category_chips_scope;
use super::drag_overlay::build_asset_drag_overlays_scope;
use super::footer::build_asset_footer_scope;
use super::list::build_asset_list_table_scope;
use super::toolbar::build_asset_toolbar_scope;
use super::tree::build_folder_tree_sidebar_scope;
pub use super::types::{ASSETS_CHIPS_BAR_HEIGHT, ASSETS_FOOTER_HEIGHT, ASSETS_TOP_BAR_HEIGHT};
use super::types::{
    AssetClickTracker, AssetPreviewModalState, AssetsFilterKey, AssetsPanelAction, AssetsPanelState,
};
use crate::assets::types::{
    AssetBrowserState, AssetCategory, AssetDragPayload, AssetItem, AssetViewMode,
};
use crate::ui::panel_layout::PanelId;
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::EngineUiAction;
use irisui::core::MemoCache;
use irisui::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Disjoint mutable borrow slice over [`AssetsPanelState`] rendering fields so
/// `action_scratch` and `click_tracker` remain independently borrowable.
struct AssetsViewMut<'a> {
    search_input: &'a mut TextInputState,
    filtered_cache: &'a mut MemoCache<AssetsFilterKey, Vec<AssetItem>>,
    thumbnail_layers: &'a HashMap<PathBuf, u32>,
    preview_modal: Option<&'a AssetPreviewModalState>,
}

impl AssetsPanelState {
    /// Generates and uploads missing 64x64 asset preview thumbnails into the GPU texture atlas.
    pub(crate) fn ensure_thumbnails(
        &mut self,
        surface: &mut irisui::IrisSurfaceHost,
        queue: &wgpu::Queue,
        items: &[AssetItem],
    ) {
        items
            .iter()
            .for_each(|item| self.ensure_item_thumbnail(surface, queue, item));
    }

    fn ensure_item_thumbnail(
        &mut self,
        surface: &mut irisui::IrisSurfaceHost,
        queue: &wgpu::Queue,
        item: &AssetItem,
    ) {
        if self.thumbnail_layers.contains_key(&item.path) {
            return;
        }
        crate::assets::thumbnails::generate_thumbnail_rgba_64(&item.path, item.category)
            .and_then(|rgba| surface.alloc_atlas_rgba_layer(queue, &rgba))
            .into_iter()
            .for_each(|layer| {
                self.thumbnail_layers.insert(item.path.clone(), layer);
            });
    }

    /// Entrypoint invoked directly by [`irisui::workbench::ScopedPanelRegistry`].
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        self.action_scratch.clear();
        build_assets_panel_view(
            scope,
            ctx.data.asset_browser,
            ctx.data.is_2d_mode,
            AssetsViewMut {
                search_input: &mut self.search_input,
                filtered_cache: &mut self.filtered_cache,
                thumbnail_layers: &self.thumbnail_layers,
                preview_modal: self.preview_modal.as_ref(),
            },
            &mut self.action_scratch,
        );
        render_active_drag_overlay(scope, ctx);
        if !self.action_scratch.is_empty() {
            ctx.notifier.tag_redraw(PanelId::Assets);
        }
        self.action_scratch.drain(..).for_each(|action| {
            handle_panel_action(
                &mut self.preview_modal,
                &mut self.click_tracker,
                ctx,
                action,
            );
        });
    }
}

/// Constructs the complete Content / Asset Browser panel widget hierarchy into the Iris `UiTree`.
pub fn build_assets_panel(
    scope: &mut UiScope<'_>,
    asset_browser: &AssetBrowserState,
    is_2d_mode: bool,
    state: &mut AssetsPanelState,
    actions: &mut Vec<AssetsPanelAction>,
) {
    build_assets_panel_view(
        scope,
        asset_browser,
        is_2d_mode,
        AssetsViewMut {
            search_input: &mut state.search_input,
            filtered_cache: &mut state.filtered_cache,
            thumbnail_layers: &state.thumbnail_layers,
            preview_modal: state.preview_modal.as_ref(),
        },
        actions,
    );
}

fn build_assets_panel_view(
    scope: &mut UiScope<'_>,
    asset_browser: &AssetBrowserState,
    is_2d_mode: bool,
    view: AssetsViewMut<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    build_asset_toolbar_scope(scope, asset_browser, view.search_input, actions);
    build_category_chips_scope(scope, asset_browser.active_category, actions);
    build_assets_body_and_footer(
        scope,
        asset_browser,
        view.thumbnail_layers,
        sync_filtered_assets(
            view.filtered_cache,
            asset_browser,
            active_search_query(asset_browser, view.search_input),
            is_2d_mode,
        ),
        actions,
    );

    if super::preview::build_asset_preview_modal(scope, view.preview_modal) {
        actions.push(AssetsPanelAction::CloseInspectModal);
    }
}

#[inline]
fn active_search_query<'a>(
    asset_browser: &'a AssetBrowserState,
    search_input: &'a TextInputState,
) -> &'a str {
    if search_input.text().is_empty() {
        &asset_browser.search_query
    } else {
        search_input.text()
    }
}

fn sync_filtered_assets<'c>(
    filtered_cache: &'c mut MemoCache<AssetsFilterKey, Vec<AssetItem>>,
    asset_browser: &AssetBrowserState,
    query: &str,
    is_2d_mode: bool,
) -> &'c [AssetItem] {
    filtered_cache.sync_with(
        |key| key.matches(asset_browser, query, is_2d_mode),
        || AssetsFilterKey::capture(asset_browser, query, is_2d_mode),
        || asset_browser.filter_items(query, is_2d_mode),
    )
}

fn build_assets_body_and_footer(
    panel_scope: &mut UiScope<'_>,
    asset_browser: &AssetBrowserState,
    thumbnail_layers: &HashMap<PathBuf, u32>,
    filtered_items: &[AssetItem],
    actions: &mut Vec<AssetsPanelAction>,
) {
    panel_scope.container_named(
        "AssetsPanelBody",
        Style::new().flex_row().flex_grow(1.0).min_height(0.0),
        |body_scope| {
            if !asset_browser.sidebar_collapsed {
                build_folder_tree_sidebar_scope(
                    body_scope,
                    asset_browser.sidebar_width,
                    &asset_browser.current_folder,
                    &asset_browser.subfolders,
                    actions,
                );
            }
            build_assets_drop_area(
                body_scope,
                asset_browser,
                filtered_items,
                thumbnail_layers,
                actions,
            );
        },
    );
    build_asset_footer_scope(panel_scope, asset_browser, filtered_items.len(), actions);
}

fn build_assets_drop_area(
    body_scope: &mut UiScope<'_>,
    asset_browser: &AssetBrowserState,
    filtered_items: &[AssetItem],
    thumbnail_layers: &HashMap<PathBuf, u32>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    body_scope
        .drop_target_styled(
            "AssetsContentArea",
            Style::new()
                .flex_col()
                .flex_grow(1.0)
                .min_width(0.0)
                .min_height(0.0)
                .background(body_scope.theme().panel_bg),
            |target_scope, is_hovered| {
                if is_hovered {
                    render_asset_drop_indicator(target_scope);
                }
                render_assets_content_view(
                    target_scope,
                    asset_browser,
                    filtered_items,
                    thumbnail_layers,
                    actions,
                );
            },
        )
        .on_drop(|paths| {
            actions.push(AssetsPanelAction::ImportDroppedFiles(paths.to_vec()));
        });
}

fn render_asset_drop_indicator(scope: &mut UiScope<'_>) {
    scope.container_named(
        "AssetDropIndicator",
        Style::new()
            .height(28.0)
            .background(scope.theme().accent.with_alpha(0.20))
            .border(1.0, scope.theme().accent)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center),
        |indicator| {
            indicator.label("Drop files here to import into project");
        },
    );
}

fn render_assets_content_view(
    target_scope: &mut UiScope<'_>,
    asset_browser: &AssetBrowserState,
    filtered_items: &[AssetItem],
    thumbnail_layers: &HashMap<PathBuf, u32>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    match asset_browser.view_mode {
        AssetViewMode::Grid => build_asset_grid_cards_scope(
            target_scope,
            filtered_items,
            asset_browser.selected_asset.as_deref(),
            &asset_browser.current_folder,
            &asset_browser.search_query,
            thumbnail_layers,
            actions,
        ),
        AssetViewMode::List => build_asset_list_table_scope(
            target_scope,
            filtered_items,
            asset_browser.selected_asset.as_deref(),
            &asset_browser.current_folder,
            &asset_browser.search_query,
            actions,
        ),
    }
}

fn render_active_drag_overlay(scope: &mut UiScope<'_>, ctx: &EditorPanelContext<'_>) {
    ctx.data
        .asset_browser
        .drag_payload
        .as_ref()
        .into_iter()
        .for_each(|payload| {
            build_asset_drag_overlays_scope(
                scope,
                payload,
                ctx.cursor_pos,
                ctx.data.viewport_rect,
                ctx.data.camera,
                ctx.data.is_2d_mode,
            );
        });
}

fn handle_panel_action(
    preview_modal: &mut Option<AssetPreviewModalState>,
    click_tracker: &mut AssetClickTracker,
    ctx: &mut EditorPanelContext<'_>,
    action: AssetsPanelAction,
) {
    match action {
        AssetsPanelAction::NavigateFolder(path) => {
            ctx.data.asset_browser.current_folder = path;
        }
        AssetsPanelAction::SelectAsset(path) => {
            handle_select_asset(click_tracker, ctx, path);
        }
        AssetsPanelAction::SelectCategory(cat) => {
            ctx.data.asset_browser.active_category = cat;
        }
        AssetsPanelAction::SetViewMode(mode) => {
            ctx.data.asset_browser.view_mode = mode;
        }
        AssetsPanelAction::ToggleSidebar => {
            ctx.data.asset_browser.sidebar_collapsed = !ctx.data.asset_browser.sidebar_collapsed;
        }
        AssetsPanelAction::SearchInput(query) => {
            ctx.data.asset_browser.search_query = query;
        }
        AssetsPanelAction::ClearSearch => {
            ctx.data.asset_browser.search_query.clear();
        }
        AssetsPanelAction::ToggleEngineContent => {
            ctx.data.asset_browser.show_engine_content =
                !ctx.data.asset_browser.show_engine_content;
        }
        AssetsPanelAction::SpawnAsset(path, category) => {
            emit_spawn_asset_action(ctx, path, category);
        }
        other => handle_modal_fs_or_drag_action(preview_modal, ctx, other),
    }
}

fn handle_select_asset(
    click_tracker: &mut AssetClickTracker,
    ctx: &mut EditorPanelContext<'_>,
    path: Option<PathBuf>,
) {
    ctx.data.asset_browser.selected_asset = path.clone();
    click_tracker.last_path = path;
    click_tracker.last_instant = Some(std::time::Instant::now());
}

fn emit_spawn_asset_action(
    ctx: &mut EditorPanelContext<'_>,
    path: PathBuf,
    category: AssetCategory,
) {
    match category {
        AssetCategory::Models3D => ctx
            .data
            .asset_browser
            .cached_items
            .iter()
            .find(|item| item.path == path)
            .and_then(|item| item.model_handle)
            .map_or(EngineUiAction::SpawnModelPathAt(path, [0.0; 3]), |handle| {
                EngineUiAction::SpawnModel(handle)
            })
            .pipe_to_bus(ctx),
        AssetCategory::Textures2D => ctx
            .data
            .asset_browser
            .cached_items
            .iter()
            .find(|item| item.path == path)
            .and_then(|item| item.texture_handle)
            .map_or(
                EngineUiAction::SpawnSpritePathAt(path, [0.0; 3]),
                EngineUiAction::SpawnSprite,
            )
            .pipe_to_bus(ctx),
        AssetCategory::Scenes => ctx.action_bus.emit(EngineUiAction::LoadSceneFromPath(path)),
        _ => {}
    }
}

trait EmitToBusExt {
    fn pipe_to_bus(self, ctx: &mut EditorPanelContext<'_>);
}

impl EmitToBusExt for EngineUiAction {
    #[inline]
    fn pipe_to_bus(self, ctx: &mut EditorPanelContext<'_>) {
        ctx.action_bus.emit(self);
    }
}

fn handle_modal_fs_or_drag_action(
    preview_modal: &mut Option<AssetPreviewModalState>,
    ctx: &mut EditorPanelContext<'_>,
    action: AssetsPanelAction,
) {
    match action {
        AssetsPanelAction::OpenImportDialog => {
            ctx.action_bus.emit(EngineUiAction::OpenModelDialog);
        }
        AssetsPanelAction::RevealFolder(path) => {
            drop(crate::assets::file_ops::open_in_file_explorer(&path));
        }
        AssetsPanelAction::CleanVram => {
            ctx.action_bus.emit(EngineUiAction::GarbageCollect);
        }
        AssetsPanelAction::OpenInspectModal(item) | AssetsPanelAction::InspectAsset(item) => {
            *preview_modal = Some(AssetPreviewModalState {
                item,
                ..Default::default()
            });
        }
        AssetsPanelAction::CloseInspectModal => *preview_modal = None,
        AssetsPanelAction::InspectOrbitDelta(dy, dp) => {
            apply_preview_orbit(preview_modal, dy, dp);
        }
        AssetsPanelAction::InspectZoomDelta(dz) => apply_preview_zoom(preview_modal, dz),
        AssetsPanelAction::OpenCreateSubfolder(parent) => {
            ctx.dialog_host
                .open_input("New Subfolder", "Folder Name:", "NewFolder", move |name| {
                    EngineUiAction::CreateSubfolder {
                        parent: parent.clone(),
                        name,
                    }
                });
        }
        AssetsPanelAction::OpenRename(target, current_name, _) => {
            ctx.dialog_host.open_input(
                "Rename Asset",
                "New Name:",
                &current_name,
                move |new_name| EngineUiAction::RenameAssetOrFolder {
                    target: target.clone(),
                    new_name,
                },
            );
        }
        AssetsPanelAction::OpenDelete(target) => {
            ctx.dialog_host.open_confirm_danger(
                "Delete Asset",
                format!(
                    "Are you sure you want to permanently delete '{}'?",
                    target
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("item")
                ),
                EngineUiAction::DeleteAssetOrFolder(target),
            );
        }
        AssetsPanelAction::ConfirmDelete(target) => {
            ctx.action_bus
                .emit(EngineUiAction::DeleteAssetOrFolder(target));
        }
        AssetsPanelAction::ConfirmRename(target, new_name) => {
            ctx.action_bus
                .emit(EngineUiAction::RenameAssetOrFolder { target, new_name });
        }
        AssetsPanelAction::ConfirmNewFolder(parent, name) => {
            ctx.action_bus
                .emit(EngineUiAction::CreateSubfolder { parent, name });
        }
        AssetsPanelAction::ImportDroppedFiles(paths) => {
            ctx.action_bus.emit(EngineUiAction::ImportDroppedFiles {
                paths,
                screen_pos: None,
            });
        }
        AssetsPanelAction::StartAssetDrag(payload) => handle_start_asset_drag(ctx, &payload),
        AssetsPanelAction::EndAssetDrag => handle_end_asset_drag(ctx),
        AssetsPanelAction::CopyPath(path) => {
            log::info!("Asset path copied: {}", path.display());
        }
        AssetsPanelAction::FocusSearch
        | AssetsPanelAction::Scroll(_)
        | AssetsPanelAction::TreeScroll(_)
        | AssetsPanelAction::OpenContextMenu(_, _)
        | AssetsPanelAction::CloseContextMenu => {}
        _ => {}
    }
}

fn apply_preview_orbit(preview_modal: &mut Option<AssetPreviewModalState>, dy: f32, dp: f32) {
    preview_modal.iter_mut().for_each(|modal| {
        modal.orbit_yaw += dy;
        modal.orbit_pitch = (modal.orbit_pitch + dp).clamp(-1.45, 1.45);
    });
}

fn apply_preview_zoom(preview_modal: &mut Option<AssetPreviewModalState>, dz: f32) {
    preview_modal.iter_mut().for_each(|modal| {
        modal.zoom_distance = (modal.zoom_distance + dz).clamp(0.4, 5.0);
    });
}

fn handle_start_asset_drag(
    ctx: &mut EditorPanelContext<'_>,
    payload: &irisui::workbench::DragDropPayload,
) {
    payload
        .downcast_ref::<AssetItem>()
        .into_iter()
        .for_each(|item| {
            ctx.data.asset_browser.drag_payload = Some(AssetDragPayload {
                path: item.path.clone(),
                name: item.name.clone(),
                category: item.category,
                model_handle: item.model_handle,
                texture_handle: item.texture_handle,
            });
        });
}

fn handle_end_asset_drag(ctx: &mut EditorPanelContext<'_>) {
    crate::assets::drag_drop::resolve_viewport_drop_action(
        ctx.data.asset_browser,
        ctx.cursor_pos,
        ctx.data.viewport_rect,
        ctx.data.camera,
        ctx.data.is_2d_mode,
    )
    .into_iter()
    .for_each(|act| ctx.action_bus.emit(act));
}