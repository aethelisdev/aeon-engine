// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen UI Primitives Inspector Cards
//!
//! Provides declarative handlers for core UI Designer widgets:
//! - `UiElement`
//! - `UiPanel`
//! - `UiText`
//! - `UiProgressBar`
//! - `UiButton`
//! - `UiImage`

use super::super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::super::types::ComponentCategory;
use ae_core::ui::UiTextAlignment;
use irisui::prelude::*;

/// Inspector handler for `UiElement`.
///
/// Handled as the primary 2D Screen Transform (RectTransform) card at the top of the Inspector.
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

    fn has_component(&self, _world: &hecs::World, _entity: hecs::Entity) -> bool {
        // Rendered as the primary 2D Screen Transform card at the top of the Inspector (replacing 3D Transform)
        false
    }

    fn render_card(&self, _scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {}

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiElement::default()));
    }
}

/// Renders the declarative UI Background Panel component card with border width and corner radius.
///
/// Modifies `p.border_width` and `p.corner_radius` in-place via [`UiScope::prop_f32`], automatically
/// recording atomic Undo/Redo sessions on edit commits without intermediate imperative plumbing.
pub fn draw_ui_panel_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    p: &mut ae_core::ecs::UiPanel,
) {
    scope.card_panel("UiPanelCard", |card| {
        UiPanelHandler.render_header(card);

        card.prop_f32("Border Width:", &mut p.border_width, 0.0..=50.0, 0.5)
            .on_commit(|_| {
                ctx.begin_edit("UiPanel");
                ctx.commit_edit("UiPanel");
            });

        card.prop_f32("Corner Radius:", &mut p.corner_radius, 0.0..=100.0, 0.5)
            .on_commit(|_| {
                ctx.begin_edit("UiPanel");
                ctx.commit_edit("UiPanel");
            });
    });
}

/// Inspector handler for `🔲 UiPanel` background container component.
pub struct UiPanelHandler;

impl ComponentInspectorHandler for UiPanelHandler {
    fn component_name(&self) -> &'static str {
        "UiPanel"
    }

    fn display_title(&self) -> &'static str {
        "UI Background Panel"
    }

    fn icon(&self) -> &'static str {
        "🔲"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiPanel>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::UiPanel>(ctx.entity),
            |card, mut p| {
                draw_ui_panel_card(card, ctx, &mut p);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiPanel::default()));
    }
}

/// Renders the declarative UI Text Label component card with text content, font size, and alignment.
///
/// Modifies `t.text`, `t.font_size`, and `t.alignment` in-place via [`UiScope::prop_text`],
/// [`UiScope::prop_f32`], and [`UiScope::prop_combobox_mapped_styled`], recording atomic Undo/Redo
/// sessions without intermediate imperative plumbing.
pub fn draw_ui_text_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    t: &mut ae_core::ecs::UiText,
) {
    scope.card_panel("UiTextCard", |card| {
        UiTextHandler.render_header(card);

        // Row 1: Text string property editor
        card.prop_text("Text", &mut t.text);

        // Row 2: Font Size
        card.prop_f32("Font Size:", &mut t.font_size, 6.0..=128.0, 0.5)
            .on_commit(|_| {
                ctx.begin_edit("UiText");
                ctx.commit_edit("UiText");
            });

        // Row 3: Text Alignment Dropdown
        card.prop_combobox_mapped_styled(
            "Align:",
            match t.alignment {
                UiTextAlignment::Left => 0,
                UiTextAlignment::Center => 1,
                UiTextAlignment::Right => 2,
            },
            &["Left", "Center", "Right"],
            ComboRowLayout::new(52.0, 80.0),
            &mut t.alignment,
            |idx| match idx {
                0 => UiTextAlignment::Left,
                1 => UiTextAlignment::Center,
                _ => UiTextAlignment::Right,
            },
        )
        .on_change(|| {
            ctx.begin_edit("UiText");
            ctx.commit_edit("UiText");
        });
    });
}

/// Inspector handler for `🔤 UiText` label component.
pub struct UiTextHandler;

impl ComponentInspectorHandler for UiTextHandler {
    fn component_name(&self) -> &'static str {
        "UiText"
    }

    fn display_title(&self) -> &'static str {
        "UI Text Label"
    }

    fn icon(&self) -> &'static str {
        "🔤"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiText>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::UiText>(ctx.entity),
            |card, mut t| {
                draw_ui_text_card(card, ctx, &mut t);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiText::default()));
    }
}

