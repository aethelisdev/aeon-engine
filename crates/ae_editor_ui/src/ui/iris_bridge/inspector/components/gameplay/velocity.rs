// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Velocity Component Inspector Card
//!
//! Provides a declarative inspector card for 3D linear velocity components (`X`, `Y`, `Z`).

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Renders the declarative Velocity component card with 3D linear velocity coordinate vector (X, Y, Z).
pub fn draw_velocity_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    v: &mut ae_core::ecs::Velocity,
) {
    scope.card_panel("VelocityCard", |card| {
        VelocityHandler.render_header(card);

        card.prop_vec3_xyz(
            "Linear",
            (&mut v.x, &mut v.y, &mut v.z),
            0.05,
            Some([0.0, 0.0, 0.0]),
        )
        .on_commit(|_| {
            ctx.begin_edit("Velocity");
            ctx.commit_edit("Velocity");
        });
    });
}

/// Inspector handler for `📎 Velocity`.
pub struct VelocityHandler;

impl ComponentInspectorHandler for VelocityHandler {
    fn component_name(&self) -> &'static str {
        "Velocity"
    }

    fn display_title(&self) -> &'static str {
        "Velocity"
    }

    fn icon(&self) -> &'static str {
        "📎"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.22, 0.74, 0.98, 1.0) // Sky Blue / Cyan (#38bdf8)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Velocity>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::Velocity>(ctx.entity),
            |s, mut v| draw_velocity_card(s, ctx, &mut v),
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(
            entity,
            ae_core::ecs::Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_velocity_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
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
        let handler = VelocityHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("Velocity");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Velocity card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| {
                n.role == irisui::core::WidgetRole::NumericInput
                    && n.name
                        .as_deref()
                        .unwrap_or("")
                        .starts_with("NumBox_Linear_")
            })
            .count();
        assert_eq!(
            numeric_pills, 3,
            "Velocity card must emit 3 numeric pills (X, Y, Z)"
        );

        // Test in-place mutation and undo snapshotting
        {
            let mut v = world.get::<&mut ae_core::ecs::Velocity>(entity).unwrap();
            ctx.begin_edit(handler.component_name());
            v.x = 5.5;
            v.y = -4.2;
            v.z = 10.0;
            ctx.commit_edit(handler.component_name());
        }

        {
            let v = world.get::<&ae_core::ecs::Velocity>(entity).unwrap();
            assert_eq!(v.x, 5.5);
            assert_eq!(v.y, -4.2);
            assert_eq!(v.z, 10.0);
        }
    }

    #[test]
    fn test_draw_velocity_card_direct_structure() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
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
        let mut scope = UiScope::new(&mut tree, root);
        let mut v = world.get::<&mut ae_core::ecs::Velocity>(entity).unwrap();

        draw_velocity_card(&mut scope, &mut ctx, &mut v);

        assert!(
            tree.has_node_named("VelocityCard"),
            "VelocityCard container must be rendered"
        );
        assert!(
            tree.has_node_named("NumBox_Linear_X"),
            "NumBox_Linear_X must be present"
        );
        assert!(
            tree.has_node_named("NumBox_Linear_Y"),
            "NumBox_Linear_Y must be present"
        );
        assert!(
            tree.has_node_named("NumBox_Linear_Z"),
            "NumBox_Linear_Z must be present"
        );
    }
}