// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Destructible Target Component Inspector Card
//!
//! Provides a declarative inspector card for destructible entities with numeric health readout and visual health bar.

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

/// Renders the declarative Destructible Target component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates interactive health scrubbers and a visual progress health bar with zero intermediate variable plumbing.
pub fn draw_destructible_target_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok(
        "DestructibleTargetCard",
        UiScope::comp_card_desc(
            "DestructibleTarget",
            "Destructible Target",
            "🎯",
            Color::rgba(0.95, 0.35, 0.35, 1.0),
        ),
        world.get::<&mut ae_core::ecs::DestructibleTarget>(entity),
        |card, mut target| {
            card.prop_f32("Health:", &mut target.health, 0.0..=100_000.0, 1.0);
            card.prop_f32("Max Health:", &mut target.max_health, 1.0..=100_000.0, 1.0);
            card.progress_bar_named(
                "HealthBarTrack",
                "HealthBarFill",
                244.0,
                10.0,
                health_fraction(target.health, target.max_health),
                health_color(card.theme(), target.health, target.max_health),
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destructible_target_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::DestructibleTarget::new(100.0),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut scope = UiScope::new(&mut tree, root);
        draw_destructible_target_card(&mut scope, &world, entity);

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
        let entity = world.spawn((ae_core::ecs::DestructibleTarget::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_destructible_target_card(&mut scope, &world, entity);

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

    #[test]
    fn test_draw_destructible_target_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_destructible_target_card(&mut scope, &world, entity);

        assert_eq!(
            tree.len(),
            1,
            "DestructibleTarget card must safely do nothing when component missing"
        );
    }
}