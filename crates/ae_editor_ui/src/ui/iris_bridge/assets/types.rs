// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type Definitions, Action Enums & Interaction Targets for Iris UI Content Browser.
//!
//! Provides data models, event actions, and layout bounding rectangles for the
//! 100% native Iris UI GPU SDF Asset Browser panel.
//!

use super::events::AssetClickTracker;
use crate::assets::types::{AssetCategory, AssetItem, AssetSource, AssetViewMode};
use irisui::prelude::{Point, Rect};
use std::collections::HashMap;
use std::path::PathBuf;

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
    /// Updates keyboard focus state for the search input box.
    FocusSearch(bool),
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
    /// Initiates dragging an asset item towards the 3D viewport or editor panels.
    StartAssetDrag(AssetItem),
    /// Completes or cancels active asset dragging.
    EndAssetDrag,
    /// Toggles visibility of internal engine assets and built-in shaders.
    ToggleEngineContent,
}

/// Target subject of an active asset browser context menu.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetsContextMenuTarget {
    /// Context menu opened on an asset item (card or list row).
    Asset(AssetItem),
    /// Context menu opened on a folder node or blank area.
    Folder(PathBuf),
}

/// Numerical tags assigned to items within the Asset Browser right-click context menu.
pub const ASSET_CTX_INSPECT: u64 = 0;
/// Numerical tag for spawning an asset into the active scene.
pub const ASSET_CTX_SPAWN: u64 = 1;
/// Numerical tag for creating a new subfolder.
pub const ASSET_CTX_NEW_FOLDER: u64 = 2;
/// Numerical tag for renaming an asset or folder.
pub const ASSET_CTX_RENAME: u64 = 3;
/// Numerical tag for deleting an asset or folder.
pub const ASSET_CTX_DELETE: u64 = 4;
/// Numerical tag for copying an asset's file path to the system clipboard.
pub const ASSET_CTX_COPY_PATH: u64 = 5;
/// Numerical tag for revealing an asset or folder in the OS file manager.
pub const ASSET_CTX_REVEAL: u64 = 6;

/// Semantic interaction tag for the asset preview modal "Reveal in Explorer" button.
pub const ASSET_PREVIEW_TAG_REVEAL: u64 = 0xF010;
/// Semantic interaction tag for the asset preview 3D model orbit canvas.
pub const ASSET_PREVIEW_TAG_ORBIT: u64 = 0xF011;

/// Actions selectable from the Asset Browser right-click context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetsContextMenuAction {
    /// Opens the 3D orbital inspection preview modal for the asset.
    Inspect,
    /// Instantiates the asset into the active 3D scene.
    Spawn,
    /// Prompts creation of a new subfolder in the directory.
    NewFolder,
    /// Opens the inline renaming modal or text prompt.
    Rename,
    /// Prompts deletion confirmation for the file or folder.
    Delete,
    /// Copies the file system path to the OS clipboard.
    CopyPath,
    /// Reveals the file or directory in the native OS desktop file manager.
    Reveal,
}

/// Resolves a context menu numeric tag into a typed [`AssetsContextMenuAction`].
#[inline]
pub fn resolve_assets_ctx_action(tag: u64) -> Option<AssetsContextMenuAction> {
    match tag {
        ASSET_CTX_INSPECT => Some(AssetsContextMenuAction::Inspect),
        ASSET_CTX_SPAWN => Some(AssetsContextMenuAction::Spawn),
        ASSET_CTX_NEW_FOLDER => Some(AssetsContextMenuAction::NewFolder),
        ASSET_CTX_RENAME => Some(AssetsContextMenuAction::Rename),
        ASSET_CTX_DELETE => Some(AssetsContextMenuAction::Delete),
        ASSET_CTX_COPY_PATH => Some(AssetsContextMenuAction::CopyPath),
        ASSET_CTX_REVEAL => Some(AssetsContextMenuAction::Reveal),
        _ => None,
    }
}

// ============================================================================
// 64-BIT DOMAIN SEMANTIC TAGS & BITMASKING RULES (O(1) HIT TESTING)
// ============================================================================

