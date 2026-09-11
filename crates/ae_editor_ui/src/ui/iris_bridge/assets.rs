// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Native Iris UI Asset / Content Browser Subsystem
//!
//! Provides a 100% GPU SDF-accelerated Asset Browser panel,
//! completely free of emojis, featuring breadcrumb navigation, live search,
//! canonical vector icons, folder tree sidebar, floating right-click context menus,
//! interactive quick asset preview modal, and responsive grid/table views.
//!

pub mod cards;
pub mod context_menu;
pub mod events;
pub mod list;
pub mod panel;
pub mod preview;
pub mod sync;
#[cfg(test)]
mod tests;
pub mod tree;
pub mod types;

pub use events::{
    AssetClickTracker, AssetsEventContext, handle_assets_click, handle_assets_panel_event,
    handle_assets_right_click, handle_assets_scroll,
};
pub use panel::{build_assets_panel, build_assets_panel_retained};
pub use sync::sync_assets_panel;
pub use types::{
    AssetBrowserRetainedState, AssetCardTarget, AssetPreviewModalState, AssetPreviewModalTargets,
    AssetRowTarget, AssetsContextMenuTarget, AssetsContextMenuTargets, AssetsPanelAction,
    AssetsPanelParams, AssetsPanelTargets, BreadcrumbTarget, FolderTreeNodeTarget,
    RetainedAssetCard,
};