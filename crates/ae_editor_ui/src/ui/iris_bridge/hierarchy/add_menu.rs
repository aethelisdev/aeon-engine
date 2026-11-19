// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy `➕` Static Cascading Add Menu
//!
//! Defines the compile-time static menu trees (`const &[CascadingMenuEntry]`) for
//! spawning 3D shapes, 2D sprites, UI Canvas widgets, asset imports, and benchmarks
//! with zero heap allocations and direct `UiTree` + `ActionBus` wiring.

use super::types::{AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HierarchyPanelParams};
use crate::ui::UiElementType;
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_FOLDER, ICON_SPHERE};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

const ADD_MENU_HUD_PRESETS: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::item_with_icon(
        "Health Bar (Player Tag)",
        UiIcon::Glyph("❤️"),
        EngineUiAction::SpawnUiElement(UiElementType::HealthBar),
    ),
    CascadingMenuEntry::item_with_icon(
        "Score Display (Score Tag)",
        UiIcon::Glyph("⭐"),
        EngineUiAction::SpawnUiElement(UiElementType::ScoreDisplay),
    ),
];

const ADD_MENU_UI_CANVAS: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::item_with_icon(
        "Panel / Canvas Box",
        UiIcon::Glyph("🟩"),
        EngineUiAction::SpawnUiElement(UiElementType::Panel),
    ),
    CascadingMenuEntry::item_with_icon(
        "Text Label",
        UiIcon::Glyph("🔤"),
        EngineUiAction::SpawnUiElement(UiElementType::Text),
    ),
    CascadingMenuEntry::item_with_icon(
        "Interactive Button",
        UiIcon::Glyph("🔘"),
        EngineUiAction::SpawnUiElement(UiElementType::Button),
    ),
    CascadingMenuEntry::item_with_icon(
        "Image / Icon",
        UiIcon::Glyph("🖼️"),
        EngineUiAction::SpawnUiElement(UiElementType::Image),
    ),
    CascadingMenuEntry::item_with_icon(
        "Progress Bar",
        UiIcon::Glyph("📊"),
        EngineUiAction::SpawnUiElement(UiElementType::ProgressBar),
    ),
    CascadingMenuEntry::item_with_icon(
        "Numeric Slider",
        UiIcon::Glyph("🎚️"),
        EngineUiAction::SpawnUiElement(UiElementType::Slider),
    ),
    CascadingMenuEntry::item_with_icon(
        "Toggle Checkbox",
        UiIcon::Glyph("☑️"),
        EngineUiAction::SpawnUiElement(UiElementType::Checkbox),
    ),
    CascadingMenuEntry::item_with_icon(
        "Text Input Field",
        UiIcon::Glyph("📝"),
        EngineUiAction::SpawnUiElement(UiElementType::TextInput),
    ),
    CascadingMenuEntry::separator(),
    CascadingMenuEntry::submenu(
        "HUD Presets",
        Some(UiIcon::Glyph("🎮")),
        AddSubmenuId::HudPresets.tag(),
        ADD_MENU_HUD_PRESETS,
    ),
];

const ADD_MENU_ASSETS_PREFABS: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::item_with_icon(
        "3D Model...",
        UiIcon::Atlas(ICON_CUBE),
        EngineUiAction::OpenModelDialog,
    ),
    CascadingMenuEntry::item_with_icon(
        "Load Prefab...",
        UiIcon::Atlas(ICON_FOLDER),
        EngineUiAction::OpenLoadPrefabDialog,
    ),
];

const ADD_MENU_2D_OBJECTS: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::item_with_icon(
        "Sprite",
        UiIcon::Glyph("🖼️"),
        EngineUiAction::SpawnDefaultSprite,
    ),
    CascadingMenuEntry::item_with_icon(
        "Player Sprite",
        UiIcon::Glyph("🏃"),
        EngineUiAction::SpawnPlayerSprite,
    ),
    CascadingMenuEntry::item_with_icon(
        "Empty 2D Object",
        UiIcon::Glyph("📦"),
        EngineUiAction::SpawnEmpty2D,
    ),
];

const ADD_MENU_3D_OBJECTS: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::item_with_icon(
        "Cube",
        UiIcon::Atlas(ICON_CUBE),
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Cube),
    ),
    CascadingMenuEntry::item_with_icon(
        "Sphere",
        UiIcon::Atlas(ICON_SPHERE),
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Sphere),
    ),
    CascadingMenuEntry::item_with_icon(
        "Cylinder",
        UiIcon::Glyph("🧪"),
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Cylinder),
    ),
    CascadingMenuEntry::item_with_icon(
        "Capsule",
        UiIcon::Glyph("💊"),
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Capsule),
    ),
    CascadingMenuEntry::item_with_icon(
        "Torus",
        UiIcon::Glyph("🍩"),
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Torus),
    ),
    CascadingMenuEntry::item_with_icon(
        "Triangle",
        UiIcon::Glyph("📐"),
        EngineUiAction::SpawnShape(ae_core::ecs::Shape::Triangle),
    ),
];

