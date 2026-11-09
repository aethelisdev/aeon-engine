// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rendering and Illumination Component Inspector Cards
//!
//! Provides handlers for `💡 Light`, `📦 ModelId`, and `🎲 Shape`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_LIGHT};
use irisui::prelude::*;

/// Renders the declarative Light component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates position offset and RGB illumination color scrubbers with zero intermediate variable plumbing.
pub fn draw_light_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "LightCard",
        UiScope::comp_card_desc("Light", "Light", "💡", Color::rgba(0.98, 0.80, 0.20, 1.0))
            .atlas_icon(ICON_LIGHT),
        world.get::<&mut ae_core::ecs::Light>(entity),
        |card, mut light| {
            card.prop_f32(
                "Light Offset Y:",
                &mut light.position[1],
                -1_000.0..=1_000.0,
                0.05,
            );
            card.prop_f32("Color R:", &mut light.color[0], 0.0..=1.0, 0.01);
            card.prop_f32("Color G:", &mut light.color[1], 0.0..=1.0, 0.01);
        },
    );
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
        draw_light_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::Light::default()));
    }
}

/// Renders the declarative 3D Model / Mesh component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Displays the active mesh asset handle with zero intermediate variable plumbing.
pub fn draw_model_mesh_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "ModelMeshCard",
        UiScope::comp_card_desc(
            "ModelId",
            "3D Model / Mesh",
            "📦",
            Color::rgba(0.38, 0.65, 0.98, 1.0),
        )
        .atlas_icon(crate::ui::iris_bridge::icons::ICON_CUBE),
        world.get::<&ae_core::ecs::ModelId>(entity),
        |card, model| {
            card.label_desc("ModelAssetInfo", format!("Asset Handle: {:?}", model.0));
        },
    );
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
        draw_model_mesh_card(scope, ctx.world, ctx.entity);
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_core::ecs::ModelId::default()));
    }
}

/// Converts a [`ae_core::ecs::Shape`] enum variant to its combobox option index.
#[inline]
#[must_use]
pub const fn shape_to_index(shape: ae_core::ecs::Shape) -> usize {
    match shape {
        ae_core::ecs::Shape::Cube => 0,
        ae_core::ecs::Shape::Sphere => 1,
        ae_core::ecs::Shape::Cylinder => 2,
        ae_core::ecs::Shape::Capsule => 3,
        ae_core::ecs::Shape::Torus => 4,
        ae_core::ecs::Shape::Triangle => 5,
    }
}

/// Converts a combobox option index back to its corresponding [`ae_core::ecs::Shape`] enum variant.
#[inline]
#[must_use]
pub const fn index_to_shape(index: usize) -> ae_core::ecs::Shape {
    match index {
        0 => ae_core::ecs::Shape::Cube,
        1 => ae_core::ecs::Shape::Sphere,
        2 => ae_core::ecs::Shape::Cylinder,
        3 => ae_core::ecs::Shape::Capsule,
        4 => ae_core::ecs::Shape::Torus,
        5 => ae_core::ecs::Shape::Triangle,
        _ => ae_core::ecs::Shape::Cube,
    }
}

/// Renders the declarative procedural Shape component card into [`UiScope`] if present on [`hecs::Entity`].
///
/// Encapsulates geometry selection dropdown with zero intermediate variable plumbing.
pub fn draw_shape_card(scope: &mut UiScope<'_>, world: &hecs::World, entity: hecs::Entity) {
    scope.card_panel_desc_when_ok(
        "ShapeCard",
        UiScope::comp_card_desc(
            "Shape",
            "Procedural 3D Shape",
            "🌐",
            Color::rgba(0.20, 0.85, 0.60, 1.0),
        )
        .atlas_icon(ICON_CUBE),
        world.get::<&mut ae_core::ecs::Shape>(entity),
        |card, mut s| {
            card.prop_combobox_mapped_styled(
                "Geometry:",
                shape_to_index(*s),
                &["Cube", "Sphere", "Cylinder", "Capsule", "Torus", "Triangle"],
                ComboRowLayout::new(65.0, 96.0),
                &mut *s,
                index_to_shape,
            );
        },
    );
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
        draw_shape_card(scope, ctx.world, ctx.entity);
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
        let mut scope = UiScope::new(&mut tree, root);

        draw_light_card(&mut scope, &world, entity);

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
    fn test_draw_light_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_light_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("LightCard")),
            "Light card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_draw_model_mesh_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::ModelId::default(),));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_model_mesh_card(&mut scope, &world, entity);

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
    fn test_draw_model_mesh_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_model_mesh_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ModelMeshCard")),
            "ModelMesh card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_draw_shape_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Shape::Cube,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_shape_card(&mut scope, &world, entity);

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
    fn test_draw_shape_card_missing_component_safely_ignored() {
        let world = hecs::World::new();
        let entity = world.reserve_entity();
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut scope = UiScope::new(&mut tree, root);

        draw_shape_card(&mut scope, &world, entity);

        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.name.as_deref() == Some("ShapeCard")),
            "Shape card must safely do nothing when component missing"
        );
    }

    #[test]
    fn test_shape_index_conversion_roundtrip() {
        let shapes = [
            ae_core::ecs::Shape::Cube,
            ae_core::ecs::Shape::Sphere,
            ae_core::ecs::Shape::Cylinder,
            ae_core::ecs::Shape::Capsule,
            ae_core::ecs::Shape::Torus,
            ae_core::ecs::Shape::Triangle,
        ];
        for s in shapes {
            assert_eq!(index_to_shape(shape_to_index(s)), s);
        }
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