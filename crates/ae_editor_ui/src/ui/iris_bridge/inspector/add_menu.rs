// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading `➕ Add Component` Menu Builder
//!
//! Provides the data-driven cascading dropdown menu tree for attaching ECS components
//! via [`CascadingMenuBuilder`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use super::registry::InspectorRegistry;
use super::tags::{TAG_ADD_MENU_COMPONENT_BASE, TAG_INSPECTOR_ADD_COMPONENT};
use super::types::{ComponentCategory, InspectorPanelParams};
use irisui::prelude::*;

/// Computes a stable, collision-resistant 64-bit FNV-1a hash tag for an attachable component type name.
///
/// Masks the 64-bit hash into a 24-bit range and prefixes it with [`TAG_ADD_MENU_COMPONENT_BASE`]
/// (`0xDB00_0000`), guaranteeing that tag bounds stay strictly inside the Inspector domain.
#[inline]
pub fn component_name_to_tag(name: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    TAG_ADD_MENU_COMPONENT_BASE | (hash & 0x00FF_FFFF)
}

/// Resolves a numeric menu item tag into its canonical component type name.
pub fn resolve_component_name_from_tag(tag: u64) -> Option<&'static str> {
    let registry = InspectorRegistry::global();
    for handler in registry.handlers() {
        let name = handler.component_name();
        if component_name_to_tag(name) == tag {
            return Some(name);
        }
    }

    let comp_registry = ae_core::registry::ComponentRegistry::global();
    for handler in comp_registry.handlers() {
        let name = handler.type_name();
        if component_name_to_tag(name) == tag {
            return Some(name);
        }
    }

    None
}

