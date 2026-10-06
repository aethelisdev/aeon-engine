// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rendering and Illumination Component Inspector Cards
//!
//! Provides handlers for `💡 Light`, `📦 ModelId`, and `🎲 Shape`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::{ComponentCategory, InspectorDropdownId, InspectorNumberInputId};
use super::physics::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, DeclarativeNumericRowParams,
    build_declarative_card_header, render_declarative_combobox_row, render_declarative_numeric_row,
};
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (pos, col) = ctx
            .world
            .get::<&ae_core::ecs::Light>(ctx.entity)
            .map(|l| (l.position, l.color))
            .unwrap_or(([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));

        let get_edit = |id| {
            ctx.params
                .active_number_input
                .filter(|s| s.id == id)
                .map(|s| s.to_edit_state(ctx.params.blink_caret))
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("LightCard", card_style, |card| {
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

            // Row 1: Light Offset Y
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::LightOffsetY,
                    label: "Light Offset Y:",
                    val: pos[1],
                    label_w: 95.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::LightOffsetY),
                    is_hovered: false,
                },
            );

            // Row 2: Color R
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::LightColorR,
                    label: "Color R:",
                    val: col[0],
                    label_w: 95.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::LightColorR),
                    is_hovered: false,
                },
            );

            // Row 3: Color G
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::LightColorG,
                    label: "Color G:",
                    val: col[1],
                    label_w: 95.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::LightColorG),
                    is_hovered: false,
                },
            );
        });
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
        let handle_str = ctx
            .world
            .get::<&ae_core::ecs::ModelId>(ctx.entity)
            .map(|m| format!("{:?}", m.0))
            .unwrap_or_else(|_| "Default".to_string());

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("ModelMeshCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: self.atlas_icon(),
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            card.label_styled_passive(
                "ModelAssetInfo",
                format!("Asset Handle: {}", handle_str),
                10.5,
                Color::rgba(0.54, 0.56, 0.60, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
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
        "🌐"
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
        let shape_str = match ctx.world.get::<&ae_core::ecs::Shape>(ctx.entity).as_deref() {
            Ok(ae_core::ecs::Shape::Cube) => "Cube",
            Ok(ae_core::ecs::Shape::Sphere) => "Sphere",
            Ok(ae_core::ecs::Shape::Cylinder) => "Cylinder",
            Ok(ae_core::ecs::Shape::Capsule) => "Capsule",
            Ok(ae_core::ecs::Shape::Torus) => "Torus",
            Ok(ae_core::ecs::Shape::Triangle) => "Triangle",
            _ => "Procedural Mesh",
        };

        let is_open = ctx.params.active_dropdown == Some(InspectorDropdownId::ShapeType);

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("ShapeCard", card_style, |card| {
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

            render_declarative_combobox_row(
                card,
                DeclarativeComboboxRowParams {
                    dropdown_id: InspectorDropdownId::ShapeType,
                    label: Some("Geometry:"),
                    label_w: 65.0,
                    selected_text: shape_str,
                    is_open,
                    is_hovered: false,
                    combo_w: 96.0,
                },
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::Shape::Cube);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::icons::{ICON_CHEVRON_DOWN, ICON_CHEVRON_UP};
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::tags::{
        encode_component_delete_tag, encode_inspector_dropdown_tag,
    };
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

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
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
            hovered_tag: None,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        let mut scope = UiScope::new(&mut tree, root);
        let handler = ShapeHandler;
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("Shape");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Shape delete button must be tagged"
        );

        let dd_tag = encode_inspector_dropdown_tag(InspectorDropdownId::ShapeType);
        assert!(
            tree.iter().any(|(_, n)| n.tag == dd_tag),
            "Shape dropdown button must be tagged"
        );

        let has_chevron_down = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.texture_uv == Some(ICON_CHEVRON_DOWN)
        });
        assert!(
            has_chevron_down,
            "Combobox must render ICON_CHEVRON_DOWN when closed"
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
            scroll_y: 0.0,
            active_dropdown: Some(InspectorDropdownId::ShapeType),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
            hovered_tag: None,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        let mut scope = UiScope::new(&mut tree, root);
        let handler = ShapeHandler;
        handler.render_card(&mut scope, &mut ctx);

        let has_chevron_up = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.texture_uv == Some(ICON_CHEVRON_UP)
        });
        assert!(
            has_chevron_up,
            "Combobox must render ICON_CHEVRON_UP when open"
        );
    }
}