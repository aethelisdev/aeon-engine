// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Moving Platform Component Inspector Card
//!
//! Provides a declarative inspector card for waypoint-driven moving platforms.

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, build_declarative_card_header,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for `🚡 MovingPlatform` waypoint translation component.
pub struct MovingPlatformHandler;

impl ComponentInspectorHandler for MovingPlatformHandler {
    fn component_name(&self) -> &'static str {
        "MovingPlatform"
    }

    fn display_title(&self) -> &'static str {
        "Moving Platform"
    }

    fn icon(&self) -> &'static str {
        "🚡"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.70, 0.50, 1.0, 1.0) // Soft Purple
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::MovingPlatform>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (speed, target_pos) =
            if let Ok(plat) = ctx.world.get::<&ae_core::ecs::MovingPlatform>(ctx.entity) {
                (plat.speed, plat.target_position)
            } else {
                (2.5, [0.0, 5.0, 0.0])
            };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("MovingPlatformCard", card_style, |card| {
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

            // Row 1: Speed
            let speed_str = format!("Speed: {:.1} m/s", speed);
            card.label_styled_passive(
                "PlatformSpeedLbl",
                &speed_str,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 2: Target Position
            let target_str = format!(
                "Target Pos: ({:.1}, {:.1}, {:.1})",
                target_pos[0], target_pos[1], target_pos[2]
            );
            card.label_styled_passive(
                "PlatformTargetLbl",
                &target_str,
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::MovingPlatform::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_moving_platform_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::MovingPlatform::default(),));
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
            hovered_tag: None,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let handler = MovingPlatformHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("MovingPlatform");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));
    }
}