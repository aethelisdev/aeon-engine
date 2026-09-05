// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Level of Detail (LOD) Group Inspector Card
//!
//! Provides inspection and distance threshold visualization for `📊 LodGroup` components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use super::physics::render_component_header;
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

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
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

        let padding = 8.0;
        let row_h = 22.0;
        let spacing = 4.0;
        let card_h = 24.0 + 3.0 * (row_h + spacing) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("LodGroupCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let mut cur_y = ctx.base_y + padding + 24.0 + 4.0;

        // Row 1: Slots summary
        let lbl1_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl1_id) {
            node.set_name("LodSlotsInfo");
            let l1_str = if lod1_set { "Set" } else { "None" };
            let l2_str = if lod2_set { "Set" } else { "None" };
            node.set_text(format!(
                "Slots: LOD0 (Active) | LOD1: {} | LOD2: {}",
                l1_str, l2_str
            ));
            node.font_size = 10.5;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl1_id);
        cur_y += row_h + spacing;

        // Row 2: LOD 0 -> 1 Threshold
        let lbl2_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl2_id) {
            node.set_name("LodThresh1Lbl");
            node.set_text(format!("LOD 0 ➔ 1 Distance: {:.1} m", t1));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl2_id);
        cur_y += row_h + spacing;

        // Row 3: LOD 1 -> 2 Threshold
        let lbl3_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl3_id) {
            node.set_name("LodThresh2Lbl");
            node.set_text(format!("LOD 1 ➔ 2 Distance: {:.1} m", t2));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl3_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::LodGroup::default());
    }
}