/// Checks if a component category has at least one attachable component for the selected entity.
fn category_has_available(
    cat: ComponentCategory,
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> bool {
    if cat == ComponentCategory::CustomDynamic {
        let registry = InspectorRegistry::global();
        let comp_registry = ae_core::registry::ComponentRegistry::global();
        comp_registry.handlers().iter().any(|h| {
            let name = h.type_name();
            !registry
                .handlers()
                .iter()
                .any(|ih| ih.component_name() == name)
                && !super::dynamic_reflection::is_internal_or_specialized(name)
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

/// Constructs the declarative cascading menu item hierarchy for `➕ Add Component`.
pub fn get_add_component_menu_items(
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> Vec<CascadingMenuItem> {
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

    let mut result = Vec::with_capacity(all_categories.len());

    for cat in all_categories {
        if !category_has_available(cat, world, entity) {
            continue;
        }

        let child_items: Vec<CascadingMenuItem> = if cat == ComponentCategory::CustomDynamic {
            let registry = InspectorRegistry::global();
            let comp_registry = ae_core::registry::ComponentRegistry::global();
            comp_registry
                .handlers()
                .iter()
                .filter(|h| {
                    let name = h.type_name();
                    !registry
                        .handlers()
                        .iter()
                        .any(|ih| ih.component_name() == name)
                        && !super::dynamic_reflection::is_internal_or_specialized(name)
                        && if let Some(ent) = entity {
                            !h.has_component(world, ent)
                        } else {
                            true
                        }
                })
                .map(|h| {
                    let name = h.type_name();
                    CascadingMenuItem::item_with_icon(
                        name,
                        CascadingMenuIcon::Text("🧩"),
                        component_name_to_tag(name),
                    )
                })
                .collect()
        } else {
            let registry = InspectorRegistry::global();
            registry
                .find_by_category(cat)
                .into_iter()
                .filter(|h| {
                    if let Some(ent) = entity {
                        !h.has_component(world, ent)
                    } else {
                        true
                    }
                })
                .map(|h| {
                    let name = h.component_name();
                    let icon = if let Some(uv) = h.atlas_icon() {
                        CascadingMenuIcon::Texture(uv)
                    } else {
                        CascadingMenuIcon::Text(h.icon())
                    };
                    CascadingMenuItem::item_with_icon(
                        h.display_title(),
                        icon,
                        component_name_to_tag(name),
                    )
                })
                .collect()
        };

        result.push(CascadingMenuItem::branch(
            cat.title(),
            Some(CascadingMenuIcon::Text(cat.icon())),
            cat.to_tag(),
            child_items,
        ));
    }

    result
}

/// Builds the cascading `➕ Add Component` menu in the [`UiTree`] using declarative [`UiScope`].
///
/// Returns the screen-space bounding box enclosing the main menu and cascading submenu, if open.
pub fn build_add_component_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    panel_tree: Option<&UiTree>,
) -> Option<Rect> {
    if !params.is_add_menu_open {
        return None;
    }

    let menu_items = get_add_component_menu_items(params.world, params.selected_entity);
    if menu_items.is_empty() {
        return None;
    }

    let anchor_rect = panel_tree
        .and_then(|ptree| {
            ptree
                .iter()
                .find(|(_, node)| node.tag == TAG_INSPECTOR_ADD_COMPONENT)
                .map(|(_, node)| {
                    Rect::new(
                        params.panel_rect.x + node.computed_rect.x,
                        params.panel_rect.y + node.computed_rect.y,
                        node.computed_rect.width,
                        node.computed_rect.height,
                    )
                })
        })
        .unwrap_or(Rect::new(
            params.panel_rect.x + 8.0,
            params.panel_rect.bottom() - 32.0,
            140.0,
            24.0,
        ));

    let menu_w = 175.0;
    let item_h = 24.0;
    let menu_h = menu_items.len() as f32 * item_h + 8.0;

    let menu_x = anchor_rect.x;
    let menu_y = (anchor_rect.y - menu_h - 4.0).max(params.panel_rect.y + 10.0);

    let mut scope = UiScope::new(tree, parent_id);
    scope.dropdown_menu_card_named("AddComponentMenu", menu_x, menu_y, menu_w, |card| {
        for item in &menu_items {
            let icon_str = match &item.icon {
                Some(CascadingMenuIcon::Text(t)) => *t,
                _ => "",
            };
            card.dropdown_item(item.tag, icon_str, &item.label, Some("▶"), true);
        }
    });

    let mut min_x = menu_x;
    let mut min_y = menu_y;
    let mut max_x = menu_x + menu_w;
    let mut max_y = menu_y + menu_h;

    if let Some(active_cat) = params.active_submenu
        && let Some(cat_item) = menu_items.iter().find(|i| i.tag == active_cat.to_tag())
        && let Some(ref children) = cat_item.submenu
        && !children.is_empty()
    {
        let sub_w = 195.0;
        let sub_h = children.len() as f32 * item_h + 8.0;

        let sub_x = if menu_x + menu_w + sub_w > params.panel_rect.right() {
            (menu_x - sub_w - 4.0).max(params.panel_rect.x + 4.0)
        } else {
            menu_x + menu_w + 4.0
        };

        let cat_idx = menu_items
            .iter()
            .position(|i| i.tag == active_cat.to_tag())
            .unwrap_or(0);
        let cat_y = menu_y + 4.0 + cat_idx as f32 * item_h;
        let sub_y = (cat_y)
            .min(params.panel_rect.bottom() - sub_h - 10.0)
            .max(params.panel_rect.y + 10.0);

        scope.dropdown_menu_card_named("AddComponentSubmenu", sub_x, sub_y, sub_w, |sub_card| {
            for child in children {
                let icon_str = match &child.icon {
                    Some(CascadingMenuIcon::Text(t)) => *t,
                    _ => "",
                };
                sub_card.dropdown_item(child.tag, icon_str, &child.label, None, true);
            }
        });

        min_x = min_x.min(sub_x);
        min_y = min_y.min(sub_y);
        max_x = max_x.max(sub_x + sub_w);
        max_y = max_y.max(sub_y + sub_h);
    }

    Some(Rect::new(
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_add_component_menu_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let world = hecs::World::new();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];

        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 600.0),
            world: &world,
            selected_entity: None,
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: Some(ComponentCategory::Physics),
            is_add_menu_open: true,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        build_add_component_menu(&mut tree, root, &params, None);

        let menu_node = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("AddComponentMenu"));
        assert!(menu_node.is_some(), "AddComponentMenu must be built");
        assert_eq!(menu_node.unwrap().1.layer, UiLayer::Popup);

        let submenu_node = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("AddComponentSubmenu"));
        assert!(
            submenu_node.is_some(),
            "AddComponentSubmenu must be built for active category"
        );
        assert_eq!(submenu_node.unwrap().1.layer, UiLayer::Popup);

        let cat_items: Vec<_> = tree
            .iter()
            .filter(|(_, n)| {
                n.role == WidgetRole::DropdownItem && n.tag == ComponentCategory::Physics.to_tag()
            })
            .collect();
        assert!(
            !cat_items.is_empty(),
            "Physics category item must be listed"
        );

        let comp_items: Vec<_> = tree
            .iter()
            .filter(|(_, n)| {
                n.role == WidgetRole::DropdownItem && n.tag != ComponentCategory::Physics.to_tag()
            })
            .collect();
        assert!(
            !comp_items.is_empty(),
            "Physics components must be listed in submenu"
        );
    }

    #[test]
    fn test_component_name_to_tag_domain_conformance() {
        let tag = component_name_to_tag("RigidBody");
        assert!(
            (TAG_ADD_MENU_COMPONENT_BASE..=(TAG_ADD_MENU_COMPONENT_BASE | 0x00FF_FFFF))
                .contains(&tag),
            "Component tag must stay strictly within TAG_ADD_MENU_COMPONENT_BASE domain"
        );
        let resolved = resolve_component_name_from_tag(tag);
        assert_eq!(resolved, Some("RigidBody"));
    }
}