/// 64-bit Domain identifier for native Iris UI Content / Asset Browser panel elements.
pub const ASSETS_TAG_DOMAIN: u64 = 0x0090_0000_0000_0000;
/// Bitmask isolating the 16-bit Asset Browser domain prefix.
pub const ASSETS_TAG_DOMAIN_MASK: u64 = 0xFFFF_0000_0000_0000;

// --- Primary Panel Controls ---
/// Semantic tag for the primary Asset Browser panel root container.
pub const ASSETS_TAG_PANEL_ROOT: u64 = ASSETS_TAG_DOMAIN | 0x0001;
/// Semantic tag for toggling the folder tree sidebar.
pub const ASSETS_TAG_TOGGLE_SIDEBAR: u64 = ASSETS_TAG_DOMAIN | 0x0002;
/// Semantic tag for triggering model/texture import dialog.
pub const ASSETS_TAG_IMPORT: u64 = ASSETS_TAG_DOMAIN | 0x0003;
/// Semantic tag for sweeping unreferenced VRAM GPU assets.
pub const ASSETS_TAG_CLEAN_VRAM: u64 = ASSETS_TAG_DOMAIN | 0x0004;
/// Semantic tag for switching presentation to responsive grid card view.
pub const ASSETS_TAG_VIEW_GRID: u64 = ASSETS_TAG_DOMAIN | 0x0005;
/// Semantic tag for switching presentation to detailed tabular list view.
pub const ASSETS_TAG_VIEW_LIST: u64 = ASSETS_TAG_DOMAIN | 0x0006;
/// Semantic tag for toggling visibility of built-in engine content.
pub const ASSETS_TAG_ENGINE_CONTENT: u64 = ASSETS_TAG_DOMAIN | 0x0007;
/// Semantic tag for the live asset search text input box.
pub const ASSETS_TAG_SEARCH_INPUT: u64 = ASSETS_TAG_DOMAIN | 0x0008;
/// Semantic tag for clearing the active search filter query.
pub const ASSETS_TAG_SEARCH_CLEAR: u64 = ASSETS_TAG_DOMAIN | 0x0009;
/// Semantic tag for the primary assets scrollable content area viewport.
pub const ASSETS_TAG_CONTENT_VIEWPORT: u64 = ASSETS_TAG_DOMAIN | 0x000E;
/// Semantic tag for primary content scrollbar track.
pub const ASSETS_TAG_SCROLLBAR_TRACK: u64 = ASSETS_TAG_DOMAIN | 0x000A;
/// Semantic tag for primary content scrollbar draggable thumb.
pub const ASSETS_TAG_SCROLLBAR_THUMB: u64 = ASSETS_TAG_DOMAIN | 0x000B;
/// Semantic tag for folder tree sidebar scrollbar track.
pub const ASSETS_TAG_TREE_SCROLLBAR_TRACK: u64 = ASSETS_TAG_DOMAIN | 0x000C;
/// Semantic tag for folder tree sidebar scrollbar draggable thumb.
pub const ASSETS_TAG_TREE_SCROLLBAR_THUMB: u64 = ASSETS_TAG_DOMAIN | 0x000D;
/// Semantic tag for creating a new subfolder in the folder tree sidebar.
pub const ASSETS_TAG_NEW_SUBFOLDER: u64 = ASSETS_TAG_DOMAIN | 0x000E;
/// Semantic tag for revealing the active folder in the OS file explorer.
pub const ASSETS_TAG_REVEAL: u64 = ASSETS_TAG_DOMAIN | 0x000F;

// --- Modal & Floating Preview Controls ---
/// Semantic tag for closing the quick asset preview modal.
pub const ASSETS_TAG_PREVIEW_CLOSE: u64 = ASSETS_TAG_DOMAIN | 0x0010;
/// Semantic tag for revealing previewed asset in the OS file explorer.
pub const ASSETS_TAG_PREVIEW_REVEAL_BTN: u64 = ASSETS_TAG_DOMAIN | 0x0011;
/// Semantic tag for the 3D asset orbital camera canvas.
pub const ASSETS_TAG_PREVIEW_ORBIT_CANVAS: u64 = ASSETS_TAG_DOMAIN | 0x0012;
/// Semantic tag for toggling 3D preview wireframe edges.
pub const ASSETS_TAG_PREVIEW_WIREFRAME_BTN: u64 = ASSETS_TAG_DOMAIN | 0x0013;

