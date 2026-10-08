// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collider Component Inspector Card
//!
//! Provides the `ColliderHandler` implementing UI rendering, shape combobox
//! selection (`Capsule`, `Box`, `Sphere`, `Trimesh`, `Convex Hull`), dynamic
//! dimension rows, and default component attachment.

use super::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, DeclarativeNumericRowParams,
    build_declarative_card_header, render_declarative_checkbox_row,
    render_declarative_combobox_row, render_declarative_numeric_row,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{
    ComponentCategory, ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId,
};
use irisui::prelude::*;

/// Inspector handler for Collider.
pub struct ColliderHandler;

impl ComponentInspectorHandler for ColliderHandler {
    fn component_name(&self) -> &'static str {
        "Collider"
    }

    fn display_title(&self) -> &'static str {
        "Collider"
    }

    fn icon(&self) -> &'static str {
        "🛡"
    }

    fn atlas_icon(&self) -> Option<[f32; 4]> {
        Some(crate::ui::iris_bridge::icons::ICON_WIREFRAME)
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.20, 0.88, 0.45, 1.0) // Emerald Green (#2ecc71)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::Collider>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (shape, friction, restitution, is_sensor) = ctx
            .world
            .get::<&ae_core::ecs::Collider>(ctx.entity)
            .map(|c| (c.shape, c.friction, c.restitution, c.is_sensor))
            .unwrap_or((
                ae_core::ecs::ColliderShape::Capsule {
                    half_height: 0.50,
                    radius: 0.40,
                    center_y: 0.0,
                },
                0.70,
                0.0,
                false,
            ));

        let shape_str = match shape {
            ae_core::ecs::ColliderShape::Box { .. } => "Box",
            ae_core::ecs::ColliderShape::Sphere { .. } => "Sphere",
            ae_core::ecs::ColliderShape::Capsule { .. } => "Capsule",
            ae_core::ecs::ColliderShape::Trimesh => "Trimesh",
            ae_core::ecs::ColliderShape::ConvexHull => "Convex Hull",
        };

        let is_open = ctx.params.active_dropdown == Some(InspectorDropdownId::ColliderShape);

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        let label_w = 85.0;
        let box_w = 44.0;

        scope.container_named("ColliderCard", card_style, |card| {
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
                    dropdown_id: InspectorDropdownId::ColliderShape,
                    label: Some("Shape:"),
                    label_w,
                    selected_text: shape_str,
                    is_open,
                    is_hovered: false,
                    combo_w: 88.0,
                },
            );

            match shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => {
                    let id_x = InspectorNumberInputId::ColliderBoxX;
                    let edit_x = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_x)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_x,
                            label: "Half Extent X:",
                            val: half_extents[0],
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_x,
                            is_hovered: false,
                        },
                    );

                    let id_y = InspectorNumberInputId::ColliderBoxY;
                    let edit_y = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_y)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_y,
                            label: "Half Extent Y:",
                            val: half_extents[1],
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_y,
                            is_hovered: false,
                        },
                    );

                    let id_z = InspectorNumberInputId::ColliderBoxZ;
                    let edit_z = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_z)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_z,
                            label: "Half Extent Z:",
                            val: half_extents[2],
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_z,
                            is_hovered: false,
                        },
                    );
                }
                ae_core::ecs::ColliderShape::Sphere { radius } => {
                    let id_r = InspectorNumberInputId::ColliderRadius;
                    let edit_r = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_r)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_r,
                            label: "Radius:",
                            val: radius,
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_r,
                            is_hovered: false,
                        },
                    );
                }
                ae_core::ecs::ColliderShape::Capsule {
                    half_height,
                    radius,
                    center_y,
                } => {
                    let id_hh = InspectorNumberInputId::ColliderHalfHeight;
                    let edit_hh = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_hh)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_hh,
                            label: "Half Height:",
                            val: half_height,
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_hh,
                            is_hovered: false,
                        },
                    );

                    let id_r = InspectorNumberInputId::ColliderRadius;
                    let edit_r = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_r)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_r,
                            label: "Radius:",
                            val: radius,
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_r,
                            is_hovered: false,
                        },
                    );

                    let id_cy = InspectorNumberInputId::ColliderCenterY;
                    let edit_cy = ctx
                        .params
                        .active_number_input
                        .filter(|s| s.id == id_cy)
                        .map(|s| s.to_edit_state(ctx.params.blink_caret));
                    render_declarative_numeric_row(
                        card,
                        DeclarativeNumericRowParams {
                            input_id: id_cy,
                            label: "Center Y:",
                            val: center_y,
                            label_w,
                            box_w,
                            unit: None,
                            edit_state: edit_cy,
                            is_hovered: false,
                        },
                    );
                }
                ae_core::ecs::ColliderShape::Trimesh | ae_core::ecs::ColliderShape::ConvexHull => {}
            }

            let id_f = InspectorNumberInputId::ColliderFriction;
            let edit_f = ctx
                .params
                .active_number_input
                .filter(|s| s.id == id_f)
                .map(|s| s.to_edit_state(ctx.params.blink_caret));
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: id_f,
                    label: "Friction:",
                    val: friction,
                    label_w,
                    box_w,
                    unit: None,
                    edit_state: edit_f,
                    is_hovered: false,
                },
            );

            let id_re = InspectorNumberInputId::ColliderRestitution;
            let edit_re = ctx
                .params
                .active_number_input
                .filter(|s| s.id == id_re)
                .map(|s| s.to_edit_state(ctx.params.blink_caret));
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: id_re,
                    label: "Restitution:",
                    val: restitution,
                    label_w,
                    box_w,
                    unit: None,
                    edit_state: edit_re,
                    is_hovered: false,
                },
            );

            render_declarative_checkbox_row(
                card,
                ComponentCheckboxId::ColliderIsSensor,
                "Is Sensor (Trigger)",
                is_sensor,
                false,
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let collider = ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Capsule {
                half_height: 0.50,
                radius: 0.40,
                center_y: 0.0,
            },
            friction: 0.70,
            restitution: 0.0,
            is_sensor: false,
        };
        let _ = world.insert_one(entity, collider);
        if world.get::<&ae_core::ecs::RigidBody>(entity).is_err() {
            let _ = world.insert_one(entity, ae_core::ecs::RigidBody::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::icons::{ICON_CHEVRON_DOWN, ICON_WIREFRAME};
    use crate::ui::iris_bridge::inspector::tags::{
        encode_component_checkbox_tag, encode_component_delete_tag, encode_inspector_dropdown_tag,
        encode_inspector_number_input_tag,
    };
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_collider_declarative_scope_renders_wireframe_and_chevron() {
        let handler = ColliderHandler;
        assert_eq!(handler.atlas_icon(), Some(ICON_WIREFRAME));

        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [1.0, 2.0, 3.0],
            },
            friction: 0.5,
            restitution: 0.2,
            is_sensor: true,
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

        let has_wireframe_icon = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("CardAtlasIcon") && n.texture_uv == Some(ICON_WIREFRAME)
        });
        assert!(
            has_wireframe_icon,
            "Collider card header must render ICON_WIREFRAME"
        );

        let has_chevron_down = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("ComboChevron") && n.texture_uv == Some(ICON_CHEVRON_DOWN)
        });
        assert!(
            has_chevron_down,
            "Collider Shape combobox must render ICON_CHEVRON_DOWN when closed"
        );

        let dropdown_tag = encode_inspector_dropdown_tag(InspectorDropdownId::ColliderShape);
        assert!(
            tree.iter().any(|(_, n)| n.tag == dropdown_tag),
            "Collider dropdown must be tagged"
        );

        let tag_x = encode_inspector_number_input_tag(InspectorNumberInputId::ColliderBoxX);
        let tag_y = encode_inspector_number_input_tag(InspectorNumberInputId::ColliderBoxY);
        let tag_z = encode_inspector_number_input_tag(InspectorNumberInputId::ColliderBoxZ);
        let tag_f = encode_inspector_number_input_tag(InspectorNumberInputId::ColliderFriction);
        let tag_re = encode_inspector_number_input_tag(InspectorNumberInputId::ColliderRestitution);

        assert!(tree.iter().any(|(_, n)| n.tag == tag_x));
        assert!(tree.iter().any(|(_, n)| n.tag == tag_y));
        assert!(tree.iter().any(|(_, n)| n.tag == tag_z));
        assert!(tree.iter().any(|(_, n)| n.tag == tag_f));
        assert!(tree.iter().any(|(_, n)| n.tag == tag_re));

        let sensor_tag = encode_component_checkbox_tag(ComponentCheckboxId::ColliderIsSensor);
        assert!(
            tree.iter().any(|(_, n)| n.tag == sensor_tag),
            "Collider Is Sensor checkbox must be tagged"
        );

        let del_tag = encode_component_delete_tag("Collider");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "Collider delete button must be tagged"
        );
    }
}