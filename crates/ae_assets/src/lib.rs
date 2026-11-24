// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Aeon Engine Asset Subsystem — headless content discovery, disk operations,
//! external asset ingestion, CPU thumbnail rasterization, and background parsing pipelines.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(clippy::allow_attributes)]

pub mod file_ops;
pub mod ingestion;
pub mod pipeline;
pub mod scanner;
pub mod scene_assets;
pub mod thumbnails;
pub mod types;

pub use ingestion::import_asset_path;
pub use pipeline::{
    AsyncImportResult, ImportAssetKind, resolve_fbx2gltf_tool_path, spawn_fbx_import,
    spawn_gltf_import, validate_asset_drop_for_dimension,
};
pub use scene_assets::{
    ParsedTextureEntry, ParsedTextureItem, PendingSceneData, SceneAssetParseRequest,
    collect_scene_asset_paths, spawn_scene_asset_parse,
};
pub use types::{
    AssetBrowserState, AssetCategory, AssetDragPayload, AssetItem, AssetSource, AssetViewMode,
    RenamingState,
};