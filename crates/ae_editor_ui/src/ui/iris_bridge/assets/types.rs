// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type Definitions, Action Enums & Interaction Targets for Iris UI Content Browser.
//!
//! Provides data models, event actions, and layout bounding rectangles for the
//! 100% native Iris UI GPU SDF Asset Browser panel.

use crate::assets::types::{
    AssetBrowserState, AssetCategory, AssetItem, AssetSource, AssetViewMode,
};
use irisui::core::{MemoCache, UiTag};
use irisui::prelude::Point;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// State tracking for double-click asset spawning detection and 3D preview orbital dragging.
#[derive(Debug, Clone, Default)]
pub struct AssetClickTracker {
    /// Last clicked asset path.
    pub last_path: Option<PathBuf>,
    /// Instant of the previous click.
    pub last_instant: Option<Instant>,
    /// Whether the user is actively dragging the 3D model preview canvas.
    pub is_orbit_dragging: bool,
    /// Last cursor position recorded during orbit dragging.
    pub last_drag_pos: Option<Point>,
    /// Candidate asset item queued for potential drag and drop upon cursor move threshold.
    pub potential_drag_item: Option<AssetItem>,
    /// Initial screen coordinates where the candidate asset item was clicked.
    pub drag_start_pos: Option<Point>,
    /// Whether an asset item is actively being dragged across the editor.
    pub is_dragging_asset: bool,
}

/// High-level interaction actions emitted by the Iris UI Asset Browser panel.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetsPanelAction {
    /// Navigates the active folder path to the specified directory.
    NavigateFolder(PathBuf),
    /// Selects an asset item by path, or clears active selection.
    SelectAsset(Option<PathBuf>),
    /// Changes the active category filter chip.
    SelectCategory(AssetCategory),
    /// Switches between Grid and List view presentation modes.
    SetViewMode(AssetViewMode),
    /// Toggles the left folder tree sidebar collapsed state.
    ToggleSidebar,
    /// Updates the live search filter query string.
    SearchInput(String),
    /// Clears the active search filter query.
    ClearSearch,
    /// Requests keyboard focus for the search input box.
    FocusSearch,
    /// Triggers model import dialog request.
    OpenImportDialog,
    /// Reveals the active directory in the operating system's native file explorer.
    RevealFolder(PathBuf),
    /// Sweeps unreferenced GPU textures and models to free video memory.
    CleanVram,
    /// Requests creation of a new subfolder in the specified parent directory.
    OpenCreateSubfolder(PathBuf),
    /// Spawns the specified asset into the active 3D scene.
    SpawnAsset(PathBuf, AssetCategory),
    /// Opens the Quick Asset Inspector modal preview window for the item.
    InspectAsset(AssetItem),
    /// Adjusts vertical scroll offset in the primary content area.
    Scroll(f32),
    /// Adjusts vertical scroll offset in the left folder tree sidebar.
    TreeScroll(f32),
    /// Opens the context menu at the specified cursor position.
    OpenContextMenu(AssetsContextMenuTarget, Point),
    /// Closes the active context menu.
    CloseContextMenu,
    /// Opens the Quick Asset Preview modal for an item.
    OpenInspectModal(AssetItem),
    /// Closes the Quick Asset Preview modal.
    CloseInspectModal,
    /// In the Quick Asset Preview modal, applies orbit delta (yaw, pitch).
    InspectOrbitDelta(f32, f32),
    /// In the Quick Asset Preview modal, applies zoom delta.
    InspectZoomDelta(f32),
    /// Requests copying the file path to clipboard.
    CopyPath(PathBuf),
    /// Requests opening the Rename dialog for an asset or folder.
    OpenRename(PathBuf, String, bool),
    /// Requests opening the Delete confirmation dialog for an asset or folder.
    OpenDelete(PathBuf),
    /// Confirms permanent deletion of an asset or folder.
    ConfirmDelete(PathBuf),
    /// Confirms renaming of an asset or folder.
    ConfirmRename(PathBuf, String),
    /// Confirms creation of a new subfolder.
    ConfirmNewFolder(PathBuf, String),
    /// Initiates dragging an asset item towards the 3D viewport or editor panels via generic workbench payload.
    StartAssetDrag(irisui::workbench::DragDropPayload),
    /// Completes or cancels active asset dragging.
    EndAssetDrag,
    /// Toggles visibility of internal engine assets and built-in shaders.
    ToggleEngineContent,
    /// Ingests external files dropped from the host operating system into the active asset folder.
    ImportDroppedFiles(Vec<PathBuf>),
}

