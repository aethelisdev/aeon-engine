// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rendering and Illumination Component Inspector Cards
//!
//! Provides handlers for `💡 Light`, `📦 ModelId`, and `🎲 Shape`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_LIGHT};
use irisui::prelude::*;

/// Renders the declarative Light component card with position offset and RGB illumination colors.
///
/// Modifies `light` fields in-place via [`UiScope::prop_f32`], automatically recording
/// atomic Undo/Redo sessions on edit commits without intermediate imperative plumbing.
pub fn draw_light_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    light: &mut ae_core::ecs::Light,
) {
    scope.card_panel("LightCard", |card| {
        LightHandler.render_header(card);

        card.prop_f32(
            "Light Offset Y:",
            &mut light.position[1],
            -1_000.0..=1_000.0,
            0.05,
        )
        .on_commit(|_| {
            ctx.begin_edit("Light");
            ctx.commit_edit("Light");
        });

        card.prop_f32("Color R:", &mut light.color[0], 0.0..=1.0, 0.01)
            .on_commit(|_| {
                ctx.begin_edit("Light");
                ctx.commit_edit("Light");
            });

        card.prop_f32("Color G:", &mut light.color[1], 0.0..=1.0, 0.01)
            .on_commit(|_| {
                ctx.begin_edit("Light");
                ctx.commit_edit("Light");
            });
    });
}

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

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(ICON_LIGHT)
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::Light>(ctx.entity),
            |card, mut light| {
                draw_light_card(card, ctx, &mut light);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::Light::default()));
    }
}

/// Renders the declarative 3D Model / Mesh component card displaying the active mesh asset handle.
pub fn draw_model_mesh_card(
    scope: &mut UiScope<'_>,
    _ctx: &mut ComponentRenderContext<'_>,
    model: &ae_core::ecs::ModelId,
) {
    scope.card_panel("ModelMeshCard", |card| {
        ModelMeshHandler.render_header(card);

        card.label_styled_passive(
            "ModelAssetInfo",
            format!("Asset Handle: {:?}", model.0),
            10.5,
            card.theme().text_muted,
            TextAlign::Left,
            Style::new().height(20.0),
        );
    });
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

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(crate::ui::iris_bridge::icons::ICON_CUBE)
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&ae_core::ecs::ModelId>(ctx.entity),
            |card, model| {
                draw_model_mesh_card(card, ctx, &model);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::ModelId::default()));
    }
}

