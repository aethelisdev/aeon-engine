// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading `➕ Add Component` Menu Builder
//!
//! Provides the data-driven cascading dropdown menu tree for attaching ECS components
//! via [`CascadingMenuBuilder`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

use super::registry::InspectorRegistry;
use super::types::{ComponentCategory, InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;

/// Computes a stable, collision-resistant 64-bit FNV-1a hash tag for an attachable component type name.
#[inline]
pub fn component_name_to_tag(name: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    if hash < 1000 { hash + 1000 } else { hash }
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
        let handled_names: std::collections::HashSet<_> = registry
            .handlers()
            .iter()
            .map(|h| h.component_name())
            .collect();
        let comp_registry = ae_core::registry::ComponentRegistry::global();
        comp_registry.handlers().iter().any(|h| {
            let name = h.type_name();
            !handled_names.contains(name)
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

/// Builds the cascading `➕ Add Component` menu in the [`UiTree`] using [`CascadingMenuBuilder`].
pub fn build_add_component_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
) {
    targets.active_add_component_rects.clear();

    if !params.is_add_menu_open {
        return;
    }

    let menu_items = get_add_component_menu_items(params.world, params.selected_entity);
    if menu_items.is_empty() {
        return;
    }

    let mut active_path = Vec::new();
    if let Some(cat) = params.active_submenu {
        active_path.push(cat.to_tag());
    }

    if let Some(frame) =
        CascadingMenuBuilder::new(targets.add_component_btn_rect, &menu_items, &active_path)
            .cursor_pos(params.cursor_pos)
            .viewport_bounds(params.panel_rect)
            .open_upward(true)
            .name("AddComponentMenu")
            .build(tree, parent_id)
    {
        targets.active_add_component_rects = frame.rendered_popup_rects;
    }
}