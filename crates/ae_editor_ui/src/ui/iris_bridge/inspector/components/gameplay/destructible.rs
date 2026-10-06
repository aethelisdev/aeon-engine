// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Destructible Target Component Inspector Card
//!
//! Provides a declarative inspector card for destructible entities with numeric health readout and visual health bar.

use crate::ui::iris_bridge::inspector::components::physics::helpers::{
    ComponentHeaderProps, build_declarative_card_header,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for `🎯 DestructibleTarget` health component.
pub struct DestructibleTargetHandler;

impl ComponentInspectorHandler for DestructibleTargetHandler {
    fn component_name(&self) -> &'static str {
        "DestructibleTarget"
    }

    fn display_title(&self) -> &'static str {
        "Destructible Target"
    }

    fn icon(&self) -> &'static str {
        "🎯"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.95, 0.35, 0.35, 1.0) // Coral Red
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world
            .get::<&ae_core::ecs::DestructibleTarget>(entity)
            .is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (health, max_health) = if let Ok(target) = ctx
            .world
            .get::<&ae_core::ecs::DestructibleTarget>(ctx.entity)
        {
            (target.health, target.max_health)
        } else {
            (100.0, 100.0)
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(6.0);

        scope.container_named("DestructibleTargetCard", card_style, |card| {
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

            // Row 1: Health Text
            let health_str = format!("Health: {:.0} / {:.0} HP", health, max_health);
            card.label_styled_passive(
                "TargetHealthLbl",
                &health_str,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 2: Health Bar Visual
            let frac = (health / max_health.max(1.0)).clamp(0.0, 1.0);
            let bar_w = (ctx.card_w - 16.0).max(10.0);
            let bar_h = 10.0;

            let track_style = Style::new()
                .width(bar_w)
                .height(bar_h)
                .background(Color::rgba(0.15, 0.16, 0.19, 0.95))
                .border(1.0, Color::rgba(0.22, 0.24, 0.28, 0.80))
                .border_radius(3.0);

            card.container_named("HealthBarTrack", track_style, |track| {
                if frac > 0.001 {
                    let fill_w = (bar_w * frac).max(2.0);
                    let fill_col = if frac > 0.5 {
                        Color::rgba(0.20, 0.85, 0.40, 0.95)
                    } else if frac > 0.25 {
                        Color::rgba(0.95, 0.75, 0.15, 0.95)
                    } else {
                        Color::rgba(0.95, 0.25, 0.25, 0.95)
                    };

                    let fill_style = Style::new()
                        .width(fill_w)
                        .height(bar_h)
                        .background(fill_col)
                        .border_radius(3.0);

                    track.empty_box_passive_named("HealthBarFill", fill_style);
                }
            });
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::DestructibleTarget::new(100.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_destructible_target_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::DestructibleTarget::new(100.0),));
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
        let handler = DestructibleTargetHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("DestructibleTarget");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));
    }
}