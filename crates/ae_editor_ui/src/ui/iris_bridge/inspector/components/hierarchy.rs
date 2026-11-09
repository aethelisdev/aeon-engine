// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchy & Parenting Inspector Card
//!
//! Provides declarative inspection and unparenting actions for entity parent-child relationships.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;

/// Resolves the parent target entity's display name if present, without heap formatting or allocations.
#[inline]
#[must_use]
pub fn parent_name(world: &hecs::World, entity: hecs::Entity) -> Option<String> {
    world
        .get::<&ae_core::ecs::Parent>(entity)
        .ok()
        .and_then(|parent| {
            world
                .get::<&ae_core::ecs::Name>(parent.0)
                .ok()
                .map(|n| n.0.clone())
        })
}

/// Resolves the parent target entity's integer identifier if present, without heap formatting or allocations.
#[inline]
#[must_use]
pub fn parent_entity_id(world: &hecs::World, entity: hecs::Entity) -> Option<u64> {
    world
        .get::<&ae_core::ecs::Parent>(entity)
        .ok()
        .map(|parent| parent.0.to_bits().get())
}

/// Renders the Parent / Hierarchy card declaratively and returns [`EngineUiAction::UnparentEntity`]
/// directly when the `"❌ Unparent"` button is clicked.
pub fn draw_parent_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) -> Option<EngineUiAction> {
    scope
        .card_builder("Parent / Hierarchy")
        .name("ParentingCard")
        .icon("🔗")
        .color(Color::rgba(0.98, 0.80, 0.08, 1.0))
        .show(|card| {
            card.prop_optional_action_row(
                PropertyActionRow::reference_target(
                    "Parent: ",
                    parent_name(world, entity),
                    parent_entity_id(world, entity),
                    "Entity #",
                    "Parent: None (Root Entity)",
                    "❌ Unparent",
                )
                .with_names(
                    "ParentRow",
                    "ParentLbl",
                    "UnparentBtn",
                    "UnparentTxt",
                    "ParentRootLbl",
                ),
            )
            .map_click(|| EngineUiAction::UnparentEntity(entity))
        })
        .into_inner()
}

/// Renders the Parent / Hierarchy card if `entity` participates in a parent or child relationship.
pub fn draw_parent_card_if_present(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) -> Option<EngineUiAction> {
    scope
        .when(
            world.get::<&ae_core::ecs::Parent>(entity).is_ok()
                || world.get::<&ae_core::ecs::Children>(entity).is_ok(),
            |s| draw_parent_card(s, world, entity),
        )
        .flatten()
}

/// Inspector handler for entity hierarchy and parenting relationships.
pub struct ParentHandler;

impl ComponentInspectorHandler for ParentHandler {
    fn component_name(&self) -> &'static str {
        "Parenting"
    }

    fn display_title(&self) -> &'static str {
        "Parent / Hierarchy"
    }

    fn icon(&self) -> &'static str {
        "🔗"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.98, 0.80, 0.08, 1.0) // Gold / Warm Amber (#facc15)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Hierarchy
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Parent>(entity).is_ok()
            || world.get::<&ae_core::ecs::Children>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        drop(draw_parent_card_if_present(scope, ctx.world, ctx.entity));
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::Children::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_parent_card_emits_unparent_entity_directly_on_click() {
        let mut world = hecs::World::new();
        let parent = world.spawn((ae_core::ecs::Name("RootSceneNode".to_string()),));
        let child = world.spawn((ae_core::ecs::Parent(parent),));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.configure_container("TestRoot", Style::new().flex_col(), false);
            let action = draw_parent_card(&mut scope, &world, child);
            assert!(action.is_none());
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 120.0));
        }

        let unparent_btn = tree.assert_node("UnparentBtn", WidgetRole::Button);
        assert_ne!(unparent_btn.tag, 0);

        let btn_rect = tree.assert_node_rect("UnparentBtn");
        let click_pt = Point::new(btn_rect.x + 10.0, btn_rect.y + 10.0);
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        let emitted = {
            let mut scope = UiScope::new(&mut tree, root);
            draw_parent_card(&mut scope, &world, child)
        };
        let Some(EngineUiAction::UnparentEntity(ent)) = emitted else {
            panic!("Expected EngineUiAction::UnparentEntity");
        };
        assert_eq!(ent, child);
    }

    #[test]
    fn test_draw_parent_card_root_entity_display() {
        let mut world = hecs::World::new();
        let root_ent = world.spawn((ae_core::ecs::Children(vec![]),));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut scope = UiScope::new(&mut tree, root);
        let action = draw_parent_card(&mut scope, &world, root_ent);
        assert!(action.is_none());

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ParentingCard")),
            "ParentingCard must be rendered"
        );
        let has_root_lbl = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ParentRootLbl")
                && n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Parent: None (Root Entity)"))
        });
        assert!(has_root_lbl, "ParentRootLbl must display Root Entity");
        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("UnparentBtn")),
            "Unparent button must not exist for root entity"
        );
    }

    #[test]
    fn test_draw_parent_card_if_present_skips_orphan() {
        let mut world = hecs::World::new();
        let orphan = world.spawn((ae_core::ecs::Name("Orphan".to_string()),));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut scope = UiScope::new(&mut tree, root);
        let action = draw_parent_card_if_present(&mut scope, &world, orphan);
        assert!(action.is_none());

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ParentingCard")),
            "ParentingCard must not be rendered for non-participating entity"
        );
    }
}