// --- Dynamic Sub-Domains (Bitmasked Payloads) ---
/// Base prefix for category filter chips (payload: category index `0..8`).
pub const ASSETS_TAG_CHIP_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0000_0100;
/// Bitmask isolating category filter chip payload.
pub const ASSETS_TAG_CHIP_MASK: u64 = 0xFFFF_FFFF_FFFF_FF00;

/// Base prefix for breadcrumb navigation segments (payload: segment index `0..255`).
pub const ASSETS_TAG_BREADCRUMB_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0000_0200;
/// Bitmask isolating breadcrumb segment payload.
pub const ASSETS_TAG_BREADCRUMB_MASK: u64 = 0xFFFF_FFFF_FFFF_FF00;

/// Base prefix for context menu actions (payload: action index `0..15`).
pub const ASSETS_TAG_CTX_ITEM_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0000_0300;
/// Bitmask isolating context menu action payload.
pub const ASSETS_TAG_CTX_ITEM_MASK: u64 = 0xFFFF_FFFF_FFFF_FF00;

/// Base prefix for folder tree directory rows (payload: node index `0..u32::MAX`).
pub const ASSETS_TAG_TREE_ROW_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0001_0000_0000;
/// Base prefix for folder tree expand/collapse chevron buttons.
pub const ASSETS_TAG_TREE_CHEVRON_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0002_0000_0000;
/// Bitmask isolating folder tree category prefix.
pub const ASSETS_TAG_TREE_PREFIX_MASK: u64 = 0xFFFF_FFFF_0000_0000;
/// Bitmask isolating 32-bit payload index.
pub const ASSETS_TAG_PAYLOAD_MASK: u64 = 0x0000_0000_FFFF_FFFF;

/// Base prefix for asset items in grid/list (primary card selection, double click, drag).
pub const ASSETS_TAG_ITEM_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0010_0000_0000;
/// Base prefix for table row direct Spawn button.
pub const ASSETS_TAG_ITEM_SPAWN_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0020_0000_0000;
/// Base prefix for table row direct Inspect button.
pub const ASSETS_TAG_ITEM_INSPECT_BASE: u64 = ASSETS_TAG_DOMAIN | 0x0000_0030_0000_0000;
/// Bitmask isolating item action type prefix.
pub const ASSETS_TAG_ITEM_PREFIX_MASK: u64 = 0xFFFF_FFF0_0000_0000;

/// Discriminator for interaction targets within an individual asset card or row.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AssetItemAction {
    /// Selects the asset, or opens it on double-click.
    SelectOrOpen,
    /// Spawns the asset into the active 3D scene immediately.
    Spawn,
    /// Opens the Quick Asset Inspector orbital preview window.
    Inspect,
}

/// Encodes a category filter chip index into a 64-bit semantic tag.
#[inline]
pub fn encode_chip_tag(category_idx: u8) -> u64 {
    ASSETS_TAG_CHIP_BASE | (category_idx as u64)
}

/// Decodes a 64-bit semantic tag into a category filter chip index, if matching.
#[inline]
pub fn parse_chip_tag(tag: u64) -> Option<u8> {
    if (tag & ASSETS_TAG_CHIP_MASK) == ASSETS_TAG_CHIP_BASE {
        Some((tag & 0xFF) as u8)
    } else {
        None
    }
}

/// Encodes a breadcrumb segment index into a 64-bit semantic tag.
#[inline]
pub fn encode_breadcrumb_tag(segment_idx: u8) -> u64 {
    ASSETS_TAG_BREADCRUMB_BASE | (segment_idx as u64)
}

