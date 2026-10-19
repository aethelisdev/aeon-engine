// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Collider Component Inspector Card
//!
//! Provides the `ColliderHandler` implementing UI rendering, shape combobox
//! selection (`Capsule`, `Box`, `Sphere`, `Trimesh`, `Convex Hull`), dynamic
//! dimension rows, and default component attachment.

use super::helpers::{
    ComponentHeaderProps, DeclarativeComboboxRowParams, build_declarative_card_header,
    render_declarative_checkbox_row, render_declarative_combobox_row,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::tags::encode_inspector_number_input_tag;
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
        let (shape, mut friction, mut restitution, is_sensor) = ctx
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

        let label_w = 85.0;
        let box_w = 44.0;

        scope.card_panel("ColliderCard", |card| {
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
                    dropdown_id: InspectorDropdownId::ColliderShape,
                    label: Some("Shape:"),
                    label_w,
                    selected_text: shape_str,
                    combo_w: 88.0,
                    options: &["Capsule", "Box", "Sphere", "Trimesh", "Convex Hull"],
                },
            );

            match shape {
                ae_core::ecs::ColliderShape::Box { half_extents } => {
                    let [mut hx, mut hy, mut hz] = half_extents;
                    let mut rows = [
                        (
                            InspectorNumberInputId::ColliderBoxX,
                            "Half Extent X:",
                            &mut hx,
                        ),
                        (
                            InspectorNumberInputId::ColliderBoxY,
                            "Half Extent Y:",
                            &mut hy,
                        ),
                        (
                            InspectorNumberInputId::ColliderBoxZ,
                            "Half Extent Z:",
                            &mut hz,
                        ),
                    ];
                    for (id, lbl, val) in &mut rows {
                        let tag = encode_inspector_number_input_tag(*id);
                        let edit = ctx.edit_state(*id);
                        let resp = card.add(
                            NumericDrag::new(lbl, val)
                                .tag(tag)
                                .label_width(label_w)
                                .box_width(box_w)
                                .edit_state(edit),
                        );
                        if resp.changed() {
                            ctx.set_number(*id, **val);
                        }
                        if resp.committed() {
                            ctx.commit_number(*id);
                        }
                    }
                }
                ae_core::ecs::ColliderShape::Sphere { mut radius } => {
                    let id_r = InspectorNumberInputId::ColliderRadius;
                    let resp = card.add(
                        NumericDrag::new("Radius:", &mut radius)
                            .tag(encode_inspector_number_input_tag(id_r))
                            .label_width(label_w)
                            .box_width(box_w)
                            .edit_state(ctx.edit_state(id_r)),
                    );
                    if resp.changed() {
                        ctx.set_number(id_r, radius);
                    }
                    if resp.committed() {
                        ctx.commit_number(id_r);
                    }
                }
                ae_core::ecs::ColliderShape::Capsule {
                    mut half_height,
                    mut radius,
                    mut center_y,
                } => {
                    let mut rows = [
                        (
                            InspectorNumberInputId::ColliderHalfHeight,
                            "Half Height:",
                            &mut half_height,
                        ),
                        (
                            InspectorNumberInputId::ColliderRadius,
                            "Radius:",
                            &mut radius,
                        ),
                        (
                            InspectorNumberInputId::ColliderCenterY,
                            "Center Y:",
                            &mut center_y,
                        ),
                    ];
                    for (id, lbl, val) in &mut rows {
                        let tag = encode_inspector_number_input_tag(*id);
                        let edit = ctx.edit_state(*id);
                        let resp = card.add(
                            NumericDrag::new(lbl, val)
                                .tag(tag)
                                .label_width(label_w)
                                .box_width(box_w)
                                .edit_state(edit),
                        );
                        if resp.changed() {
                            ctx.set_number(*id, **val);
                        }
                        if resp.committed() {
                            ctx.commit_number(*id);
                        }
                    }
                }
                ae_core::ecs::ColliderShape::Trimesh | ae_core::ecs::ColliderShape::ConvexHull => {}
            }

            let id_f = InspectorNumberInputId::ColliderFriction;
            let resp_f = card.add(
                NumericDrag::new("Friction:", &mut friction)
                    .tag(encode_inspector_number_input_tag(id_f))
                    .label_width(label_w)
                    .box_width(box_w)
                    .edit_state(ctx.edit_state(id_f)),
            );
            if resp_f.changed() {
                ctx.set_number(id_f, friction);
            }
            if resp_f.committed() {
                ctx.commit_number(id_f);
            }

            let id_re = InspectorNumberInputId::ColliderRestitution;
            let resp_re = card.add(
                NumericDrag::new("Restitution:", &mut restitution)
                    .tag(encode_inspector_number_input_tag(id_re))
                    .label_width(label_w)
                    .box_width(box_w)
                    .edit_state(ctx.edit_state(id_re)),
            );
            if resp_re.changed() {
                ctx.set_number(id_re, restitution);
            }
            if resp_re.committed() {
                ctx.commit_number(id_re);
            }

            let resp_sensor = render_declarative_checkbox_row(
                card,
                ComponentCheckboxId::ColliderIsSensor,
                "Is Sensor (Trigger)",
                is_sensor,
            );
            if resp_sensor.clicked() {
                ctx.toggle_checkbox(ComponentCheckboxId::ColliderIsSensor);
            }
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
    use crate::ui::iris_bridge::icons::ICON_WIREFRAME;
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
            n.name.as_deref() == Some("ComboChevron")
                && n.texture_uv == Some(crate::ui::iris_bridge::icons::ICON_CHEVRON_DOWN)
        });
        assert!(
            has_chevron_down,
            "Collider Shape combobox must render atlas chevron down when closed"
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