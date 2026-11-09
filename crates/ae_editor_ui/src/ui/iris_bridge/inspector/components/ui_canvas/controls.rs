// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Controls & Layout Inspector Cards
//!
//! Provides declarative inspector cards and handlers for UI Designer interactive controls:
//! - `UiSlider`: Numeric slider with min, max, value scrubbers
//! - `UiCheckbox`: Toggleable checkbox state
//! - `UiTextInput`: Text input field with placeholder property
//! - `UiLayoutGroup`: Automatic horizontal, vertical, and grid layout arranger

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::ComponentCategory;
use irisui::prelude::*;

/// Maps [`ae_core::ecs::UiLayoutType`] enum variant to combo dropdown selection index.
#[inline]
const fn layout_type_to_index(layout_type: ae_core::ecs::UiLayoutType) -> usize {
    match layout_type {
        ae_core::ecs::UiLayoutType::Horizontal => 0,
        ae_core::ecs::UiLayoutType::Vertical => 1,
        ae_core::ecs::UiLayoutType::Grid => 2,
    }
}

/// Maps combo dropdown selection index back to [`ae_core::ecs::UiLayoutType`].
#[inline]
const fn index_to_layout_type(index: usize) -> ae_core::ecs::UiLayoutType {
    match index {
        0 => ae_core::ecs::UiLayoutType::Horizontal,
        1 => ae_core::ecs::UiLayoutType::Vertical,
        _ => ae_core::ecs::UiLayoutType::Grid,
    }
}

/// Returns the static label prefix string for the layout properties readout.
#[inline]
const fn layout_props_prefix(layout_type: ae_core::ecs::UiLayoutType) -> &'static str {
    match layout_type {
        ae_core::ecs::UiLayoutType::Horizontal => "Type: Horizontal  |  Spacing: ",
        ae_core::ecs::UiLayoutType::Vertical => "Type: Vertical  |  Spacing: ",
        ae_core::ecs::UiLayoutType::Grid => "Type: Grid  |  Spacing: ",
    }
}

/// Renders the declarative UI Slider component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Modifies slider min, max, and current value in-place with zero intermediate variable plumbing.
pub fn draw_ui_slider_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok_mut(
        "UiSliderCard",
        UiScope::comp_card_desc(
            "UiSlider",
            "UI Numeric Slider",
            "🎚️",
            Color::rgba(0.20, 0.85, 1.0, 1.0),
        ),
        world.get::<&mut ae_core::ecs::UiSlider>(entity),
        |card, s| {
            card.prop_f32("Min:", &mut s.min, -100_000.0..=100_000.0, 0.1);
            card.prop_f32("Max:", &mut s.max, -100_000.0..=100_000.0, 0.1);
            card.prop_f32("Value:", &mut s.value, s.min..=s.max, 0.05);
        },
    );
}

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
        draw_ui_slider_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiSlider::default()));
    }
}

/// Renders the declarative UI Checkbox component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Modifies checkbox checked state in-place with zero intermediate variable plumbing.
pub fn draw_ui_checkbox_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok_mut(
        "UiCheckboxCard",
        UiScope::comp_card_desc(
            "UiCheckbox",
            "UI Checkbox",
            "☑️",
            Color::rgba(0.20, 0.85, 1.0, 1.0),
        ),
        world.get::<&mut ae_core::ecs::UiCheckbox>(entity),
        |card, c| {
            card.prop_checkbox("Checked", &mut c.is_checked);
        },
    );
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
        draw_ui_checkbox_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiCheckbox::default()));
    }
}

/// Renders the declarative UI Text Input component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Modifies placeholder text in-place with zero intermediate variable plumbing.
pub fn draw_ui_text_input_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok_mut(
        "UiTextInputCard",
        UiScope::comp_card_desc(
            "UiTextInput",
            "UI Text Input Field",
            "📝",
            Color::rgba(0.20, 0.85, 1.0, 1.0),
        ),
        world.get::<&mut ae_core::ecs::UiTextInput>(entity),
        |card, input| {
            card.prop_text("Placeholder", &mut input.placeholder);
        },
    );
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
        draw_ui_text_input_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiTextInput::default()));
    }
}

