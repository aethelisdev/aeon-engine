// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Material Component Inspector Card
//!
//! Provides the `PhysicsMaterialHandler` implementing UI rendering, surface type
//! combobox with preset action button, and friction / restitution numeric controls.

use super::helpers::{
    ComponentHeaderProps, DeclarativeNumericRowParams, build_declarative_card_header,
    render_declarative_numeric_row,
};
use crate::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use crate::ui::iris_bridge::inspector::types::{
    ComponentCategory, InspectorDropdownId, InspectorNumberInputId,
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
        let mat_data = ctx
            .world
            .get::<&ae_core::ecs::PhysicsMaterial>(ctx.entity)
            .map(|m| (m.friction, m.restitution, m.surface_type.display_name()))
            .unwrap_or((0.70, 0.0, "Default"));

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

        scope.container_named("PhysicsMaterialCard", card_style, |card| {
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
                row.label_styled_passive(
                    "SurfaceTypeLbl",
                    "Surface Type:",
                    11.0,
                    Color::rgba(0.620, 0.635, 0.678, 1.0),
                    TextAlign::Left,
                    Style::new().width(85.0).height(22.0),
                );

                let is_open = ctx.params.active_dropdown == Some(InspectorDropdownId::SurfaceType);
                let pill_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(80.0)
                    .height(22.0)
                    .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
                    .background(if is_open {
                        Color::rgba(0.118, 0.125, 0.145, 1.0)
                    } else {
                        Color::rgba(0.157, 0.165, 0.188, 0.98)
                    })
                    .border(
                        1.0,
                        if is_open {
                            Color::rgba(0.353, 0.376, 0.439, 0.95)
                        } else {
                            Color::rgba(0.212, 0.220, 0.259, 0.85)
                        },
                    )
                    .border_radius(5.0);

                let dropdown_tag =
                    crate::ui::iris_bridge::inspector::tags::encode_inspector_dropdown_tag(
                        InspectorDropdownId::SurfaceType,
                    );
                row.container_tagged(
                    "ComboPill",
                    pill_style,
                    WidgetRole::Button,
                    dropdown_tag,
                    |pill| {
                        pill.label_styled_passive(
                            "ComboText",
                            mat_data.2,
                            10.5,
                            if is_open {
                                Color::WHITE
                            } else {
                                Color::rgba(0.886, 0.894, 0.918, 1.0)
                            },
                            TextAlign::Left,
                            Style::new().flex_grow(1.0),
                        );

                        let chevron_uv = if is_open {
                            crate::ui::iris_bridge::icons::ICON_CHEVRON_UP
                        } else {
                            crate::ui::iris_bridge::icons::ICON_CHEVRON_DOWN
                        };
                        let chevron_color = if is_open {
                            Color::WHITE
                        } else {
                            Color::rgba(0.70, 0.72, 0.78, 0.9)
                        };
                        pill.icon_named("ComboChevron", chevron_uv, chevron_color, 9.0);
                    },
                );

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
                row.container_tagged(
                    "PresetBtn",
                    preset_style,
                    WidgetRole::Button,
                    preset_tag,
                    |btn| {
                        btn.label_styled_passive(
                            "PresetText",
                            "↺ Preset",
                            10.5,
                            Color::rgba(0.886, 0.894, 0.918, 1.0),
                            TextAlign::Center,
                            Style::new().width(62.0).height(22.0),
                        );
                    },
                );
            });

            // Row 2: Friction
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::PhysMatFriction,
                    label: "Friction:",
                    val: mat_data.0,
                    label_w: 85.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::PhysMatFriction),
                    is_hovered: false,
                },
            );

            // Row 3: Restitution (Bounciness)
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::PhysMatRestitution,
                    label: "Restitution (Bounciness):",
                    val: mat_data.1,
                    label_w: 155.0,
                    box_w: 44.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::PhysMatRestitution),
                    is_hovered: false,
                },
            );
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