/// Decodes a 64-bit semantic tag into a breadcrumb segment index, if matching.
#[inline]
pub fn parse_breadcrumb_tag(tag: u64) -> Option<u8> {
    if (tag & ASSETS_TAG_BREADCRUMB_MASK) == ASSETS_TAG_BREADCRUMB_BASE {
        Some((tag & 0xFF) as u8)
    } else {
        None
    }
}

/// Encodes a context menu action index into a 64-bit semantic tag.
#[inline]
pub fn encode_ctx_item_tag(action_idx: u8) -> u64 {
    ASSETS_TAG_CTX_ITEM_BASE | (action_idx as u64)
}

/// Decodes a 64-bit semantic tag into a context menu action index, if matching.
#[inline]
pub fn parse_ctx_item_tag(tag: u64) -> Option<u8> {
    if (tag & ASSETS_TAG_CTX_ITEM_MASK) == ASSETS_TAG_CTX_ITEM_BASE {
        Some((tag & 0xFF) as u8)
    } else {
        None
    }
}

/// Encodes a folder tree row node index into a 64-bit semantic tag.
#[inline]
pub fn encode_tree_row_tag(node_idx: u32) -> u64 {
    ASSETS_TAG_TREE_ROW_BASE | (node_idx as u64)
}

/// Encodes a folder tree chevron button index into a 64-bit semantic tag.
#[inline]
pub fn encode_tree_chevron_tag(node_idx: u32) -> u64 {
    ASSETS_TAG_TREE_CHEVRON_BASE | (node_idx as u64)
}

/// Decodes a 64-bit semantic tag into a folder tree node index and chevron flag, if matching.
///
/// Returns `Some((node_idx, is_chevron))`.
#[inline]
pub fn parse_tree_tag(tag: u64) -> Option<(u32, bool)> {
    let prefix = tag & ASSETS_TAG_TREE_PREFIX_MASK;
    let idx = (tag & ASSETS_TAG_PAYLOAD_MASK) as u32;
    if prefix == ASSETS_TAG_TREE_ROW_BASE {
        Some((idx, false))
    } else if prefix == ASSETS_TAG_TREE_CHEVRON_BASE {
        Some((idx, true))
    } else {
        None
    }
}

/// Encodes an asset card or row selection target index into a 64-bit semantic tag.
#[inline]
pub fn encode_item_tag(item_idx: u32) -> u64 {
    ASSETS_TAG_ITEM_BASE | (item_idx as u64)
}

/// Encodes a list table row direct Spawn button index into a 64-bit semantic tag.
#[inline]
pub fn encode_item_spawn_tag(item_idx: u32) -> u64 {
    ASSETS_TAG_ITEM_SPAWN_BASE | (item_idx as u64)
}

/// Encodes a list table row direct Inspect button index into a 64-bit semantic tag.
#[inline]
pub fn encode_item_inspect_tag(item_idx: u32) -> u64 {
    ASSETS_TAG_ITEM_INSPECT_BASE | (item_idx as u64)
}

/// Decodes a 64-bit semantic tag into an asset item index and action type, if matching.
///
/// Returns `Some((item_idx, action))`.
#[inline]
pub fn parse_item_tag(tag: u64) -> Option<(u32, AssetItemAction)> {
    let prefix = tag & ASSETS_TAG_ITEM_PREFIX_MASK;
    let idx = (tag & ASSETS_TAG_PAYLOAD_MASK) as u32;
    if prefix == ASSETS_TAG_ITEM_BASE {
        Some((idx, AssetItemAction::SelectOrOpen))
    } else if prefix == ASSETS_TAG_ITEM_SPAWN_BASE {
        Some((idx, AssetItemAction::Spawn))
    } else if prefix == ASSETS_TAG_ITEM_INSPECT_BASE {
        Some((idx, AssetItemAction::Inspect))
    } else {
        None
    }
}

/// Evaluates whether a 64-bit semantic tag belongs to the Asset Browser domain.
#[inline]
pub fn is_assets_tag(tag: u64) -> bool {
    (tag & ASSETS_TAG_DOMAIN_MASK) == ASSETS_TAG_DOMAIN
}

