// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Skeletal Animation Inspector Card
//!
//! Provides inspection and playback feedback for AnimationPlayer components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use irisui::prelude::*;

#[inline]
const fn status_color(theme: Theme, state: ae_animation::AnimationState) -> Color {
    match state {
        ae_animation::AnimationState::Playing => theme.success,
        ae_animation::AnimationState::Paused => theme.warning,
        ae_animation::AnimationState::Stopped => theme.text_muted,
    }
}

/// Renders the declarative Animation Player component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates playback state, clip info, skeleton joints, speed scrubber, and looping toggle with zero intermediate variable plumbing.
pub fn draw_animation_player_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "AnimationPlayerCard",
        UiScope::comp_card_desc(
            "AnimationPlayer",
            "Animation Player",
            "🎬",
            Color::rgba(0.95, 0.45, 0.70, 1.0),
        ),
        world.get::<&mut ae_animation::AnimationPlayer>(entity),
        |card, mut player| {
            card.labeled_status(
                "Status:",
                player.state.label(),
                status_color(card.theme(), player.state),
            );
            card.key_value("Clip", player.clip_name());
            card.branch_ok(
                world.get::<&ae_animation::Skeleton>(entity),
                |card, skel| {
                    card.key_value_number_named(
                        "AnimSkeletonInfo",
                        "Joints",
                        skel.joints.len() as f64,
                        0,
                    );
                    card.key_value_number("Duration (s)", player.clip_duration() as f64, 2);
                },
                |card, _| {
                    card.label_desc_colored(
                        "AnimSkeletonInfo",
                        "ℹ Static 3D Mesh (No Armature found)",
                        card.theme().warning,
                    );
                },
            );
            card.prop_f32("Speed:", &mut player.speed, 0.0..=10.0, 0.05);
            card.prop_checkbox("Looping", &mut player.looping);
        },
    );
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
        draw_animation_player_card(scope, ctx.world, ctx.entity);
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
        let mut scope = UiScope::new(&mut tree, root);

        draw_animation_player_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("AnimationPlayerCard")),
            "AnimationPlayerCard container must be rendered"
        );
        let has_playing = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("LabeledStatusValue")
                && n.text.as_deref().is_some_and(|t| t.contains("PLAYING"))
        });
        assert!(has_playing, "LabeledStatusValue must display PLAYING");
        assert!(
            tree.iter().any(|(_, n)| n
                .text
                .as_deref()
                .is_some_and(|t| t.contains("No Clip Selected"))),
            "Clip readout must be rendered"
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

    #[test]
    fn test_draw_animation_player_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_animation_player_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("AnimationPlayerCard")),
            "AnimationPlayer card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_animation_domain_methods() {
        assert_eq!(ae_animation::AnimationState::Playing.label(), "▶ PLAYING");
        assert_eq!(ae_animation::AnimationState::Paused.label(), "⏸ PAUSED");
        assert_eq!(ae_animation::AnimationState::Stopped.label(), "⏹ STOPPED");
        let player = ae_animation::AnimationPlayer::default();
        assert_eq!(player.clip_name(), "No Clip Selected");
        assert_eq!(player.clip_duration(), 0.0);
    }
}