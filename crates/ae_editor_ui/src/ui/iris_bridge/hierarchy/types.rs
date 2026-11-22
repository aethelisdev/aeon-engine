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

/// Persistent UI state owned by the Scene Hierarchy panel.
///
/// Popup visibility, expansion state, selection, and flattened DFS row caches are all
/// managed autonomously inside `irisui` keyed by semantic tags.
#[derive(Debug, Clone, Default)]
pub struct HierarchyPanelState {
    /// Search input text state managing filter buffer, cursor, and selection.
    pub search_input: TextInputState,
}

/// Semantic alias for [`HierarchyPanelState`].
pub type HierarchyState = HierarchyPanelState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_default_state() {
        let state = HierarchyPanelState::default();
        assert!(state.search_input.text().is_empty());
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