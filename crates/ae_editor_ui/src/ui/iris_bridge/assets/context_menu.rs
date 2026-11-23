// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset & Folder Floating Context Menu Builder
//!
//! Renders declarative right-click context menu items for individual assets
//! and directories on [`UiLayer::Popup`] via [`UiScope`].

use super::types::AssetsPanelAction;
use ae_assets::types::{AssetCategory, AssetItem};
use irisui::prelude::*;
use std::path::{Path, PathBuf};

/// Context menu target in the Asset Browser.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetsContextMenuTarget {
    /// Right-clicked on a specific asset card or row.
    Asset(AssetItem),
    /// Right-clicked on a folder in the sidebar tree or empty content area.
    Folder(PathBuf),
}

/// Resolves the category-specific spawn action label for an asset item.
#[inline]
#[must_use]
fn category_spawn_label(category: AssetCategory) -> &'static str {
    match category {
        AssetCategory::Models3D => "Spawn into 3D Scene",
        AssetCategory::Textures2D => "Spawn as 2D Sprite",
        AssetCategory::Scenes => "Load World Scene",
        _ => "Open / Apply Asset",
    }
}

/// Renders declarative context menu items for an individual [`AssetItem`] into an active menu [`UiScope`].
pub fn render_asset_context_menu_items(
    menu: &mut UiScope<'_>,
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
            ))
        });

    menu.context_menu_separator();

    menu.context_menu_item("Rename").on_click(|| {
        actions.push(AssetsPanelAction::OpenRename(
            item.path.clone(),
            item.name.clone(),
            false,
        ))
    });

    menu.context_menu_item("Delete")
        .on_click(|| actions.push(AssetsPanelAction::OpenDelete(item.path.clone())));

    menu.context_menu_separator();

    menu.context_menu_item("Copy File Path")
        .on_click(|| actions.push(AssetsPanelAction::CopyPath(item.path.clone())));

    menu.context_menu_item("Reveal in Explorer")
        .on_click(|| actions.push(AssetsPanelAction::RevealFolder(item.path.clone())));
}

/// Renders declarative context menu items for a directory into an active menu [`UiScope`].
pub fn render_folder_context_menu_items(
    menu: &mut UiScope<'_>,
    folder: &Path,
    actions: &mut Vec<AssetsPanelAction>,
) {
    menu.context_menu_item("New Subfolder").on_click(|| {
        actions.push(AssetsPanelAction::OpenCreateSubfolder(folder.to_path_buf()));
    });

    if folder != Path::new("assets") && !folder.as_os_str().is_empty() {
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