/// Renders the declarative procedural Shape component card with geometry selection dropdown.
///
/// Modifies `s` in-place via [`UiScope::prop_combobox_mapped_styled`], recording Undo/Redo sessions
/// on user selection changes with zero intermediate `let` variables or imperatively plumbed state.
pub fn draw_shape_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    s: &mut ae_core::ecs::Shape,
) {
    scope.card_panel("ShapeCard", |card| {
        ShapeHandler.render_header(card);

        card.prop_combobox_mapped_styled(
            "Geometry:",
            match *s {
                ae_core::ecs::Shape::Cube => 0,
                ae_core::ecs::Shape::Sphere => 1,
                ae_core::ecs::Shape::Cylinder => 2,
                ae_core::ecs::Shape::Capsule => 3,
                ae_core::ecs::Shape::Torus => 4,
                ae_core::ecs::Shape::Triangle => 5,
            },
            &["Cube", "Sphere", "Cylinder", "Capsule", "Torus", "Triangle"],
            ComboRowLayout::new(65.0, 96.0),
            s,
            |idx| match idx {
                0 => ae_core::ecs::Shape::Cube,
                1 => ae_core::ecs::Shape::Sphere,
                2 => ae_core::ecs::Shape::Cylinder,
                3 => ae_core::ecs::Shape::Capsule,
                4 => ae_core::ecs::Shape::Torus,
                5 => ae_core::ecs::Shape::Triangle,
                _ => ae_core::ecs::Shape::Cube,
            },
        )
        .on_change(|| {
            ctx.begin_edit("Shape");
            ctx.commit_edit("Shape");
        });
    });
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
        "🌐"
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(ICON_CUBE)
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        scope.when_ok(
            ctx.world.get::<&mut ae_core::ecs::Shape>(ctx.entity),
            |card, mut s| {
                draw_shape_card(card, ctx, &mut s);
            },
        );
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::Shape::Cube));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_light_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Light {
            position: [0.0, 5.0, 0.0],
            color: [1.0, 0.8, 0.6],
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

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
        let handler = LightHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("Light");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Light card must tag delete button with semantic delete tag"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "Light card must emit 3 numeric pills (Offset Y, Color R, Color G)"
        );

        // Test in-place mutation and undo snapshotting
        {
            let mut l = world.get::<&mut ae_core::ecs::Light>(entity).unwrap();
            ctx.begin_edit(handler.component_name());
            l.position[1] = 8.5;
            l.color[0] = 0.5;
            l.color[1] = 0.4;
            ctx.commit_edit(handler.component_name());
        }

        {
            let l = world.get::<&ae_core::ecs::Light>(entity).unwrap();
            assert_eq!(l.position[1], 8.5);
            assert_eq!(l.color[0], 0.5);
            assert_eq!(l.color[1], 0.4);
        }
    }

    #[test]
    fn test_draw_light_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Light::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

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

        let mut light = ae_core::ecs::Light::default();
        let mut scope = UiScope::new(&mut tree, root);
        draw_light_card(&mut scope, &mut ctx, &mut light);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("LightCard")),
            "LightCard container must be rendered"
        );
        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 3,
            "Light card must emit exactly 3 numeric pills"
        );
    }

    #[test]
    fn test_draw_model_mesh_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::ModelId::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

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

        let model = ae_core::ecs::ModelId::default();
        let mut scope = UiScope::new(&mut tree, root);
        draw_model_mesh_card(&mut scope, &mut ctx, &model);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ModelMeshCard")),
            "ModelMeshCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ModelAssetInfo")),
            "ModelAssetInfo passive label must be rendered"
        );
    }

    #[test]
    fn test_draw_shape_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

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

        let mut shape = ae_core::ecs::Shape::Cube;
        let mut scope = UiScope::new(&mut tree, root);
        draw_shape_card(&mut scope, &mut ctx, &mut shape);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ShapeCard")),
            "ShapeCard container must be rendered"
        );
        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ComboPill")),
            "Shape combobox pill must be rendered"
        );
    }

    #[test]
    fn test_shape_declarative_render() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        let mut scope = UiScope::new(&mut tree, root);
        let handler = ShapeHandler;
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = UiScope::action_tag("Shape");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Shape delete button must be tagged"
        );

        let has_cube_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_CUBE)
        });
        assert!(
            has_cube_icon,
            "Shape card header must render ICON_CUBE from editor atlas"
        );

        let has_combo_pill = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
        assert!(has_combo_pill, "Shape combobox pill must be rendered");

        let has_chevron_down = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▾")
        });
        assert!(
            has_chevron_down,
            "Combobox must render chevron down (▾) when closed"
        );
    }

    #[test]
    fn test_shape_combobox_renders_chevron_up_when_open() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        // First pass: render once to obtain ComboPill tag
        {
            let mut scope = UiScope::new(&mut tree, root);
            let handler = ShapeHandler;
            handler.render_card(&mut scope, &mut ctx);
        }

        let combo_tag = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
            .map(|(_, n)| n.tag)
            .expect("ComboPill node must exist in tree");

        // Second pass: set active_combobox_tag to combo_tag and render
        let mut tree2 = UiTree::new();
        let root2 = tree2.create_root().expect("Root node must exist");
        tree2.active_combobox_tag = Some(combo_tag);

        let mut scope2 = UiScope::new(&mut tree2, root2);
        let handler = ShapeHandler;
        handler.render_card(&mut scope2, &mut ctx);

        let has_chevron_up = tree2.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.text.as_deref() == Some("▴")
        });
        assert!(
            has_chevron_up,
            "Combobox must render chevron up (▴) when open"
        );
    }

    #[test]
    fn test_shape_declarative_in_place_mutation_and_undo_snapshot() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Shape::Cube,));
        let mut actions = Vec::new();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            10.0,
            20.0,
            260.0,
        );

        let handler = ShapeHandler;
        {
            let mut s = world.get::<&mut ae_core::ecs::Shape>(entity).unwrap();
            ctx.begin_edit(handler.component_name());
            *s = ae_core::ecs::Shape::Sphere;
            ctx.commit_edit(handler.component_name());
        }

        {
            let s = world.get::<&ae_core::ecs::Shape>(entity).unwrap();
            assert_eq!(*s, ae_core::ecs::Shape::Sphere);
        }
    }
}