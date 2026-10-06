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
    /// Whether the search input box currently has keyboard focus.
    pub is_search_focused: bool,
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
    /// Optional semantic tag currently under the mouse cursor for hover reactivity.
    pub hovered_tag: Option<u64>,
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
            current_folder: PathBuf::from("assets"),
            click_tracker: AssetClickTracker::default(),
            context_menu: None,
            preview_modal: None,
            selected_asset: None,
            thumbnail_layers: HashMap::new(),
            next_thumbnail_layer: 32,
            filtered_items_cache: Vec::new(),
            subfolders_cache: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_semantic_tags_encoding_roundtrip() {
        // 1. Domain verification
        assert!(is_assets_tag(ASSETS_TAG_PANEL_ROOT));
        assert!(is_assets_tag(ASSETS_TAG_TOGGLE_SIDEBAR));
        assert!(is_assets_tag(ASSETS_TAG_IMPORT));
        assert!(is_assets_tag(ASSETS_TAG_VIEW_GRID));
        assert!(is_assets_tag(ASSETS_TAG_VIEW_LIST));
        assert!(!is_assets_tag(0x0080_0000_0000_0001)); // Preferences domain
        assert!(!is_assets_tag(0x0070_0000_0000_0001)); // UI designer domain
        assert!(!is_assets_tag(0));

        // 2. Category Chips Roundtrip (0..8)
        for cat_idx in 0..=8 {
            let tag = encode_chip_tag(cat_idx);
            assert!(is_assets_tag(tag));
            assert_eq!(parse_chip_tag(tag), Some(cat_idx));
        }
        assert_eq!(parse_chip_tag(ASSETS_TAG_PANEL_ROOT), None);

        // 3. Breadcrumb Navigation Segments Roundtrip (0..255)
        for seg_idx in [0, 1, 5, 128, 255] {
            let tag = encode_breadcrumb_tag(seg_idx);
            assert!(is_assets_tag(tag));
            assert_eq!(parse_breadcrumb_tag(tag), Some(seg_idx));
        }
        assert_eq!(parse_breadcrumb_tag(ASSETS_TAG_PANEL_ROOT), None);

        // 4. Context Menu Actions Roundtrip (0..15)
        for action_idx in 0..=6 {
            let tag = encode_ctx_item_tag(action_idx);
            assert!(is_assets_tag(tag));
            assert_eq!(parse_ctx_item_tag(tag), Some(action_idx));
        }
        assert_eq!(parse_ctx_item_tag(ASSETS_TAG_PANEL_ROOT), None);

        // 5. Folder Tree Rows and Chevrons Roundtrip (0..u32::MAX)
        for node_idx in [0, 1, 42, 1024, 0x00FF_FFFF] {
            let row_tag = encode_tree_row_tag(node_idx);
            let chev_tag = encode_tree_chevron_tag(node_idx);
            assert!(is_assets_tag(row_tag));
            assert!(is_assets_tag(chev_tag));
            assert_eq!(parse_tree_tag(row_tag), Some((node_idx, false)));
            assert_eq!(parse_tree_tag(chev_tag), Some((node_idx, true)));
        }
        assert_eq!(parse_tree_tag(ASSETS_TAG_PANEL_ROOT), None);

        // 6. Asset Items (Select, Spawn, Inspect) Roundtrip
        for item_idx in [0, 1, 99, 10000, 0x00FF_FFFF] {
            let select_tag = encode_item_tag(item_idx);
            let spawn_tag = encode_item_spawn_tag(item_idx);
            let inspect_tag = encode_item_inspect_tag(item_idx);

            assert!(is_assets_tag(select_tag));
            assert!(is_assets_tag(spawn_tag));
            assert!(is_assets_tag(inspect_tag));

            assert_eq!(
                parse_item_tag(select_tag),
                Some((item_idx, AssetItemAction::SelectOrOpen))
            );
            assert_eq!(
                parse_item_tag(spawn_tag),
                Some((item_idx, AssetItemAction::Spawn))
            );
            assert_eq!(
                parse_item_tag(inspect_tag),
                Some((item_idx, AssetItemAction::Inspect))
            );
        }
        assert_eq!(parse_item_tag(ASSETS_TAG_PANEL_ROOT), None);
    }
}