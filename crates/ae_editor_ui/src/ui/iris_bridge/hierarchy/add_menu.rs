// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy `➕` Cascading Add Menu Builder
//!
//! Renders the cascading multi-level dropdown menus for spawning 3D shapes,
//! 2D UI elements, HUD presets, asset imports, and stress test benchmarks.

use super::types::{AddSubmenuId, HierarchyAction, HierarchyPanelParams, HierarchyPanelTargets};
use irisui::prelude::*;

/// Descriptor for a top-level category item in the Add Menu.
struct AddMenuCategoryItem {
    icon: &'static str,
    label: &'static str,
    has_sub: bool,
    sub_id: Option<AddSubmenuId>,
    action: Option<HierarchyAction>,
}

/// Builds the cascading `➕` Add Menu and its active submenus in the `UiTree`.
pub fn build_add_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) {
    targets.active_add_menu_rect = None;
    targets.active_submenu_rect = None;
    targets.add_menu_items.clear();
    targets.submenu_items.clear();

    if !params.is_add_menu_open {
        return;
    }

    let menu_x = (targets.add_btn_rect.x)
        .min(params.panel_rect.right() - 190.0)
        .max(4.0);
    let menu_y = targets.add_btn_rect.bottom() + 2.0;
    let menu_w = 185.0;

    let root_items = [
        AddMenuCategoryItem {
            icon: "📦",
            label: "3D Objects",
            has_sub: true,
            sub_id: Some(AddSubmenuId::Objects3D),
            action: None,
        },
        AddMenuCategoryItem {
            icon: "🎨",
            label: "UI & Canvas",
            has_sub: true,
            sub_id: Some(AddSubmenuId::UiCanvas),
            action: None,
        },
        AddMenuCategoryItem {
            icon: "📁",
            label: "Assets & Prefabs",
            has_sub: true,
            sub_id: Some(AddSubmenuId::AssetsPrefabs),
            action: None,
        },
        AddMenuCategoryItem {
            icon: "─",
            label: "",
            has_sub: false,
            sub_id: None,
            action: None,
        },
        AddMenuCategoryItem {
            icon: "🎮",
            label: "Phase 1 Test Sandbox",
            has_sub: false,
            sub_id: None,
            action: Some(HierarchyAction::SpawnPhase1TestSandbox),
        },
        AddMenuCategoryItem {
            icon: "⚡",
            label: "Stress Benchmarks",
            has_sub: true,
            sub_id: Some(AddSubmenuId::StressBenchmarks),
            action: None,
        },
    ];

    let item_h = 24.0;
    let sep_h = 5.0;
    let total_h = 5.0 * item_h + sep_h + 8.0;

    let card_rect = Rect::new(menu_x, menu_y, menu_w, total_h);
    targets.active_add_menu_rect = Some(card_rect);

    // Root Add Menu Card Container
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("AddMenuCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85)) // Cyan border
            .border_radius(6.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.80));
    }
    let _ = tree.add_child(parent_id, card_id);

    let mut cur_y = menu_y + 4.0;
    let mut submenu_anchor_y = cur_y;

    for item in &root_items {
        if item.icon == "─" {
            let sep_id = tree.create_node();
            if let Some(node) = tree.get_mut(sep_id) {
                node.set_name("MenuSeparator");
                node.computed_rect = Rect::new(menu_x + 6.0, cur_y + 2.0, menu_w - 12.0, 1.0);
                node.style = Style::new().background(Color::rgba(0.18, 0.20, 0.26, 0.70));
            }
            let _ = tree.add_child(card_id, sep_id);
            cur_y += sep_h;
            continue;
        }

        let item_rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, item_h);
        let is_hovered = item_rect.contains_point(params.cursor_pos);
        let is_active_sub = item.sub_id.is_some() && item.sub_id == params.active_submenu;

        if is_active_sub {
            submenu_anchor_y = cur_y;
        }

        let (bg, text_col) = if is_active_sub || is_hovered {
            (
                Color::rgba(0.0, 0.35, 0.45, 0.80),
                Color::rgba(0.0, 0.95, 1.0, 1.0),
            )
        } else {
            (Color::TRANSPARENT, Color::rgba(0.88, 0.90, 0.96, 1.0))
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name(format!("AddMenuItem_{}", item.label));
            node.computed_rect = item_rect;
            node.style = Style::new().background(bg).border_radius(4.0);
        }
        let _ = tree.add_child(card_id, row_id);

        // Icon
        let ic_id = tree.create_node();
        if let Some(node) = tree.get_mut(ic_id) {
            node.set_name("ItemIcon");
            node.set_text(item.icon);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.computed_rect = Rect::new(item_rect.x + 6.0, cur_y, 16.0, item_h);
        }
        let _ = tree.add_child(row_id, ic_id);

        // Label
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("ItemLabel");
            node.set_text(item.label);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.text_color = text_col;
            node.computed_rect =
                Rect::new(item_rect.x + 24.0, cur_y, item_rect.width - 40.0, item_h);
        }
        let _ = tree.add_child(row_id, lbl_id);

        // Submenu arrow "▸"
        if item.has_sub {
            let arw_id = tree.create_node();
            if let Some(node) = tree.get_mut(arw_id) {
                node.set_name("SubmenuArrow");
                node.set_text("▸");
                node.font_size = 10.0;
                node.line_height = item_h;
                node.text_align = TextAlign::Right;
                node.text_color = Color::rgba(0.60, 0.63, 0.72, 0.85);
                node.computed_rect = Rect::new(item_rect.right() - 16.0, cur_y, 12.0, item_h);
            }
            let _ = tree.add_child(row_id, arw_id);
        }

        let target_payload = if let Some(sid) = item.sub_id {
            Ok(sid)
        } else if let Some(ref action) = item.action {
            Err(action.clone())
        } else {
            continue;
        };

        targets.add_menu_items.push((item_rect, target_payload));
        cur_y += item_h;
    }

    // Build Active Submenu if open
    if let Some(submenu_id) = params.active_submenu {
        build_submenu(
            tree,
            parent_id,
            menu_x + menu_w + 2.0,
            submenu_anchor_y,
            submenu_id,
            params,
            targets,
        );
    }
}