/// Renders the declarative UI Layout Group auto-layout container component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Modifies layout type and spacing in-place without heap allocations or intermediate plumbing.
pub fn draw_ui_layout_group_card(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    entity: hecs::Entity,
) {
    scope.card_panel_desc_when_ok_mut(
        "UiLayoutGroupCard",
        UiScope::comp_card_desc(
            "UiLayoutGroup",
            "UI Layout Group",
            "🗂️",
            Color::rgba(0.20, 0.85, 1.0, 1.0),
        ),
        world.get::<&mut ae_core::ecs::UiLayoutGroup>(entity),
        |card, lg| {
            card.label_number_desc(
                "UiLayoutProps",
                NumberLabelDescriptor::new(
                    layout_props_prefix(lg.layout_type),
                    lg.spacing as f64,
                    " px",
                )
                .precision(1)
                .color(card.theme().text_primary),
            );

            card.prop_combobox_mapped_styled(
                "Layout Type:",
                layout_type_to_index(lg.layout_type),
                &["Horizontal", "Vertical", "Grid"],
                ComboRowLayout::default(),
                &mut lg.layout_type,
                index_to_layout_type,
            );

            card.prop_f32("Spacing:", &mut lg.spacing, 0.0..=500.0, 0.5);
        },
    );
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
        draw_ui_layout_group_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiLayoutGroup::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_ui_slider_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiSlider {
            value: 0.5,
            min: 0.0,
            max: 1.0,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_slider_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiSliderCard")),
            "UiSliderCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "UiSliderCard must emit 3 numeric pills (Min, Max, Value)"
        );
    }

    #[test]
    fn test_draw_ui_checkbox_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiCheckbox {
            is_checked: true,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_checkbox_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiCheckboxCard")),
            "UiCheckboxCard container must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| n.role == WidgetRole::Checkbox),
            "UiCheckbox must emit a Checkbox widget"
        );
    }

    #[test]
    fn test_draw_ui_text_input_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiTextInput {
            placeholder: "Search...".to_string(),
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_text_input_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiTextInputCard")),
            "UiTextInputCard container must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| n.role == WidgetRole::TextInput),
            "UiTextInput must emit a TextInput widget"
        );
    }

    #[test]
    fn test_draw_ui_layout_group_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiLayoutGroup {
            layout_type: ae_core::ecs::UiLayoutType::Vertical,
            spacing: 12.0,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_layout_group_card(&mut scope, &world, entity);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiLayoutGroupCard")),
            "UiLayoutGroupCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ComboPill")),
            "Layout Type combobox pill must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("NumericDrag")),
            "Spacing numeric pill must be rendered"
        );
        let has_props = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("UiLayoutProps")
                && n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Vertical") && t.contains("12.0 px"))
        });
        assert!(
            has_props,
            "UiLayoutProps label must render layout type and spacing"
        );
    }

    #[test]
    fn test_layout_type_mappings() {
        assert_eq!(
            layout_type_to_index(ae_core::ecs::UiLayoutType::Horizontal),
            0
        );
        assert_eq!(
            layout_type_to_index(ae_core::ecs::UiLayoutType::Vertical),
            1
        );
        assert_eq!(layout_type_to_index(ae_core::ecs::UiLayoutType::Grid), 2);

        assert_eq!(
            index_to_layout_type(0),
            ae_core::ecs::UiLayoutType::Horizontal
        );
        assert_eq!(
            index_to_layout_type(1),
            ae_core::ecs::UiLayoutType::Vertical
        );
        assert_eq!(index_to_layout_type(2), ae_core::ecs::UiLayoutType::Grid);
        assert_eq!(index_to_layout_type(99), ae_core::ecs::UiLayoutType::Grid);

        assert_eq!(
            layout_props_prefix(ae_core::ecs::UiLayoutType::Horizontal),
            "Type: Horizontal  |  Spacing: "
        );
        assert_eq!(
            layout_props_prefix(ae_core::ecs::UiLayoutType::Vertical),
            "Type: Vertical  |  Spacing: "
        );
        assert_eq!(
            layout_props_prefix(ae_core::ecs::UiLayoutType::Grid),
            "Type: Grid  |  Spacing: "
        );
    }
}