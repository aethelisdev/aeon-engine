// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading `➕ Add Component` Menu Builder
//!
//! Renders the 8-category cascading floating dropdown menu for attaching components.

use super::registry::InspectorRegistry;
use super::types::{ComponentCategory, InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;

/// Builds the cascading Add Component menu and its active category submenu.
pub fn build_add_component_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
) {
    targets.active_add_menu_rect = None;
    targets.active_submenu_rect = None;
    targets.add_menu_categories.clear();
    targets.submenu_components.clear();

    if !params.is_add_menu_open {
        return;
    }

    let menu_x = targets.add_component_btn_rect.x;
    let menu_w = 175.0;
    let item_h = 22.0;

    let all_categories = [
        ComponentCategory::Animation,
        ComponentCategory::Audio,
        ComponentCategory::Gameplay,
        ComponentCategory::Hierarchy,
        ComponentCategory::Physics,
        ComponentCategory::Rendering,
        ComponentCategory::UiHud,
        ComponentCategory::CustomDynamic,
    ];

    let categories: Vec<ComponentCategory> = all_categories
        .into_iter()
        .filter(|&cat| category_has_available(cat, params.world, params.selected_entity))
        .collect();

    if categories.is_empty() {
        return;
    }

    let total_h = (categories.len() as f32) * item_h + 8.0;
    let menu_y = (targets.add_component_btn_rect.y - total_h - 2.0).max(30.0);
    let card_rect = Rect::new(menu_x, menu_y, menu_w, total_h);
    targets.active_add_menu_rect = Some(card_rect);

    // Root Add Menu Card
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("AddComponentMenuPopup");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.086, 0.090, 0.106, 0.98))
            .border(1.0, Color::rgba(0.173, 0.180, 0.208, 0.90)) // Clean neutral dark border
            .border_radius(5.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }
    let _ = tree.add_child(parent_id, card_id);

    let mut cur_y = menu_y + 4.0;
    let mut submenu_anchor_y = cur_y;

    for cat in categories {
        let item_rect = Rect::new(menu_x + 4.0, cur_y, menu_w - 8.0, item_h);
        let is_hovered = item_rect.contains_point(params.cursor_pos);
        let is_active_sub = params.active_submenu == Some(cat);

        if is_active_sub {
            submenu_anchor_y = cur_y;
        }

        let (bg, text_col) = if is_active_sub || is_hovered {
            (Color::rgba(0.157, 0.165, 0.188, 0.98), Color::WHITE)
        } else {
            (Color::TRANSPARENT, Color::rgba(0.886, 0.894, 0.918, 1.0))
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name(format!("AddCategory_{:?}", cat));
            node.computed_rect = item_rect;
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(card_id, row_id);

        // Category Icon
        let ic_id = tree.create_node();
        if let Some(node) = tree.get_mut(ic_id) {
            node.set_name("CategoryIcon");
            node.set_text(cat.icon());
            node.font_size = 11.0;
            node.line_height = item_h;
            node.computed_rect = Rect::new(item_rect.x + 6.0, cur_y, 16.0, item_h);
        }
        let _ = tree.add_child(row_id, ic_id);

        // Category Label
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("CategoryLabel");
            node.set_text(cat.title());
            node.font_size = 11.0;
            node.line_height = item_h;
            node.text_color = text_col;
            node.computed_rect =
                Rect::new(item_rect.x + 24.0, cur_y, item_rect.width - 40.0, item_h);
        }
        let _ = tree.add_child(row_id, lbl_id);

        // Submenu Arrow "▸"
        let arw_id = tree.create_node();
        if let Some(node) = tree.get_mut(arw_id) {
            node.set_name("CategoryArrow");
            node.set_text("▸");
            node.font_size = 10.0;
            node.line_height = item_h;
            node.text_align = TextAlign::Right;
            node.text_color = Color::rgba(0.54, 0.56, 0.60, 1.0);
            node.computed_rect = Rect::new(item_rect.right() - 16.0, cur_y, 12.0, item_h);
        }
        let _ = tree.add_child(row_id, arw_id);

        targets.add_menu_categories.push((cat, item_rect));
        cur_y += item_h;
    }

    // Build Active Category Submenu if open
    if let Some(cat) = params.active_submenu {
        build_category_submenu(
            tree,
            parent_id,
            menu_x + menu_w + 2.0,
            submenu_anchor_y,
            cat,
            params,
            targets,
        );
    }
}

