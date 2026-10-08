// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Trigger Zone Component Inspector Card
//!
//! Provides a declarative inspector card for proximity-activated trigger zones and elevation targets.

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, build_declarative_card_header,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for `⚡ TriggerZone` proximity sensor component.
pub struct TriggerZoneHandler;

impl ComponentInspectorHandler for TriggerZoneHandler {
    fn component_name(&self) -> &'static str {
        "TriggerZone"
    }

    fn display_title(&self) -> &'static str {
        "Trigger Zone"
    }

    fn icon(&self) -> &'static str {
        "⚡"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.98, 0.80, 0.15, 1.0) // Gold / Warm Amber
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::TriggerZone>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (is_triggered, speed, target_y) =
            if let Ok(zone) = ctx.world.get::<&ae_core::ecs::TriggerZone>(ctx.entity) {
                (zone.is_triggered, zone.speed, zone.target_position[1])
            } else {
                (false, 3.0, 4.0)
            };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("TriggerZoneCard", card_style, |card| {
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

            // Row 1: Status
            let (status_str, status_col) = if is_triggered {
                ("Status: ● ACTIVATED", Color::rgba(0.20, 0.85, 0.40, 1.0))
            } else {
                ("Status: ○ IDLE", Color::rgba(0.60, 0.62, 0.68, 1.0))
            };
            card.label_styled_passive(
                "TriggerStatusLbl",
                status_str,
                11.0,
                status_col,
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 2: Speed & Target Elevation
            let param_str = format!("Speed: {:.1} m/s  |  Target Y: {:.1} m", speed, target_y);
            card.label_styled_passive(
                "TriggerParamLbl",
                &param_str,
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::TriggerZone::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_trigger_zone_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::TriggerZone::default(),));
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
        let handler = TriggerZoneHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("TriggerZone");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));
    }
}