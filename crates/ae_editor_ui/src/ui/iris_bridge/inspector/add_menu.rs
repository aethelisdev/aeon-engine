// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading `➕ Add Component` Menu Builder
//!
//! Provides the data-driven cascading dropdown menu tree for attaching ECS components
//! via [`UiScope`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`) and pure declarative
//! closure workflows without imperative intermediate `let` variables or pyramid nesting.

use std::ops::BitXor;

use super::registry::InspectorRegistry;
use super::tags::{TAG_ADD_MENU_COMPONENT_BASE, TAG_INSPECTOR_ADD_COMPONENT};
use super::types::{ComponentCategory, InspectorPanelParams};
use irisui::prelude::*;
use irisui::widgets::cascading_menu::{CascadingMenuIcon, CascadingMenuItem};
use irisui::widgets::declarative::scope::{CascadingPopupConfig, PopupPlacement};

/// Canonical list of all attachable component categories available in the cascading Add Menu.
///
/// Relational topology categories (e.g. `ComponentCategory::Hierarchy`) are excluded as
/// parent-child relationships are managed directly via the Hierarchy scene graph.
const ALL_COMPONENT_CATEGORIES: [ComponentCategory; 7] = [
    ComponentCategory::Animation,
    ComponentCategory::Audio,
    ComponentCategory::Gameplay,
    ComponentCategory::Physics,
    ComponentCategory::Rendering,
    ComponentCategory::UiHud,
    ComponentCategory::CustomDynamic,
];

/// Computes a stable, collision-resistant 64-bit FNV-1a hash tag for an attachable component type name.
///
/// Masks the 64-bit hash into a 24-bit range and prefixes it with [`TAG_ADD_MENU_COMPONENT_BASE`]
/// (`0xDB00_0000`), guaranteeing that tag bounds stay strictly inside the Inspector domain.
#[inline]
#[must_use]
pub fn component_name_to_tag(name: &str) -> u64 {
    TAG_ADD_MENU_COMPONENT_BASE
        | (name.bytes().fold(0xcbf2_9ce4_8422_2325u64, |acc, b| {
            acc.bitxor(b as u64).wrapping_mul(0x100_0000_01b3)
        }) & 0x00FF_FFFF)
}

/// Resolves a numeric menu item tag into its canonical component type name.
#[must_use]
pub fn resolve_component_name_from_tag(tag: u64) -> Option<&'static str> {
    InspectorRegistry::global()
        .handlers()
        .iter()
        .map(|h| h.component_name())
        .find(|name| component_name_to_tag(name) == tag)
        .or_else(|| {
            ae_core::registry::ComponentRegistry::global()
                .handlers()
                .iter()
                .map(|h| h.type_name())
                .find(|name| component_name_to_tag(name) == tag)
        })
}

/// Returns whether a component type name is already handled by a built-in inspector handler.
#[inline]
fn is_built_in_inspector_component(type_name: &str) -> bool {
    InspectorRegistry::global()
        .handlers()
        .iter()
        .any(|ih| ih.component_name() == type_name)
}