/// Target destination or action resolved from an Asset Browser semantic tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetsTagTarget {
    /// Toggles the folder tree sidebar open/closed.
    ToggleSidebar,
    /// Prompts file import dialog.
    Import,
    /// Sweeps unreferenced VRAM assets.
    CleanVram,
    /// Switches to grid view mode.
    ViewGrid,
    /// Switches to tabular list view mode.
    ViewList,
    /// Toggles display of built-in engine content.
    EngineContent,
    /// Focuses search text input.
    SearchInput,
    /// Clears active search text query.
    SearchClear,
    /// Opens new subfolder creation dialog.
    NewSubfolder,
    /// Reveals folder in OS file explorer.
    Reveal,
    /// Asset card or list row target: `(item_idx, action)`.
    Item(u32, AssetItemAction),
    /// Folder tree row or chevron target: `(node_idx, is_chevron)`.
    Tree(u32, bool),
    /// Breadcrumb navigation item: `segment_idx`.
    Breadcrumb(u8),
    /// Category filter chip: `category_idx`.
    Chip(u8),
    /// Closes quick asset preview modal.
    PreviewClose,
    /// Reveals previewed asset in OS file manager.
    PreviewReveal,
    /// Quick asset preview 3D orbit canvas.
    PreviewOrbit,
    /// Toggles wireframe view in 3D preview.
    PreviewWireframe,
}

/// Resolves a 64-bit semantic tag into a typed [`AssetsTagTarget`].
#[inline]
pub fn resolve_assets_tag(tag: u64) -> Option<AssetsTagTarget> {
    if !is_assets_tag(tag) {
        return None;
    }
    match tag {
        ASSETS_TAG_TOGGLE_SIDEBAR => Some(AssetsTagTarget::ToggleSidebar),
        ASSETS_TAG_IMPORT => Some(AssetsTagTarget::Import),
        ASSETS_TAG_CLEAN_VRAM => Some(AssetsTagTarget::CleanVram),
        ASSETS_TAG_VIEW_GRID => Some(AssetsTagTarget::ViewGrid),
        ASSETS_TAG_VIEW_LIST => Some(AssetsTagTarget::ViewList),
        ASSETS_TAG_ENGINE_CONTENT => Some(AssetsTagTarget::EngineContent),
        ASSETS_TAG_SEARCH_INPUT => Some(AssetsTagTarget::SearchInput),
        ASSETS_TAG_SEARCH_CLEAR => Some(AssetsTagTarget::SearchClear),
        ASSETS_TAG_NEW_SUBFOLDER => Some(AssetsTagTarget::NewSubfolder),
        ASSETS_TAG_REVEAL => Some(AssetsTagTarget::Reveal),
        ASSETS_TAG_PREVIEW_CLOSE => Some(AssetsTagTarget::PreviewClose),
        ASSETS_TAG_PREVIEW_REVEAL_BTN => Some(AssetsTagTarget::PreviewReveal),
        ASSETS_TAG_PREVIEW_ORBIT_CANVAS => Some(AssetsTagTarget::PreviewOrbit),
        ASSETS_TAG_PREVIEW_WIREFRAME_BTN => Some(AssetsTagTarget::PreviewWireframe),
        _ => {
            if let Some((item_idx, action)) = parse_item_tag(tag) {
                Some(AssetsTagTarget::Item(item_idx, action))
            } else if let Some((node_idx, is_chevron)) = parse_tree_tag(tag) {
                Some(AssetsTagTarget::Tree(node_idx, is_chevron))
            } else if let Some(segment_idx) = parse_breadcrumb_tag(tag) {
                Some(AssetsTagTarget::Breadcrumb(segment_idx))
            } else {
                parse_chip_tag(tag).map(AssetsTagTarget::Chip)
            }
        }
    }
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

/// Calculated geometry metrics from the Asset Browser panel declarative layout.
///
/// Encapsulates the outer panel bounds, collapsible sidebar area, asset content viewport,
/// and optional floating context menu card bounding box resolved during declarative layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetsPanelLayoutMetrics {
    /// Total bounding rectangle of the asset browser panel.
    pub panel_rect: Rect,
    /// Bounding rectangle of the left folder tree sidebar, if not collapsed.
    pub sidebar_rect: Option<Rect>,
    /// Bounding rectangle of the main scrollable asset content viewport.
    pub content_viewport_rect: Rect,
    /// Bounding rectangle of the active floating right-click context menu card, if open.
    pub context_menu_card_rect: Option<Rect>,
}

