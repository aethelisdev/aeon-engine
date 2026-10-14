// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Browser Right-Click Context Menu Builder
//!
//! Renders floating context menus for asset cards and folder tree nodes,
//! supporting quick inspection, spawning, renaming, deletion, path copying,
//! and OS explorer reveal operations using the unified `ContextMenuBuilder`.
//!

use super::types::{AssetsPanelAction, AssetsPanelParams, truncate_display_name};
use crate::assets::types::{AssetCategory, AssetItem};
use crate::ui::iris_bridge::icons::ICON_FOLDER;
use irisui::prelude::*;
use std::path::PathBuf;

/// Width of the floating context menu popup card in logical pixels.
pub const CONTEXT_MENU_WIDTH: f32 = 190.0;

/// Numerical tags assigned to items within the Asset Browser right-click context menu.
pub const ASSET_CTX_INSPECT: u64 = 0;
/// Numerical tag for spawning an asset into the active scene.
pub const ASSET_CTX_SPAWN: u64 = 1;
/// Numerical tag for creating a new subfolder.
pub const ASSET_CTX_NEW_FOLDER: u64 = 2;
/// Numerical tag for renaming an asset or folder.
pub const ASSET_CTX_RENAME: u64 = 3;
/// Numerical tag for deleting an asset or folder.
pub const ASSET_CTX_DELETE: u64 = 4;
/// Numerical tag for copying an asset's file path to the system clipboard.
pub const ASSET_CTX_COPY_PATH: u64 = 5;
/// Numerical tag for revealing an asset or folder in the OS file manager.
pub const ASSET_CTX_REVEAL: u64 = 6;

/// Target subject of an active asset browser context menu.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetsContextMenuTarget {
    /// Context menu opened on an asset item (card or list row).
    Asset(AssetItem),
    /// Context menu opened on a folder node or blank area.
    Folder(PathBuf),
}

/// Actions selectable from the Asset Browser right-click context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetsContextMenuAction {
    /// Opens the 3D orbital inspection preview modal for the asset.
    Inspect,
    /// Instantiates the asset into the active 3D scene.
    Spawn,
    /// Prompts creation of a new subfolder in the directory.
    NewFolder,
    /// Opens the inline renaming modal or text prompt.
    Rename,
    /// Prompts deletion confirmation for the file or folder.
    Delete,
    /// Copies the file system path to the OS clipboard.
    CopyPath,
    /// Reveals the file or directory in the native OS desktop file manager.
    Reveal,
}

/// Resolves a context menu numeric tag into a typed [`AssetsContextMenuAction`].
#[inline]
pub fn resolve_assets_ctx_action(tag: u64) -> Option<AssetsContextMenuAction> {
    match tag {
        ASSET_CTX_INSPECT => Some(AssetsContextMenuAction::Inspect),
        ASSET_CTX_SPAWN => Some(AssetsContextMenuAction::Spawn),
        ASSET_CTX_NEW_FOLDER => Some(AssetsContextMenuAction::NewFolder),
        ASSET_CTX_RENAME => Some(AssetsContextMenuAction::Rename),
        ASSET_CTX_DELETE => Some(AssetsContextMenuAction::Delete),
        ASSET_CTX_COPY_PATH => Some(AssetsContextMenuAction::CopyPath),
        ASSET_CTX_REVEAL => Some(AssetsContextMenuAction::Reveal),
        _ => None,
    }
}

/// Declaratively renders right-click context menu items for an asset card or list row.
///
/// Dispatches semantic actions into `actions` when clicked, closing the context menu
/// automatically via internal framework bindings.
/// Helper to return the contextual spawn label for an asset category.
#[inline]
fn category_spawn_label(category: AssetCategory) -> &'static str {
    match category {
        AssetCategory::Models3D => "Spawn in Viewport",
        AssetCategory::Textures2D => "Apply / Spawn Sprite",
        AssetCategory::Shaders => "Open Shader Editor",
        AssetCategory::Materials => "Edit Material",
        AssetCategory::Audio => "Add Sound to Scene",
        AssetCategory::Scenes => "Load Scene",
        AssetCategory::All => "Spawn / Load",
    }
}

