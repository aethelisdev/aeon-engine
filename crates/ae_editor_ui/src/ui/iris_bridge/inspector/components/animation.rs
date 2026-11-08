// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Skeletal Animation Inspector Card
//!
//! Provides inspection and playback feedback for AnimationPlayer components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use irisui::prelude::*;

/// Renders the declarative Animation Player component card with playback state, clip info,
/// skeleton joints, speed scrubber, and looping toggle.
///
/// Modifies `speed` and `looping` in-place via [`UiScope::prop_f32`] and [`UiScope::prop_checkbox`],
/// automatically recording atomic Undo/Redo sessions on edit commits without intermediate imperative plumbing.
pub fn draw_animation_player_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    player: &mut ae_animation::AnimationPlayer,
) {
    scope.card_panel("AnimationPlayerCard", |card| {
        AnimationPlayerHandler.render_header(card);

        // Row 1: Status Row (Horizontal: "Status:" + state badge)
        card.container_named(
            "AnimStatusRow",
            Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(20.0)
                .gap(6.0),
            |row| {
                row.label_styled_passive(
                    "AnimStatusLbl",
                    "Status:",
                    11.0,
                    row.theme().text_muted,
                    TextAlign::Left,
                    Style::new().width(48.0).height(20.0),
                );
                row.label_styled_passive(
                    "AnimStatusVal",
                    match player.state {
                        ae_animation::AnimationState::Playing => "▶ PLAYING",
                        ae_animation::AnimationState::Paused => "⏸ PAUSED",
                        ae_animation::AnimationState::Stopped => "⏹ STOPPED",
                    },
                    11.0,
                    match player.state {
                        ae_animation::AnimationState::Playing => Color::rgba(0.20, 0.85, 0.40, 1.0),
                        ae_animation::AnimationState::Paused => Color::rgba(0.95, 0.75, 0.15, 1.0),
                        ae_animation::AnimationState::Stopped => Color::rgba(0.60, 0.62, 0.68, 1.0),
                    },
                    TextAlign::Left,
                    Style::new().flex_grow(1.0).height(20.0),
                );
            },
        );

        // Row 2: Active Clip
        card.label_styled_passive(
            "AnimClipLbl",
            format!(
                "Clip: {}",
                player
                    .current_clip
                    .as_ref()
                    .map_or("No Clip Selected", |c| c.name.as_str())
            ),
            11.0,
            card.theme().text_primary,
            TextAlign::Left,
            Style::new().height(20.0),
        );

        // Row 3: Skeleton Info / Static Warning
        card.label_styled_passive(
            "AnimSkeletonInfo",
            match ctx
                .world
                .get::<&ae_animation::Skeleton>(ctx.entity)
                .map(|s| s.joints.len())
                .ok()
            {
                Some(joints) => format!(
                    "🦴 Joints: {} | ⏱ Duration: {:.2}s",
                    joints,
                    player.current_clip.as_ref().map_or(0.0, |c| c.duration)
                ),
                None => "ℹ Static 3D Mesh (No Armature found)".to_string(),
            },
            10.0,
            match ctx.world.get::<&ae_animation::Skeleton>(ctx.entity).is_ok() {
                true => card.theme().info,
                false => card.theme().warning,
            },
            TextAlign::Left,
            Style::new().height(20.0),
        );

        // Row 4: Speed & Looping Indicator
        card.label_styled_passive(
            "AnimSpeedLoop",
            format!(
                "Speed: {:.2}x  |  Loop: {}",
                player.speed,
                if player.looping { "Yes" } else { "No" }
            ),
            10.5,
            card.theme().text_muted,
            TextAlign::Left,
            Style::new().height(20.0),
        );

        // Interactive controls
        card.prop_f32("Speed:", &mut player.speed, 0.0..=10.0, 0.05)
            .on_commit(|_| {
                ctx.begin_edit("AnimationPlayer");
                ctx.commit_edit("AnimationPlayer");
            });

        card.prop_checkbox("Looping", &mut player.looping)
            .on_click(|| {
                ctx.begin_edit("AnimationPlayer");
                ctx.commit_edit("AnimationPlayer");
            });
    });
}

/// Inspector handler for AnimationPlayer component.
pub struct AnimationPlayerHandler;

impl ComponentInspectorHandler for AnimationPlayerHandler {
    fn component_name(&self) -> &'static str {
        "AnimationPlayer"
    }

    fn display_title(&self) -> &'static str {
        "Animation Player"
    }

    fn icon(&self) -> &'static str {
        "🎬"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.95, 0.45, 0.70, 1.0) // Vibrant Rose / Magenta
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Animation
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_animation::AnimationPlayer>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world
                .get::<&mut ae_animation::AnimationPlayer>(ctx.entity),
            |card, mut player| {
                draw_animation_player_card(card, ctx, &mut player);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_animation::AnimationPlayer::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_animation_player_card_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_animation::AnimationPlayer::default(),));
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

        let handler = AnimationPlayerHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("AnimationPlayer");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "AnimationPlayer card must tag its delete button"
        );
    }

    #[test]
    fn test_draw_animation_player_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_animation::AnimationPlayer {
            state: ae_animation::AnimationState::Playing,
            speed: 1.5,
            looping: true,
            ..Default::default()
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
            20.0,
            260.0,
        );

        let mut player = ae_animation::AnimationPlayer {
            state: ae_animation::AnimationState::Playing,
            speed: 1.5,
            looping: true,
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_animation_player_card(&mut scope, &mut ctx, &mut player);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("AnimationPlayerCard")),
            "AnimationPlayerCard container must be rendered"
        );
        let has_playing = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AnimStatusVal")
                && n.text.as_deref().is_some_and(|t| t.contains("PLAYING"))
        });
        assert!(has_playing, "AnimStatusVal must display PLAYING");
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("AnimClipLbl")),
            "AnimClipLbl must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("AnimSkeletonInfo")),
            "AnimSkeletonInfo must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("NumericDrag")),
            "Speed NumericDrag must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("Checkbox")),
            "Looping Checkbox must be rendered"
        );
    }
}