/// Rendering parameters supplied to `build_assets_panel`.
pub struct AssetsPanelParams<'a> {
    /// Bounding rectangle assigned to the panel by the docking manager.
    pub panel_rect: Rect,
    /// Full dimensions of the entire editor window in logical pixels: `(width, height)`.
    pub screen_size: (f32, f32),
    /// Currently active folder path in the asset browser.
    pub current_folder: &'a std::path::Path,
    /// Live search filter query string.
    pub search_query: &'a str,
    /// Currently active asset category filter.
    pub active_category: AssetCategory,
    /// Active presentation mode (Grid vs List).
    pub view_mode: AssetViewMode,
    /// Currently selected asset path, if any.
    pub selected_asset: Option<&'a std::path::Path>,
    /// Slice of all discovered cached items across the workspace.
    pub cached_items: &'a [AssetItem],
    /// Filtered asset items matching current folder, category, and search query.
    pub filtered_items: &'a [AssetItem],
    /// Whether the editor is currently running in 2D mode.
    pub is_2d_mode: bool,
    /// Whether internal engine assets and built-in shaders are displayed.
    pub show_engine_content: bool,
    /// Width of the left folder tree sidebar in pixels.
    pub sidebar_width: f32,
    /// Whether the left folder tree sidebar is currently collapsed.
    pub sidebar_collapsed: bool,
    /// Vertical scroll offset of the primary asset content area.
    pub scroll_y: f32,
    /// Vertical scroll offset of the left folder tree sidebar.
    pub tree_scroll_y: f32,
    /// Current mouse cursor coordinates for hover evaluation.
    pub cursor_pos: Point,
    /// Whether the blinking caret cursor should be drawn (500ms cycle).
    pub blink_caret: bool,
    /// Active right-click context menu target and click position, if open.
    pub active_context_menu: Option<&'a (AssetsContextMenuTarget, Point)>,
    /// Active Quick Asset Preview modal state, if open.
    pub active_preview_modal: Option<&'a AssetPreviewModalState>,
    /// Discovered subfolders tree cached in engine asset state.
    pub subfolders: &'a [PathBuf],
    /// Map of asset paths to allocated 2D Texture Array thumbnail layer indices.
    pub thumbnail_layers: &'a HashMap<PathBuf, u32>,
}

/// Truncates a UTF-8 string safely at Unicode code point boundaries.
///
/// If the character count exceeds `max_chars`, the string is sliced at the `keep_chars`-th
/// Unicode character boundary and an ellipsis (`"..."`) is appended. If the character
/// count is within limits, a clone of the original string is returned unchanged.
///
/// This prevents byte index panics on multi-byte characters such as Turkish (`ğ`, `ü`, `ş`, `ı`, `ö`, `ç`),
/// Cyrillic, CJK ideographs, or emojis.
pub fn truncate_display_name(text: &str, max_chars: usize, keep_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count > max_chars {
        let split_idx = text
            .char_indices()
            .nth(keep_chars)
            .map(|(idx, _)| idx)
            .unwrap_or(text.len());
        format!("{}...", &text[..split_idx])
    } else {
        text.to_string()
    }
}

