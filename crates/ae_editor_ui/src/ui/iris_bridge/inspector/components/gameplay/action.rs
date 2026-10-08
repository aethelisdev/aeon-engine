// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character Action Component Inspector Card
//!
//! Provides a declarative inspector card for configuring action speed, range, and cooldown times.

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, DeclarativeNumericRowParams, build_declarative_card_header,
    render_declarative_numeric_row_custom,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
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
        let act_data = ctx
            .world
            .get::<&ae_core::ecs::CharacterAction>(ctx.entity)
            .map(|a| (a.speed, a.cooldown))
            .unwrap_or((50.0, 0.20));

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

        scope.container_named("CharacterActionCard", card_style, |card| {
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
            let custom_speed = format!("{:.0}", act_data.0);
            render_declarative_numeric_row_custom(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::ActionSpeedRange,
                    label: "Speed / Range:",
                    val: act_data.0,
                    label_w,
                    box_w,
                    unit: Some("m/s"),
                    edit_state: get_edit(InspectorNumberInputId::ActionSpeedRange),
                    is_hovered: false,
                },
                Some(&custom_speed),
            );

            // Cooldown [ 0.20 ] s
            render_declarative_numeric_row_custom(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::ActionCooldown,
                    label: "Cooldown:",
                    val: act_data.1,
                    label_w,
                    box_w,
                    unit: Some("s"),
                    edit_state: get_edit(InspectorNumberInputId::ActionCooldown),
                    is_hovered: false,
                },
                None,
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
        let handler = CharacterActionHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("CharacterAction");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let speed_tag = encode_inspector_number_input_tag(InspectorNumberInputId::ActionSpeedRange);
        assert!(tree.iter().any(|(_, n)| n.tag == speed_tag));
    }
}