/// Builds an active cascading submenu.
fn build_submenu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    sub_x: f32,
    sub_y: f32,
    submenu_id: AddSubmenuId,
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) {
    let items: Vec<(&str, &str, Option<HierarchyAction>, Option<AddSubmenuId>)> = match submenu_id {
        AddSubmenuId::Objects3D => vec![
            (
                "📦",
                "Cube",
                Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Cube)),
                None,
            ),
            (
                "🔮",
                "Sphere",
                Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Sphere)),
                None,
            ),
            (
                "🧪",
                "Cylinder",
                Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Cylinder)),
                None,
            ),
            (
                "💊",
                "Capsule",
                Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Capsule)),
                None,
            ),
            (
                "🍩",
                "Torus",
                Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Torus)),
                None,
            ),
            (
                "📐",
                "Triangle",
                Some(HierarchyAction::SpawnShape(ae_core::ecs::Shape::Triangle)),
                None,
            ),
        ],
        AddSubmenuId::UiCanvas => vec![
            (
                "🟩",
                "Panel / Canvas Box",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::Panel,
                )),
                None,
            ),
            (
                "🔤",
                "Text Label",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::Text,
                )),
                None,
            ),
            (
                "🖼️",
                "Image / Icon",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::Image,
                )),
                None,
            ),
            (
                "🔘",
                "Interactive Button",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::Button,
                )),
                None,
            ),
            (
                "📊",
                "Progress Bar",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::ProgressBar,
                )),
                None,
            ),
            (
                "🎚️",
                "Numeric Slider",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::Slider,
                )),
                None,
            ),
            (
                "☑️",
                "Toggle Checkbox",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::Checkbox,
                )),
                None,
            ),
            (
                "📝",
                "Text Input Field",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::TextInput,
                )),
                None,
            ),
            ("─", "", None, None),
            ("🎮", "HUD Presets ▸", None, Some(AddSubmenuId::HudPresets)),
        ],
        AddSubmenuId::HudPresets => vec![
            (
                "❤️",
                "Health Bar (Player Tag)",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::HealthBar,
                )),
                None,
            ),
            (
                "⭐",
                "Score Display (Score Tag)",
                Some(HierarchyAction::SpawnUiElement(
                    crate::ui::UiElementType::ScoreDisplay,
                )),
                None,
            ),
        ],
        AddSubmenuId::AssetsPrefabs => vec![
            (
                "📁",
                "3D Model...",
                Some(HierarchyAction::OpenModelDialog),
                None,
            ),
            ("📦", "Load Prefab...", None, None), // Triggers rfd::FileDialog in handler
        ],
        AddSubmenuId::StressBenchmarks => vec![
            (
                "🏰",
                "OpenWorld (10km)",
                Some(HierarchyAction::AaaOpenWorldTest),
                None,
            ),
            (
                "⚡",
                "10,000 Entities",
                Some(HierarchyAction::StressTest(10_000)),
                None,
            ),
            (
                "⚡",
                "100,000 Entities",
                Some(HierarchyAction::StressTest(100_000)),
                None,
            ),
            (
                "⚡",
                "10,000,000 Universe",
                Some(HierarchyAction::StressTest(10_000_000)),
                None,
            ),
            ("💥", "Explode!", Some(HierarchyAction::Explode), None),
        ],
    };

    let item_h = 24.0;
    let mut total_h = 8.0;
    for (ic, _, _, _) in &items {
        if *ic == "─" {
            total_h += 5.0;
        } else {
            total_h += item_h;
        }
    }

    let sub_w = 195.0;
    let card_rect = Rect::new(sub_x, sub_y, sub_w, total_h);
    targets.active_submenu_rect = Some(card_rect);

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("AddSubmenuCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
            .border_radius(6.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.80));
    }
    let _ = tree.add_child(parent_id, card_id);

    let mut cur_y = sub_y + 4.0;

    for (icon, label, action_opt, sub_opt) in items {
        if icon == "─" {
            let sep_id = tree.create_node();
            if let Some(node) = tree.get_mut(sep_id) {
                node.set_name("SubmenuSeparator");
                node.computed_rect = Rect::new(sub_x + 6.0, cur_y + 2.0, sub_w - 12.0, 1.0);
                node.style = Style::new().background(Color::rgba(0.18, 0.20, 0.26, 0.70));
            }
            let _ = tree.add_child(card_id, sep_id);
            cur_y += 5.0;
            continue;
        }

        let item_rect = Rect::new(sub_x + 4.0, cur_y, sub_w - 8.0, item_h);
        let is_hovered = item_rect.contains_point(params.cursor_pos);

        let (bg, text_col) = if is_hovered {
            (
                Color::rgba(0.0, 0.35, 0.45, 0.80),
                Color::rgba(0.0, 0.95, 1.0, 1.0),
            )
        } else {
            (Color::TRANSPARENT, Color::rgba(0.88, 0.90, 0.96, 1.0))
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name(format!("SubItem_{}", label));
            node.computed_rect = item_rect;
            node.style = Style::new().background(bg).border_radius(4.0);
        }
        let _ = tree.add_child(card_id, row_id);

        let ic_id = tree.create_node();
        if let Some(node) = tree.get_mut(ic_id) {
            node.set_name("SubItemIcon");
            node.set_text(icon);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.computed_rect = Rect::new(item_rect.x + 6.0, cur_y, 16.0, item_h);
        }
        let _ = tree.add_child(row_id, ic_id);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("SubItemLabel");
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.text_color = text_col;
            node.computed_rect =
                Rect::new(item_rect.x + 24.0, cur_y, item_rect.width - 28.0, item_h);
        }
        let _ = tree.add_child(row_id, lbl_id);

        if let Some(act) = action_opt {
            targets.submenu_items.push((item_rect, act));
        } else if let Some(sub) = sub_opt {
            targets.add_menu_items.push((item_rect, Ok(sub)));
        }

        cur_y += item_h;
    }
}