// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Level of Detail (LOD) Group Inspector Card
//!
//! Provides inspection and distance threshold visualization for `📊 LodGroup` components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use super::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};
use irisui::prelude::*;

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
        let (t1, t2, lod1_set, lod2_set) =
            if let Ok(lod) = ctx.world.get::<&ae_core::ecs::LodGroup>(ctx.entity) {
                (
                    lod.threshold_1,
                    lod.threshold_2,
                    lod.lod_1.is_some(),
                    lod.lod_2.is_some(),
                )
            } else {
                (15.0, 35.0, false, false)
            };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        let l1_str = if lod1_set { "Set" } else { "None" };
        let l2_str = if lod2_set { "Set" } else { "None" };

        scope.container_named("LodGroupCard", card_style, |card| {
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

            // Row 1: Slots summary
            card.label_styled_passive(
                "LodSlotsInfo",
                format!("Slots: LOD0 (Active) | LOD1: {} | LOD2: {}", l1_str, l2_str),
                10.5,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 2: LOD 0 -> 1 Threshold
            card.label_styled_passive(
                "LodThresh1Lbl",
                format!("LOD 0 ➔ 1 Distance: {:.1} m", t1),
                11.0,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 3: LOD 1 -> 2 Threshold
            card.label_styled_passive(
                "LodThresh2Lbl",
                format!("LOD 1 ➔ 2 Distance: {:.1} m", t2),
                11.0,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::LodGroup::default());
    }
}