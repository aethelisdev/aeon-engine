// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Entity Name and Identity Header Builder
//!
//! Renders the top entity rename input box and label using high-level
//! declarative [`UiScope`] property primitives and flexbox layout.

use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::widgets::declarative::hash_label_with_seed;

/// Returns the persistent 64-bit semantic tag identifying the entity rename text input box for $O(1)$ hit-testing.
#[inline]
pub fn entity_name_input_tag() -> u64 {
    hash_label_with_seed(0, "InspectorEntityName")
}

/// Builds the top Entity Name header declaratively into the parent [`UiScope`].
///
/// Encapsulates the entity name editor within a standardized declarative [`UiScope::prop_text_with_options`]
/// row, managing clicking, focusing, keyboard typing, and inline string editing autonomously via Iris UI.
/// Emits [`WidgetResponse`] where [`WidgetResponse::changed`] signals a committed name change.
pub fn build_entity_header(
    scope: &mut UiScope<'_>,
    name: &mut String,
    _panel_w: f32,
) -> WidgetResponse {
    scope.prop_text_with_options(
        "🏷 Name:",
        name,
        PropertyTextOptions::new()
            .with_placeholder("Entity name...")
            .with_custom_tag(entity_name_input_tag())
            .with_label_width(55.0)
            .with_margin_insets(Insets::new(6.0, 6.0, 4.0, 6.0)),
    )
}

/// Emits the top Entity Name header declaratively and invokes `on_modify` with [`EngineUiAction::ModifyName`] when changed.
///
/// Eliminates intermediate imperative variable plumbing and directly generates the engine UI action.
pub fn build_entity_header_modify<F>(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    old_name: &str,
    name: &mut String,
    panel_w: f32,
    mut on_modify: F,
) -> WidgetResponse
where
    F: FnMut(EngineUiAction),
{
    build_entity_header(scope, name, panel_w).on_change(|| {
        if name != old_name {
            on_modify(EngineUiAction::ModifyName(
                entity,
                old_name.to_string(),
                name.clone(),
            ));
        }
    })
}

/// Emits the top Entity Name header declaratively and returns an optional [`EngineUiAction::ModifyName`] when committed.
///
/// Emits the top Entity Name header declaratively and returns an optional [`EngineUiAction::ModifyName`] when committed.
///
/// Eliminates intermediate imperative variable plumbing and directly generates the engine UI action.
pub fn build_entity_header_modify_action(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    old_name: &str,
    name: &mut String,
    panel_w: f32,
) -> Option<EngineUiAction> {
    (build_entity_header(scope, name, panel_w).changed() && name != old_name)
        .then(|| EngineUiAction::ModifyName(entity, old_name.to_string(), name.clone()))
}

/// Maps a click on the card delete button of `component_name` during the active frame directly
/// into [`EngineUiAction::RemoveComponent`].
#[inline]
pub fn component_delete_action(
    scope: &UiScope<'_>,
    entity: hecs::Entity,
    component_name: &'static str,
) -> Option<EngineUiAction> {
    scope.map_action_clicked(component_name, || {
        EngineUiAction::RemoveComponent(entity, component_name)
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_entity_header_structure_and_layout() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut name = "Dynamic Cube".to_string();

        let mut scope = UiScope::new(&mut tree, root);
        let _resp = build_entity_header(&mut scope, &mut name, 320.0);
        scope.finish_layout(Rect::new(0.0, 0.0, 320.0, 800.0));

        let box_id = tree
            .find_node_by_tag(entity_name_input_tag())
            .expect("Box node must exist");
        let box_node = tree.get(box_id).expect("Box node must exist");
        assert_eq!(box_node.tag, entity_name_input_tag());
        assert_eq!(box_node.role, WidgetRole::TextInput);
        assert_eq!(box_node.cursor, Some(WidgetCursor::Text));
        assert!(box_node.computed_rect.width > 100.0);
        assert!(box_node.computed_rect.height >= 24.0);
        assert!(
            box_node.computed_rect.y >= 6.0,
            "Box node must respect top margin inset >= 6.0"
        );
        assert!(
            box_node.computed_rect.x >= 6.0,
            "Box node must respect left margin inset >= 6.0"
        );
    }

    #[test]
    fn test_declarative_entity_header_modify_action_emission() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let mut name = "New Cube".to_string();
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        let mut emitted_action = None;
        {
            let mut scope = UiScope::new(&mut tree, root);
            let _resp = build_entity_header_modify(
                &mut scope,
                entity,
                "Old Cube",
                &mut name,
                320.0,
                |act| {
                    emitted_action = Some(act);
                },
            );
        }

        assert!(emitted_action.is_none());
    }

    #[test]
    fn test_card_header_structure_and_delete_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let desc = CardHeaderDescriptor::new("Test Component Title")
            .icon("🧪")
            .color(Color::rgba(0.2, 0.8, 0.4, 1.0))
            .action("×", UiScope::action_tag("TestComponent"));

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.card_header_desc(desc);
        }

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("CardHeader")),
            "CardHeader container must exist"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("CardActionButton")),
            "CardActionButton must exist"
        );

        let entity = hecs::Entity::DANGLING;
        {
            let scope = UiScope::new(&mut tree, root);
            assert!(component_delete_action(&scope, entity, "TestComponent").is_none());
        }

        // Frame click on action tag
        tree.event_state.frame_clicked_tag = Some(UiScope::action_tag("TestComponent"));
        let scope_clicked = UiScope::new(&mut tree, root);
        let action = component_delete_action(&scope_clicked, entity, "TestComponent");
        assert!(matches!(
            action,
            Some(EngineUiAction::RemoveComponent(e, "TestComponent")) if e == entity
        ));
    }
}