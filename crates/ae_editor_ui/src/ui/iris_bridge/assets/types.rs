// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type Definitions, Action Enums & Interaction Targets for Iris UI Content Browser.
//!
//! Provides data models, event actions, and layout bounding rectangles for the
//! 100% native Iris UI GPU SDF Asset Browser panel.
//!

use crate::ui::panels::assets::types::{AssetCategory, AssetItem, AssetViewMode};
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
}

/// Target subject of an active asset browser context menu.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetsContextMenuTarget {
    /// Context menu opened on an asset item (card or list row).
    Asset(AssetItem),
    /// Context menu opened on a folder node or blank area.
    Folder(PathBuf),
}

/// Hit-testing bounding boxes and target descriptors for breadcrumb navigation items.
#[derive(Debug, Clone)]
pub struct BreadcrumbTarget {
    /// Bounding rectangle of the breadcrumb button.
    pub rect: Rect,
    /// Target folder path when clicked.
    pub path: PathBuf,
}

/// Hit-testing bounding boxes and target descriptors for folder tree sidebar rows.
#[derive(Debug, Clone)]
pub struct FolderTreeNodeTarget {
    /// Bounding rectangle of the complete folder tree row.
    pub row_rect: Rect,
    /// Optional bounding rectangle of the expand/collapse chevron icon.
    pub chevron_rect: Option<Rect>,
    /// Target directory path for this tree node.
    pub path: PathBuf,
    /// Whether this directory has child subdirectories.
    pub has_children: bool,
    /// Whether this folder node is currently expanded.
    pub is_expanded: bool,
}

/// Hit-testing bounding boxes and target descriptors for asset grid cards.
#[derive(Debug, Clone)]
pub struct AssetCardTarget {
    /// Bounding rectangle of the complete asset card.
    pub rect: Rect,
    /// Reference path to the underlying asset on disk.
    pub path: PathBuf,
    /// Classified category of the asset.
    pub category: AssetCategory,
    /// Clone of item metadata for double-click spawning or inspection.
    pub item: AssetItem,
}

/// Hit-testing bounding boxes and target descriptors for asset list view table rows.
#[derive(Debug, Clone)]
pub struct AssetRowTarget {
    /// Bounding rectangle of the complete table row.
    pub rect: Rect,
    /// Bounding rectangle of the direct Spawn action button, if available.
    pub spawn_btn_rect: Option<Rect>,
    /// Bounding rectangle of the direct Inspect action button.
    pub inspect_btn_rect: Option<Rect>,
    /// Reference path to the underlying asset on disk.
    pub path: PathBuf,
    /// Classified category of the asset.
    pub category: AssetCategory,
    /// Clone of item metadata for double-click spawning or inspection.
    pub item: AssetItem,
}

/// Hit-testing targets for an active Asset Browser right-click context menu.
#[derive(Debug, Clone)]
pub struct AssetsContextMenuTargets {
    /// Full bounding box of the floating context menu card.
    pub card_rect: Rect,
    /// Hit target of the 'Quick Inspect' item (for assets).
    pub inspect_rect: Option<Rect>,
    /// Hit target of the primary 'Spawn / Load' item (for assets).
    pub spawn_rect: Option<Rect>,
    /// Hit target of the 'New Subfolder' item (for folders).
    pub new_folder_rect: Option<Rect>,
    /// Hit target of the 'Rename' item.
    pub rename_rect: Option<Rect>,
    /// Hit target of the 'Delete' item.
    pub delete_rect: Option<Rect>,
    /// Hit target of the 'Copy File Path' item (for assets).
    pub copy_path_rect: Option<Rect>,
    /// Hit target of the 'Reveal in Explorer' item.
    pub reveal_rect: Option<Rect>,
    /// Target subject of the active context menu.
    pub target: AssetsContextMenuTarget,
}

