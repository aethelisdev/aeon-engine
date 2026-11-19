// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Data Structures and Semantic Tag Definitions
//!
//! Provides semantic [`UiTag`] identifiers, cached row aliases, panel view parameters,
//! and retained panel state for the Iris UI-accelerated Scene Hierarchy outliner.

use irisui::prelude::*;

/// Alias for a flattened hierarchy row owned by [`TreeModel`](irisui::prelude::TreeModel).
pub type HierarchyRow = TreeRowEntry<hecs::Entity>;

/// Active cascading submenu category within the Hierarchy `➕` Add Menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSubmenuId {
    /// 3D primitive geometry submenu.
    Objects3D,
    /// 2D primitive sprite objects submenu.
    Objects2D,
    /// 2D Canvas & UI elements submenu.
    UiCanvas,
    /// Preset in-game HUD widgets submenu.
    HudPresets,
    /// Assets and prefabs submenu.
    AssetsPrefabs,
    /// Stress testing and performance benchmarks submenu.
    StressBenchmarks,
}

impl AddSubmenuId {
    /// Returns the deterministic [`UiTag`] identifying this cascading submenu branch.
    #[inline]
    #[must_use]
    pub const fn tag(self) -> UiTag {
        match self {
            Self::Objects3D => UiTag::from_str("ae.hierarchy.add_menu.objects_3d"),
            Self::Objects2D => UiTag::from_str("ae.hierarchy.add_menu.objects_2d"),
            Self::UiCanvas => UiTag::from_str("ae.hierarchy.add_menu.ui_canvas"),
            Self::HudPresets => UiTag::from_str("ae.hierarchy.add_menu.hud_presets"),
            Self::AssetsPrefabs => UiTag::from_str("ae.hierarchy.add_menu.assets_prefabs"),
            Self::StressBenchmarks => UiTag::from_str("ae.hierarchy.add_menu.stress_benchmarks"),
        }
    }
}

// ── Deterministic Semantic UiTags ─────────────────────────────────────────

/// Semantic [`UiTag`] assigned to the Hierarchy Panel root container.
pub const HIERARCHY_TAG_PANEL_ROOT: UiTag = UiTag::from_str("ae.hierarchy.panel_root");

/// Semantic [`UiTag`] assigned to the Search Bar text input container.
pub const HIERARCHY_TAG_SEARCH_INPUT: UiTag = UiTag::from_str("ae.hierarchy.search_input");

/// Semantic [`UiTag`] assigned to the Search Bar clear `✖` button.
pub const HIERARCHY_TAG_SEARCH_CLEAR: UiTag = UiTag::from_str("ae.hierarchy.search_clear");

/// Semantic [`UiTag`] assigned to the Header `➕` Add Menu toggle button.
pub const HIERARCHY_TAG_ADD_BUTTON: UiTag = UiTag::from_str("ae.hierarchy.add_button");

/// Semantic [`UiTag`] assigned to the Header `🗑` Delete Selected button.
pub const HIERARCHY_TAG_DELETE_BUTTON: UiTag = UiTag::from_str("ae.hierarchy.delete_button");

/// Semantic [`UiTag`] assigned to the Hierarchy virtual rows scroll viewport.
pub const HIERARCHY_TAG_VIEWPORT: UiTag = UiTag::from_str("ae.hierarchy.viewport");

/// Semantic [`UiTag`] assigned to the Hierarchy scrollbar track.
pub const HIERARCHY_TAG_SCROLLBAR_TRACK: UiTag = UiTag::from_str("ae.hierarchy.scrollbar_track");

/// Semantic [`UiTag`] assigned to the Hierarchy draggable scrollbar thumb.
pub const HIERARCHY_TAG_SCROLLBAR_THUMB: UiTag = UiTag::from_str("ae.hierarchy.scrollbar_thumb");

/// Base [`UiTag`] used to derive per-row selection tags via [`UiTag::indexed`].
pub const HIERARCHY_TAG_ROW_BASE: UiTag = UiTag::from_str("ae.hierarchy.row");

