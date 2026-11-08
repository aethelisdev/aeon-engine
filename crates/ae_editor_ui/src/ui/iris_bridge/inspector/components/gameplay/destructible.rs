// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Destructible Target Component Inspector Card
//!
//! Provides a declarative inspector card for destructible entities with numeric health readout and visual health bar.

use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::ComponentCategory;
use irisui::prelude::*;

#[inline]
const fn health_fraction(health: f32, max_health: f32) -> f32 {
    if max_health > 0.0 {
        health / max_health
    } else {
        0.0
    }
}

#[inline]
fn health_color(theme: Theme, health: f32, max_health: f32) -> Color {
    match health_fraction(health, max_health) {
        f if f > 0.5 => theme.success,
        f if f > 0.25 => theme.warning,
        _ => theme.danger,
    }
}

/// Renders the declarative Destructible Target component card with interactive health scrubbers and visual health bar.
pub fn draw_destructible_target_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    target: &mut ae_core::ecs::DestructibleTarget,
) {
    scope.card_panel("DestructibleTargetCard", |card| {
        DestructibleTargetHandler.render_header(card);

        card.prop_f32("Health:", &mut target.health, 0.0..=100_000.0, 1.0)
            .on_commit(|_| {
                ctx.begin_edit("DestructibleTarget");
                ctx.commit_edit("DestructibleTarget");
            });

        card.prop_f32("Max Health:", &mut target.max_health, 1.0..=100_000.0, 1.0)
            .on_commit(|_| {
                ctx.begin_edit("DestructibleTarget");
                ctx.commit_edit("DestructibleTarget");
            });

        card.progress_bar_named(
            "HealthBarTrack",
            "HealthBarFill",
            (ctx.card_w - 16.0).max(10.0),
            10.0,
            health_fraction(target.health, target.max_health),
            health_color(card.theme(), target.health, target.max_health),
        );
    });
}

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
        scope.when_ok(
            ctx.world
                .get::<&mut ae_core::ecs::DestructibleTarget>(ctx.entity),
            |s, mut target| draw_destructible_target_card(s, ctx, &mut target),
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::DestructibleTarget::new(100.0)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_destructible_target_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::DestructibleTarget::new(100.0),));
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
        let handler = DestructibleTargetHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("DestructibleTarget");
        assert!(tree.iter().any(|(_, n)| n.tag == del_tag));

        let has_track = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("HealthBarTrack"));
        let has_fill = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("HealthBarFill"));
        assert!(
            has_track && has_fill,
            "HealthBarTrack and HealthBarFill must be rendered"
        );
    }

    #[test]
    fn test_draw_destructible_target_card_direct_structure() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::DestructibleTarget::new(100.0),));
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
        let mut target = world
            .get::<&mut ae_core::ecs::DestructibleTarget>(entity)
            .unwrap();

        draw_destructible_target_card(&mut scope, &mut ctx, &mut target);

        assert!(
            tree.has_node_named("DestructibleTargetCard"),
            "DestructibleTargetCard container must be rendered"
        );
        assert!(
            tree.has_node_named("HealthBarTrack"),
            "HealthBarTrack must be present"
        );
        assert!(
            tree.has_node_named("HealthBarFill"),
            "HealthBarFill must be present"
        );
    }
}