// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Data Structures and Action Types
//!
//! Provides the data structures, hit-test targets, and action dispatch enums
//! for the 100% Iris UI GPU-accelerated Scene Hierarchy panel.

use irisui::prelude::*;
use std::path::PathBuf;

/// Pre-flattened lightweight POD representation of a single entity row in the scene hierarchy tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HierarchyRow {
    /// Target ECS entity.
    pub entity: hecs::Entity,
    /// Indentation nesting depth (0 = root entity).
    pub depth: u16,
    /// True if the entity has at least one valid child entity.
    pub has_children: bool,
    /// True if this node is currently expanded in the tree view.
    pub is_expanded: bool,
}

/// Active hierarchical submenu currently open within the `➕` Add Menu.
/// Represents the hierarchical cascading depth tier of an Add Menu branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSubmenuTier {
    /// Primary level-1 submenu opening directly from the main Add Menu card.
    Primary,
    /// Secondary nested level-2 sub-submenu opening from a primary branch card.
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSubmenuId {
    /// 3D primitive geometry submenu (Cube, Sphere, Plane, Cylinder, Capsule, Torus, Triangle).
    Objects3D,
    /// 2D primitive sprite objects submenu (Sprite, Player Sprite, Empty 2D Object).
    Objects2D,
    /// 2D Canvas & UI elements submenu (Panel, Label, Image, Button, Progress Bar, Slider, Checkbox, Input).
    UiCanvas,
    /// Preset in-game HUD widgets (HealthBar, ScoreDisplay).
    HudPresets,
    /// Assets and prefabs submenu (Load Model, Load Prefab).
    AssetsPrefabs,
    /// Stress testing and performance benchmarks submenu.
    StressBenchmarks,
}

impl AddSubmenuId {
    /// Returns the hierarchical cascading depth tier of this submenu branch.
    ///
    /// Enables fully data-driven submenu activation without hardcoded enum branching.
    #[inline]
    pub const fn tier(self) -> AddSubmenuTier {
        match self {
            Self::HudPresets => AddSubmenuTier::Secondary,
            Self::Objects3D
            | Self::Objects2D
            | Self::UiCanvas
            | Self::AssetsPrefabs
            | Self::StressBenchmarks => AddSubmenuTier::Primary,
        }
    }

    /// Converts the submenu variant into a unique numeric tag for widget routing.
    #[inline]
    pub const fn to_tag(self) -> u64 {
        match self {
            Self::Objects3D => 1,
            Self::Objects2D => 2,
            Self::UiCanvas => 3,
            Self::HudPresets => 4,
            Self::AssetsPrefabs => 5,
            Self::StressBenchmarks => 6,
        }
    }

    /// Reconstructs the submenu variant from a widget tag if it represents a branch.
    #[inline]
    pub const fn from_tag(tag: u64) -> Option<Self> {
        match tag {
            1 => Some(Self::Objects3D),
            2 => Some(Self::Objects2D),
            3 => Some(Self::UiCanvas),
            4 => Some(Self::HudPresets),
            5 => Some(Self::AssetsPrefabs),
            6 => Some(Self::StressBenchmarks),
            _ => None,
        }
    }
}

