// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Data Models & State Containers.
//!
//! Provides category classifications, card/list item view models,
//! and persistent browser navigation state.
//!

use ae_renderer::asset::AssetHandle;
use std::path::PathBuf;
use std::time::Instant;

/// Asset categories for top-level filter chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetCategory {
    #[default]
    All,
    Models3D,
    Textures2D,
    Shaders,
    Materials,
    Scenes,
    Audio,
}

impl AssetCategory {
    /// Human-readable label with category icon.
    pub fn label(self) -> &'static str {
        match self {
            AssetCategory::All => "All Assets",
            AssetCategory::Models3D => "📦 3D Meshes",
            AssetCategory::Textures2D => "🖼 Textures",
            AssetCategory::Shaders => "⚡ Shaders",
            AssetCategory::Materials => "🎨 Materials",
            AssetCategory::Scenes => "🎬 Scenes",
            AssetCategory::Audio => "🔊 Audio",
        }
    }

    /// Short badge identifier.
    pub fn badge(self) -> &'static str {
        match self {
            AssetCategory::All => "ALL",
            AssetCategory::Models3D => "3D",
            AssetCategory::Textures2D => "TEX",
            AssetCategory::Shaders => "WGSL",
            AssetCategory::Materials => "MAT",
            AssetCategory::Scenes => "SCENE",
            AssetCategory::Audio => "AUD",
        }
    }

    /// Primary accent color for UI category badges.
    pub fn badge_color(self) -> egui::Color32 {
        match self {
            AssetCategory::All => egui::Color32::from_rgb(180, 180, 190),
            AssetCategory::Models3D => egui::Color32::from_rgb(0, 229, 255), // Aeon Cyan
            AssetCategory::Textures2D => egui::Color32::from_rgb(100, 220, 120), // Green
            AssetCategory::Shaders => egui::Color32::from_rgb(255, 190, 60), // Amber / Yellow
            AssetCategory::Materials => egui::Color32::from_rgb(220, 100, 220), // Magenta
            AssetCategory::Scenes => egui::Color32::from_rgb(80, 160, 255),  // Sky Blue
            AssetCategory::Audio => egui::Color32::from_rgb(255, 120, 100),  // Coral
        }
    }
}

/// Presentation mode for the content browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetViewMode {
    #[default]
    Grid,
    List,
}

/// Metadata item model representing an individual asset on disk or in GPU memory.
#[derive(Debug, Clone)]
pub struct AssetItem {
    /// File stem or display name.
    pub name: String,
    /// Absolute or workspace-relative path.
    pub path: PathBuf,
    /// Relative display path from `assets/`.
    pub relative_path: String,
    /// Classified category.
    pub category: AssetCategory,
    /// File size in bytes.
    pub file_size_bytes: u64,
    /// Formatted metadata string (e.g. "24.5k Verts", "2048x2048", "15.9 KB").
    pub metadata_badge: String,
    /// Whether this asset is currently resident in GPU / CPU memory.
    pub is_loaded_in_memory: bool,
    /// Associated model asset handle if loaded.
    pub model_handle: Option<AssetHandle>,
    /// Associated texture asset handle if loaded.
    pub texture_handle: Option<AssetHandle>,
    /// Associated shader asset handle if loaded.
    pub shader_handle: Option<AssetHandle>,
}

/// Central state manager for the Asset / Content Browser panel.
pub struct AssetBrowserState {
    /// Current folder path for breadcrumb navigation.
    pub current_folder: PathBuf,
    /// Live search filter query.
    pub search_query: String,
    /// Active category filter.
    pub active_category: AssetCategory,
    /// Grid vs List display mode.
    pub view_mode: AssetViewMode,
    /// Currently selected asset path for context operations.
    pub selected_asset: Option<PathBuf>,
    /// Cached list of discovered disk and memory assets.
    pub cached_items: Vec<AssetItem>,
    /// Timestamp of last directory sweep.
    pub last_scan_time: Instant,
    /// Discovered subfolder tree under `assets/`.
    pub subfolders: Vec<PathBuf>,
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetBrowserState {
    /// Creates a new default asset browser state.
    pub fn new() -> Self {
        Self {
            current_folder: PathBuf::from("assets"),
            search_query: String::new(),
            active_category: AssetCategory::All,
            view_mode: AssetViewMode::Grid,
            selected_asset: None,
            cached_items: Vec::new(),
            last_scan_time: Instant::now()
                .checked_sub(std::time::Duration::from_secs(60))
                .unwrap_or_else(Instant::now),
            subfolders: Vec::new(),
        }
    }

    /// Formats human-readable file size string (KB, MB).
    pub fn format_file_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}