// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character Action Component Inspector Card
//!
//! Provides a declarative inspector card for configuring action speed, range, and cooldown times.

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for `💨 CharacterAction`.
pub struct CharacterActionHandler;

impl ComponentInspectorHandler for CharacterActionHandler {
    fn component_name(&self) -> &'static str {
        "CharacterAction"
    }

    fn display_title(&self) -> &'static str {
        "CharacterAction"
    }

    fn icon(&self) -> &'static str {
        "💨"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.68, 0.40, 0.96, 1.0) // Purple / Violet (#a855f7)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::CharacterAction>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut a) = ctx
            .world
            .get::<&mut ae_core::ecs::CharacterAction>(ctx.entity)
        else {
            return;
        };

        scope.card_panel("CharacterActionCard", |card| {
            self.render_header(card);

            ctx.prop_f32(
                card,
                self.component_name(),
                "Speed / Range:",
                &mut a.speed,
                0.0..=10_000.0,
                0.5,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Cooldown:",
                &mut a.cooldown,
                0.0..=3_600.0,
                0.05,
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::CharacterAction::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_character_action_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::CharacterAction::new(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        let mut ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            10.0,
            20.0,
            260.0,
        );
        let handler = CharacterActionHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("CharacterAction");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "CharacterAction card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "CharacterAction card must emit 2 numeric pills (Speed, Cooldown)"
        );

        // Test in-place mutation and undo snapshotting
        {
            let mut a = world
                .get::<&mut ae_core::ecs::CharacterAction>(entity)
                .unwrap();
            ctx.begin_edit(handler.component_name());
            a.speed = 120.0;
            a.cooldown = 1.5;
            ctx.commit_edit(handler.component_name());
        }

        {
            let a = world.get::<&ae_core::ecs::CharacterAction>(entity).unwrap();
            assert_eq!(a.speed, 120.0);
            assert_eq!(a.cooldown, 1.5);
        }
    }
}