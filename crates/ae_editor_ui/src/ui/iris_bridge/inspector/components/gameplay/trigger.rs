// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Trigger Zone Component Inspector Card
//!
//! Provides a declarative inspector card for proximity-activated trigger zones and elevation targets.

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

#[inline]
const fn trigger_status_text(is_triggered: bool) -> &'static str {
    if is_triggered {
        "Status: ● ACTIVATED"
    } else {
        "Status: ○ IDLE"
    }
}

#[inline]
fn trigger_status_color(theme: Theme, is_triggered: bool) -> Color {
    if is_triggered {
        theme.success
    } else {
        theme.text_muted
    }
}

/// Renders the declarative Trigger Zone component card with speed scrubber, target position vector, and proximity status pill.
pub fn draw_trigger_zone_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    zone: &mut ae_core::ecs::TriggerZone,
) {
    scope.card_panel("TriggerZoneCard", |card| {
        TriggerZoneHandler.render_header(card);

        card.prop_f32("Speed (m/s):", &mut zone.speed, 0.0..=100.0, 0.1)
            .on_commit(|_| {
                ctx.begin_edit("TriggerZone");
                ctx.commit_edit("TriggerZone");
            });

        card.prop_vec3("Target Pos:", &mut zone.target_position, 0.1, None)
            .on_commit(|_| {
                ctx.begin_edit("TriggerZone");
                ctx.commit_edit("TriggerZone");
            });

        card.status_pill(
            "TriggerStatusPill",
            "TriggerStatusLbl",
            trigger_status_text(zone.is_triggered),
            trigger_status_color(card.theme(), zone.is_triggered),
        );
    });
}

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
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::TriggerZone>(ctx.entity),
            |s, mut zone| draw_trigger_zone_card(s, ctx, &mut zone),
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::TriggerZone::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_trigger_zone_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::TriggerZone::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let handler = TriggerZoneHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("TriggerZone");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let has_status = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("TriggerStatusLbl"));
        assert!(has_status, "TriggerStatusLbl must be rendered in tree");
    }

    #[test]
    fn test_draw_trigger_zone_card_direct_structure() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::TriggerZone::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let mut scope = UiScope::new(&mut tree, root);
        let mut zone = world.get::<&mut ae_core::ecs::TriggerZone>(entity).unwrap();

        draw_trigger_zone_card(&mut scope, &mut ctx, &mut zone);

        assert!(
            tree.has_node_named("TriggerZoneCard"),
            "TriggerZoneCard container must be rendered"
        );
        assert!(
            tree.has_node_named("TriggerStatusPill"),
            "TriggerStatusPill container must be present"
        );
        assert!(
            tree.has_node_named("TriggerStatusLbl"),
            "TriggerStatusLbl label must be present"
        );
    }
}