/// Actions dispatched from the Hierarchy panel to the engine UI processor.
#[derive(Debug, Clone, PartialEq)]
pub enum HierarchyAction {
    /// Select or deselect an entity.
    SelectEntity(Option<hecs::Entity>),
    /// Toggle visibility of a specific entity.
    ToggleVisibility(hecs::Entity),
    /// Delete the currently selected entity.
    DeleteSelected,
    /// Spawn a primitive 3D mesh shape.
    SpawnShape(ae_core::ecs::Shape),
    /// Spawn a standard 2D Sprite entity.
    SpawnDefaultSprite,
    /// Spawn a 2D Player Sprite entity with `PlayerTag`.
    SpawnPlayerSprite,
    /// Spawn an empty 2D entity.
    SpawnEmpty2D,
    /// Spawn a 2D UI element or preset HUD component.
    SpawnUiElement(crate::ui::UiElementType),
    /// Open the 3D model asset import file picker dialog.
    OpenModelDialog,
    /// Open the prefab asset import file picker dialog.
    OpenLoadPrefabDialog,
    /// Instantiate a prefab from the given filesystem path.
    InstantiatePrefab(PathBuf),
    /// Spawns the complete interactive test sandbox.
    SpawnPhase1TestSandbox,
    /// Triggers an entity stress benchmark test with N entities.
    StressTest(usize),
    /// Triggers the 10km OpenWorld stress test.
    AaaOpenWorldTest,
    /// Triggers the particle physics explosion stress test.
    Explode,
    /// Sets the search filter query string.
    SetSearchQuery(String),
    /// Clears the active search filter query.
    ClearSearchQuery,
    /// Opens the `➕` Add Menu at the specified screen anchor.
    OpenAddMenu(Option<Point>),
    /// Closes the active `➕` Add Menu.
    CloseAddMenu,
    /// Opens a specific cascading submenu within the Add Menu.
    OpenSubmenu(AddSubmenuId),
    /// Closes the active cascading submenu.
    CloseSubmenu,
    /// Opens a specific nested sub-submenu within the cascading submenu.
    OpenSubSubmenu(AddSubmenuId),
    /// Closes the active nested sub-submenu.
    CloseSubSubmenu,
    /// Opens the right-click context menu for an entity at the cursor position.
    OpenContextMenu(hecs::Entity, Point),
    /// Closes the right-click context menu.
    CloseContextMenu,
}

// ── Semantic 64-bit Tags ──────────────────────────────────────────────────

/// Semantic tag assigned to the Hierarchy Panel root container.
pub const HIERARCHY_TAG_PANEL_ROOT: u64 = 0x4849_4552_0000_0000;

/// Semantic tag assigned to the Search Bar text input container.
pub const HIERARCHY_TAG_SEARCH_INPUT: u64 = 0x4849_4552_0000_0001;

/// Semantic tag assigned to the Search Bar clear `✖` button.
pub const HIERARCHY_TAG_SEARCH_CLEAR: u64 = 0x4849_4552_0000_0002;

/// Semantic tag assigned to the Header `➕` Add Menu button.
pub const HIERARCHY_TAG_ADD_BUTTON: u64 = 0x4849_4552_0000_0003;

/// Semantic tag assigned to the Header `🗑` Delete Selected button.
pub const HIERARCHY_TAG_DELETE_BUTTON: u64 = 0x4849_4552_0000_0004;

/// Semantic tag assigned to the Hierarchy virtual rows scroll viewport.
pub const HIERARCHY_TAG_VIEWPORT: u64 = 0x4849_4552_0000_0005;

/// Bitmask prefix for dynamic entity row selection tags.
pub const HIERARCHY_TAG_ROW_PREFIX: u64 = 0x4849_4552_1000_0000;

/// Bitmask prefix for dynamic entity eye visibility toggle tags.
pub const HIERARCHY_TAG_EYE_PREFIX: u64 = 0x4849_4552_2000_0000;

/// Bitmask prefix for dynamic entity foldout chevron expand/collapse tags.
pub const HIERARCHY_TAG_FOLDOUT_PREFIX: u64 = 0x4849_4552_3000_0000;

/// Bitmask isolating the row index within dynamic row and eye tags (lower 24 bits).
pub const HIERARCHY_TAG_INDEX_MASK: u64 = 0x0000_0000_00FF_FFFF;

/// Constructs a 64-bit semantic tag for an interactive entity row selection item.
#[inline]
pub const fn make_row_tag(row_idx: usize) -> u64 {
    HIERARCHY_TAG_ROW_PREFIX | ((row_idx as u64) & HIERARCHY_TAG_INDEX_MASK)
}

/// Parses and extracts the row index from an interactive entity row semantic tag.
#[inline]
pub const fn parse_row_tag(tag: u64) -> Option<usize> {
    if (tag & 0xFFFF_FFFF_FF00_0000) == HIERARCHY_TAG_ROW_PREFIX {
        Some((tag & HIERARCHY_TAG_INDEX_MASK) as usize)
    } else {
        None
    }
}

/// Constructs a 64-bit semantic tag for an entity eye visibility toggle button.
#[inline]
pub const fn make_eye_tag(row_idx: usize) -> u64 {
    HIERARCHY_TAG_EYE_PREFIX | ((row_idx as u64) & HIERARCHY_TAG_INDEX_MASK)
}

