// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Material Component Inspector Card
//!
//! Provides the `PhysicsMaterialHandler` implementing declarative UI rendering, surface type
//! combobox with preset action button, and friction / restitution numeric controls with two-way data-binding.

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext, PropComboRowLayout,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

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
        let Ok(mut m) = ctx
            .world
            .get::<&mut ae_core::ecs::PhysicsMaterial>(ctx.entity)
        else {
            return;
        };

        let mut surface_type_idx = match m.surface_type {
            ae_core::ecs::SurfaceType::Default => 0,
            ae_core::ecs::SurfaceType::Metal => 1,
            ae_core::ecs::SurfaceType::Wood => 2,
            ae_core::ecs::SurfaceType::Stone => 3,
            ae_core::ecs::SurfaceType::Flesh => 4,
            ae_core::ecs::SurfaceType::Dirt => 5,
            ae_core::ecs::SurfaceType::Glass => 6,
            ae_core::ecs::SurfaceType::Rubber => 7,
        };

        scope.card_panel("PhysicsMaterialCard", |card| {
            self.render_header(card);

            // Row 1: Surface Type Dropdown + Preset Reset Button
            let row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(22.0)
                .gap(4.0);

            card.container_named("SurfaceTypeRow", row_style, |row| {
                let options = &[
                    "Default", "Metal", "Wood", "Stone", "Flesh", "Dirt", "Glass", "Rubber",
                ];
                ctx.prop_combobox_row(
                    row,
                    self.component_name(),
                    "Surface Type:",
                    &mut surface_type_idx,
                    options,
                    PropComboRowLayout::default(),
                );
                m.surface_type = match surface_type_idx {
                    0 => ae_core::ecs::SurfaceType::Default,
                    1 => ae_core::ecs::SurfaceType::Metal,
                    2 => ae_core::ecs::SurfaceType::Wood,
                    3 => ae_core::ecs::SurfaceType::Stone,
                    4 => ae_core::ecs::SurfaceType::Flesh,
                    5 => ae_core::ecs::SurfaceType::Dirt,
                    6 => ae_core::ecs::SurfaceType::Glass,
                    _ => ae_core::ecs::SurfaceType::Rubber,
                };

                // ↺ Preset button
                let preset_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(62.0)
                    .height(22.0)
                    .background(Color::rgba(0.16, 0.17, 0.20, 0.95))
                    .border(1.0, Color::rgba(0.24, 0.26, 0.30, 0.85))
                    .border_radius(4.0);

                let preset_tag =
                    crate::ui::iris_bridge::inspector::tags::TAG_INSPECTOR_PRESET_RESET;
                let preset_resp = row.container_tagged(
                    "PresetBtn",
                    preset_style,
                    WidgetRole::Button,
                    preset_tag,
                    |btn| {
                        btn.label_styled_passive(
                            "PresetText",
                            "↺ Preset",
                            10.5,
                            btn.theme().text_secondary,
                            TextAlign::Center,
                            Style::new().width(62.0).height(22.0),
                        );
                    },
                );
                if preset_resp.response.clicked() {
                    ctx.begin_edit(self.component_name());
                    let surf = m.surface_type;
                    *m = ae_core::ecs::PhysicsMaterial::from_preset(surf);
                    ctx.commit_edit(self.component_name());
                }
            });

            // Row 2: Friction
            ctx.prop_f32(
                card,
                self.component_name(),
                "Friction:",
                &mut m.friction,
                0.0..=100.0,
                0.05,
            );

            // Row 3: Restitution
            ctx.prop_f32(
                card,
                self.component_name(),
                "Restitution:",
                &mut m.restitution,
                0.0..=1.0,
                0.05,
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let mat = ae_core::ecs::PhysicsMaterial {
            friction: 0.70,
            restitution: 0.0,
            surface_type: ae_core::ecs::SurfaceType::Default,
        };
        let _ = world.insert_one(entity, mat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::InspectorAction;
    use crate::ui::iris_bridge::inspector::tags::{
        TAG_INSPECTOR_PRESET_RESET, encode_component_delete_tag,
    };
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
            .any(|(_, n)| n.tag == TAG_INSPECTOR_PRESET_RESET);
        assert!(has_preset, "Preset reset button must be tagged");

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
    fn test_physics_material_declarative_in_place_mutation_and_undo() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PhysicsMaterial {
            friction: 0.70,
            restitution: 0.0,
            surface_type: ae_core::ecs::SurfaceType::Default,
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

        ctx.begin_edit("PhysicsMaterial");
        if let Ok(mut m) = ctx
            .world
            .get::<&mut ae_core::ecs::PhysicsMaterial>(ctx.entity)
        {
            m.friction = 0.40;
            m.restitution = 0.80;
            m.surface_type = ae_core::ecs::SurfaceType::Rubber;
        }
        ctx.commit_edit("PhysicsMaterial");

        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[0],
            InspectorAction::StartComponentEdit(entity, "PhysicsMaterial", None)
        );
        assert_eq!(
            actions[1],
            InspectorAction::CommitComponentEdit(entity, "PhysicsMaterial")
        );

        let mat = world.get::<&ae_core::ecs::PhysicsMaterial>(entity).unwrap();
        assert!((mat.friction - 0.40).abs() < f32::EPSILON);
        assert!((mat.restitution - 0.80).abs() < f32::EPSILON);
        assert_eq!(mat.surface_type, ae_core::ecs::SurfaceType::Rubber);
    }
}