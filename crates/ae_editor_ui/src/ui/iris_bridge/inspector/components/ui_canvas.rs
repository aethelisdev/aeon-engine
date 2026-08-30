// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI and HUD Component Inspector Cards
//!
//! Provides handlers for UI Designer widgets (`UiElement`, `UiProgressBar`, `UiButton`).

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use super::physics::render_component_header;
use irisui::prelude::*;

/// Inspector handler for `📐 UiElement`.
pub struct UiElementHandler;

impl ComponentInspectorHandler for UiElementHandler {
    fn component_name(&self) -> &'static str {
        "UiElement"
    }

    fn display_title(&self) -> &'static str {
        "2D Screen UI Element"
    }

    fn icon(&self) -> &'static str {
        "📐"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiElement>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 2.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiElementCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
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

        let cur_y = ctx.base_y + padding + 22.0;

        let (offset, size) = ctx
            .world
            .get::<&ae_core::ecs::UiElement>(ctx.entity)
            .map(|st| (st.offset, st.size))
            .unwrap_or(([0.0, 0.0], [100.0, 30.0]));

        let lbl1_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl1_id) {
            node.set_name("ScreenPosInfo");
            node.set_text(format!("Offset: ({:.0}, {:.0}) px", offset[0], offset[1]));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.75, 0.78, 0.88, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl1_id);

        let lbl2_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl2_id) {
            node.set_name("ScreenSizeInfo");
            node.set_text(format!("Size: ({:.0} × {:.0}) px", size[0], size[1]));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.75, 0.78, 0.88, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y + row_h + 3.0,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl2_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiElement::default());
    }
}

/// Inspector handler for `📊 UiProgressBar`.
pub struct UiProgressBarHandler;

impl ComponentInspectorHandler for UiProgressBarHandler {
    fn component_name(&self) -> &'static str {
        "UiProgressBar"
    }

    fn display_title(&self) -> &'static str {
        "Progress / Health Bar"
    }

    fn icon(&self) -> &'static str {
        "📊"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 0.50, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiProgressBar>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 2.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ProgressBarCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
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

        let cur_y = ctx.base_y + padding + 22.0;

        let (val, max_val, frac) = ctx
            .world
            .get::<&ae_core::ecs::UiProgressBar>(ctx.entity)
            .map(|pb| (pb.value, pb.max, pb.fraction()))
            .unwrap_or((65.0, 100.0, 0.65));

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("ProgressValInfo");
            node.set_text(format!(
                "Fill Amount: {:.1}% ({:.1}/{:.1})",
                frac * 100.0,
                val,
                max_val
            ));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.75, 0.78, 0.88, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                cur_y,
                ctx.card_w - padding * 2.0,
                row_h,
            );
        }
        let _ = tree.add_child(card_id, lbl_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiProgressBar::default());
    }
}

/// Inspector handler for `🔘 UiButton`.
pub struct UiButtonHandler;

impl ComponentInspectorHandler for UiButtonHandler {
    fn component_name(&self) -> &'static str {
        "UiButton"
    }

    fn display_title(&self) -> &'static str {
        "Interactive Button"
    }

    fn icon(&self) -> &'static str {
        "🔘"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.40, 0.75, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiButton>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiButtonCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
                .border(1.0, Color::rgba(0.16, 0.19, 0.26, 0.80))
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

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiButton::default());
    }
}