/// Renders the declarative UI Progress Bar component card with min, max, value scrubbers and visual track.
///
/// Modifies `pb.min`, `pb.max`, and `pb.value` in-place via [`UiScope::prop_f32`], automatically
/// recording atomic Undo/Redo sessions on edit commits without intermediate imperative plumbing.
pub fn draw_ui_progress_bar_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    pb: &mut ae_core::ecs::UiProgressBar,
) {
    scope.card_panel("UiProgressBarCard", |card| {
        UiProgressBarHandler.render_header(card);

        card.prop_f32("Min:", &mut pb.min, -100_000.0..=100_000.0, 1.0)
            .on_commit(|_| {
                ctx.begin_edit("UiProgressBar");
                ctx.commit_edit("UiProgressBar");
            });

        card.prop_f32("Max:", &mut pb.max, -100_000.0..=100_000.0, 1.0)
            .on_commit(|_| {
                ctx.begin_edit("UiProgressBar");
                ctx.commit_edit("UiProgressBar");
            });

        card.prop_f32("Value:", &mut pb.value, pb.min..=pb.max, 1.0)
            .on_commit(|_| {
                ctx.begin_edit("UiProgressBar");
                ctx.commit_edit("UiProgressBar");
            });

        // Visual Mini Progress Bar track + fill
        card.progress_bar_named(
            "MiniBarTrack",
            "MiniBarFill",
            (ctx.card_w - 20.0).max(10.0),
            10.0,
            if pb.max > pb.min {
                ((pb.value - pb.min) / (pb.max - pb.min)).clamp(0.0, 1.0)
            } else {
                0.0
            },
            card.theme().accent,
        );
    });
}

/// Inspector handler for `📊 UiProgressBar`.
pub struct UiProgressBarHandler;

impl ComponentInspectorHandler for UiProgressBarHandler {
    fn component_name(&self) -> &'static str {
        "UiProgressBar"
    }

    fn display_title(&self) -> &'static str {
        "UI Progress Bar"
    }

    fn icon(&self) -> &'static str {
        "📊"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiProgressBar>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world
                .get::<&mut ae_core::ecs::UiProgressBar>(ctx.entity),
            |card, mut pb| {
                draw_ui_progress_bar_card(card, ctx, &mut pb);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiProgressBar::default()));
    }
}

/// Renders the declarative Interactive Button component card with enable toggle.
///
/// Modifies `b.is_enabled` in-place via [`UiScope::prop_checkbox`], automatically
/// recording atomic Undo/Redo sessions on user click without intermediate imperative plumbing.
pub fn draw_ui_button_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    b: &mut ae_core::ecs::UiButton,
) {
    scope.card_panel("UiButtonCard", |card| {
        UiButtonHandler.render_header(card);

        card.prop_checkbox("Enabled", &mut b.is_enabled)
            .on_click(|| {
                ctx.begin_edit("UiButton");
                ctx.commit_edit("UiButton");
            });
    });
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::UiButton>(ctx.entity),
            |card, mut b| {
                draw_ui_button_card(card, ctx, &mut b);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiButton::default()));
    }
}

/// Renders the declarative UI Sprite Image component card with slice mode display.
pub fn draw_ui_image_card(
    scope: &mut UiScope<'_>,
    _ctx: &mut ComponentRenderContext<'_>,
    img: &ae_core::ecs::UiImage,
) {
    scope.card_panel("UiImageCard", |card| {
        UiImageHandler.render_header(card);

        card.label_styled_passive(
            "UiImageMode",
            match img.slice_mode {
                ae_core::ecs::UiSliceMode::Stretch => "Slice Mode: Stretch",
                ae_core::ecs::UiSliceMode::Fit => "Slice Mode: Fit",
                ae_core::ecs::UiSliceMode::NineSlice => "Slice Mode: NineSlice",
                ae_core::ecs::UiSliceMode::Tile => "Slice Mode: Tile",
            },
            11.0,
            card.theme().text_primary,
            TextAlign::Left,
            Style::new().height(20.0),
        );
    });
}

/// Inspector handler for `🖼️ UiImage` sprite component.
pub struct UiImageHandler;