/// Parses and extracts the row index from an entity eye visibility semantic tag.
#[inline]
pub const fn parse_eye_tag(tag: u64) -> Option<usize> {
    if (tag & 0xFFFF_FFFF_FF00_0000) == HIERARCHY_TAG_EYE_PREFIX {
        Some((tag & HIERARCHY_TAG_INDEX_MASK) as usize)
    } else {
        None
    }
}

/// Constructs a 64-bit semantic tag for an entity branch foldout chevron toggle button.
#[inline]
pub const fn make_foldout_tag(row_idx: usize) -> u64 {
    HIERARCHY_TAG_FOLDOUT_PREFIX | ((row_idx as u64) & HIERARCHY_TAG_INDEX_MASK)
}

/// Parses and extracts the row index from an entity branch foldout chevron semantic tag.
#[inline]
pub const fn parse_foldout_tag(tag: u64) -> Option<usize> {
    if (tag & 0xFFFF_FFFF_FF00_0000) == HIERARCHY_TAG_FOLDOUT_PREFIX {
        Some((tag & HIERARCHY_TAG_INDEX_MASK) as usize)
    } else {
        None
    }
}

/// Returns `true` if the given 64-bit widget tag belongs to the Scene Hierarchy panel subsystem.
#[inline]
pub const fn is_hierarchy_tag(tag: u64) -> bool {
    (tag & 0xFFFF_FFFF_0000_0000) == 0x4849_4552_0000_0000
}

/// Strongly typed semantic hit target within the Scene Hierarchy panel.
///
/// Eliminates raw bitwise masking and arithmetic across event handling routines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchyTagTarget {
    /// Primary panel root container.
    PanelRoot,
    /// Search query text input box.
    SearchInput,
    /// Search query clear button.
    SearchClear,
    /// Header Add Entity `➕` button.
    AddButton,
    /// Header Delete Selected `🗑` button.
    DeleteButton,
    /// Virtual rows scroll viewport.
    Viewport,
    /// Dynamic entity row item by index.
    Row(usize),
    /// Dynamic entity visibility toggle button by index.
    Eye(usize),
    /// Dynamic entity chevron expand/collapse toggle button by index.
    Foldout(usize),
}

/// Resolves a 64-bit semantic tag into a strongly typed [`HierarchyTagTarget`].
///
/// Returns `None` if the tag does not belong to the Scene Hierarchy panel domain.
#[inline]
pub fn resolve_hierarchy_tag(tag: u64) -> Option<HierarchyTagTarget> {
    match tag {
        HIERARCHY_TAG_PANEL_ROOT => Some(HierarchyTagTarget::PanelRoot),
        HIERARCHY_TAG_SEARCH_INPUT => Some(HierarchyTagTarget::SearchInput),
        HIERARCHY_TAG_SEARCH_CLEAR => Some(HierarchyTagTarget::SearchClear),
        HIERARCHY_TAG_ADD_BUTTON => Some(HierarchyTagTarget::AddButton),
        HIERARCHY_TAG_DELETE_BUTTON => Some(HierarchyTagTarget::DeleteButton),
        HIERARCHY_TAG_VIEWPORT => Some(HierarchyTagTarget::Viewport),
        _ => {
            if let Some(row_idx) = parse_eye_tag(tag) {
                Some(HierarchyTagTarget::Eye(row_idx))
            } else if let Some(row_idx) = parse_foldout_tag(tag) {
                Some(HierarchyTagTarget::Foldout(row_idx))
            } else {
                parse_row_tag(tag).map(HierarchyTagTarget::Row)
            }
        }
    }
}

/// Numerical tag for deleting the target entity via right-click context menu.
pub const HIERARCHY_CTX_DELETE: u64 = 0;

/// Numerical tag for toggling target entity visibility via right-click context menu.
pub const HIERARCHY_CTX_VISIBILITY: u64 = 1;

