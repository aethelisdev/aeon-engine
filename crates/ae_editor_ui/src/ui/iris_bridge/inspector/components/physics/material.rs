// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Material Component Inspector Card
//!
//! Provides the `PhysicsMaterialHandler` implementing declarative UI rendering, surface type
//! combobox with preset action button, and friction / restitution numeric controls with two-way data-binding.

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Maps a [`ae_core::ecs::SurfaceType`] variant to its zero-based dropdown option index.
#[inline]
#[must_use]
const fn surface_type_to_index(surface: ae_core::ecs::SurfaceType) -> usize {
    match surface {
        ae_core::ecs::SurfaceType::Default => 0,
        ae_core::ecs::SurfaceType::Metal => 1,
        ae_core::ecs::SurfaceType::Wood => 2,
        ae_core::ecs::SurfaceType::Stone => 3,
        ae_core::ecs::SurfaceType::Flesh => 4,
        ae_core::ecs::SurfaceType::Dirt => 5,
        ae_core::ecs::SurfaceType::Glass => 6,
        ae_core::ecs::SurfaceType::Rubber => 7,
    }
}

/// Maps a zero-based dropdown option index to its corresponding [`ae_core::ecs::SurfaceType`] variant.
#[inline]
#[must_use]
const fn index_to_surface_type(idx: usize) -> ae_core::ecs::SurfaceType {
    match idx {
        0 => ae_core::ecs::SurfaceType::Default,
        1 => ae_core::ecs::SurfaceType::Metal,
        2 => ae_core::ecs::SurfaceType::Wood,
        3 => ae_core::ecs::SurfaceType::Stone,
        4 => ae_core::ecs::SurfaceType::Flesh,
        5 => ae_core::ecs::SurfaceType::Dirt,
        6 => ae_core::ecs::SurfaceType::Glass,
        _ => ae_core::ecs::SurfaceType::Rubber,
    }
}

/// Renders the declarative Physics Material component card with two-way bound surface type
/// dropdown, in-place preset reset action, and friction/restitution numeric scrubbers.
pub fn draw_physics_material_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    m: &mut ae_core::ecs::PhysicsMaterial,
) {
    scope.card_panel("PhysicsMaterialCard", |card| {
        PhysicsMaterialHandler.render_header(card);
        card.prop_combobox_mapped_with_action(
            "Surface Type:",
            surface_type_to_index(m.surface_type),
            &[
                "Default", "Metal", "Wood", "Stone", "Flesh", "Dirt", "Glass", "Rubber",
            ],
            "↺ Preset",
            &mut m.surface_type,
            index_to_surface_type,
        )
        .on_action(|| {
            *m = ae_core::ecs::PhysicsMaterial::from_preset(m.surface_type);
        })
        .on_change(|| {
            ctx.begin_edit("PhysicsMaterial");
            ctx.commit_edit("PhysicsMaterial");
        });
        ctx.prop_f32(
            card,
            "PhysicsMaterial",
            "Friction:",
            &mut m.friction,
            0.0..=100.0,
            0.05,
        );
        ctx.prop_f32(
            card,
            "PhysicsMaterial",
            "Restitution:",
            &mut m.restitution,
            0.0..=1.0,
            0.05,
        );
    });
}

/// Inspector handler for Physics Material.
pub struct PhysicsMaterialHandler;

impl ComponentInspectorHandler for PhysicsMaterialHandler {
    fn component_name(&self) -> &'static str {
        "PhysicsMaterial"
    }

    fn display_title(&self) -> &'static str {
        "Physics Material"
    }

    fn icon(&self) -> &'static str {
        "🧱"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.62, 0.15, 1.0) // Amber / Orange (#f59e0b)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::PhysicsMaterial>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        if let Ok(mut m) = ctx
            .world
            .get::<&mut ae_core::ecs::PhysicsMaterial>(ctx.entity)
        {
            draw_physics_material_card(scope, ctx, &mut m);
        }
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(
            entity,
            ae_core::ecs::PhysicsMaterial {
                friction: 0.70,
                restitution: 0.0,
                surface_type: ae_core::ecs::SurfaceType::Default,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::InspectorAction;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_physics_material_declarative_scope_renders_structure() {
        let handler = PhysicsMaterialHandler;
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PhysicsMaterial {
            friction: 0.75,
            restitution: 0.25,
            surface_type: ae_core::ecs::SurfaceType::Wood,
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

        let has_del = tree
            .iter()
            .any(|(_, n)| n.tag == encode_component_delete_tag("PhysicsMaterial"));
        assert!(has_del, "Delete button must be tagged");

        let has_preset = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("PresetBtn") && n.role == WidgetRole::Button);
        assert!(has_preset, "Preset reset button must exist");

        let has_combo = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
        assert!(has_combo, "Surface Type combobox pill must exist");

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "Must emit 2 numeric drag pills (Friction & Restitution)"
        );
    }

    #[test]
    fn test_physics_material_preset_button_in_place_mutation_and_undo() {
        let handler = PhysicsMaterialHandler;
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PhysicsMaterial {
            friction: 0.99,
            restitution: 0.99,
            surface_type: ae_core::ecs::SurfaceType::Rubber,
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

        {
            let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 50.0, 280.0);
            let mut scope = UiScope::new(&mut tree, root);
            handler.render_card(&mut scope, &mut ctx);
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 600.0));
        }

        let preset_rect = tree.assert_node_rect("PresetBtn");
        let click_pt = Point::new(preset_rect.x + 10.0, preset_rect.y + 10.0);

        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        let mut actions = Vec::new();
        {
            let mut ctx = ComponentRenderContext::with_actions(
                entity,
                &world,
                &params,
                &mut actions,
                10.0,
                50.0,
                280.0,
            );
            let mut scope = UiScope::new(&mut tree, root);
            handler.render_card(&mut scope, &mut ctx);
        }

        let expected =
            ae_core::ecs::PhysicsMaterial::from_preset(ae_core::ecs::SurfaceType::Rubber);
        let mat = world.get::<&ae_core::ecs::PhysicsMaterial>(entity).unwrap();
        assert!((mat.friction - expected.friction).abs() < f32::EPSILON);
        assert!((mat.restitution - expected.restitution).abs() < f32::EPSILON);
        assert_eq!(mat.surface_type, ae_core::ecs::SurfaceType::Rubber);

        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[0],
            InspectorAction::StartComponentEdit(entity, "PhysicsMaterial", None)
        );
        assert_eq!(
            actions[1],
            InspectorAction::CommitComponentEdit(entity, "PhysicsMaterial")
        );
    }
}