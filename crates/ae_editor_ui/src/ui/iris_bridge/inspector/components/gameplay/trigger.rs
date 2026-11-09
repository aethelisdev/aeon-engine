// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Trigger Zone Component Inspector Card
//!
//! Provides a declarative inspector card for proximity-activated trigger zones and elevation targets.

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

/// Renders the declarative Trigger Zone component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates speed scrubber, target position 3D vector, and proximity status pill with zero intermediate variable plumbing.
pub fn draw_trigger_zone_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "TriggerZoneCard",
        UiScope::comp_card_desc(
            "TriggerZone",
            "Trigger Zone",
            "⚡",
            Color::rgba(0.98, 0.80, 0.15, 1.0),
        ),
        world.get::<&mut ae_core::ecs::TriggerZone>(entity),
        |card, mut zone| {
            card.prop_f32("Speed (m/s):", &mut zone.speed, 0.0..=100.0, 0.1);
            card.prop_vec3("Target Pos:", &mut zone.target_position, 0.1, None);
            card.status_pill(
                "TriggerStatusPill",
                "TriggerStatusLbl",
                trigger_status_text(zone.is_triggered),
                trigger_status_color(card.theme(), zone.is_triggered),
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_zone_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::TriggerZone::default(),));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");

        let mut scope = UiScope::new(&mut tree, root);
        draw_trigger_zone_card(&mut scope, &world, entity);

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
        let mut scope = UiScope::new(&mut tree, root);

        draw_trigger_zone_card(&mut scope, &world, entity);

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

    #[test]
    fn test_draw_trigger_zone_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node");
        let mut scope = UiScope::new(&mut tree, root);

        draw_trigger_zone_card(&mut scope, &world, entity);

        assert_eq!(
            tree.len(),
            1,
            "TriggerZone card must safely do nothing when component missing"
        );
    }
}