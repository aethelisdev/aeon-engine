// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser UI Panel Module.
//!
//! Provides inspection of loaded 3D meshes, 2D textures, and WGSL shaders,
//! category filtering, responsive card grid views, and VRAM garbage collection.
//!

pub mod browser;
pub mod context_menu;
pub mod grid_view;
pub mod list_view;
pub mod scanner;
pub mod toolbar;
pub mod types;

pub use types::{AssetBrowserState, AssetCategory, AssetItem, AssetViewMode};