pub use super::context_menu::AssetsContextMenuTarget;

/// Deterministic semantic tag for the asset preview modal window state in [`irisui::core::UiTree`].
pub const ASSETS_TAG_PREVIEW_MODAL: u64 = UiTag::from_str("assets.preview.modal").raw();
/// Semantic interaction tag for the asset preview modal "Reveal in Explorer" button.
pub const ASSET_PREVIEW_TAG_REVEAL: u64 = UiTag::from_str("assets.preview.reveal").raw();
/// Semantic interaction tag for the asset preview 3D model orbit canvas.
pub const ASSET_PREVIEW_TAG_ORBIT: u64 = UiTag::from_str("assets.preview.orbit").raw();

// ============================================================================
// DETERMINISTIC FNV-1A SEMANTIC TAGS (ZERO HARDCODED HEX LITERALS)
// ============================================================================

/// Semantic tag for the primary Asset Browser panel root container.
pub const ASSETS_TAG_PANEL_ROOT: u64 = UiTag::from_str("assets.panel.root").raw();
/// Semantic tag for toggling the folder tree sidebar.
pub const ASSETS_TAG_TOGGLE_SIDEBAR: u64 = UiTag::from_str("assets.toggle_sidebar").raw();
/// Semantic tag for triggering model/texture import dialog.
pub const ASSETS_TAG_IMPORT: u64 = UiTag::from_str("assets.import").raw();
/// Semantic tag for sweeping unreferenced VRAM GPU assets.
pub const ASSETS_TAG_CLEAN_VRAM: u64 = UiTag::from_str("assets.clean_vram").raw();
/// Semantic tag for switching presentation to responsive grid card view.
pub const ASSETS_TAG_VIEW_GRID: u64 = UiTag::from_str("assets.view_grid").raw();
/// Semantic tag for switching presentation to detailed tabular list view.
pub const ASSETS_TAG_VIEW_LIST: u64 = UiTag::from_str("assets.view_list").raw();
/// Semantic tag for toggling visibility of built-in engine content.
pub const ASSETS_TAG_ENGINE_CONTENT: u64 = UiTag::from_str("assets.engine_content").raw();
/// Semantic tag for the live asset search text input box.
pub const ASSETS_TAG_SEARCH_INPUT: u64 = UiTag::from_str("assets.search.input").raw();
/// Semantic tag for clearing the active search filter query.
pub const ASSETS_TAG_SEARCH_CLEAR: u64 = UiTag::from_str("assets.search.clear").raw();
/// Semantic [`UiTag`] for the folder tree sidebar scrollable viewport.
pub const ASSETS_TAG_TREE_VIEWPORT_ID: UiTag = UiTag::from_str("assets.tree.viewport");
/// Semantic tag for the folder tree sidebar scrollable viewport.
pub const ASSETS_TAG_TREE_VIEWPORT: u64 = ASSETS_TAG_TREE_VIEWPORT_ID.raw();
/// Semantic tag for the primary assets scrollable content area viewport.
pub const ASSETS_TAG_CONTENT_VIEWPORT: u64 = UiTag::from_str("assets.content.viewport").raw();
/// Semantic tag for primary content scrollbar track.
pub const ASSETS_TAG_SCROLLBAR_TRACK: u64 = UiTag::from_str("assets.scrollbar.track").raw();
/// Semantic tag for primary content scrollbar draggable thumb.
pub const ASSETS_TAG_SCROLLBAR_THUMB: u64 = UiTag::from_str("assets.scrollbar.thumb").raw();
/// Semantic [`UiTag`] for folder tree sidebar scrollbar track.
pub const ASSETS_TAG_TREE_SCROLLBAR_TRACK_ID: UiTag =
    UiTag::from_str("assets.tree.scrollbar.track");
