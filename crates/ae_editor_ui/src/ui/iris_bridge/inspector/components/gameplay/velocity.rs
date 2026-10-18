// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Velocity Component Inspector Card
//!
//! Provides a declarative inspector card for 3D linear velocity components (`X`, `Y`, `Z`).

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, build_declarative_card_header,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::tags::encode_inspector_number_input_tag;
use crate::ui::iris_bridge::inspector::types::{ComponentCategory, InspectorNumberInputId};
use irisui::prelude::*;

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
        let [mut vx, mut vy, mut vz] = ctx
            .world
            .get::<&ae_core::ecs::Velocity>(ctx.entity)
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0.0, 0.0, 0.0]);

        scope.card_panel("VelocityCard", |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            let mut row_defs = [
                ("X:", &mut vx, InspectorNumberInputId::VelocityX),
                ("Y:", &mut vy, InspectorNumberInputId::VelocityY),
                ("Z:", &mut vz, InspectorNumberInputId::VelocityZ),
            ];
            for (lbl, val, input_id) in &mut row_defs {
                let tag = encode_inspector_number_input_tag(*input_id);
                let edit = ctx.edit_state(*input_id);
                if card
                    .add(
                        NumericDrag::new(lbl, val)
                            .tag(tag)
                            .label_width(20.0)
                            .box_width(44.0)
                            .edit_state(edit),
                    )
                    .changed()
                {
                    ctx.set_number(*input_id, **val);
                }
            }
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(
            entity,
            ae_core::ecs::Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::tags::{
        encode_component_delete_tag, encode_inspector_number_input_tag,
    };
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_velocity_render_tags() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Velocity {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },));
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
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let handler = VelocityHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("Velocity");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let vx_tag = encode_inspector_number_input_tag(InspectorNumberInputId::VelocityX);
        assert!(tree.iter().any(|(_, n)| n.tag == vx_tag));
    }
}