/// Checks if a component category has at least one attachable component for the entity.
fn category_has_available(
    cat: ComponentCategory,
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> bool {
    if cat == ComponentCategory::CustomDynamic {
        let registry = InspectorRegistry::global();
        let handled_names: std::collections::HashSet<_> = registry
            .handlers()
            .iter()
            .map(|h| h.component_name())
            .collect();
        let comp_registry = ae_core::registry::ComponentRegistry::global();
        comp_registry.handlers().iter().any(|h| {
            let name = h.type_name();
            !handled_names.contains(name)
                && !crate::ui::panels::inspector::dynamic_reflection::is_internal_or_specialized(
                    name,
                )
                && if let Some(ent) = entity {
                    !h.has_component(world, ent)
                } else {
                    true
                }
        })
    } else {
        let registry = InspectorRegistry::global();
        registry.find_by_category(cat).into_iter().any(|h| {
            if let Some(ent) = entity {
                !h.has_component(world, ent)
            } else {
                true
            }
        })
    }
}

/// Component item descriptor representing an attachable entry in the submenu.
struct SubmenuItemEntry {
    comp_name: &'static str,
    display_title: &'static str,
    icon: &'static str,
}

/// Builds the cascading flyout submenu for a specific component category.
fn build_category_submenu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    sub_x: f32,
    sub_y: f32,
    cat: ComponentCategory,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
) {
    let items: Vec<SubmenuItemEntry> = if cat == ComponentCategory::CustomDynamic {
        let registry = InspectorRegistry::global();
        let handled_names: std::collections::HashSet<_> = registry
            .handlers()
            .iter()
            .map(|h| h.component_name())
            .collect();
        let comp_registry = ae_core::registry::ComponentRegistry::global();
        comp_registry
            .handlers()
            .iter()
            .filter(|h| {
                let name = h.type_name();
                !handled_names.contains(name)
                    && !crate::ui::panels::inspector::dynamic_reflection::is_internal_or_specialized(
                        name,
                    )
                    && if let Some(entity) = params.selected_entity {
                        !h.has_component(params.world, entity)
                    } else {
                        true
                    }
            })
            .map(|h| SubmenuItemEntry {
                comp_name: h.type_name(),
                display_title: h.type_name(),
                icon: "🧩",
            })
            .collect()
    } else {
        let registry = InspectorRegistry::global();
        let all_handlers = registry.find_by_category(cat);
        all_handlers
            .into_iter()
            .filter(|h| {
                if let Some(entity) = params.selected_entity {
                    !h.has_component(params.world, entity)
                } else {
                    true
                }
            })
            .map(|h| SubmenuItemEntry {
                comp_name: h.component_name(),
                display_title: h.display_title(),
                icon: h.icon(),
            })
            .collect()
    };

    if items.is_empty() {
        return;
    }

    let item_h = 22.0;
    let sub_w = 210.0;
    let total_h = (items.len() as f32) * item_h + 8.0;

    let sub_rect = Rect::new(sub_x, sub_y, sub_w, total_h);
    targets.active_submenu_rect = Some(sub_rect);

    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("AddComponentSubmenuPopup");
        node.computed_rect = sub_rect;
        node.style = Style::new()
            .background(Color::rgba(0.086, 0.090, 0.106, 0.98))
            .border(1.0, Color::rgba(0.173, 0.180, 0.208, 0.90))
            .border_radius(5.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }
    let _ = tree.add_child(parent_id, sub_id);

    let mut cur_y = sub_y + 4.0;

    for item in items {
        let item_rect = Rect::new(sub_x + 4.0, cur_y, sub_w - 8.0, item_h);
        let is_hovered = item_rect.contains_point(params.cursor_pos);

        let (bg, text_col) = if is_hovered {
            (Color::rgba(0.157, 0.165, 0.188, 0.98), Color::WHITE)
        } else {
            (Color::TRANSPARENT, Color::rgba(0.886, 0.894, 0.918, 1.0))
        };

        let row_id = tree.create_node();
        if let Some(node) = tree.get_mut(row_id) {
            node.set_name(format!("SubmenuItem_{}", item.comp_name));
            node.computed_rect = item_rect;
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(sub_id, row_id);

        // Component Icon
        let ic_id = tree.create_node();
        if let Some(node) = tree.get_mut(ic_id) {
            node.set_name("SubmenuItemIcon");
            node.set_text(item.icon);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.computed_rect = Rect::new(item_rect.x + 6.0, cur_y, 16.0, item_h);
        }
        let _ = tree.add_child(row_id, ic_id);

        // Component Label
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("SubmenuItemLabel");
            node.set_text(item.display_title);
            node.font_size = 11.0;
            node.line_height = item_h;
            node.text_color = text_col;
            node.computed_rect =
                Rect::new(item_rect.x + 24.0, cur_y, item_rect.width - 28.0, item_h);
        }
        let _ = tree.add_child(row_id, lbl_id);

        targets.submenu_components.push((item.comp_name, item_rect));
        cur_y += item_h;
    }
}