/// Semantic tag for folder tree sidebar scrollbar track.
pub const ASSETS_TAG_TREE_SCROLLBAR_TRACK: u64 = ASSETS_TAG_TREE_SCROLLBAR_TRACK_ID.raw();
/// Semantic [`UiTag`] for folder tree sidebar scrollbar draggable thumb.
pub const ASSETS_TAG_TREE_SCROLLBAR_THUMB_ID: UiTag =
    UiTag::from_str("assets.tree.scrollbar.thumb");
/// Semantic tag for folder tree sidebar scrollbar draggable thumb.
pub const ASSETS_TAG_TREE_SCROLLBAR_THUMB: u64 = ASSETS_TAG_TREE_SCROLLBAR_THUMB_ID.raw();
/// Semantic tag for creating a new subfolder in the folder tree sidebar.
pub const ASSETS_TAG_NEW_SUBFOLDER: u64 = UiTag::from_str("assets.new_subfolder").raw();
/// Semantic tag for revealing the active folder in the OS file explorer.
pub const ASSETS_TAG_REVEAL: u64 = UiTag::from_str("assets.reveal").raw();

const CHIP_BASE_TAG: UiTag = UiTag::from_str("assets.chip");
const BREADCRUMB_BASE_TAG: UiTag = UiTag::from_str("assets.breadcrumb");
/// Base [`UiTag`] indexed for folder tree rows.
pub const TREE_ROW_BASE_TAG: UiTag = UiTag::from_str("assets.tree.row");
/// Base [`UiTag`] indexed for folder tree foldout chevrons.
pub const TREE_CHEVRON_BASE_TAG: UiTag = UiTag::from_str("assets.tree.chevron");
const ITEM_BASE_TAG: UiTag = UiTag::from_str("assets.item.select");
const ITEM_SPAWN_BASE_TAG: UiTag = UiTag::from_str("assets.item.spawn");
const ITEM_INSPECT_BASE_TAG: UiTag = UiTag::from_str("assets.item.inspect");

/// Encodes a category filter chip index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_chip_tag(category_idx: u8) -> u64 {
    UiTag::indexed(CHIP_BASE_TAG, category_idx as u64).raw()
}

/// Encodes a breadcrumb segment index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_breadcrumb_tag(segment_idx: u8) -> u64 {
    UiTag::indexed(BREADCRUMB_BASE_TAG, segment_idx as u64).raw()
}

/// Encodes a folder tree row node index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_tree_row_tag(node_idx: u32) -> u64 {
    UiTag::indexed(TREE_ROW_BASE_TAG, node_idx as u64).raw()
}

/// Encodes a folder tree chevron button index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_tree_chevron_tag(node_idx: u32) -> u64 {
    UiTag::indexed(TREE_CHEVRON_BASE_TAG, node_idx as u64).raw()
}

/// Encodes an asset card or row selection target index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_item_tag(item_idx: u32) -> u64 {
    UiTag::indexed(ITEM_BASE_TAG, item_idx as u64).raw()
}

/// Encodes a list table row direct Spawn button index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_item_spawn_tag(item_idx: u32) -> u64 {
    UiTag::indexed(ITEM_SPAWN_BASE_TAG, item_idx as u64).raw()
}

/// Encodes a list table row direct Inspect button index into a deterministic 64-bit semantic tag.
#[inline]
#[must_use]
pub const fn encode_item_inspect_tag(item_idx: u32) -> u64 {
    UiTag::indexed(ITEM_INSPECT_BASE_TAG, item_idx as u64).raw()
}

/// Dynamic runtime state parameters for the 3D Quick Asset Preview orbital camera.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetPreviewModalState {
    /// Inspected asset metadata.
    pub item: AssetItem,
    /// Orbital yaw angle in radians for 3D model preview.
    pub orbit_yaw: f32,
    /// Orbital pitch angle in radians for 3D model preview.
    pub orbit_pitch: f32,
    /// Camera distance zoom factor for 3D model preview.
    pub zoom_distance: f32,
    /// Whether 3D wireframe edges should be drawn.
    pub show_wireframe: bool,
}

