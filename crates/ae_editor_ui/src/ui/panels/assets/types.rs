// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Data Models & State Containers.
//!
//! Provides category classifications, card/list item view models,
//! drag-and-drop payloads, inspection modal states, and persistent browser navigation state.
//!

use ae_renderer::asset::AssetHandle;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

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
#[derive(Debug, Clone, PartialEq)]
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

/// Active drag-and-drop payload when dragging an asset card or list row.
#[derive(Debug, Clone)]
pub struct AssetDragPayload {
    /// Path to the dragged asset on disk.
    pub path: PathBuf,
    /// Display name of the asset.
    pub name: String,
    /// Asset category classification.
    pub category: AssetCategory,
    /// Optional in-memory model handle.
    pub model_handle: Option<AssetHandle>,
    /// Optional in-memory texture handle.
    pub texture_handle: Option<AssetHandle>,
}

/// In-progress inline renaming state for assets or folders.
#[derive(Debug, Clone)]
pub struct RenamingState {
    /// Target file or directory path.
    pub target_path: PathBuf,
    /// Active text edit buffer for the new name.
    pub current_name: String,
    /// Whether the target is a directory.
    pub is_folder: bool,
}

/// State container for the interactive quick asset inspection modal window.
#[derive(Debug, Clone)]
pub struct PreviewModalState {
    /// Inspected asset item reference.
    pub item: AssetItem,
    /// 3D model orbit rotation yaw angle in radians.
    pub orbit_yaw: f32,
    /// 3D model orbit rotation pitch angle in radians.
    pub orbit_pitch: f32,
    /// 3D model camera zoom distance multiplier.
    pub zoom_distance: f32,
    /// Whether to render the 3D model in wireframe mode.
    pub show_wireframe: bool,
    /// RGBA channel visibility toggles [R, G, B, A] for texture inspection.
    pub channel_mask: [bool; 4],
    /// Cached WGSL shader code content for syntax inspection.
    pub wgsl_source: Option<String>,
}

/// Memory-cached texture preview entry.
#[derive(Clone)]
pub struct ThumbnailEntry {
    /// Registered egui texture handle.
    pub texture_handle: egui::TextureHandle,
    /// Last recorded file modification timestamp.
    pub last_modified: SystemTime,
}

/// Dynamic cache holding downscaled texture and model preview thumbnails.
#[derive(Default)]
pub struct ThumbnailCache {
    /// Map of canonical file path to cached egui texture preview.
    pub entries: HashMap<PathBuf, ThumbnailEntry>,
}

/// Central state manager for the Asset / Content Browser panel.
pub struct AssetBrowserState {
    /// Current folder path for breadcrumb and tree navigation.
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
    /// Sidebar folder tree width in pixels.
    pub sidebar_width: f32,
    /// Whether the folder tree sidebar is currently collapsed.
    pub sidebar_collapsed: bool,
    /// Active drag-and-drop payload if an asset is being dragged.
    pub drag_payload: Option<AssetDragPayload>,
    /// Inline renaming state if an asset or folder is being renamed.
    pub rename_state: Option<RenamingState>,
    /// Target path pending deletion confirmation.
    pub delete_confirmation: Option<PathBuf>,
    /// Active quick asset inspection modal window state.
    pub preview_modal: Option<PreviewModalState>,
    /// Active new folder creation parent path if dialog is open.
    pub new_folder_parent: Option<PathBuf>,
    /// Input buffer for newly created folder names.
    pub new_folder_name: String,
    /// In-memory thumbnail preview cache.
    pub thumbnail_cache: ThumbnailCache,
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
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            drag_payload: None,
            rename_state: None,
            delete_confirmation: None,
            preview_modal: None,
            new_folder_parent: None,
            new_folder_name: String::new(),
            thumbnail_cache: ThumbnailCache::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_categories() {
        assert_eq!(AssetCategory::All.badge(), "ALL");
        assert_eq!(AssetCategory::Models3D.badge(), "3D");
        assert_eq!(AssetCategory::Textures2D.badge(), "TEX");
        assert_eq!(AssetCategory::Shaders.badge(), "WGSL");
        assert_eq!(AssetCategory::Materials.badge(), "MAT");
        assert_eq!(AssetCategory::Scenes.badge(), "SCENE");
        assert_eq!(AssetCategory::Audio.badge(), "AUD");
    }

    #[test]
    fn test_asset_browser_state_defaults() {
        let state = AssetBrowserState::new();
        assert_eq!(state.current_folder, PathBuf::from("assets"));
        assert_eq!(state.active_category, AssetCategory::All);
        assert_eq!(state.view_mode, AssetViewMode::Grid);
        assert!(!state.sidebar_collapsed);
        assert_eq!(state.sidebar_width, 180.0);
        assert!(state.drag_payload.is_none());
        assert!(state.preview_modal.is_none());
    }
}