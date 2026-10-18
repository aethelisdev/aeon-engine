// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character Action Component Inspector Card
//!
//! Provides a declarative inspector card for configuring action speed, range, and cooldown times.

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, build_declarative_card_header,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::tags::encode_inspector_number_input_tag;
use crate::ui::iris_bridge::inspector::types::{ComponentCategory, InspectorNumberInputId};
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
        let (mut speed, mut cooldown) = ctx
            .world
            .get::<&ae_core::ecs::CharacterAction>(ctx.entity)
            .map(|a| (a.speed, a.cooldown))
            .unwrap_or((50.0, 0.20));

        scope.card_panel("CharacterActionCard", |card| {
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

            // Speed / Range [ 50 ] m/s
            let speed_id = InspectorNumberInputId::ActionSpeedRange;
            let custom_speed = format!("{:.0}", speed);
            if card
                .add(
                    NumericDrag::new("Speed / Range:", &mut speed)
                        .tag(encode_inspector_number_input_tag(speed_id))
                        .label_width(label_w)
                        .box_width(box_w)
                        .unit("m/s")
                        .custom_text(&custom_speed)
                        .edit_state(ctx.edit_state(speed_id)),
                )
                .changed()
            {
                ctx.set_number(speed_id, speed);
            }

            // Cooldown [ 0.20 ] s
            let cd_id = InspectorNumberInputId::ActionCooldown;
            if card
                .add(
                    NumericDrag::new("Cooldown:", &mut cooldown)
                        .tag(encode_inspector_number_input_tag(cd_id))
                        .label_width(label_w)
                        .box_width(box_w)
                        .unit("s")
                        .edit_state(ctx.edit_state(cd_id)),
                )
                .changed()
            {
                ctx.set_number(cd_id, cooldown);
            }
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::CharacterAction::new());
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
    fn test_character_action_render_tags() {
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
        let handler = CharacterActionHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("CharacterAction");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let speed_tag = encode_inspector_number_input_tag(InspectorNumberInputId::ActionSpeedRange);
        assert!(tree.iter().any(|(_, n)| n.tag == speed_tag));
    }
}