/// Parameters passed to the Hierarchy panel declarative builder.
pub struct HierarchyPanelParams<'a> {
    /// Docked panel bounding rectangle.
    pub panel_rect: Rect,
    /// Active ECS world.
    pub world: &'a hecs::World,
    /// Currently selected entity.
    pub selected_entity: Option<hecs::Entity>,
    /// Active search filter query string.
    pub search_query: &'a str,
    /// Whether the editor is currently in edit mode (vs play mode).
    pub is_editing: bool,
    /// Whether the active dimension mode is 2D Sprite Engine mode.
    pub is_2d: bool,
    /// Vertical scroll offset in physical pixels.
    pub scroll_y: f32,
    /// Active cascading Add Menu submenu (if open).
    pub active_submenu: Option<AddSubmenuId>,
    /// Active nested Add Menu sub-submenu (if open).
    pub active_sub_submenu: Option<AddSubmenuId>,
    /// Whether the Add Menu is open.
    pub is_add_menu_open: bool,
    /// Active right-click context menu (target entity, position) if open.
    pub active_context_menu: Option<(hecs::Entity, Point)>,
    /// Current mouse cursor position.
    pub cursor_pos: Point,
    /// Caret blink phase indicator for text inputs.
    pub blink_caret: bool,
    /// Set of currently collapsed entity identifiers.
    pub collapsed_entities: &'a std::collections::HashSet<hecs::Entity>,
}

/// Persistent interactive state for the Scene Hierarchy panel overlay.
#[derive(Debug, Default, Clone)]
pub struct HierarchyPanelState {
    /// Common panel interaction state (scroll_y, search, actions).
    pub interactions: crate::ui::iris_bridge::types::PanelInteractionState<(), HierarchyAction>,
    /// Maximum computed vertical scrollable overflow extent.
    pub max_scroll: f32,
    /// Persistent pre-allocated row cache to eliminate per-frame allocations.
    pub rows_cache: Vec<HierarchyRow>,
    /// Set of currently collapsed entity identifiers.
    pub collapsed_entities: std::collections::HashSet<hecs::Entity>,
    /// Whether the `➕` Add Menu is open in Scene Hierarchy.
    pub is_add_menu_open: bool,
    /// Currently open cascading submenu in Scene Hierarchy Add Menu.
    pub active_submenu: Option<AddSubmenuId>,
    /// Currently open cascading sub-submenu (Level 3) in Scene Hierarchy Add Menu.
    pub active_sub_submenu: Option<AddSubmenuId>,
    /// Currently open right-click context menu in Scene Hierarchy.
    pub active_context_menu: Option<(hecs::Entity, Point)>,
    /// Last bounding rectangle allocated for the Hierarchy panel inside docking.
    pub last_rect: Option<Rect>,
    /// Last recorded selected entity for detecting Hierarchy selection invalidation.
    pub last_selected_entity: Option<hecs::Entity>,
    /// Last recorded entity count in ECS world for detecting hierarchy additions/deletions.
    pub last_entity_count: usize,
}

/// Semantic alias for [`HierarchyPanelState`].
pub type HierarchyState = HierarchyPanelState;

impl std::ops::Deref for HierarchyPanelState {
    type Target = crate::ui::iris_bridge::types::PanelInteractionState<(), HierarchyAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for HierarchyPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}

impl HierarchyPanelState {
    /// Evaluates whether the Scene Hierarchy panel requires an in-place repaint.
    ///
    /// Inspects selected entity shifts, entity additions/deletions in the ECS world,
    /// and active floating popup menus (`➕ Add Menu` or right-click context menu).
    pub fn is_dirty(
        &self,
        params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>,
    ) -> bool {
        let world_len = params.scene.world.len() as usize;
        self.last_selected_entity != params.scene.selected_entity
            || self.last_entity_count != world_len
            || (self.scroll_y - self.last_scroll_y).abs() > 0.001
            || self.is_add_menu_open
            || self.active_context_menu.is_some()
    }

