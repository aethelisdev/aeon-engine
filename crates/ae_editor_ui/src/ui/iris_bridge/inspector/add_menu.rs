// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading `➕ Add Component` Menu Builder
//!
//! Provides the data-driven cascading dropdown menu tree for attaching ECS components
//! via [`UiScope`].
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`) and pure declarative
//! closure workflows without imperative intermediate `let` variables or pyramid nesting.

use super::footer::TAG_INSPECTOR_ADD_COMPONENT;
use super::types::ComponentCategory;
use irisui::prelude::*;
use irisui::widgets::cascading_menu::{CascadingMenuIcon, CascadingMenuItem};
use irisui::widgets::declarative::scope::{CascadingPopupConfig, PopupPlacement};

/// Semantic base domain tag for attachable component items inside the Add Component menu.
const TAG_ADD_MENU_COMPONENT: UiTag = UiTag::from_str("ae.inspector.add_menu.component");

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

pub mod catalog;
pub use catalog::*;

/// Computes a deterministic scoped 64-bit [`UiTag`] for an attachable component type name
/// derived directly from [`TAG_ADD_MENU_COMPONENT`].
#[inline]
#[must_use]
pub const fn component_name_to_tag(name: &str) -> u64 {
    TAG_ADD_MENU_COMPONENT.child(name).raw()
}

/// Resolves a numeric menu item tag into its canonical component type name.
#[must_use]
pub fn resolve_component_name_from_tag(tag: u64) -> Option<&'static str> {
    BUILTIN_COMPONENT_MENU_CATALOG
        .iter()
        .find(|e| component_name_to_tag(e.component_name) == tag)
        .map(|e| e.component_name)
        .or_else(|| {
            ae_plugin_api::engine_component_registry()
                .handlers()
                .iter()
                .map(|h| h.type_name())
                .find(|name| component_name_to_tag(name) == tag)
        })
}

/// Returns whether a component type name is already handled by a built-in inspector handler.
#[inline]
fn is_built_in_inspector_component(type_name: &str) -> bool {
    BUILTIN_COMPONENT_MENU_CATALOG
        .iter()
        .any(|e| e.component_name == type_name)
}

/// Checks if a component is an internal system marker or handled by top-level inspector headers.
#[inline]
fn is_internal_or_specialized(type_name: &str) -> bool {
    matches!(
        type_name,
        "Position"
            | "Rotation"
            | "Scale"
            | "Name"
            | "TransformDirty"
            | "GlobalTransform"
            | "Hidden"
            | "BoundingBox"
            | "BoundingRadius"
            | "Shape"
            | "Color"
            | "Light"
            | "Parent"
            | "Children"
            | "ModelId"
            | "SpriteId"
            | "EntityGuid"
    )
}

/// Checks if a component category has at least one attachable component for the selected entity.
fn category_has_available(
    cat: ComponentCategory,
    world: &ae_ecs::World,
    entity: Option<ae_ecs::Entity>,
) -> bool {
    if cat == ComponentCategory::CustomDynamic {
        ae_plugin_api::engine_component_registry()
            .handlers()
            .iter()
            .any(|h| {
                !is_built_in_inspector_component(h.type_name())
                    && !is_internal_or_specialized(h.type_name())
                    && entity.is_none_or(|ent| !h.has_component(world, ent))
            })
    } else {
        BUILTIN_COMPONENT_MENU_CATALOG
            .iter()
            .filter(|e| e.category == cat)
            .any(|e| entity.is_none_or(|ent| !(e.has_component)(world, ent)))
    }
}

/// Constructs the child menu items for custom dynamic reflected components.
fn get_custom_dynamic_child_items(
    world: &ae_ecs::World,
    entity: Option<ae_ecs::Entity>,
) -> Vec<CascadingMenuItem> {
    ae_plugin_api::engine_component_registry()
        .handlers()
        .iter()
        .filter(|h| {
            !is_built_in_inspector_component(h.type_name())
                && !is_internal_or_specialized(h.type_name())
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
    world: &ae_ecs::World,
    entity: Option<ae_ecs::Entity>,
) -> Vec<CascadingMenuItem> {
    if cat == ComponentCategory::CustomDynamic {
        get_custom_dynamic_child_items(world, entity)
    } else {
        BUILTIN_COMPONENT_MENU_CATALOG
            .iter()
            .filter(|e| {
                e.category == cat && entity.is_none_or(|ent| !(e.has_component)(world, ent))
            })
            .map(|e| {
                CascadingMenuItem::item_with_icon(
                    e.display_title,
                    e.atlas_icon
                        .map(CascadingMenuIcon::Texture)
                        .unwrap_or(CascadingMenuIcon::Text(e.icon)),
                    component_name_to_tag(e.component_name),
                )
            })
            .collect()
    }
}

/// Constructs the declarative cascading menu item hierarchy for `➕ Add Component`.
#[must_use]
pub fn get_add_component_menu_items(
    world: &ae_ecs::World,
    entity: Option<ae_ecs::Entity>,
) -> Vec<CascadingMenuItem> {
    ALL_COMPONENT_CATEGORIES
        .into_iter()
        .filter(|&cat| category_has_available(cat, world, entity))
        .map(|cat| {
            CascadingMenuItem::branch(
                cat.title(),
                cat.atlas_icon()
                    .map(CascadingMenuIcon::Texture)
                    .or_else(|| Some(CascadingMenuIcon::Text(cat.icon()))),
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
    selected_entity: Option<ae_ecs::Entity>,
) -> Option<crate::ui::types::EngineUiAction> {
    children.iter().find_map(|child| {
        sub_card.dropdown_menu_item_autoclose(child).then_click(|| {
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
    selected_entity: Option<ae_ecs::Entity>,
) -> Option<crate::ui::types::EngineUiAction> {
    if !item.has_submenu() {
        card.dropdown_menu_item_autoclose(item);
        return None;
    }

    card.submenu_named_with_icon_inner(
        "AddComponentSubmenu",
        &item.label,
        item.icon,
        item.tag,
        |sub_card| {
            render_submenu_children(
                sub_card,
                item.submenu.as_deref().unwrap_or(&[]),
                selected_entity,
            )
        },
    )
    .inner
    .flatten()
}

/// Renders all category items in the root menu card and returns any component addition action triggered.
fn render_category_items(
    card: &mut UiScope<'_>,
    menu_items: &[CascadingMenuItem],
    selected_entity: Option<ae_ecs::Entity>,
) -> Option<crate::ui::types::EngineUiAction> {
    menu_items
        .iter()
        .find_map(|item| render_category_item(card, item, selected_entity))
}

/// Renders the root Add Component popup menu card into [`UiScope`] when active in `UiTree`,
/// returning the outer bounding bounds and any triggered action.
pub fn draw_add_component_menu(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    selected_entity: Option<ae_ecs::Entity>,
) -> (Option<Rect>, Option<crate::ui::types::EngineUiAction>) {
    match get_add_component_menu_items(world, selected_entity) {
        items if !items.is_empty() => scope
            .scoped_cascading_popup_anchored_auto(
                "AddComponentMenu",
                TAG_INSPECTOR_ADD_COMPONENT,
                CascadingPopupConfig::new(175.0, PopupPlacement::Above)
                    .with_item_count(items.len()),
                |card| render_category_items(card, &items, selected_entity),
            )
            .map(|res| res.map_response(|rect, action| (Some(rect), action)))
            .unwrap_or((None, None)),
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::widgets::declarative::hash_label;

    #[test]
    fn test_declarative_add_component_menu_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let world = ae_ecs::World::new();

        tree.open_popup(TAG_INSPECTOR_ADD_COMPONENT);
        tree.set_active_submenu(
            hash_label("AddComponentMenu"),
            ComponentCategory::Physics.to_tag(),
        );
        draw_add_component_menu(&mut UiScope::new(&mut tree, root), &world, None);

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
        assert_eq!(tag, TAG_ADD_MENU_COMPONENT.child("RigidBody").raw());
        assert_ne!(tag, 0);
        assert_ne!(tag, component_name_to_tag("Collider"));
        let resolved = resolve_component_name_from_tag(tag);
        assert_eq!(resolved, Some("RigidBody"));
    }

    #[test]
    fn test_add_component_click_directly_emits_engine_ui_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut world = ae_ecs::World::new();
        let entity = world.spawn(());

        tree.open_popup(TAG_INSPECTOR_ADD_COMPONENT);
        tree.set_active_submenu(
            hash_label("AddComponentMenu"),
            ComponentCategory::Physics.to_tag(),
        );
        let (_, action) =
            draw_add_component_menu(&mut UiScope::new(&mut tree, root), &world, Some(entity));
        assert!(action.is_none());

        let tag = component_name_to_tag("RigidBody");
        tree.event_state.frame_clicked_tag = Some(tag);

        let (rect, ui_action) =
            draw_add_component_menu(&mut UiScope::new(&mut tree, root), &world, Some(entity));

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
            !tree.is_popup_tag_active(TAG_INSPECTOR_ADD_COMPONENT),
            "Clicking a component item must clear TAG_INSPECTOR_ADD_COMPONENT in UiTree"
        );
        assert!(
            !tree.is_submenu_open(
                hash_label("AddComponentMenu"),
                ComponentCategory::Physics.to_tag()
            ),
            "Clicking a component item must clear active_submenus in UiTree"
        );
    }

    #[test]
    fn test_relational_hierarchy_excluded_from_add_menu() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn(());
        let items = get_add_component_menu_items(&world, Some(entity));

        let has_hierarchy_category = items.iter().any(|item| {
            item.tag == ComponentCategory::Hierarchy.to_tag() || item.label == "Hierarchy"
        });
        assert!(
            !has_hierarchy_category,
            "Relational Hierarchy category must be excluded from Add Component menu"
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

    #[test]
    fn test_autonomous_add_menu_without_inspector_registry() {
        let mut world = ae_ecs::World::new();
        let entity = world.spawn(());

        // 1. Initial items: all 31 components should be resolvable from their tags
        let initial_items = get_add_component_menu_items(&world, Some(entity));
        assert!(
            !initial_items.is_empty(),
            "Add Component menu must produce items autonomously without InspectorRegistry"
        );

        for cat_item in &initial_items {
            if let Some(sub) = &cat_item.submenu {
                for child in sub {
                    let resolved = resolve_component_name_from_tag(child.tag);
                    assert!(
                        resolved.is_some(),
                        "Child tag 0x{:X} must resolve to component name",
                        child.tag
                    );
                    let name = resolved.unwrap();
                    if cat_item.tag != ComponentCategory::CustomDynamic.to_tag() {
                        let entry = find_builtin_component_entry(name);
                        assert!(
                            entry.is_some(),
                            "Built-in component '{}' must exist in SSOT catalog",
                            name
                        );
                    } else {
                        let has_dynamic = ae_plugin_api::engine_component_registry()
                            .handlers()
                            .iter()
                            .any(|h| h.type_name() == name);
                        assert!(
                            has_dynamic,
                            "Dynamic component '{}' must exist in ComponentRegistry::global",
                            name
                        );
                    }
                }
            }
        }

        // 2. Attach a component and verify it is autonomously filtered out from the menu
        let rb_entry = find_builtin_component_entry("RigidBody").expect("RigidBody entry");
        (rb_entry.spawn_default)(&mut world, entity);

        let filtered_items = get_add_component_menu_items(&world, Some(entity));
        let has_rigidbody_child = filtered_items.iter().any(|cat| {
            cat.submenu.as_ref().is_some_and(|sub| {
                sub.iter()
                    .any(|child| child.tag == component_name_to_tag("RigidBody"))
            })
        });
        assert!(
            !has_rigidbody_child,
            "Attached component 'RigidBody' must be filtered out from the Add Component menu"
        );
    }
}