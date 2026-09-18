// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy `➕` Cascading Add Menu Builder
//!
//! Provides the data-driven cascading multi-level dropdown menu tree for spawning 3D shapes,
//! 2D UI elements, HUD presets, asset imports, and stress test benchmarks via [`CascadingMenuBuilder`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use super::types::{AddSubmenuId, HierarchyAction, HierarchyPanelParams, HierarchyPanelTargets};
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_FOLDER, ICON_SPHERE};
use irisui::prelude::*;

/// Resolves a numeric menu item tag into its corresponding [`HierarchyAction`].
#[inline]
pub fn get_hierarchy_add_menu_action(tag: u64) -> Option<HierarchyAction> {
    match tag {
        // 3D Shapes
        101 => Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Cube)),
        102 => Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Sphere)),
        103 => Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Cylinder)),
        104 => Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Capsule)),
        105 => Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Torus)),
        106 => Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Triangle)),

        // 2D Objects
        201 => Some(HierarchyAction::SpawnDefaultSprite),
        202 => Some(HierarchyAction::SpawnPlayerSprite),
        203 => Some(HierarchyAction::SpawnEmpty2D),

        // UI & Canvas Elements
        301 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::Panel,
        )),
        302 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::Text,
        )),
        303 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::Button,
        )),
        304 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::Image,
        )),
        305 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::ProgressBar,
        )),
        306 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::Slider,
        )),
        307 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::Checkbox,
        )),
        308 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::TextInput,
        )),
        309 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::HealthBar,
        )),
        310 => Some(HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::ScoreDisplay,
        )),

        // Asset Imports
        401 => Some(HierarchyAction::OpenModelDialog),
        402 => Some(HierarchyAction::OpenLoadPrefabDialog),

        // Benchmarks & Sandboxes
        501 => Some(HierarchyAction::SpawnPhase1TestSandbox),
        502 => Some(HierarchyAction::AaaOpenWorldTest),
        503 => Some(HierarchyAction::StressTest(10_000)),
        504 => Some(HierarchyAction::StressTest(100_000)),
        505 => Some(HierarchyAction::StressTest(10_000_000)),
        506 => Some(HierarchyAction::Explode),

        _ => None,
    }
}

