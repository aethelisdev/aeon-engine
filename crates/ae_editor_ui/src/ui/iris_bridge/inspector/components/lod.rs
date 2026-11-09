// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Level of Detail (LOD) Group Inspector Card
//!
//! Provides inspection and distance threshold visualization for `📊 LodGroup` components.

use irisui::prelude::*;

/// Returns the static summary string for LOD slots availability without heap allocations.
#[inline]
#[must_use]
pub const fn lod_slots_text(has_lod1: bool, has_lod2: bool) -> &'static str {
    match (has_lod1, has_lod2) {
        (false, false) => "Slots: LOD0 (Active) | LOD1: None | LOD2: None",
        (true, false) => "Slots: LOD0 (Active) | LOD1: Set | LOD2: None",
        (false, true) => "Slots: LOD0 (Active) | LOD1: None | LOD2: Set",
        (true, true) => "Slots: LOD0 (Active) | LOD1: Set | LOD2: Set",
    }
}

/// Renders the declarative Level of Detail (LOD) Group component card with slots and distance thresholds.
///
/// Modifies `threshold_1` and `threshold_2` in-place via [`UiScope::prop_f32`] with zero heap allocations.
pub fn draw_lod_group_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok_mut(
        "LodGroupCard",
        UiScope::comp_card_desc(
            "LodGroup",
            "LOD Group",
            "📊",
            Color::rgba(0.20, 0.85, 0.65, 1.0),
        ),
        world.get::<&mut ae_core::ecs::LodGroup>(entity),
        |card, lod| {
            card.label_styled_passive(
                "LodSlotsInfo",
                lod_slots_text(lod.lod_1.is_some(), lod.lod_2.is_some()),
                10.5,
                card.theme().text_primary,
                TextAlign::Left,
                Style::new().height(20.0),
            );

            card.label_number_desc(
                "LodThresh1Lbl",
                NumberLabelDescriptor::new("LOD 0 ➔ 1 Distance: ", lod.threshold_1 as f64, " m")
                    .color(card.theme().text_muted),
            );

            card.prop_f32("LOD 0 ➔ 1 (m):", &mut lod.threshold_1, 1.0..=100.0, 0.5);

            card.label_number_desc(
                "LodThresh2Lbl",
                NumberLabelDescriptor::new("LOD 1 ➔ 2 Distance: ", lod.threshold_2 as f64, " m")
                    .color(card.theme().text_muted),
            );

            card.prop_f32("LOD 1 ➔ 2 (m):", &mut lod.threshold_2, 5.0..=200.0, 1.0);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_group_card_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::LodGroup::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_lod_group_card(&mut scope, &world, entity);

        let del_tag = UiScope::action_tag("LodGroup");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "LodGroup card must tag its delete button"
        );
    }

    #[test]
    fn test_draw_lod_group_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::LodGroup {
            threshold_1: 15.0,
            threshold_2: 35.0,
            lod_1: None,
            lod_2: None,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_lod_group_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("LodGroupCard")),
            "LodGroupCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("LodSlotsInfo")),
            "LodSlotsInfo passive label must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| {
                n.name.as_deref() == Some("LodThresh1Lbl")
                    && n.text.as_deref().is_some_and(|t| t.contains("15.0 m"))
            }),
            "LodThresh1Lbl must contain 15.0 m"
        );
        assert!(
            tree.iter().any(|(_, n)| {
                n.name.as_deref() == Some("LodThresh2Lbl")
                    && n.text.as_deref().is_some_and(|t| t.contains("35.0 m"))
            }),
            "LodThresh2Lbl must contain 35.0 m"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "LodGroup card must emit 2 numeric drag pills (Threshold 1, Threshold 2)"
        );
    }

    #[test]
    fn test_draw_lod_group_card_missing_safely_ignored() {
        let world = hecs::World::new();
        let entity = hecs::Entity::DANGLING;
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_lod_group_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("LodGroupCard")),
            "Dangling entity must safely omit LodGroupCard"
        );
    }
}