    /// Synchronizes internal cached snapshot values against active frame parameters.
    pub fn sync_dirty(&mut self, params: &crate::ui::iris_bridge::types::OverlayUpdateParams<'_>) {
        self.last_selected_entity = params.scene.selected_entity;
        self.last_entity_count = params.scene.world.len() as usize;
        self.last_scroll_y = self.scroll_y;
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

    /// Dynamically activates a cascading submenu branch based on its hierarchical tier.
    ///
    /// Opening a primary tier submenu clears any active secondary sub-submenus,
    /// while opening a secondary tier branch preserves the parent primary branch.
    ///
    /// Returns `true` if the state was mutated and requires UI repaint.
    pub fn activate_submenu(&mut self, sub_id: AddSubmenuId) -> bool {
        match sub_id.tier() {
            AddSubmenuTier::Primary => {
                if self.active_submenu != Some(sub_id) {
                    self.active_submenu = Some(sub_id);
                    self.active_sub_submenu = None;
                    return true;
                }
            }
            AddSubmenuTier::Secondary => {
                if self.active_sub_submenu != Some(sub_id) {
                    self.active_sub_submenu = Some(sub_id);
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_submenu_tier_and_dynamic_activation() {
        let mut state = HierarchyPanelState::default();

        // 1. Activate Primary tier (UiCanvas)
        assert_eq!(AddSubmenuId::UiCanvas.tier(), AddSubmenuTier::Primary);
        assert!(state.activate_submenu(AddSubmenuId::UiCanvas));
        assert_eq!(state.active_submenu, Some(AddSubmenuId::UiCanvas));
        assert_eq!(state.active_sub_submenu, None);

        // Activating same primary branch returns false (no mutation)
        assert!(!state.activate_submenu(AddSubmenuId::UiCanvas));

        // 2. Activate Secondary tier (HudPresets) while preserving primary
        assert_eq!(AddSubmenuId::HudPresets.tier(), AddSubmenuTier::Secondary);
        assert!(state.activate_submenu(AddSubmenuId::HudPresets));
        assert_eq!(state.active_submenu, Some(AddSubmenuId::UiCanvas));
        assert_eq!(state.active_sub_submenu, Some(AddSubmenuId::HudPresets));

        // 3. Switching to another Primary tier (Objects3D) clears secondary tier
        assert!(state.activate_submenu(AddSubmenuId::Objects3D));
        assert_eq!(state.active_submenu, Some(AddSubmenuId::Objects3D));
        assert_eq!(state.active_sub_submenu, None);
    }

    #[test]
    fn test_resolve_hierarchy_tag_invariants() {
        assert_eq!(
            resolve_hierarchy_tag(HIERARCHY_TAG_SEARCH_INPUT),
            Some(HierarchyTagTarget::SearchInput)
        );
        assert_eq!(
            resolve_hierarchy_tag(HIERARCHY_TAG_SEARCH_CLEAR),
            Some(HierarchyTagTarget::SearchClear)
        );
        assert_eq!(
            resolve_hierarchy_tag(HIERARCHY_TAG_ADD_BUTTON),
            Some(HierarchyTagTarget::AddButton)
        );
        assert_eq!(
            resolve_hierarchy_tag(HIERARCHY_TAG_DELETE_BUTTON),
            Some(HierarchyTagTarget::DeleteButton)
        );
        assert_eq!(
            resolve_hierarchy_tag(HIERARCHY_TAG_VIEWPORT),
            Some(HierarchyTagTarget::Viewport)
        );
        assert_eq!(
            resolve_hierarchy_tag(make_eye_tag(4)),
            Some(HierarchyTagTarget::Eye(4))
        );
        assert_eq!(
            resolve_hierarchy_tag(make_foldout_tag(8)),
            Some(HierarchyTagTarget::Foldout(8))
        );
        assert_eq!(
            resolve_hierarchy_tag(make_row_tag(15)),
            Some(HierarchyTagTarget::Row(15))
        );
        assert_eq!(resolve_hierarchy_tag(0), None);
        assert_eq!(resolve_hierarchy_tag(0xFFFF_FFFF), None);
    }

    #[test]
    fn test_hierarchy_scroll_dirty_detection() {
        let mut state = HierarchyPanelState::default();
        assert_eq!(state.scroll_y, 0.0);
        assert_eq!(state.last_scroll_y, 0.0);

        // Modifying scroll_y reflects a dirty scroll delta
        state.scroll_y = 50.0;
        assert!((state.scroll_y - state.last_scroll_y).abs() > 0.001);

        // Synchronizing updates last_scroll_y to match active scroll_y
        state.last_scroll_y = state.scroll_y;
        assert!((state.scroll_y - state.last_scroll_y).abs() <= 0.001);
    }
}