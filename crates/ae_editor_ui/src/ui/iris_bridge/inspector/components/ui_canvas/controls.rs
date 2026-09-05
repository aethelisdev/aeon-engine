// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Controls & Layout Inspector Cards
//!
//! Provides handlers for UI Designer interactive controls:
//! - `UiSlider`
//! - `UiCheckbox`
//! - `UiTextInput`
//! - `UiLayoutGroup`

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::ComponentCategory;
use super::super::physics::render_component_header;
use irisui::prelude::*;

/// Inspector handler for `🎚️ UiSlider` component.
pub struct UiSliderHandler;

impl ComponentInspectorHandler for UiSliderHandler {
    fn component_name(&self) -> &'static str {
        "UiSlider"
    }

    fn display_title(&self) -> &'static str {
        "UI Numeric Slider"
    }

    fn icon(&self) -> &'static str {
        "🎚️"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiSlider>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (val, min, max) = if let Ok(s) = ctx.world.get::<&ae_core::ecs::UiSlider>(ctx.entity) {
            (s.value, s.min, s.max)
        } else {
            (0.5, 0.0, 1.0)
        };

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiSliderCard");
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
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("UiSliderRange");
            node.set_text(format!(
                "Value: {:.2} (Range: {:.1} - {:.1})",
                val, min, max
            ));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
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
        let _ = world.insert_one(entity, ae_core::ecs::UiSlider::default());
    }
}

/// Inspector handler for `☑️ UiCheckbox` component.
pub struct UiCheckboxHandler;

impl ComponentInspectorHandler for UiCheckboxHandler {
    fn component_name(&self) -> &'static str {
        "UiCheckbox"
    }

    fn display_title(&self) -> &'static str {
        "UI Checkbox"
    }

    fn icon(&self) -> &'static str {
        "☑️"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiCheckbox>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (label, is_checked) =
            if let Ok(c) = ctx.world.get::<&ae_core::ecs::UiCheckbox>(ctx.entity) {
                (c.label.clone(), c.is_checked)
            } else {
                ("Option".to_string(), false)
            };

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiCheckboxCard");
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
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("UiCheckboxState");
            let mark = if is_checked { "[x]" } else { "[ ]" };
            node.set_text(format!("{} Label: \"{}\"", mark, label));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
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
        let _ = world.insert_one(entity, ae_core::ecs::UiCheckbox::default());
    }
}

/// Inspector handler for `📝 UiTextInput` component.
pub struct UiTextInputHandler;

impl ComponentInspectorHandler for UiTextInputHandler {
    fn component_name(&self) -> &'static str {
        "UiTextInput"
    }

    fn display_title(&self) -> &'static str {
        "UI Text Input Field"
    }

    fn icon(&self) -> &'static str {
        "📝"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiTextInput>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let placeholder = if let Ok(input) = ctx.world.get::<&ae_core::ecs::UiTextInput>(ctx.entity)
        {
            input.placeholder.clone()
        } else {
            "Enter text...".to_string()
        };

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiTextInputCard");
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
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("UiInputPlaceholder");
            node.set_text(format!("Placeholder: \"{}\"", placeholder));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
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
        let _ = world.insert_one(entity, ae_core::ecs::UiTextInput::default());
    }
}

/// Inspector handler for `🗂️ UiLayoutGroup` auto-layout container component.
pub struct UiLayoutGroupHandler;

impl ComponentInspectorHandler for UiLayoutGroupHandler {
    fn component_name(&self) -> &'static str {
        "UiLayoutGroup"
    }

    fn display_title(&self) -> &'static str {
        "UI Layout Group"
    }

    fn icon(&self) -> &'static str {
        "🗂️"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiLayoutGroup>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (layout_type, spacing) =
            if let Ok(lg) = ctx.world.get::<&ae_core::ecs::UiLayoutGroup>(ctx.entity) {
                (format!("{:?}", lg.layout_type), lg.spacing)
            } else {
                ("Vertical".to_string(), 8.0)
            };

        let padding = 8.0;
        let row_h = 22.0;
        let card_h = 24.0 + 1.0 * (row_h + 3.0) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("UiLayoutGroupCard");
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
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("UiLayoutProps");
            node.set_text(format!(
                "Type: {}  |  Spacing: {:.1} px",
                layout_type, spacing
            ));
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
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
        let _ = world.insert_one(entity, ae_core::ecs::UiLayoutGroup::default());
    }
}