/// Declaratively renders right-click context menu items for an asset card or list row.
///
/// Dispatches semantic actions into `actions` when clicked, closing the context menu
/// automatically via internal framework bindings.
pub fn render_asset_context_menu_items(
    menu: &mut UiScope,
    item: &AssetItem,
    actions: &mut Vec<AssetsPanelAction>,
) {
    menu.context_menu_item("Quick Inspect (Space)")
        .on_click(|| actions.push(AssetsPanelAction::OpenInspectModal(item.clone())));

    menu.context_menu_item(category_spawn_label(item.category))
        .on_click(|| {
            actions.push(AssetsPanelAction::SpawnAsset(
                item.path.clone(),
                item.category,
            ));
        });

    menu.context_menu_separator();
    menu.context_menu_item("Rename").on_click(|| {
        actions.push(AssetsPanelAction::OpenRename(
            item.path.clone(),
            item.name.clone(),
            false,
        ));
    });
    menu.context_menu_item("Delete")
        .on_click(|| actions.push(AssetsPanelAction::OpenDelete(item.path.clone())));

    menu.context_menu_separator();
    menu.context_menu_item("Copy File Path")
        .on_click(|| actions.push(AssetsPanelAction::CopyPath(item.path.clone())));
    menu.context_menu_item("Reveal in Explorer")
        .on_click(|| actions.push(AssetsPanelAction::RevealFolder(item.path.clone())));
}

/// Declaratively renders right-click context menu items for a folder or container area.
///
/// Dispatches semantic folder management actions directly into `actions`.
pub fn render_folder_context_menu_items(
    menu: &mut UiScope,
    folder: &std::path::Path,
    actions: &mut Vec<AssetsPanelAction>,
) {
    let is_folder_root = folder == std::path::Path::new("assets") || folder.as_os_str().is_empty();

    menu.context_menu_item("New Subfolder").on_click(|| {
        actions.push(AssetsPanelAction::OpenCreateSubfolder(folder.to_path_buf()));
    });

    if !is_folder_root {
        menu.context_menu_item("Rename").on_click(|| {
            actions.push(AssetsPanelAction::OpenRename(
                folder.to_path_buf(),
                folder
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("assets")
                    .to_string(),
                true,
            ));
        });
        menu.context_menu_item("Delete Folder")
            .on_click(|| actions.push(AssetsPanelAction::OpenDelete(folder.to_path_buf())));
    }

    menu.context_menu_separator();
    menu.context_menu_item("Reveal in Explorer")
        .on_click(|| actions.push(AssetsPanelAction::RevealFolder(folder.to_path_buf())));
}

/// Builds the floating context menu for an asset or folder into the top-level overlay tree.
pub fn build_assets_context_menu_overlay(
    tree: &mut UiTree,
    parent_id: WidgetId,
    target: &AssetsContextMenuTarget,
    click_pos: Point,
    cursor_pos: Point,
    viewport_bounds: Option<Rect>,
) -> Rect {
    let mut builder = ContextMenuBuilder::new(click_pos)
        .cursor_pos(cursor_pos)
        .width(CONTEXT_MENU_WIDTH);

    if let Some(bounds) = viewport_bounds {
        builder = builder.viewport_bounds(bounds);
    }

    match target {
        AssetsContextMenuTarget::Folder(path) => {
            let is_folder_root =
                path == std::path::Path::new("assets") || path.as_os_str().is_empty();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("assets");

            builder = builder.header(name, Some(ContextMenuIcon::Texture(ICON_FOLDER)));
            builder = builder.item(ASSET_CTX_NEW_FOLDER, "New Subfolder");

            if !is_folder_root {
                builder = builder.item(ASSET_CTX_RENAME, "Rename");
                builder = builder.destructive_item(ASSET_CTX_DELETE, "Delete Folder");
            }

            builder = builder.separator();
            builder = builder.item(ASSET_CTX_REVEAL, "Reveal in Explorer");
        }
        AssetsContextMenuTarget::Asset(item) => {
            builder = builder
                .header(truncate_display_name(&item.name, 20, 17), None)
                .item(ASSET_CTX_INSPECT, "Quick Inspect (Space)")
                .item(ASSET_CTX_SPAWN, category_spawn_label(item.category))
                .separator()
                .item(ASSET_CTX_RENAME, "Rename")
                .destructive_item(ASSET_CTX_DELETE, "Delete")
                .separator()
                .item(ASSET_CTX_COPY_PATH, "Copy File Path")
                .item(ASSET_CTX_REVEAL, "Reveal in Explorer");
        }
    }

    builder.build(tree, parent_id)
}

/// Builds the floating right-click context menu into the `UiTree` if currently open.
///
/// Returns `Some(Rect)` representing the context menu card bounds for outside-click dismissal,
/// or `None` if no context menu is active.
pub fn build_assets_context_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AssetsPanelParams<'_>,
) -> Option<Rect> {
    params.active_context_menu.map(|(target, click_pos)| {
        build_assets_context_menu_overlay(
            tree,
            parent_id,
            target,
            *click_pos,
            params.cursor_pos,
            Some(params.panel_rect),
        )
    })
}