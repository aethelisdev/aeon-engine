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

/// Renders the declarative UI Slider component card with min, max, and current value properties.
///
/// Modifies slider fields in-place via [`UiScope::prop_f32`], automatically recording
/// atomic Undo/Redo sessions on edit commits without intermediate imperative plumbing.
pub fn draw_ui_slider_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    s: &mut ae_core::ecs::UiSlider,
) {
    scope.card_panel("UiSliderCard", |card| {
        UiSliderHandler.render_header(card);

        card.prop_f32("Min:", &mut s.min, -100_000.0..=100_000.0, 0.1)
            .on_commit(|_| {
                ctx.begin_edit("UiSlider");
                ctx.commit_edit("UiSlider");
            });

        card.prop_f32("Max:", &mut s.max, -100_000.0..=100_000.0, 0.1)
            .on_commit(|_| {
                ctx.begin_edit("UiSlider");
                ctx.commit_edit("UiSlider");
            });

        card.prop_f32("Value:", &mut s.value, s.min..=s.max, 0.05)
            .on_commit(|_| {
                ctx.begin_edit("UiSlider");
                ctx.commit_edit("UiSlider");
            });
    });
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
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::UiSlider>(ctx.entity),
            |card, mut s| {
                draw_ui_slider_card(card, ctx, &mut s);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiSlider::default()));
    }
}

/// Renders the declarative UI Checkbox component card.
///
/// Modifies `is_checked` in-place via [`UiScope::prop_checkbox`], recording atomic Undo/Redo sessions.
pub fn draw_ui_checkbox_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    c: &mut ae_core::ecs::UiCheckbox,
) {
    scope.card_panel("UiCheckboxCard", |card| {
        UiCheckboxHandler.render_header(card);

        card.prop_checkbox("Checked", &mut c.is_checked)
            .on_click(|| {
                ctx.begin_edit("UiCheckbox");
                ctx.commit_edit("UiCheckbox");
            });
    });
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
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::UiCheckbox>(ctx.entity),
            |card, mut c| {
                draw_ui_checkbox_card(card, ctx, &mut c);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiCheckbox::default()));
    }
}

/// Renders the declarative UI Text Input component card.
///
/// Modifies `placeholder` in-place via [`UiScope::prop_text`], recording atomic Undo/Redo sessions.
pub fn draw_ui_text_input_card(
    scope: &mut UiScope<'_>,
    _ctx: &mut ComponentRenderContext<'_>,
    input: &mut ae_core::ecs::UiTextInput,
) {
    scope.card_panel("UiTextInputCard", |card| {
        UiTextInputHandler.render_header(card);

        card.prop_text("Placeholder", &mut input.placeholder);
    });
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
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::UiTextInput>(ctx.entity),
            |card, mut input| {
                draw_ui_text_input_card(card, ctx, &mut input);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiTextInput::default()));
    }
}

/// Renders the declarative UI Layout Group auto-layout container component card.
///
/// Provides layout type combobox selection (`Horizontal`, `Vertical`, `Grid`) and spacing adjustment.
pub fn draw_ui_layout_group_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    lg: &mut ae_core::ecs::UiLayoutGroup,
) {
    scope.card_panel("UiLayoutGroupCard", |card| {
        UiLayoutGroupHandler.render_header(card);

        card.label_styled_passive(
            "UiLayoutProps",
            format!(
                "Type: {:?}  |  Spacing: {:.1} px",
                lg.layout_type, lg.spacing
            ),
            11.0,
            card.theme().text_primary,
            TextAlign::Left,
            Style::new().height(20.0),
        );

        card.prop_combobox_mapped_styled(
            "Layout Type:",
            match lg.layout_type {
                ae_core::ecs::UiLayoutType::Horizontal => 0,
                ae_core::ecs::UiLayoutType::Vertical => 1,
                ae_core::ecs::UiLayoutType::Grid => 2,
            },
            &["Horizontal", "Vertical", "Grid"],
            ComboRowLayout::default(),
            &mut lg.layout_type,
            |idx| match idx {
                0 => ae_core::ecs::UiLayoutType::Horizontal,
                1 => ae_core::ecs::UiLayoutType::Vertical,
                _ => ae_core::ecs::UiLayoutType::Grid,
            },
        )
        .on_change(|| {
            ctx.begin_edit("UiLayoutGroup");
            ctx.commit_edit("UiLayoutGroup");
        });

        card.prop_f32("Spacing:", &mut lg.spacing, 0.0..=500.0, 0.5)
            .on_commit(|_| {
                ctx.begin_edit("UiLayoutGroup");
                ctx.commit_edit("UiLayoutGroup");
            });
    });
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
        scope.when_ok(
            ctx.world
                .get::<&mut ae_core::ecs::UiLayoutGroup>(ctx.entity),
            |card, mut lg| {
                draw_ui_layout_group_card(card, ctx, &mut lg);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiLayoutGroup::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    fn create_dummy_params<'a>(
        world: &'a hecs::World,
        entity: hecs::Entity,
        euler: &'a [f32; 3],
        swatches: &'a [[f32; 4]],
    ) -> InspectorPanelParams<'a> {
        InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world,
            selected_entity: Some(entity),
            inspector_euler: euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        }
    }

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
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

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

        let mut slider = ae_core::ecs::UiSlider {
            value: 0.5,
            min: 0.0,
            max: 1.0,
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_slider_card(&mut scope, &mut ctx, &mut slider);

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
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

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

        let mut cb = ae_core::ecs::UiCheckbox {
            is_checked: true,
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_checkbox_card(&mut scope, &mut ctx, &mut cb);

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
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

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

        let mut input = ae_core::ecs::UiTextInput {
            placeholder: "Search...".to_string(),
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_text_input_card(&mut scope, &mut ctx, &mut input);

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
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

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

        let mut lg = ae_core::ecs::UiLayoutGroup {
            layout_type: ae_core::ecs::UiLayoutType::Vertical,
            spacing: 12.0,
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_layout_group_card(&mut scope, &mut ctx, &mut lg);

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
    }
}