/// State container for the -style interactive Asset Preview modal.
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
                file_size_bytes: 0,
                metadata_badge: String::new(),
                is_loaded_in_memory: false,
                model_handle: None,
                texture_handle: None,
                shader_handle: None,
            },
            orbit_yaw: 0.0,
            orbit_pitch: 0.3,
            zoom_distance: 1.0,
            show_wireframe: true,
        }
    }
}

/// Hit-testing targets for the Quick Asset Preview modal window.
#[derive(Debug, Clone)]
pub struct AssetPreviewModalTargets {
    /// Bounding box of the complete modal dialog card.
    pub dialog_rect: Rect,
    /// Hit target of the '✖' top-right close button.
    pub close_btn_rect: Rect,
    /// Hit target of the central interactive 3D orbit preview canvas.
    pub orbit_canvas_rect: Option<Rect>,
    /// Hit target of the primary 'Spawn into Scene' / 'Spawn as Sprite' / 'Load Scene' button.
    pub action_btn_rect: Option<Rect>,
    /// Hit target of the 'Reveal in Explorer' button.
    pub reveal_btn_rect: Rect,
    /// Inspected asset metadata.
    pub item: AssetItem,
}

/// Interactive hit-testing target collection populated during Asset Browser panel construction.
#[derive(Debug, Default, Clone)]
pub struct AssetsPanelTargets {
    /// Total bounding rectangle of the asset browser panel.
    pub panel_rect: Rect,
    /// Bounding rectangle of the top header toolbar.
    pub toolbar_rect: Rect,
    /// Bounding rectangle of the category chips toolbar.
    pub chips_rect: Rect,
    /// Interactive breadcrumb navigation items in the top bar.
    pub breadcrumbs: Vec<BreadcrumbTarget>,
    /// Bounding rectangle of the "+ Import" button.
    pub import_btn_rect: Rect,
    /// Bounding rectangle of the "Reveal" button.
    pub reveal_btn_rect: Rect,
    /// Bounding rectangle of the "Clean" button.
    pub clean_btn_rect: Rect,
    /// Bounding rectangle of the "Grid" view toggle button.
    pub grid_toggle_rect: Rect,
    /// Bounding rectangle of the "List" view toggle button.
    pub list_toggle_rect: Rect,
    /// Bounding rectangle of the search input field box.
    pub search_input_rect: Rect,
    /// Bounding rectangle of the "✖" search query clear button, if query is non-empty.
    pub search_clear_btn_rect: Option<Rect>,
    /// Interactive category filter chip rectangles: `(Category, Rect)`.
    pub category_chips: Vec<(AssetCategory, Rect)>,
    /// Bounding rectangle of the left folder tree sidebar.
    pub sidebar_rect: Option<Rect>,
    /// Bounding rectangle of the "+" new subfolder button in the sidebar header.
    pub new_subfolder_btn_rect: Option<Rect>,
    /// Interactive folder tree node rows.
    pub folder_nodes: Vec<FolderTreeNodeTarget>,
    /// Bounding rectangle of the main scrollable asset content viewport.
    pub content_viewport_rect: Rect,
    /// Interactive grid card targets when in Grid view mode.
    pub grid_cards: Vec<AssetCardTarget>,
    /// Interactive table row targets when in List view mode.
    pub list_rows: Vec<AssetRowTarget>,
    /// Bounding rectangle of the bottom status bar / footer.
    pub footer_rect: Rect,
    /// Bounding rectangle of the sidebar collapse/expand toggle button in the footer.
    pub sidebar_toggle_btn_rect: Rect,
    /// Bounding rectangle of the active folder path display in the footer.
    pub footer_folder_rect: Rect,
    /// Active right-click context menu targets, if open.
    pub context_menu: Option<AssetsContextMenuTargets>,
    /// Active Quick Asset Preview modal targets, if open.
    pub preview_modal: Option<AssetPreviewModalTargets>,
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
    /// Map of asset paths to allocated 2D Texture Array thumbnail layer indices.
    pub thumbnail_layers: &'a HashMap<PathBuf, u32>,
}