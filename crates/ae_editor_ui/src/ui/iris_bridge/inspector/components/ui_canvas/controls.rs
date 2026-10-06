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
use super::super::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};

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
        let (val, min, max) = if let Ok(s) = ctx.world.get::<&ae_core::ecs::UiSlider>(ctx.entity) {
            (s.value, s.min, s.max)
        } else {
            (0.5, 0.0, 1.0)
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiSliderCard", card_style, |card| {
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

            let label_text = format!("Value: {:.2} (Range: {:.1} - {:.1})", val, min, max);
            card.label_styled_passive(
                "UiSliderRange",
                &label_text,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
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
        let (label, is_checked) =
            if let Ok(c) = ctx.world.get::<&ae_core::ecs::UiCheckbox>(ctx.entity) {
                (c.label.clone(), c.is_checked)
            } else {
                ("Option".to_string(), false)
            };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiCheckboxCard", card_style, |card| {
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

            let mark = if is_checked { "[x]" } else { "[ ]" };
            let label_text = format!("{} Label: \"{}\"", mark, label);
            card.label_styled_passive(
                "UiCheckboxState",
                &label_text,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
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
        let placeholder = if let Ok(input) = ctx.world.get::<&ae_core::ecs::UiTextInput>(ctx.entity)
        {
            input.placeholder.clone()
        } else {
            "Enter text...".to_string()
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiTextInputCard", card_style, |card| {
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

            let label_text = format!("Placeholder: \"{}\"", placeholder);
            card.label_styled_passive(
                "UiInputPlaceholder",
                &label_text,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
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
        let (layout_type, spacing) =
            if let Ok(lg) = ctx.world.get::<&ae_core::ecs::UiLayoutGroup>(ctx.entity) {
                (format!("{:?}", lg.layout_type), lg.spacing)
            } else {
                ("Vertical".to_string(), 8.0)
            };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("UiLayoutGroupCard", card_style, |card| {
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

            let props_str = format!("Type: {}  |  Spacing: {:.1} px", layout_type, spacing);
            card.label_styled_passive(
                "UiLayoutProps",
                &props_str,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::UiLayoutGroup::default());
    }
}