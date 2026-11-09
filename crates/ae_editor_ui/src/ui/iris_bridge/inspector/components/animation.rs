// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Skeletal Animation Inspector Card
//!
//! Provides inspection and playback feedback for AnimationPlayer components.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_player_card_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_animation::AnimationPlayer::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_animation_player_card(&mut scope, &world, entity);

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