/// Constructs the declarative cascading menu item hierarchy for Scene Hierarchy.
pub fn get_hierarchy_add_menu_items(is_2d: bool) -> Vec<CascadingMenuItem> {
    let ui_canvas_items = vec![
        CascadingMenuItem::item_with_icon("Panel / Canvas Box", CascadingMenuIcon::Text("🟩"), 301),
        CascadingMenuItem::item_with_icon("Text Label", CascadingMenuIcon::Text("🔤"), 302),
        CascadingMenuItem::item_with_icon("Interactive Button", CascadingMenuIcon::Text("🔘"), 303),
        CascadingMenuItem::item_with_icon("Image / Icon", CascadingMenuIcon::Text("🖼️"), 304),
        CascadingMenuItem::item_with_icon("Progress Bar", CascadingMenuIcon::Text("📊"), 305),
        CascadingMenuItem::item_with_icon("Numeric Slider", CascadingMenuIcon::Text("🎚️"), 306),
        CascadingMenuItem::item_with_icon("Toggle Checkbox", CascadingMenuIcon::Text("☑️"), 307),
        CascadingMenuItem::item_with_icon("Text Input Field", CascadingMenuIcon::Text("📝"), 308),
        CascadingMenuItem::separator(),
        CascadingMenuItem::branch(
            "HUD Presets",
            Some(CascadingMenuIcon::Text("🎮")),
            AddSubmenuId::HudPresets.to_tag(),
            vec![
                CascadingMenuItem::item_with_icon(
                    "Health Bar (Player Tag)",
                    CascadingMenuIcon::Text("❤️"),
                    309,
                ),
                CascadingMenuItem::item_with_icon(
                    "Score Display (Score Tag)",
                    CascadingMenuIcon::Text("⭐"),
                    310,
                ),
            ],
        ),
    ];

    let asset_items = vec![
        CascadingMenuItem::item_with_icon(
            "3D Model...",
            CascadingMenuIcon::Texture(ICON_CUBE),
            401,
        ),
        CascadingMenuItem::item_with_icon(
            "Load Prefab...",
            CascadingMenuIcon::Texture(ICON_FOLDER),
            402,
        ),
    ];

    if is_2d {
        vec![
            CascadingMenuItem::branch(
                "2D Objects",
                Some(CascadingMenuIcon::Text("🖼️")),
                AddSubmenuId::Objects2D.to_tag(),
                vec![
                    CascadingMenuItem::item_with_icon("Sprite", CascadingMenuIcon::Text("🖼️"), 201),
                    CascadingMenuItem::item_with_icon(
                        "Player Sprite",
                        CascadingMenuIcon::Text("🏃"),
                        202,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "Empty 2D Object",
                        CascadingMenuIcon::Text("📦"),
                        203,
                    ),
                ],
            ),
            CascadingMenuItem::branch(
                "UI & Canvas",
                Some(CascadingMenuIcon::Text("🎨")),
                AddSubmenuId::UiCanvas.to_tag(),
                ui_canvas_items,
            ),
            CascadingMenuItem::branch(
                "Assets & Prefabs",
                Some(CascadingMenuIcon::Texture(ICON_FOLDER)),
                AddSubmenuId::AssetsPrefabs.to_tag(),
                asset_items,
            ),
        ]
    } else {
        vec![
            CascadingMenuItem::branch(
                "3D Objects",
                Some(CascadingMenuIcon::Texture(ICON_CUBE)),
                AddSubmenuId::Objects3D.to_tag(),
                vec![
                    CascadingMenuItem::item_with_icon(
                        "Cube",
                        CascadingMenuIcon::Texture(ICON_CUBE),
                        101,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "Sphere",
                        CascadingMenuIcon::Texture(ICON_SPHERE),
                        102,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "Cylinder",
                        CascadingMenuIcon::Text("🧪"),
                        103,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "Capsule",
                        CascadingMenuIcon::Text("💊"),
                        104,
                    ),
                    CascadingMenuItem::item_with_icon("Torus", CascadingMenuIcon::Text("🍩"), 105),
                    CascadingMenuItem::item_with_icon(
                        "Triangle",
                        CascadingMenuIcon::Text("📐"),
                        106,
                    ),
                ],
            ),
            CascadingMenuItem::branch(
                "UI & Canvas",
                Some(CascadingMenuIcon::Text("🎨")),
                AddSubmenuId::UiCanvas.to_tag(),
                ui_canvas_items,
            ),
            CascadingMenuItem::branch(
                "Assets & Prefabs",
                Some(CascadingMenuIcon::Texture(ICON_FOLDER)),
                AddSubmenuId::AssetsPrefabs.to_tag(),
                asset_items,
            ),
            CascadingMenuItem::separator(),
            CascadingMenuItem::item_with_icon(
                "Phase 1 Test Sandbox",
                CascadingMenuIcon::Text("🎮"),
                501,
            ),
            CascadingMenuItem::branch(
                "Stress Benchmarks",
                Some(CascadingMenuIcon::Text("⚡")),
                AddSubmenuId::StressBenchmarks.to_tag(),
                vec![
                    CascadingMenuItem::item_with_icon(
                        "OpenWorld (10km)",
                        CascadingMenuIcon::Text("🏰"),
                        502,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "10,000 Entities",
                        CascadingMenuIcon::Text("⚡"),
                        503,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "100,000 Entities",
                        CascadingMenuIcon::Text("⚡"),
                        504,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "10,000,000 Universe",
                        CascadingMenuIcon::Text("⚡"),
                        505,
                    ),
                    CascadingMenuItem::item_with_icon(
                        "Explode!",
                        CascadingMenuIcon::Text("💥"),
                        506,
                    ),
                ],
            ),
        ]
    }
}

/// Builds the cascading `➕` Add Menu in the [`UiTree`] using [`CascadingMenuBuilder`].
pub fn build_add_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) {
    targets.active_add_menu_rects.clear();

    if !params.is_add_menu_open {
        return;
    }

    let menu_items = get_hierarchy_add_menu_items(params.is_2d);

    let mut active_path = Vec::new();
    if let Some(sub) = params.active_submenu {
        active_path.push(sub.to_tag());
        if let Some(sub_sub) = params.active_sub_submenu {
            active_path.push(sub_sub.to_tag());
        }
    }

    if let Some(frame) = CascadingMenuBuilder::new(targets.add_btn_rect, &menu_items, &active_path)
        .cursor_pos(params.cursor_pos)
        .viewport_bounds(params.panel_rect)
        .name("HierarchyAddMenu")
        .build(tree, parent_id)
    {
        targets.active_add_menu_rects = frame.rendered_popup_rects;
    }
}