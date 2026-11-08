// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Level of Detail (LOD) Group Inspector Card
//!
//! Provides inspection and distance threshold visualization for `📊 LodGroup` components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use irisui::prelude::*;

/// Renders the declarative Level of Detail (LOD) Group component card with slots and distance thresholds.
///
/// Modifies `threshold_1` and `threshold_2` in-place via [`UiScope::prop_f32`], automatically recording
/// atomic Undo/Redo sessions on edit commits without intermediate imperative plumbing.
pub fn draw_lod_group_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    lod: &mut ae_core::ecs::LodGroup,
) {
    scope.card_panel("LodGroupCard", |card| {
        LodGroupHandler.render_header(card);

        // Row 1: Slots summary
        card.label_styled_passive(
            "LodSlotsInfo",
            format!(
                "Slots: LOD0 (Active) | LOD1: {} | LOD2: {}",
                if lod.lod_1.is_some() { "Set" } else { "None" },
                if lod.lod_2.is_some() { "Set" } else { "None" },
            ),
            10.5,
            card.theme().text_primary,
            TextAlign::Left,
            Style::new().height(20.0),
        );

        // Row 2: LOD 0 -> 1 Threshold label & prop_f32 scrubber
        card.label_styled_passive(
            "LodThresh1Lbl",
            format!("LOD 0 ➔ 1 Distance: {:.1} m", lod.threshold_1),
            11.0,
            card.theme().text_muted,
            TextAlign::Left,
            Style::new().height(20.0),
        );

        card.prop_f32("LOD 0 ➔ 1 (m):", &mut lod.threshold_1, 1.0..=100.0, 0.5)
            .on_commit(|_| {
                ctx.begin_edit("LodGroup");
                ctx.commit_edit("LodGroup");
            });

        // Row 3: LOD 1 -> 2 Threshold label & prop_f32 scrubber
        card.label_styled_passive(
            "LodThresh2Lbl",
            format!("LOD 1 ➔ 2 Distance: {:.1} m", lod.threshold_2),
            11.0,
            card.theme().text_muted,
            TextAlign::Left,
            Style::new().height(20.0),
        );

        card.prop_f32("LOD 1 ➔ 2 (m):", &mut lod.threshold_2, 5.0..=200.0, 1.0)
            .on_commit(|_| {
                ctx.begin_edit("LodGroup");
                ctx.commit_edit("LodGroup");
            });
    });
}

/// Inspector handler for `📊 LodGroup` component.
pub struct LodGroupHandler;

impl ComponentInspectorHandler for LodGroupHandler {
    fn component_name(&self) -> &'static str {
        "LodGroup"
    }

    fn display_title(&self) -> &'static str {
        "LOD Group"
    }

    fn icon(&self) -> &'static str {
        "📊"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 0.65, 1.0) // Mint Emerald
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Rendering
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::LodGroup>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::LodGroup>(ctx.entity),
            |card, mut lod| {
                draw_lod_group_card(card, ctx, &mut lod);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::LodGroup::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_lod_group_card_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::LodGroup::default(),));
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

        let handler = LodGroupHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("LodGroup");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "LodGroup card must tag its delete button"
        );
    }

    #[test]
    fn test_draw_lod_group_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::LodGroup::default(),));
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

        let mut lod = ae_core::ecs::LodGroup::default();
        let mut scope = UiScope::new(&mut tree, root);
        draw_lod_group_card(&mut scope, &mut ctx, &mut lod);

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
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "LodGroup card must emit 2 numeric drag pills (Threshold 1, Threshold 2)"
        );
    }
}