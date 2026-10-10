// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Browser Right-Click Context Menu Builder
//!
//! Renders floating context menus for asset cards and folder tree nodes,
//! supporting quick inspection, spawning, renaming, deletion, path copying,
//! and OS explorer reveal operations using the unified `ContextMenuBuilder`.
//!

use super::types::{
    ASSET_CTX_COPY_PATH, ASSET_CTX_DELETE, ASSET_CTX_INSPECT, ASSET_CTX_NEW_FOLDER,
    ASSET_CTX_RENAME, ASSET_CTX_REVEAL, ASSET_CTX_SPAWN, AssetsContextMenuTarget,
    AssetsPanelParams, truncate_display_name,
};
use crate::assets::types::AssetCategory;
use crate::ui::iris_bridge::icons::ICON_FOLDER;
use irisui::prelude::*;

/// Width of the floating context menu popup card in logical pixels.
pub const CONTEXT_MENU_WIDTH: f32 = 190.0;

/// Builds the floating right-click context menu into the `UiTree` if currently open.
///
/// Returns `Some(Rect)` representing the context menu card bounds for outside-click dismissal,
/// or `None` if no context menu is active.
/// Builds the floating context menu for an asset or folder into the top-level overlay tree.
pub fn build_assets_context_menu_overlay(
    tree: &mut UiTree,
    parent_id: WidgetId,
    target: &AssetsContextMenuTarget,
    click_pos: Point,
    cursor_pos: Point,
    viewport_bounds: Option<Rect>,
) -> Rect {
    let is_folder_root = match target {
        AssetsContextMenuTarget::Folder(path) => {
            path == std::path::Path::new("assets") || path.as_os_str().is_empty()
        }
        AssetsContextMenuTarget::Asset(_) => false,
    };

    let mut builder = ContextMenuBuilder::new(click_pos)
        .cursor_pos(cursor_pos)
        .width(CONTEXT_MENU_WIDTH);

    if let Some(bounds) = viewport_bounds {
        builder = builder.viewport_bounds(bounds);
    }

    match target {
        AssetsContextMenuTarget::Folder(path) => {
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
            let title = truncate_display_name(&item.name, 20, 17);
            builder = builder.header(title, None);

            builder = builder.item(ASSET_CTX_INSPECT, "Quick Inspect (Space)");

            let spawn_label = match item.category {
                AssetCategory::Models3D => "Spawn in Viewport",
                AssetCategory::Textures2D => "Apply / Spawn Sprite",
                AssetCategory::Shaders => "Open Shader Editor",
                AssetCategory::Materials => "Edit Material",
                AssetCategory::Audio => "Add Sound to Scene",
                AssetCategory::Scenes => "Load Scene",
                AssetCategory::All => "Spawn / Load",
            };
            builder = builder.item(ASSET_CTX_SPAWN, spawn_label);

            builder = builder.separator();
            builder = builder.item(ASSET_CTX_RENAME, "Rename");
            builder = builder.destructive_item(ASSET_CTX_DELETE, "Delete");

            builder = builder.separator();
            builder = builder.item(ASSET_CTX_COPY_PATH, "Copy File Path");
            builder = builder.item(ASSET_CTX_REVEAL, "Reveal in Explorer");
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
    let (target, click_pos) = params.active_context_menu?;
    let card_rect = build_assets_context_menu_overlay(
        tree,
        parent_id,
        target,
        *click_pos,
        params.cursor_pos,
        Some(params.panel_rect),
    );
    Some(card_rect)
}