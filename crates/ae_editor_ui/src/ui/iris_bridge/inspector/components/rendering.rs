// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rendering and Illumination Component Inspector Cards
//!
//! Provides handlers for `💡 Light`, `📦 ModelId`, and `🎲 Shape`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::{
    ComboboxRowParams, CompactNumericRowParams, ComponentCategory, InspectorDropdownId,
    InspectorNumberInputId,
};
use super::physics::{render_combobox_row, render_component_header, render_numeric_row_compact};
use irisui::prelude::*;

/// Inspector handler for `💡 Light`.
pub struct LightHandler;

impl ComponentInspectorHandler for LightHandler {
    fn component_name(&self) -> &'static str {
        "Light"
    }

    fn display_title(&self) -> &'static str {
        "Light"
    }

    fn icon(&self) -> &'static str {
        "💡"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.98, 0.80, 0.20, 1.0) // Gold / Bright Yellow (#fbbf24)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Rendering
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Light>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 3.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("LightCard");
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

        let mut cur_y = ctx.base_y + padding + 22.0;

        let (pos, col) = ctx
            .world
            .get::<&ae_core::ecs::Light>(ctx.entity)
            .map(|l| (l.position, l.color))
            .unwrap_or(([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));

        let label_w = 95.0;
        let box_w = 44.0;

        // Position Offset
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Light Offset Y:",
                input_id: InspectorNumberInputId::LightIntensity,
                val: pos[1],
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        // Color RGB
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Color R:",
                input_id: InspectorNumberInputId::LightRange,
                val: col[0],
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );
        cur_y += row_h + 3.0;

        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Color G:",
                input_id: InspectorNumberInputId::LightRange,
                val: col[1],
                row_y: cur_y,
                label_w,
                box_w,
                unit: None,
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::Light::default());
    }
}

/// Inspector handler for `📦 ModelId`.
pub struct ModelMeshHandler;

impl ComponentInspectorHandler for ModelMeshHandler {
    fn component_name(&self) -> &'static str {
        "ModelId"
    }

    fn display_title(&self) -> &'static str {
        "3D Model / Mesh"
    }

    fn icon(&self) -> &'static str {
        "📦"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.38, 0.65, 0.98, 1.0) // Sky Blue (#60a5fa)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Rendering
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::ModelId>(entity).is_ok()
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
            node.set_name("ModelMeshCard");
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

        let cur_y = ctx.base_y + padding + 22.0;

        let handle_str = ctx
            .world
            .get::<&ae_core::ecs::ModelId>(ctx.entity)
            .map(|m| format!("{:?}", m.0))
            .unwrap_or_else(|_| "Default".to_string());

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("ModelAssetInfo");
            node.set_text(format!("Asset Handle: {}", handle_str));
            node.font_size = 10.5;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.54, 0.56, 0.60, 1.0);
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
        let _ = world.insert_one(entity, ae_core::ecs::ModelId::default());
    }
}

/// Inspector handler for `🎲 Shape`.
pub struct ShapeHandler;

impl ComponentInspectorHandler for ShapeHandler {
    fn component_name(&self) -> &'static str {
        "Shape"
    }

    fn display_title(&self) -> &'static str {
        "Procedural 3D Shape"
    }

    fn icon(&self) -> &'static str {
        "🎲"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 0.60, 1.0) // Mint Green (#34d399)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Rendering
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Shape>(entity).is_ok()
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
            node.set_name("ShapeCard");
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

        let cur_y = ctx.base_y + padding + 22.0;

        let shape_str = match ctx.world.get::<&ae_core::ecs::Shape>(ctx.entity).as_deref() {
            Ok(ae_core::ecs::Shape::Cube) => "Cube",
            Ok(ae_core::ecs::Shape::Sphere) => "Sphere",
            Ok(ae_core::ecs::Shape::Cylinder) => "Cylinder",
            Ok(ae_core::ecs::Shape::Capsule) => "Capsule",
            Ok(ae_core::ecs::Shape::Torus) => "Torus",
            Ok(ae_core::ecs::Shape::Triangle) => "Triangle",
            _ => "Procedural Mesh",
        };

        render_combobox_row(
            tree,
            card_id,
            ctx,
            ComboboxRowParams {
                label: "Geometry:",
                selected_text: shape_str,
                dropdown_id: InspectorDropdownId::ShapeType,
                label_w: 65.0,
                row_y: cur_y,
            },
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::Shape::Cube);
    }
}