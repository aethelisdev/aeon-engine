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
/// Encapsulates the entity name editor within a pure declarative [`UiScope`] property row,
/// managing clicking, focusing, keyboard typing, and inline string editing autonomously via
/// [`UiScope::text_input_with_width_tagged`] without external state bags or engine-side rename flags.
/// Emits [`WidgetResponse`] where [`WidgetResponse::changed`] signals a committed name change.
pub fn build_entity_header(
    scope: &mut UiScope<'_>,
    name: &mut String,
    panel_w: f32,
) -> WidgetResponse {
    scope
        .column_with()
        .name("EntityHeaderCard")
        .width(panel_w - 12.0)
        .height(30.0)
        .margin_insets(Insets::new(4.0, 6.0, 4.0, 6.0))
        .show(|header_scope| {
            header_scope
                .container_named(
                    "EntityHeaderRow",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .height(26.0)
                        .gap(6.0),
                    |row| {
                        row.label_styled_passive(
                            "NameLabel",
                            "🏷 Name:",
                            11.0,
                            row.theme().text_muted,
                            TextAlign::Left,
                            Style::new().width(55.0).height(24.0),
                        );
                        row.text_input_with_width_tagged(
                            "Entity name...",
                            entity_name_input_tag(),
                            (panel_w - 85.0).max(60.0),
                            name,
                        )
                    },
                )
                .inner
        })
        .inner
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
}