/// Persistent interactive state for the Asset Browser panel overlay.
#[derive(Debug, Clone)]
pub struct AssetsPanelState {
    /// Common panel interaction state (scroll_y, search, actions).
    pub interactions: crate::ui::iris_bridge::types::PanelInteractionState<(), AssetsPanelAction>,
    /// Bounding rectangle of the Asset Browser panel from the last layout pass.
    pub panel_rect: Option<Rect>,
    /// Bounding rectangle of the folder tree sidebar if active.
    pub sidebar_rect: Option<Rect>,
    /// Bounding rectangle of the scrollable content viewport.
    pub content_viewport_rect: Option<Rect>,
    /// Bounding rectangle of the active floating right-click context menu card, if open.
    pub context_menu_card_rect: Option<Rect>,
    /// Folder tree sidebar vertical scroll offset.
    pub tree_scroll_y: f32,
    /// Last synchronized folder tree sidebar vertical scroll offset.
    pub last_tree_scroll_y: f32,
    /// Current folder path in Asset Browser panel.
    pub current_folder: PathBuf,
    /// Double-click tracking state for asset spawning.
    pub click_tracker: AssetClickTracker,
    /// Active right-click context menu: `(target, click_pos)`.
    pub context_menu: Option<(AssetsContextMenuTarget, Point)>,
    /// Active Quick Asset Preview modal state.
    pub preview_modal: Option<AssetPreviewModalState>,
    /// Currently selected asset path.
    pub selected_asset: Option<PathBuf>,
    /// Dynamic thumbnail layer cache mapping asset paths to 2D Texture Array layers (32..255).
    pub thumbnail_layers: HashMap<PathBuf, u32>,
    /// Next available layer index in the 2D Texture Array (32..255).
    pub next_thumbnail_layer: u32,
    /// Cached filtered asset items from the active frame for O(1) semantic hit-testing.
    pub filtered_items_cache: Vec<AssetItem>,
    /// Cached subfolders from the active frame for O(1) semantic hit-testing.
    pub subfolders_cache: Vec<PathBuf>,
    /// Last recorded directory items count for detecting file additions or deletions.
    pub last_cached_items_count: usize,
}

impl Default for AssetsPanelState {
    fn default() -> Self {
        Self {
            interactions: crate::ui::iris_bridge::types::PanelInteractionState::default(),
            panel_rect: None,
            sidebar_rect: None,
            content_viewport_rect: None,
            context_menu_card_rect: None,
            tree_scroll_y: 0.0,
            last_tree_scroll_y: 0.0,
            current_folder: PathBuf::from("assets"),
            click_tracker: AssetClickTracker::default(),
            context_menu: None,
            preview_modal: None,
            selected_asset: None,
            thumbnail_layers: HashMap::new(),
            next_thumbnail_layer: 32,
            filtered_items_cache: Vec::new(),
            subfolders_cache: Vec::new(),
            last_cached_items_count: 0,
        }
    }
}

impl std::ops::Deref for AssetsPanelState {
    type Target = crate::ui::iris_bridge::types::PanelInteractionState<(), AssetsPanelAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for AssetsPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}

/// Semantic alias for [`AssetsPanelState`].
pub type AssetsState = AssetsPanelState;

impl AssetsPanelState {
    /// Evaluates whether the Content / Asset Browser panel requires an in-place repaint.
    ///
    /// Inspects current folder changes, selected asset changes, item count changes,
    /// active context menu popups, quick preview modals, or active asset drag-and-drop payloads.
    pub fn is_dirty(
        &self,
        params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>,
    ) -> bool {
        self.current_folder != params.panel_data.asset_browser.current_folder
            || self.selected_asset != params.panel_data.asset_browser.selected_asset
            || self.last_cached_items_count != params.panel_data.asset_browser.cached_items.len()
            || self.context_menu.is_some()
            || self.preview_modal.is_some()
            || params.panel_data.asset_browser.drag_payload.is_some()
    }

    /// Synchronizes internal cached snapshot values against active frame parameters.
    pub fn sync_dirty(&mut self, params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>) {
        self.current_folder = params.panel_data.asset_browser.current_folder.clone();
        self.selected_asset = params.panel_data.asset_browser.selected_asset.clone();
        self.last_cached_items_count = params.panel_data.asset_browser.cached_items.len();
    }

    /// Evaluates `is_dirty` and automatically updates snapshot caches if dirty.
    ///
    /// Returns `true` if the panel state changed and requires redraw tagging.
    pub fn check_and_sync_dirty(
        &mut self,
        params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>,
    ) -> bool {
        let dirty = self.is_dirty(params);
        if dirty {
            self.sync_dirty(params);
        }
        dirty
    }
}