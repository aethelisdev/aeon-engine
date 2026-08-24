// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser UI Panel Module.
//!
//! Provides hierarchical directory tree navigation, responsive card grid views,
//! structured list views, safe file operations, thumbnail generation,
//! drag-and-drop viewport spawning, and interactive asset inspection.
//!

pub mod browser;
pub mod context_menu;
pub mod drag_drop;
pub mod file_ops;
pub mod folder_tree;
pub mod grid_view;
pub mod list_view;
pub mod preview_modal;
pub mod scanner;
pub mod thumbnails;
pub mod toolbar;
pub mod types;

pub use types::{
    AssetBrowserState, AssetCategory, AssetDragPayload, AssetItem, AssetViewMode,
    PreviewModalState, RenamingState,
};