/// Base [`UiTag`] used to derive per-row eye visibility tags via [`UiTag::indexed`].
pub const HIERARCHY_TAG_EYE_BASE: UiTag = UiTag::from_str("ae.hierarchy.eye");

/// Base [`UiTag`] used to derive per-row foldout chevron tags via [`UiTag::indexed`].
pub const HIERARCHY_TAG_FOLDOUT_BASE: UiTag = UiTag::from_str("ae.hierarchy.foldout");

/// Derives a deterministic 64-bit tag for an interactive entity row at `row_idx`.
#[inline]
#[must_use]
pub const fn make_row_tag(row_idx: usize) -> u64 {
    HIERARCHY_TAG_ROW_BASE.indexed(row_idx as u64).raw()
}

/// Derives a deterministic 64-bit tag for an entity eye visibility button at `row_idx`.
#[inline]
#[must_use]
pub const fn make_eye_tag(row_idx: usize) -> u64 {
    HIERARCHY_TAG_EYE_BASE.indexed(row_idx as u64).raw()
}

/// Derives a deterministic 64-bit tag for an entity branch foldout chevron at `row_idx`.
#[inline]
#[must_use]
pub const fn make_foldout_tag(row_idx: usize) -> u64 {
    HIERARCHY_TAG_FOLDOUT_BASE.indexed(row_idx as u64).raw()
}

/// Read-only view parameters passed to the Hierarchy panel declarative builder.
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
    /// Caret blink phase indicator for text inputs.
    pub blink_caret: bool,
    /// Byte offset of the text cursor inside [`Self::search_query`].
    pub cursor_byte_idx: usize,
}

/// Persistent UI state owned by the Scene Hierarchy panel.
///
/// All popup open/close flags and context menu visibility are owned directly by
/// [`UiTree`](irisui::core::UiTree); flattened row caches live inside [`TreeModel`].
#[derive(Debug, Clone)]
pub struct HierarchyPanelState {
    /// Search input text state managing filter buffer, cursor, and selection.
    pub search_input: TextInputState,
    /// Cached flattened DFS entity rows and filtered view managed by `irisui-widgets`.
    pub model: TreeModel<hecs::Entity>,
    /// Unified TreeState managing expanded and selected entity nodes.
    pub tree_state: TreeState<hecs::Entity>,
}

impl Default for HierarchyPanelState {
    #[inline]
    fn default() -> Self {
        Self {
            search_input: TextInputState::default(),
            model: TreeModel::new(),
            tree_state: TreeState::expanded_by_default(),
        }
    }
}

/// Semantic alias for [`HierarchyPanelState`].
pub type HierarchyState = HierarchyPanelState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_default_state() {
        let state = HierarchyPanelState::default();
        assert_eq!(state.model.total_count(), 0);
        assert!(state.tree_state.default_expanded);
    }

    #[test]
    fn test_hierarchy_tags_are_unique_and_non_zero() {
        let tags = [
            HIERARCHY_TAG_PANEL_ROOT.raw(),
            HIERARCHY_TAG_SEARCH_INPUT.raw(),
            HIERARCHY_TAG_SEARCH_CLEAR.raw(),
            HIERARCHY_TAG_ADD_BUTTON.raw(),
            HIERARCHY_TAG_DELETE_BUTTON.raw(),
            HIERARCHY_TAG_VIEWPORT.raw(),
            HIERARCHY_TAG_SCROLLBAR_TRACK.raw(),
            HIERARCHY_TAG_SCROLLBAR_THUMB.raw(),
            make_row_tag(0),
            make_row_tag(1),
            make_eye_tag(0),
            make_foldout_tag(0),
        ];
        for (i, &t1) in tags.iter().enumerate() {
            assert_ne!(t1, 0);
            for (j, &t2) in tags.iter().enumerate() {
                if i != j {
                    assert_ne!(t1, t2);
                }
            }
        }
    }
}