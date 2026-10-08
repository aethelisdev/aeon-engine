// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rotator Component Inspector Card
//!
//! Provides a declarative inspector card for continuous rotation components around an arbitrary 3D axis.

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, DeclarativeNumericRowParams, build_declarative_card_header,
    render_declarative_numeric_row,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{ComponentCategory, InspectorNumberInputId};
use irisui::prelude::*;

/// Inspector handler for `🔄 Rotator`.
pub struct RotatorHandler;

impl ComponentInspectorHandler for RotatorHandler {
    fn component_name(&self) -> &'static str {
        "Rotator"
    }

    fn display_title(&self) -> &'static str {
        "Rotator (Continuous Spin)"
    }

    fn icon(&self) -> &'static str {
        "🔄"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.35, 0.85, 0.90, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Rotator>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (speed, axis) = ctx
            .world
            .get::<&ae_core::ecs::Rotator>(ctx.entity)
            .map(|r| (r.speed, r.axis))
            .unwrap_or((1.5, [0.0, 1.0, 0.0]));

        let get_edit = |id| {
            ctx.params
                .active_number_input
                .filter(|s| s.id == id)
                .map(|s| s.to_edit_state(ctx.params.blink_caret))
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("RotatorCard", card_style, |card| {
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

            let label_w = 95.0;
            let box_w = 44.0;

            let row_defs = [
                (
                    "Speed (rad/s):",
                    speed,
                    InspectorNumberInputId::RotatorSpeed,
                ),
                ("Axis X:", axis[0], InspectorNumberInputId::RotatorAxisX),
                ("Axis Y:", axis[1], InspectorNumberInputId::RotatorAxisY),
                ("Axis Z:", axis[2], InspectorNumberInputId::RotatorAxisZ),
            ];

            for (lbl, val, input_id) in row_defs {
                render_declarative_numeric_row(
                    card,
                    DeclarativeNumericRowParams {
                        input_id,
                        label: lbl,
                        val,
                        label_w,
                        box_w,
                        unit: None,
                        edit_state: get_edit(input_id),
                        is_hovered: false,
                    },
                );
            }
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(
            entity,
            ae_core::ecs::Rotator {
                speed: 1.5,
                axis: [0.0, 1.0, 0.0],
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
    fn test_rotator_render_tags() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Rotator {
            speed: 1.5,
            axis: [0.0, 1.0, 0.0],
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
            scroll_y: 0.0,
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
        let handler = RotatorHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("Rotator");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let speed_tag = encode_inspector_number_input_tag(InspectorNumberInputId::RotatorSpeed);
        assert!(tree.iter().any(|(_, n)| n.tag == speed_tag));
    }
}