const ADD_MENU_STRESS_BENCHMARKS: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::item_with_icon(
        "OpenWorld (10km)",
        UiIcon::Glyph("🏰"),
        EngineUiAction::AaaOpenWorldTest,
    ),
    CascadingMenuEntry::item_with_icon(
        "10,000 Entities",
        UiIcon::Glyph("⚡"),
        EngineUiAction::StressTest(10_000),
    ),
    CascadingMenuEntry::item_with_icon(
        "100,000 Entities",
        UiIcon::Glyph("⚡"),
        EngineUiAction::StressTest(100_000),
    ),
    CascadingMenuEntry::item_with_icon(
        "10,000,000 Universe",
        UiIcon::Glyph("⚡"),
        EngineUiAction::StressTest(10_000_000),
    ),
    CascadingMenuEntry::item_with_icon("Explode!", UiIcon::Glyph("💥"), EngineUiAction::Explode),
];

const ADD_MENU_ROOT_2D: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::submenu(
        "2D Objects",
        Some(UiIcon::Glyph("🖼️")),
        AddSubmenuId::Objects2D.tag(),
        ADD_MENU_2D_OBJECTS,
    ),
    CascadingMenuEntry::submenu(
        "UI & Canvas",
        Some(UiIcon::Glyph("🎨")),
        AddSubmenuId::UiCanvas.tag(),
        ADD_MENU_UI_CANVAS,
    ),
    CascadingMenuEntry::submenu(
        "Assets & Prefabs",
        Some(UiIcon::Atlas(ICON_FOLDER)),
        AddSubmenuId::AssetsPrefabs.tag(),
        ADD_MENU_ASSETS_PREFABS,
    ),
];

const ADD_MENU_ROOT_3D: &[CascadingMenuEntry<'static, EngineUiAction>] = &[
    CascadingMenuEntry::submenu(
        "3D Objects",
        Some(UiIcon::Atlas(ICON_CUBE)),
        AddSubmenuId::Objects3D.tag(),
        ADD_MENU_3D_OBJECTS,
    ),
    CascadingMenuEntry::submenu(
        "UI & Canvas",
        Some(UiIcon::Glyph("🎨")),
        AddSubmenuId::UiCanvas.tag(),
        ADD_MENU_UI_CANVAS,
    ),
    CascadingMenuEntry::submenu(
        "Assets & Prefabs",
        Some(UiIcon::Atlas(ICON_FOLDER)),
        AddSubmenuId::AssetsPrefabs.tag(),
        ADD_MENU_ASSETS_PREFABS,
    ),
    CascadingMenuEntry::separator(),
    CascadingMenuEntry::item_with_icon(
        "Phase 1 Test Sandbox",
        UiIcon::Glyph("🎮"),
        EngineUiAction::SpawnPhase1TestSandbox,
    ),
    CascadingMenuEntry::submenu(
        "Stress Benchmarks",
        Some(UiIcon::Glyph("⚡")),
        AddSubmenuId::StressBenchmarks.tag(),
        ADD_MENU_STRESS_BENCHMARKS,
    ),
];

/// Returns the compile-time static menu hierarchy for either 2D or 3D editor mode.
#[inline]
#[must_use]
pub const fn hierarchy_add_menu_entries(
    is_2d: bool,
) -> &'static [CascadingMenuEntry<'static, EngineUiAction>] {
    if is_2d {
        ADD_MENU_ROOT_2D
    } else {
        ADD_MENU_ROOT_3D
    }
}

/// Draws the cascading `➕` Add Menu directly into [`UiScope`] when `HIERARCHY_TAG_ADD_BUTTON`
/// is active, emitting clicked spawn actions directly to [`ActionBus`].
pub fn draw_hierarchy_add_menu(
    scope: &mut UiScope<'_>,
    params: &HierarchyPanelParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) -> Option<Rect> {
    scope.scoped_cascading_menu_anchored(
        "HierarchyAddMenu",
        HIERARCHY_TAG_ADD_BUTTON,
        CascadingPopupConfig::new(185.0, PopupPlacement::Below).with_fallback_rect(Rect::new(
            (params.panel_rect.x + params.panel_rect.width - 56.0).max(params.panel_rect.x + 60.0),
            params.panel_rect.y + 4.0,
            24.0,
            24.0,
        )),
        hierarchy_add_menu_entries(params.is_2d),
        |action| action_bus.emit(action.clone()),
    )
}