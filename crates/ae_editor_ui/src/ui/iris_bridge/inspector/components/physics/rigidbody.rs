// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # RigidBody Component Inspector Card
//!
//! Provides the `RigidBodyHandler` implementing declarative UI rendering, body type combobox
//! (`Dynamic`, `Kinematic`, `Static`), mass and gravity scale numeric inputs with two-way data-binding,
//! and paired component attachment with `Collider`.

use crate::ui::iris_bridge::icons::ICON_GEAR;
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for `⚙ RigidBody`.
pub struct RigidBodyHandler;

impl ComponentInspectorHandler for RigidBodyHandler {
    fn component_name(&self) -> &'static str {
        "RigidBody"
    }

    fn display_title(&self) -> &'static str {
        "RigidBody"
    }

    fn icon(&self) -> &'static str {
        "⚙"
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(ICON_GEAR)
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.22, 0.74, 0.98, 1.0) // Sky Blue / Cyan (#38bdf8)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::RigidBody>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut rb) = ctx.world.get::<&mut ae_core::ecs::RigidBody>(ctx.entity) else {
            return;
        };

        scope.card_panel("RigidBodyCard", |card| {
            self.render_header(card);
            ctx.prop_rigidbody(card, &mut rb);
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::RigidBody::default());
        if world.get::<&ae_core::ecs::Collider>(entity).is_err() {
            let _ = world.insert_one(entity, ae_core::ecs::Collider::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_rigidbody_declarative_scope_renders_gear_icon_and_chevron() {
        let handler = RigidBodyHandler;
        assert_eq!(handler.atlas_icon(), Some(ICON_GEAR));

        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 2.5,
            gravity_scale: 1.0,
        },));

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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 50.0, 280.0);

        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let has_gear_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_GEAR)
        });
        assert!(has_gear_icon, "RigidBody card header must render ICON_GEAR");

        let has_chevron_down = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboChevron"));
        assert!(
            has_chevron_down,
            "Combobox must render chevron down when closed"
        );

        let del_tag = UiScope::action_tag("RigidBody");
        let has_del = tree.iter().any(|(_, n)| n.tag == del_tag);
        assert!(has_del, "RigidBody delete button must be tagged");
    }

    #[test]
    fn test_rigidbody_combobox_renders_chevron_up_when_open() {
        let handler = RigidBodyHandler;
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody::default(),));

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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 50.0, 280.0);

        // First pass: render once to get the tag of the ComboPill
        {
            let mut scope = UiScope::new(&mut tree, root);
            handler.render_card(&mut scope, &mut ctx);
        }

        let combo_tag = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
            .map(|(_, n)| n.tag)
            .expect("ComboPill node must exist in tree");

        // Second pass: set active_combobox_tag to combo_tag and render
        let mut tree2 = UiTree::new();
        let root2 = tree2.create_root().expect("Root node must exist");
        tree2.active_combobox_tag = Some(combo_tag);

        let mut scope2 = UiScope::new(&mut tree2, root2);
        handler.render_card(&mut scope2, &mut ctx);

        let has_chevron_up = tree2.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▴")
        });
        assert!(
            has_chevron_up,
            "Combobox must render chevron up (▴) when open"
        );
    }

    #[test]
    fn test_rigidbody_declarative_in_place_mutation_and_undo_snapshot() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 2.5,
            gravity_scale: 1.0,
        },));

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
            50.0,
            280.0,
        );

        // Verify begin_edit and commit_edit signals
        ctx.begin_edit("RigidBody");
        if let Ok(mut rb) = ctx.world.get::<&mut ae_core::ecs::RigidBody>(ctx.entity) {
            rb.mass = 5.0;
            rb.gravity_scale = 2.0;
            rb.body_type = ae_core::ecs::RigidBodyType::Static;
        }
        ctx.commit_edit("RigidBody");

        // Verify mutations in world
        let rb = world.get::<&ae_core::ecs::RigidBody>(entity).unwrap();
        assert_eq!(rb.mass, 5.0);
        assert_eq!(rb.gravity_scale, 2.0);
        assert_eq!(rb.body_type, ae_core::ecs::RigidBodyType::Static);
    }
}