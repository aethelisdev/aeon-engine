// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # RigidBody Component Inspector Card
//!
//! Provides the `RigidBodyHandler` implementing UI rendering, body type combobox
//! (`Dynamic`, `Kinematic`, `Static`), mass and gravity scale numeric inputs,
//! and paired component attachment with `Collider`.

use super::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, DeclarativeNumericRowParams,
    build_declarative_card_header, render_declarative_combobox_row, render_declarative_numeric_row,
};
use crate::ui::iris_bridge::icons::ICON_GEAR;
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{
    ComponentCategory, InspectorDropdownId, InspectorNumberInputId,
};
use irisui::prelude::*;

/// Inspector handler for `⚙ RigidBody`.
pub struct RigidBodyHandler;

impl ComponentInspectorHandler for RigidBodyHandler {
    fn component_name(&self) -> &'static str {
        "RigidBody"
    }

    fn display_title(&self) -> &'static str {
        "RigidBody"
    }

    fn icon(&self) -> &'static str {
        "⚙"
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(ICON_GEAR)
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.22, 0.74, 0.98, 1.0) // Sky Blue / Cyan (#38bdf8)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::RigidBody>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let rb_data = ctx
            .world
            .get::<&ae_core::ecs::RigidBody>(ctx.entity)
            .map(|rb| (rb.body_type, rb.mass, rb.gravity_scale))
            .unwrap_or((ae_core::ecs::RigidBodyType::Kinematic, 1.0, 1.0));

        let body_type_str = match rb_data.0 {
            ae_core::ecs::RigidBodyType::Dynamic => "Dynamic",
            ae_core::ecs::RigidBodyType::Kinematic => "Kinematic",
            ae_core::ecs::RigidBodyType::Static => "Static",
        };

        let is_open = ctx.params.active_dropdown == Some(InspectorDropdownId::RigidBodyType);

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("RigidBodyCard", card_style, |card| {
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
                DeclarativeComboboxRowParams {
                    dropdown_id: InspectorDropdownId::RigidBodyType,
                    label: None,
                    label_w: 0.0,
                    selected_text: body_type_str,
                    is_open,
                    is_hovered: false,
                    combo_w: 96.0,
                },
            );

            let mass_edit = ctx
                .params
                .active_number_input
                .filter(|s| s.id == InspectorNumberInputId::RigidBodyMass)
                .map(|s| s.to_edit_state(ctx.params.blink_caret));
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::RigidBodyMass,
                    label: "Mass:",
                    val: rb_data.1,
                    label_w: 55.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: mass_edit,
                    is_hovered: false,
                },
            );

            let grav_edit = ctx
                .params
                .active_number_input
                .filter(|s| s.id == InspectorNumberInputId::RigidBodyGravity)
                .map(|s| s.to_edit_state(ctx.params.blink_caret));
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::RigidBodyGravity,
                    label: "Gravity:",
                    val: rb_data.2,
                    label_w: 55.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: grav_edit,
                    is_hovered: false,
                },
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::RigidBody::default());
        if world.get::<&ae_core::ecs::Collider>(entity).is_err() {
            let _ = world.insert_one(entity, ae_core::ecs::Collider::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::icons::{ICON_CHEVRON_DOWN, ICON_CHEVRON_UP};
    use crate::ui::iris_bridge::inspector::tags::{
        encode_component_delete_tag, encode_inspector_dropdown_tag,
        encode_inspector_number_input_tag,
    };
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_rigidbody_declarative_scope_renders_gear_icon_and_chevron() {
        let handler = RigidBodyHandler;
        assert_eq!(handler.atlas_icon(), Some(ICON_GEAR));

        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody {
            body_type: ae_core::ecs::RigidBodyType::Dynamic,
            mass: 2.5,
            gravity_scale: 1.0,
        },));

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
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 50.0, 280.0);

        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let has_gear_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_GEAR)
        });
        assert!(has_gear_icon, "RigidBody card header must render ICON_GEAR");

        let has_chevron_down = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.texture_uv == Some(ICON_CHEVRON_DOWN)
        });
        assert!(
            has_chevron_down,
            "Combobox must render ICON_CHEVRON_DOWN when closed"
        );

        let dropdown_tag = encode_inspector_dropdown_tag(InspectorDropdownId::RigidBodyType);
        let has_dropdown = tree.iter().any(|(_, n)| n.tag == dropdown_tag);
        assert!(has_dropdown, "RigidBody combobox must be tagged");

        let mass_tag = encode_inspector_number_input_tag(InspectorNumberInputId::RigidBodyMass);
        let has_mass = tree.iter().any(|(_, n)| n.tag == mass_tag);
        assert!(has_mass, "RigidBody mass input must be tagged");

        let grav_tag = encode_inspector_number_input_tag(InspectorNumberInputId::RigidBodyGravity);
        let has_grav = tree.iter().any(|(_, n)| n.tag == grav_tag);
        assert!(has_grav, "RigidBody gravity input must be tagged");

        let del_tag = encode_component_delete_tag("RigidBody");
        let has_del = tree.iter().any(|(_, n)| n.tag == del_tag);
        assert!(has_del, "RigidBody delete button must be tagged");
    }

    #[test]
    fn test_rigidbody_combobox_renders_chevron_up_when_open() {
        let handler = RigidBodyHandler;
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::RigidBody::default(),));

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
            active_dropdown: Some(InspectorDropdownId::RigidBodyType),
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

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 50.0, 280.0);

        let mut scope = UiScope::new(&mut tree, root);
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