// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Data Models & State Containers.
//!
//! Provides category classifications, card/list item models,
//! drag-and-drop payloads, inline rename state, and headless browser navigation state.

use ae_renderer::asset::AssetHandle;
use std::path::PathBuf;
use std::time::Instant;

/// Asset categories for top-level filter chips and asset routing.
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
    /// All asset categories in canonical display order for zero-allocation indexed access.
    pub const ALL: [AssetCategory; 7] = [
        AssetCategory::All,
        AssetCategory::Models3D,
        AssetCategory::Textures2D,
        AssetCategory::Shaders,
        AssetCategory::Scenes,
        AssetCategory::Materials,
        AssetCategory::Audio,
    ];

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
}

/// Presentation mode for the content browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetViewMode {
    #[default]
    Grid,
    List,
}

/// Source origin domain of an asset (user project vs engine/editor internal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetSource {
    /// User project asset located under workspace project directories (e.g. `assets/`).
    #[default]
    Project,
    /// Internal engine or editor asset (e.g. built-in shaders, editor HUD atlases).
    Engine,
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
    /// Source origin domain (Project vs Engine).
    pub source: AssetSource,
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
    /// Whether this asset represents 3D-specific content (e.g. 3D mesh or 3D scene).
    pub is_3d: bool,
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
    /// Whether internal engine assets and built-in shaders are displayed in the browser.
    pub show_engine_content: bool,
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
            show_engine_content: false,
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

    /// Returns filtered asset items matching active folder, category, search query, and dimension mode.
    #[must_use]
    pub fn filter_items(&self, search_query: &str, is_2d_mode: bool) -> Vec<AssetItem> {
        let trimmed = search_query.trim();
        let query_lower = trimmed.to_ascii_lowercase();
        let is_root_folder = self.current_folder == std::path::Path::new("assets");

        self.cached_items
            .iter()
            .filter(|item| self.matches_item_filter(item, is_2d_mode, is_root_folder, &query_lower))
            .cloned()
            .collect()
    }

    fn matches_item_filter(
        &self,
        item: &AssetItem,
        is_2d_mode: bool,
        is_root_folder: bool,
        query_lower: &str,
    ) -> bool {
        if !self.show_engine_content && item.source == AssetSource::Engine {
            return false;
        }
        if is_2d_mode && item.is_3d {
            return false;
        }
        if !is_2d_mode && !item.is_3d && item.category == AssetCategory::Scenes {
            return false;
        }
        if !is_root_folder && !item.path.starts_with(&self.current_folder) {
            return false;
        }
        if self.active_category != AssetCategory::All && item.category != self.active_category {
            return false;
        }
        query_lower.is_empty()
            || item.name.to_ascii_lowercase().contains(query_lower)
            || item
                .relative_path
                .to_ascii_lowercase()
                .contains(query_lower)
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
    fn test_asset_source_defaults() {
        assert_eq!(AssetSource::default(), AssetSource::Project);
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
        assert!(!state.show_engine_content);
    }

    #[test]
    fn test_asset_browser_filter_items() {
        let mut state = AssetBrowserState::new();
        state.cached_items = vec![
            AssetItem {
                path: PathBuf::from("assets/models/cube.gltf"),
                relative_path: "assets/models/cube.gltf".to_string(),
                name: "cube.gltf".to_string(),
                category: AssetCategory::Models3D,
                source: AssetSource::Project,
                file_size_bytes: 128,
                metadata_badge: "128 B".to_string(),
                is_loaded_in_memory: false,
                model_handle: None,
                texture_handle: None,
                shader_handle: None,
                is_3d: true,
            },
            AssetItem {
                path: PathBuf::from("crates/ae_renderer/src/shaders/sky.wgsl"),
                relative_path: "crates/ae_renderer/src/shaders/sky.wgsl".to_string(),
                name: "sky.wgsl".to_string(),
                category: AssetCategory::Shaders,
                source: AssetSource::Engine,
                file_size_bytes: 256,
                metadata_badge: "WGSL".to_string(),
                is_loaded_in_memory: false,
                model_handle: None,
                texture_handle: None,
                shader_handle: None,
                is_3d: false,
            },
        ];

        assert_eq!(state.filter_items("", false).len(), 1);
        assert!(state.filter_items("", true).is_empty());
        state.show_engine_content = true;
        assert_eq!(state.filter_items("sky", false).len(), 1);
    }
}