impl Default for AssetPreviewModalState {
    fn default() -> Self {
        Self {
            item: AssetItem {
                name: String::new(),
                path: PathBuf::new(),
                relative_path: String::new(),
                category: AssetCategory::All,
                source: AssetSource::Project,
                file_size_bytes: 0,
                metadata_badge: String::new(),
                is_loaded_in_memory: false,
                model_handle: None,
                texture_handle: None,
                shader_handle: None,
                is_3d: false,
            },
            orbit_yaw: 0.0,
            orbit_pitch: 0.3,
            zoom_distance: 1.0,
            show_wireframe: true,
        }
    }
}

/// Height of the top navigation header toolbar in physical pixels.
pub const ASSETS_TOP_BAR_HEIGHT: f32 = 34.0;
/// Height of the category filter chips toolbar in physical pixels.
pub const ASSETS_CHIPS_BAR_HEIGHT: f32 = 28.0;
/// Height of the bottom telemetry status footer in physical pixels.
pub const ASSETS_FOOTER_HEIGHT: f32 = 24.0;

/// Truncates a UTF-8 string safely at Unicode code point boundaries via `irisui::text`.
pub fn truncate_display_name(text: &str, max_chars: usize, keep_chars: usize) -> String {
    irisui::text::truncate_with_ellipsis(text, max_chars, keep_chars)
}

/// Key snapshot used by [`AssetsPanelState::filtered_cache`] to detect asset view changes
/// without allocating heap keys on cache hits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AssetsFilterKey {
    pub current_folder: PathBuf,
    pub selected_asset: Option<PathBuf>,
    pub search_query: String,
    pub active_category: AssetCategory,
    pub view_mode: AssetViewMode,
    pub sidebar_collapsed: bool,
    pub is_2d_mode: bool,
    pub show_engine_content: bool,
    pub cached_items_len: usize,
}

impl AssetsFilterKey {
    #[must_use]
    pub(crate) fn matches(
        &self,
        asset_browser: &AssetBrowserState,
        query: &str,
        is_2d_mode: bool,
    ) -> bool {
        self.current_folder == asset_browser.current_folder
            && self.selected_asset == asset_browser.selected_asset
            && self.search_query == query
            && self.active_category == asset_browser.active_category
            && self.view_mode == asset_browser.view_mode
            && self.sidebar_collapsed == asset_browser.sidebar_collapsed
            && self.is_2d_mode == is_2d_mode
            && self.show_engine_content == asset_browser.show_engine_content
            && self.cached_items_len == asset_browser.cached_items.len()
    }

    #[must_use]
    pub(crate) fn capture(
        asset_browser: &AssetBrowserState,
        query: &str,
        is_2d_mode: bool,
    ) -> Self {
        Self {
            current_folder: asset_browser.current_folder.clone(),
            selected_asset: asset_browser.selected_asset.clone(),
            search_query: query.to_string(),
            active_category: asset_browser.active_category,
            view_mode: asset_browser.view_mode,
            sidebar_collapsed: asset_browser.sidebar_collapsed,
            is_2d_mode,
            show_engine_content: asset_browser.show_engine_content,
            cached_items_len: asset_browser.cached_items.len(),
        }
    }
}

/// Persistent interactive state for the Asset Browser panel overlay.
#[derive(Debug, Default)]
pub struct AssetsPanelState {
    /// Search input text state managing asset filter buffer, cursor, and selection.
    pub search_input: irisui::prelude::TextInputState,
    /// Double-click tracking state for asset spawning.
    pub click_tracker: AssetClickTracker,
    /// Active Quick Asset Preview modal state.
    pub preview_modal: Option<AssetPreviewModalState>,
    /// Dynamic thumbnail layer cache mapping asset paths to managed 2D Texture Array layers.
    pub thumbnail_layers: HashMap<PathBuf, u32>,
    /// Library-managed memoized cache storing filtered assets and key snapshot.
    pub(crate) filtered_cache: MemoCache<AssetsFilterKey, Vec<AssetItem>>,
    /// Reusable action scratch buffer avoiding per-frame allocations during panel builds.
    pub(crate) action_scratch: Vec<AssetsPanelAction>,
}

/// Semantic alias for [`AssetsPanelState`].
pub type AssetsState = AssetsPanelState;