/// Checks if a component category has at least one attachable component for the selected entity.
fn category_has_available(
    cat: ComponentCategory,
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> bool {
    if cat == ComponentCategory::CustomDynamic {
        ae_core::registry::ComponentRegistry::global()
            .handlers()
            .iter()
            .any(|h| {
                !is_built_in_inspector_component(h.type_name())
                    && !super::dynamic_reflection::is_internal_or_specialized(h.type_name())
                    && entity.is_none_or(|ent| !h.has_component(world, ent))
            })
    } else {
        InspectorRegistry::global()
            .find_by_category(cat)
            .into_iter()
            .any(|h| entity.is_none_or(|ent| !h.has_component(world, ent)))
    }
}

/// Constructs the child menu items for custom dynamic reflected components.
fn get_custom_dynamic_child_items(
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> Vec<CascadingMenuItem> {
    ae_core::registry::ComponentRegistry::global()
        .handlers()
        .iter()
        .filter(|h| {
            !is_built_in_inspector_component(h.type_name())
                && !super::dynamic_reflection::is_internal_or_specialized(h.type_name())
                && entity.is_none_or(|ent| !h.has_component(world, ent))
        })
        .map(|h| {
            CascadingMenuItem::item_with_icon(
                h.type_name(),
                CascadingMenuIcon::Text("🧩"),
                component_name_to_tag(h.type_name()),
            )
        })
        .collect()
}

/// Constructs the child menu items for built-in or custom categories.
fn get_category_child_items(
    cat: ComponentCategory,
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> Vec<CascadingMenuItem> {
    if cat == ComponentCategory::CustomDynamic {
        get_custom_dynamic_child_items(world, entity)
    } else {
        InspectorRegistry::global()
            .find_by_category(cat)
            .into_iter()
            .filter(|h| entity.is_none_or(|ent| !h.has_component(world, ent)))
            .map(|h| {
                CascadingMenuItem::item_with_icon(
                    h.display_title(),
                    h.atlas_icon()
                        .map(CascadingMenuIcon::Texture)
                        .unwrap_or(CascadingMenuIcon::Text(h.icon())),
                    component_name_to_tag(h.component_name()),
                )
            })
            .collect()
    }
}

/// Constructs the declarative cascading menu item hierarchy for `➕ Add Component`.
#[must_use]
pub fn get_add_component_menu_items(
    world: &hecs::World,
    entity: Option<hecs::Entity>,
) -> Vec<CascadingMenuItem> {
    ALL_COMPONENT_CATEGORIES
        .into_iter()
        .filter(|&cat| category_has_available(cat, world, entity))
        .map(|cat| {
            CascadingMenuItem::branch(
                cat.title(),
                Some(CascadingMenuIcon::Text(cat.icon())),
                cat.to_tag(),
                get_category_child_items(cat, world, entity),
            )
        })
        .collect()
}

/// Renders component items in a category submenu and returns any component addition action triggered.
fn render_submenu_children(
    sub_card: &mut UiScope<'_>,
    children: &[CascadingMenuItem],
    selected_entity: Option<hecs::Entity>,
    is_add_menu_open: &mut bool,
    active_submenu: &mut Option<ComponentCategory>,
) -> Option<crate::ui::types::EngineUiAction> {
    children.iter().find_map(|child| {
        sub_card.dropdown_menu_item(child).then_click(|| {
            *is_add_menu_open = false;
            *active_submenu = None;
            resolve_component_name_from_tag(child.tag)
                .zip(selected_entity)
                .map(|(name, ent)| crate::ui::types::EngineUiAction::AddComponent(ent, name))
        })
    })
}

/// Renders a single category row and its nested submenu, returning any component addition action triggered.
fn render_category_item(
    card: &mut UiScope<'_>,
    item: &CascadingMenuItem,
    params: &InspectorPanelParams<'_>,
    is_add_menu_open: &mut bool,
    active_submenu: &mut Option<ComponentCategory>,
) -> Option<crate::ui::types::EngineUiAction> {
    if !item.has_submenu() {
        card.dropdown_menu_item(item).on_click(|| {
            *is_add_menu_open = false;
            *active_submenu = None;
        });
        return None;
    }

    card.submenu_named_with_inner(
        "AddComponentSubmenu",
        &item.label,
        Some(item.text_icon()),
        item.tag,
        |sub_card| {
            render_submenu_children(
                sub_card,
                item.submenu.as_deref().unwrap_or(&[]),
                params.selected_entity,
                is_add_menu_open,
                active_submenu,
            )
        },
    )
    .on_interact(|_| {
        ComponentCategory::from_tag(item.tag)
            .filter(|&cat| *active_submenu != Some(cat))
            .inspect(|&cat| {
                *active_submenu = Some(cat);
            });
    })
    .flatten()
}

/// Renders all category items in the root menu card and returns any component addition action triggered.
fn render_category_items(
    card: &mut UiScope<'_>,
    menu_items: &[CascadingMenuItem],
    params: &InspectorPanelParams<'_>,
    is_add_menu_open: &mut bool,
    active_submenu: &mut Option<ComponentCategory>,
) -> Option<crate::ui::types::EngineUiAction> {
    menu_items
        .iter()
        .find_map(|item| render_category_item(card, item, params, is_add_menu_open, active_submenu))
}

/// Computes the fallback screen-space anchor bounds when the tagged button rectangle is unavailable.
#[inline]
#[must_use]
fn fallback_add_menu_anchor(panel_rect: Rect) -> Rect {
    Rect::new(panel_rect.x + 8.0, panel_rect.bottom() - 32.0, 140.0, 24.0)
}

/// Renders the root Add Component popup menu card into [`UiScope`], returning the outer bounding bounds and any triggered action.
pub fn draw_add_component_menu(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    is_add_menu_open: &mut bool,
    active_submenu: &mut Option<ComponentCategory>,
) -> (Option<Rect>, Option<crate::ui::types::EngineUiAction>) {
    if !*is_add_menu_open {
        return (None, None);
    }

    match get_add_component_menu_items(params.world, params.selected_entity) {
        items if !items.is_empty() => scope
            .cascading_popup_anchored_to_tag(
                "AddComponentMenu",
                TAG_INSPECTOR_ADD_COMPONENT,
                fallback_add_menu_anchor(params.panel_rect),
                CascadingPopupConfig::new(175.0, PopupPlacement::Above)
                    .with_item_count(items.len())
                    .with_active_submenu(active_submenu.map(|c| c.to_tag())),
                |card| {
                    render_category_items(card, &items, params, is_add_menu_open, active_submenu)
                },
            )
            .map_response(|rect, action| (Some(rect), action)),
        _ => (None, None),
    }
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
            active_submenu: Some(ComponentCategory::Physics),
            is_add_menu_open: true,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut is_add_menu_open = true;
        let mut active_submenu = Some(ComponentCategory::Physics);
        draw_add_component_menu(
            &mut UiScope::new(&mut tree, root),
            &params,
            &mut is_add_menu_open,
            &mut active_submenu,
        );

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

    #[test]
    fn test_add_component_click_directly_emits_engine_ui_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];

        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: Some(ComponentCategory::Physics),
            is_add_menu_open: true,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let tag = component_name_to_tag("RigidBody");
        tree.event_state.frame_clicked_tag = Some(tag);

        let mut is_add_menu_open = true;
        let mut active_submenu = Some(ComponentCategory::Physics);
        let (rect, ui_action) = draw_add_component_menu(
            &mut UiScope::new(&mut tree, root),
            &params,
            &mut is_add_menu_open,
            &mut active_submenu,
        );

        assert!(rect.is_some(), "Menu rect must be returned");
        assert!(
            matches!(
                ui_action,
                Some(crate::ui::types::EngineUiAction::AddComponent(
                    ent,
                    "RigidBody"
                )) if ent == entity
            ),
            "Clicking component must directly yield EngineUiAction::AddComponent"
        );
        assert!(
            !is_add_menu_open,
            "Clicking a component item must reset is_add_menu_open to false in-place"
        );
        assert_eq!(
            active_submenu, None,
            "Clicking a component item must reset active_submenu to None in-place"
        );
    }

    #[test]
    fn test_relational_hierarchy_excluded_from_add_menu() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());
        let items = get_add_component_menu_items(&world, Some(entity));

        let has_hierarchy_category = items.iter().any(|item| {
            item.tag == ComponentCategory::Hierarchy.to_tag() || item.label == "Hierarchy"
        });
        assert!(
            !has_hierarchy_category,
            "Relational Hierarchy category must be excluded from Add Component menu when can_add is false"
        );

        let has_parenting_item = items.iter().any(|item| {
            item.submenu.as_ref().is_some_and(|sub| {
                sub.iter().any(|child| {
                    child.label.contains("Parent")
                        || child.tag == component_name_to_tag("Parenting")
                })
            })
        });
        assert!(
            !has_parenting_item,
            "Relational Parenting component must not be present in Add Component menu"
        );
    }
}