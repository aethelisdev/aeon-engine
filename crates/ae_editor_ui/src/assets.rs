// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor Asset Management and Backend Subsystem.
//!
//! Provides disk scanning, asset categorization, thumbnail texture synthesis,
//! raycast math for drag-and-drop viewport instantiation, and secure file operations.
//!

pub mod drag_drop;
pub mod file_ops;
pub mod scanner;
pub mod thumbnails;
pub mod types;

pub use types::{
    AssetBrowserState, AssetCategory, AssetDragPayload, AssetItem, AssetSource, AssetViewMode,
    RenamingState,
};