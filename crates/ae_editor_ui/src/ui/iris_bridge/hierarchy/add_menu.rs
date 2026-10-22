// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy `➕` Cascading Add Menu Builder
//!
//! Provides the data-driven cascading multi-level dropdown menu tree for spawning 3D shapes,
//! 2D UI elements, HUD presets, asset imports, and stress test benchmarks via [`CascadingMenuBuilder`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use super::types::{AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HierarchyAction, HierarchyPanelParams};
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_FOLDER, ICON_SPHERE};
use irisui::prelude::*;
use irisui::widgets::cascading_menu::{CascadingMenuIcon, CascadingMenuItem};
use irisui::widgets::declarative::{PopupPlacement, hash_label};

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

/// Builds the cascading `➕` Add Menu in the [`UiTree`] using pure declarative [`UiScope`] popups.
///
/// Dispatches menu actions, submenu activations, and dismiss triggers autonomously via
/// [`WidgetResponse`] without leaking event loop machinery to the engine layer.
///
/// Returns the list of bounding [`Rect`]s for all open cascading menu levels.
pub fn build_add_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    actions: &mut Vec<HierarchyAction>,
) -> Vec<Rect> {
    if !params.is_add_menu_open {
        return Vec::new();
    }

    let anchor = tree
        .node_rect_tagged(HIERARCHY_TAG_ADD_BUTTON, params.panel_rect.origin())
        .filter(|r| r.width > 0.0 && r.height > 0.0)
        .unwrap_or(Rect::new(
            (params.panel_rect.x + params.panel_rect.width - 56.0).max(params.panel_rect.x + 60.0),
            params.panel_rect.y + 4.0,
            24.0,
            24.0,
        ));

    let menu_items = get_hierarchy_add_menu_items(params.is_2d);
    let root_rect = render_root_add_menu(tree, parent_id, anchor, &menu_items, params, actions);
    let mut rendered_rects = vec![root_rect];
    if let Some(sub_id) = tree.find_node_by_name("HierarchySubmenu")
        && let Some(sub_node) = tree.get(sub_id)
    {
        rendered_rects.push(sub_node.computed_rect);
    }
    if let Some(s2_id) = tree.find_node_by_name("HierarchySubSubmenu")
        && let Some(s2_node) = tree.get(s2_id)
    {
        rendered_rects.push(s2_node.computed_rect);
    }

    rendered_rects
}

fn render_root_add_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    anchor_rect: Rect,
    menu_items: &[CascadingMenuItem],
    params: &HierarchyPanelParams<'_>,
    actions: &mut Vec<HierarchyAction>,
) -> Rect {
    let parent_tag = hash_label("HierarchyAddMenu");
    if let Some(sub) = params.active_submenu {
        tree.set_active_submenu(parent_tag, sub.to_tag());
    }
    if let (Some(sub), Some(s2)) = (params.active_submenu, params.active_sub_submenu) {
        tree.set_active_submenu(sub.to_tag(), s2.to_tag());
    }

    UiScope::new(tree, parent_id)
        .popup_anchored_with_placement(
            "HierarchyAddMenu",
            anchor_rect,
            185.0,
            PopupPlacement::Below,
            |card| {
                for item in menu_items {
                    if item.is_separator {
                        card.separator();
                        continue;
                    }

                    let icon_str = match item.icon {
                        Some(CascadingMenuIcon::Text(t)) => t,
                        _ => "",
                    };

                    if item.has_submenu() {
                        let is_branch_active =
                            params.active_submenu.map(|s| s.to_tag()) == Some(item.tag);
                        let resp = card.submenu_named(
                            "HierarchySubmenu",
                            &item.label,
                            Some(icon_str),
                            item.tag,
                            |sub_card| {
                                if let Some(children) = &item.submenu {
                                    for child in children {
                                        if child.is_separator {
                                            sub_card.separator();
                                            continue;
                                        }
                                        let c_icon = match child.icon {
                                            Some(CascadingMenuIcon::Text(t)) => t,
                                            _ => "",
                                        };
                                        if child.has_submenu() {
                                            let is_s2_active =
                                                params.active_sub_submenu.map(|s| s.to_tag())
                                                    == Some(child.tag);
                                            let sub_resp = sub_card.submenu_named(
                                                "HierarchySubSubmenu",
                                                &child.label,
                                                Some(c_icon),
                                                child.tag,
                                                |s2_card| {
                                                    if let Some(s2_children) = &child.submenu {
                                                        for s2_child in s2_children {
                                                            if s2_child.is_separator {
                                                                s2_card.separator();
                                                                continue;
                                                            }
                                                            let s2_icon = match s2_child.icon {
                                                                Some(CascadingMenuIcon::Text(
                                                                    t,
                                                                )) => t,
                                                                _ => "",
                                                            };
                                                            let s2_resp = s2_card.dropdown_item(
                                                                s2_child.tag,
                                                                s2_icon,
                                                                &s2_child.label,
                                                                None,
                                                                true,
                                                            );
                                                            if s2_resp.clicked() {
                                                                if let Some(act) =
                                                                    get_hierarchy_add_menu_action(
                                                                        s2_child.tag,
                                                                    )
                                                                {
                                                                    actions.push(act);
                                                                }
                                                                actions.push(
                                                                    HierarchyAction::CloseAddMenu,
                                                                );
                                                            }
                                                        }
                                                    }
                                                },
                                            );
                                            if (sub_resp.hovered() || sub_resp.clicked())
                                                && !is_s2_active
                                                && let Some(s2) = AddSubmenuId::from_tag(child.tag)
                                            {
                                                actions.push(HierarchyAction::OpenSubSubmenu(s2));
                                            }
                                        } else {
                                            let sub_resp = sub_card.dropdown_item(
                                                child.tag,
                                                c_icon,
                                                &child.label,
                                                None,
                                                true,
                                            );
                                            if sub_resp.clicked() {
                                                if let Some(act) =
                                                    get_hierarchy_add_menu_action(child.tag)
                                                {
                                                    actions.push(act);
                                                }
                                                actions.push(HierarchyAction::CloseAddMenu);
                                            }
                                        }
                                    }
                                }
                            },
                        );
                        if (resp.hovered() || resp.clicked())
                            && !is_branch_active
                            && let Some(sub) = AddSubmenuId::from_tag(item.tag)
                        {
                            actions.push(HierarchyAction::OpenSubmenu(sub));
                        }
                    } else {
                        let resp = card.dropdown_item(item.tag, icon_str, &item.label, None, true);
                        if resp.clicked() {
                            if let Some(act) = get_hierarchy_add_menu_action(item.tag) {
                                actions.push(act);
                            }
                            actions.push(HierarchyAction::CloseAddMenu);
                        }
                    }
                }
            },
        )
        .response
        .rect
}