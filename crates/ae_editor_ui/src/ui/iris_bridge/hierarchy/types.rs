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
}

/// Active hierarchical submenu currently open within the `➕` Add Menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSubmenuId {
    /// 3D primitive geometry submenu (Cube, Sphere, Plane, Cylinder, Capsule, Torus, Triangle).
    Objects3D,
    /// 2D Canvas & UI elements submenu (Panel, Label, Image, Button, Progress Bar, Slider, Checkbox, Input).
    UiCanvas,
    /// Preset in-game HUD widgets (HealthBar, ScoreDisplay).
    HudPresets,
    /// Assets and prefabs submenu (Load Model, Load Prefab).
    AssetsPrefabs,
    /// Stress testing and performance benchmarks submenu.
    StressBenchmarks,
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
    /// Opens the right-click context menu for an entity at the cursor position.
    OpenContextMenu(hecs::Entity, Point),
    /// Closes the right-click context menu.
    CloseContextMenu,
}

/// Hit-testing targets for interactive elements in the Hierarchy panel.
#[derive(Debug, Clone, Default)]
pub struct HierarchyPanelTargets {
    /// Total bounding rectangle of the docked hierarchy panel.
    pub panel_rect: Rect,
    /// Search bar text input rectangle.
    pub search_input_rect: Rect,
    /// Search clear `✖` button rectangle.
    pub search_clear_btn_rect: Option<Rect>,
    /// Header `➕` Add button rectangle.
    pub add_btn_rect: Rect,
    /// Header `🗑` Delete selected button rectangle (visible when an entity is selected).
    pub delete_btn_rect: Option<Rect>,
    /// Bounding rectangle of the scrollable row container.
    pub scroll_container_rect: Rect,
    /// Clickable entity rows: `(entity, row_rect, eye_btn_rect, foldout_rect)`.
    pub entity_rows: Vec<(hecs::Entity, Rect, Rect, Option<Rect>)>,
    /// Bounding rectangle of the active Add Menu root card (if open).
    pub active_add_menu_rect: Option<Rect>,
    /// Bounding rectangle of the active Add Menu submenu card (if open).
    pub active_submenu_rect: Option<Rect>,
    /// Add menu main category item targets: `(item_rect, submenu_id_or_action)`.
    pub add_menu_items: Vec<(Rect, Result<AddSubmenuId, HierarchyAction>)>,
    /// Add menu submenu item targets: `(item_rect, action)`.
    pub submenu_items: Vec<(Rect, HierarchyAction)>,
    /// Right-click context menu target: `(target_entity, menu_rect, delete_btn_rect, toggle_vis_btn_rect)`.
    pub active_context_menu: Option<(hecs::Entity, Rect, Rect, Rect)>,
}

/// Parameters passed to the Hierarchy panel builder and value updater.
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
    /// Vertical scroll offset in physical pixels.
    pub scroll_y: f32,
    /// Active cascading Add Menu submenu (if open).
    pub active_submenu: Option<AddSubmenuId>,
    /// Whether the Add Menu is open.
    pub is_add_menu_open: bool,
    /// Active right-click context menu (target entity, position) if open.
    pub active_context_menu: Option<(hecs::Entity, Point)>,
    /// Current mouse cursor position.
    pub cursor_pos: Point,
    /// Search input focused state.
    pub is_search_focused: bool,
}