// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Rendering and Illumination Component Inspector Cards
//!
//! Provides handlers for `💡 Light`, `📦 ModelId`, and `🎲 Shape`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::tags::encode_inspector_number_input_tag;
use super::super::types::{ComponentCategory, InspectorDropdownId, InspectorNumberInputId};
use super::physics::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, build_declarative_card_header,
    render_declarative_combobox_row,
};
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_LIGHT};
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
        let (mut pos, mut col) = ctx
            .world
            .get::<&ae_core::ecs::Light>(ctx.entity)
            .map(|l| (l.position, l.color))
            .unwrap_or(([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));

        scope.card_panel("LightCard", |card| {
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

            // Row 1: Light Offset Y
            let id_offset = InspectorNumberInputId::LightOffsetY;
            let resp_offset = card.add(
                NumericDrag::new("Light Offset Y:", &mut pos[1])
                    .label_width(95.0)
                    .box_width(44.0)
                    .tag(encode_inspector_number_input_tag(id_offset))
                    .edit_state(ctx.edit_state(id_offset)),
            );
            if resp_offset.changed() {
                ctx.set_number(id_offset, pos[1]);
            }
            if resp_offset.committed() {
                ctx.commit_number(id_offset);
            }

            // Row 2: Color R
            let id_r = InspectorNumberInputId::LightColorR;
            let resp_r = card.add(
                NumericDrag::new("Color R:", &mut col[0])
                    .label_width(95.0)
                    .box_width(44.0)
                    .range(0.0..=1.0)
                    .speed(0.01)
                    .tag(encode_inspector_number_input_tag(id_r))
                    .edit_state(ctx.edit_state(id_r)),
            );
            if resp_r.changed() {
                ctx.set_number(id_r, col[0]);
            }
            if resp_r.committed() {
                ctx.commit_number(id_r);
            }

            // Row 3: Color G
            let id_g = InspectorNumberInputId::LightColorG;
            let resp_g = card.add(
                NumericDrag::new("Color G:", &mut col[1])
                    .label_width(95.0)
                    .box_width(44.0)
                    .range(0.0..=1.0)
                    .speed(0.01)
                    .tag(encode_inspector_number_input_tag(id_g))
                    .edit_state(ctx.edit_state(id_g)),
            );
            if resp_g.changed() {
                ctx.set_number(id_g, col[1]);
            }
            if resp_g.committed() {
                ctx.commit_number(id_g);
            }
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

        scope.card_panel("ModelMeshCard", |card| {
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
                card.theme().text_muted,
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
        let shape_str = match ctx.world.get::<&ae_core::ecs::Shape>(ctx.entity).as_deref() {
            Ok(ae_core::ecs::Shape::Cube) => "Cube",
            Ok(ae_core::ecs::Shape::Sphere) => "Sphere",
            Ok(ae_core::ecs::Shape::Cylinder) => "Cylinder",
            Ok(ae_core::ecs::Shape::Capsule) => "Capsule",
            Ok(ae_core::ecs::Shape::Torus) => "Torus",
            Ok(ae_core::ecs::Shape::Triangle) => "Triangle",
            _ => "Procedural Mesh",
        };

        scope.card_panel("ShapeCard", |card| {
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

            render_declarative_combobox_row(
                card,
                ctx,
                DeclarativeComboboxRowParams {
                    dropdown_id: InspectorDropdownId::ShapeType,
                    label: Some("Geometry:"),
                    label_w: 65.0,
                    selected_text: shape_str,
                    combo_w: 96.0,
                    options: &["Cube", "Sphere", "Cylinder", "Capsule", "Torus", "Triangle"],
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

        let has_cube_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_CUBE)
        });
        assert!(
            has_cube_icon,
            "Shape card header must render ICON_CUBE from editor atlas"
        );

        let dd_tag = encode_inspector_dropdown_tag(InspectorDropdownId::ShapeType);
        assert!(
            tree.iter().any(|(_, n)| n.tag == dd_tag),
            "Shape dropdown button must be tagged"
        );

        let has_chevron_down = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron")
                && n.texture_uv == Some(crate::ui::iris_bridge::icons::ICON_CHEVRON_DOWN)
        });
        assert!(
            has_chevron_down,
            "Combobox must render atlas chevron down when closed"
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
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

        tree.active_combobox_tag = Some(encode_inspector_dropdown_tag(
            InspectorDropdownId::ShapeType,
        ));
        let mut scope = UiScope::new(&mut tree, root);
        let handler = ShapeHandler;
        handler.render_card(&mut scope, &mut ctx);

        let has_chevron_up = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron")
                && n.texture_uv == Some(crate::ui::iris_bridge::icons::ICON_CHEVRON_UP)
        });
        assert!(
            has_chevron_up,
            "Combobox must render atlas chevron up when open"
        );
    }
}