// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Material Component Inspector Card
//!
//! Provides the `PhysicsMaterialHandler` implementing UI rendering, surface type
//! combobox with preset action button, and friction / restitution numeric controls.

use super::helpers::{ComponentHeaderProps, build_declarative_card_header};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::tags::encode_inspector_number_input_tag;
use crate::ui::iris_bridge::inspector::types::{
    ComponentCategory, InspectorAction, InspectorDropdownId, InspectorNumberInputId,
};
use irisui::prelude::*;

/// Inspector handler for Physics Material.
pub struct PhysicsMaterialHandler;

impl ComponentInspectorHandler for PhysicsMaterialHandler {
    fn component_name(&self) -> &'static str {
        "PhysicsMaterial"
    }

    fn display_title(&self) -> &'static str {
        "Physics Material"
    }

    fn icon(&self) -> &'static str {
        "🧱"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.62, 0.15, 1.0) // Amber / Orange (#f59e0b)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Physics
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::PhysicsMaterial>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (mut friction, mut restitution, surface_name) = ctx
            .world
            .get::<&ae_core::ecs::PhysicsMaterial>(ctx.entity)
            .map(|m| (m.friction, m.restitution, m.surface_type.display_name()))
            .unwrap_or((0.70, 0.0, "Default"));

        scope.card_panel("PhysicsMaterialCard", |card| {
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

            // Row 1: Surface Type Dropdown + Preset Reset Button
            let row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(22.0)
                .gap(4.0);

            card.container_named("SurfaceTypeRow", row_style, |row| {
                let options = &[
                    "Default", "Metal", "Wood", "Stone", "Flesh", "Dirt", "Glass", "Rubber",
                ];
                let combo = ComboBox::new(surface_name, options)
                    .label("Surface Type:")
                    .label_width(85.0)
                    .width(80.0)
                    .tag(
                        crate::ui::iris_bridge::inspector::tags::encode_inspector_dropdown_tag(
                            InspectorDropdownId::SurfaceType,
                        ),
                    );
                let resp = row.add(combo);
                if resp.changed()
                    && let Some(idx) = resp.selected_index()
                {
                    ctx.push_action(InspectorAction::SelectDropdown(
                        ctx.entity,
                        InspectorDropdownId::SurfaceType,
                        idx,
                    ));
                }

                // ↺ Preset button
                let preset_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(62.0)
                    .height(22.0)
                    .background(Color::rgba(0.16, 0.17, 0.20, 0.95))
                    .border(1.0, Color::rgba(0.24, 0.26, 0.30, 0.85))
                    .border_radius(4.0);

                let preset_tag =
                    crate::ui::iris_bridge::inspector::tags::TAG_INSPECTOR_PRESET_RESET;
                let preset_resp = row.container_tagged(
                    "PresetBtn",
                    preset_style,
                    WidgetRole::Button,
                    preset_tag,
                    |btn| {
                        btn.label_styled_passive(
                            "PresetText",
                            "↺ Preset",
                            10.5,
                            btn.theme().text_secondary,
                            TextAlign::Center,
                            Style::new().width(62.0).height(22.0),
                        );
                    },
                );
                if preset_resp.response.clicked() {
                    ctx.push_action(InspectorAction::ResetPhysMatPreset(ctx.entity));
                }
            });

            // Row 2: Friction
            let fric_id = InspectorNumberInputId::PhysMatFriction;
            let resp_fric = card.add(
                NumericDrag::new("Friction:", &mut friction)
                    .tag(encode_inspector_number_input_tag(fric_id))
                    .label_width(85.0)
                    .box_width(44.0)
                    .edit_state(ctx.edit_state(fric_id)),
            );
            if resp_fric.changed() {
                ctx.set_number(fric_id, friction);
            }
            if resp_fric.committed() {
                ctx.commit_number(fric_id);
            }

            // Row 3: Restitution (Bounciness)
            let rest_id = InspectorNumberInputId::PhysMatRestitution;
            let resp_rest = card.add(
                NumericDrag::new("Restitution (Bounciness):", &mut restitution)
                    .tag(encode_inspector_number_input_tag(rest_id))
                    .label_width(155.0)
                    .box_width(44.0)
                    .edit_state(ctx.edit_state(rest_id)),
            );
            if resp_rest.changed() {
                ctx.set_number(rest_id, restitution);
            }
            if resp_rest.committed() {
                ctx.commit_number(rest_id);
            }
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let mat = ae_core::ecs::PhysicsMaterial {
            friction: 0.70,
            restitution: 0.0,
            surface_type: ae_core::ecs::SurfaceType::Default,
        };
        let _ = world.insert_one(entity, mat);
    }
}