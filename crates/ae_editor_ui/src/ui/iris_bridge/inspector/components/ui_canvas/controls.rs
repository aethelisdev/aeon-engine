// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Controls & Layout Inspector Cards
//!
//! Provides declarative handlers for UI Designer interactive controls:
//! - `UiSlider`
//! - `UiCheckbox`
//! - `UiTextInput`
//! - `UiLayoutGroup`

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::ComponentCategory;

use irisui::prelude::*;

/// Inspector handler for `UiSlider` component.
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut s) = ctx.world.get::<&mut ae_core::ecs::UiSlider>(ctx.entity) else {
            return;
        };

        let val_range = s.min..=s.max;

        scope.card_panel("UiSliderCard", |card| {
            self.render_header(card);

            ctx.prop_f32(
                card,
                self.component_name(),
                "Min:",
                &mut s.min,
                -100_000.0..=100_000.0,
                0.1,
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Max:",
                &mut s.max,
                -100_000.0..=100_000.0,
                0.1,
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Value:",
                &mut s.value,
                val_range,
                0.05,
            );
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut c) = ctx.world.get::<&mut ae_core::ecs::UiCheckbox>(ctx.entity) else {
            return;
        };

        scope.card_panel("UiCheckboxCard", |card| {
            self.render_header(card);

            ctx.prop_checkbox(card, self.component_name(), "Checked", &mut c.is_checked);
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(input) = ctx.world.get::<&ae_core::ecs::UiTextInput>(ctx.entity) else {
            return;
        };

        scope.card_panel("UiTextInputCard", |card| {
            self.render_header(card);

            let label_text = format!("Placeholder: \"{}\"", input.placeholder);
            card.label_styled_passive(
                "UiInputPlaceholder",
                &label_text,
                11.0,
                card.theme().text_primary,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut lg) = ctx
            .world
            .get::<&mut ae_core::ecs::UiLayoutGroup>(ctx.entity)
        else {
            return;
        };

        let props_str = format!(
            "Type: {:?}  |  Spacing: {:.1} px",
            lg.layout_type, lg.spacing
        );

        scope.card_panel("UiLayoutGroupCard", |card| {
            self.render_header(card);

            card.label_styled_passive(
                "UiLayoutProps",
                &props_str,
                11.0,
                card.theme().text_primary,
                TextAlign::Left,
                Style::new().height(20.0),
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Spacing:",
                &mut lg.spacing,
                0.0..=500.0,
                0.5,
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiLayoutGroup::default());
    }
}