impl ComponentInspectorHandler for UiImageHandler {
    fn component_name(&self) -> &'static str {
        "UiImage"
    }

    fn display_title(&self) -> &'static str {
        "UI Sprite Image"
    }

    fn icon(&self) -> &'static str {
        "🖼️"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.85, 1.0, 1.0)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::UiHud
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::UiImage>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&ae_core::ecs::UiImage>(ctx.entity),
            |card, img| {
                draw_ui_image_card(card, ctx, &img);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::UiImage::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn create_test_context<'a>(
        world: &'a hecs::World,
        entity: hecs::Entity,
        params: &'a InspectorPanelParams<'a>,
        actions: &'a mut Vec<()>,
    ) -> ComponentRenderContext<'a> {
        ComponentRenderContext::with_actions(entity, world, params, actions, 0.0, 0.0, 260.0)
    }

    #[test]
    fn test_ui_panel_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiPanel {
            border_width: 2.0,
            corner_radius: 4.0,
            ..Default::default()
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        {
            let mut scope = UiScope::new(&mut tree, root);
            UiPanelHandler.render_card(&mut scope, &mut ctx);
        }

        // Verify direct in-place mutation
        {
            let mut p = world.get::<&mut ae_core::ecs::UiPanel>(entity).unwrap();
            p.border_width = 5.0;
            p.corner_radius = 8.0;
        }
        let panel = world.get::<&ae_core::ecs::UiPanel>(entity).unwrap();
        assert_eq!(panel.border_width, 5.0);
        assert_eq!(panel.corner_radius, 8.0);
    }

    #[test]
    fn test_draw_ui_panel_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiPanel::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        let mut panel = ae_core::ecs::UiPanel::default();
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_panel_card(&mut scope, &mut ctx, &mut panel);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiPanelCard")),
            "UiPanelCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "UiPanel must emit 2 numeric drag pills (Border Width, Corner Radius)"
        );
    }

    #[test]
    fn test_draw_ui_text_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiText::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        let mut text = ae_core::ecs::UiText::default();
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_text_card(&mut scope, &mut ctx, &mut text);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiTextCard")),
            "UiTextCard container must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| n.role == WidgetRole::TextInput),
            "TextInput must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ComboPill")),
            "ComboPill must be rendered for Alignment"
        );
    }

    #[test]
    fn test_draw_ui_progress_bar_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiProgressBar::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        let mut pb = ae_core::ecs::UiProgressBar {
            value: 50.0,
            min: 0.0,
            max: 100.0,
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_progress_bar_card(&mut scope, &mut ctx, &mut pb);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiProgressBarCard")),
            "UiProgressBarCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "UiProgressBar must emit 3 numeric pills (Min, Max, Value)"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("MiniBarTrack")),
            "MiniBarTrack must be rendered"
        );
    }

    #[test]
    fn test_ui_button_toggle_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiButton {
            is_enabled: true,
            ..Default::default()
        },));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        {
            let mut scope = UiScope::new(&mut tree, root);
            UiButtonHandler.render_card(&mut scope, &mut ctx);
        }

        {
            let mut b = world.get::<&mut ae_core::ecs::UiButton>(entity).unwrap();
            b.is_enabled = false;
        }
        let btn = world.get::<&ae_core::ecs::UiButton>(entity).unwrap();
        assert!(!btn.is_enabled);
    }

    #[test]
    fn test_draw_ui_button_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::UiButton::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        let mut btn = ae_core::ecs::UiButton::default();
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_button_card(&mut scope, &mut ctx, &mut btn);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiButtonCard")),
            "UiButtonCard container must be rendered"
        );
        assert!(
            tree.iter().any(|(_, n)| n.role == WidgetRole::Checkbox),
            "UiButton checkbox must be rendered"
        );
    }

    #[test]
    fn test_draw_ui_image_card_standalone() {
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let world = hecs::World::new();
        let entity = hecs::Entity::DANGLING;
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = create_test_context(&world, entity, &params, &mut actions);

        let img = ae_core::ecs::UiImage {
            slice_mode: ae_core::ui::UiSliceMode::NineSlice,
            ..Default::default()
        };
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_image_card(&mut scope, &mut ctx, &img);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiImageCard")),
            "UiImageCard container must be rendered"
        );
        let has_mode = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("UiImageMode")
                && n.text.as_deref().is_some_and(|t| t.contains("NineSlice"))
        });
        assert!(has_mode, "UiImageMode label must display NineSlice");
    }
}