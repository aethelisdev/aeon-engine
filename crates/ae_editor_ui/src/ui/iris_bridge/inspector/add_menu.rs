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

/// Standard row height for Add Component menu items in logical pixels.
pub const ADD_MENU_ITEM_HEIGHT: f32 = 24.0;

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

    match get_add_component_menu_items(params.world, params.selected_entity) {
        menu_items if !menu_items.is_empty() => {
            render_add_component_scope(tree, parent_id, params, panel_tree, &menu_items)
        }
        _ => None,
    }
}

fn render_add_component_scope(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    panel_tree: Option<&UiTree>,
    menu_items: &[CascadingMenuItem],
) -> Option<Rect> {
    render_add_component_popups(
        render_add_component_menu_card(tree, parent_id, params, panel_tree, menu_items),
        tree,
        parent_id,
        params,
        menu_items,
    )
}

fn render_add_component_menu_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    panel_tree: Option<&UiTree>,
    menu_items: &[CascadingMenuItem],
) -> Rect {
    UiScope::new(tree, parent_id)
        .popup_anchored_with_placement(
            "AddComponentMenu",
            panel_tree
                .and_then(|ptree| {
                    ptree.node_rect_tagged(TAG_INSPECTOR_ADD_COMPONENT, params.panel_rect.origin())
                })
                .unwrap_or(Rect::new(
                    params.panel_rect.x + 8.0,
                    params.panel_rect.bottom() - 32.0,
                    140.0,
                    24.0,
                )),
            175.0,
            PopupPlacement::Above,
            |card| {
                for item in menu_items {
                    card.dropdown_item(
                        item.tag,
                        match &item.icon {
                            Some(CascadingMenuIcon::Text(t)) => t,
                            _ => "",
                        },
                        &item.label,
                        Some("▶"),
                        true,
                    );
                }
            },
        )
        .response
        .rect
}

fn render_add_component_popups(
    card_rect: Rect,
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    menu_items: &[CascadingMenuItem],
) -> Option<Rect> {
    Some(
        params
            .active_submenu
            .and_then(|active_cat| {
                menu_items
                    .iter()
                    .position(|i| i.tag == active_cat.to_tag())
                    .and_then(|cat_idx| {
                        menu_items[cat_idx].submenu.as_ref().map(|children| {
                            UiScope::new(tree, parent_id)
                                .dropdown_menu_card_named(
                                    "AddComponentSubmenu",
                                    if card_rect.right() + 195.0 > params.panel_rect.right() {
                                        (card_rect.x - 195.0 - 4.0).max(params.panel_rect.x + 4.0)
                                    } else {
                                        card_rect.right() + 4.0
                                    },
                                    (card_rect.y + 4.0 + cat_idx as f32 * ADD_MENU_ITEM_HEIGHT)
                                        .min(
                                            params.panel_rect.bottom()
                                                - (children.len() as f32 * ADD_MENU_ITEM_HEIGHT
                                                    + 8.0)
                                                - 10.0,
                                        )
                                        .max(params.panel_rect.y + 10.0),
                                    195.0,
                                    |sub_card| {
                                        for child in children {
                                            sub_card.dropdown_item(
                                                child.tag,
                                                match &child.icon {
                                                    Some(CascadingMenuIcon::Text(t)) => t,
                                                    _ => "",
                                                },
                                                &child.label,
                                                None,
                                                true,
                                            );
                                        }
                                    },
                                )
                                .response
                                .rect
                        })
                    })
            })
            .map_or(card_rect, |sub_rect| card_rect.union(sub_rect)),
    )
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

        assert_eq!(
            tree.node_named("AddComponentMenu")
                .expect("AddComponentMenu must be built")
                .layer,
            UiLayer::Popup
        );
        assert_eq!(
            tree.node_named("AddComponentSubmenu")
                .expect("AddComponentSubmenu must be built")
                .layer,
            UiLayer::Popup
        );

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