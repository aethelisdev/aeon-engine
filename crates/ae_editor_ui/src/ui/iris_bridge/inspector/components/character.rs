// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Character and Player Component Inspector Cards
//!
//! Provides handlers for `🚶 Kinematic Character Controller` and `🎮 PlayerTag`.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::tags::encode_inspector_number_input_tag;
use super::super::types::{ComponentCategory, InspectorNumberInputId};
use super::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};
use irisui::prelude::*;

/// Inspector handler for `🚶 Kinematic Character Controller`.
pub struct CharacterControllerHandler;

impl ComponentInspectorHandler for CharacterControllerHandler {
    fn component_name(&self) -> &'static str {
        "CharacterController"
    }

    fn display_title(&self) -> &'static str {
        "Kinematic Character Controller"
    }

    fn icon(&self) -> &'static str {
        "🚶"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.30, 0.65, 1.0) // Vibrant Pink / Magenta (#ec4899)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world
            .get::<&ae_core::ecs::CharacterController>(entity)
            .is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (mut height, mut radius, mut center_y, mut max_slope, mut step_height, is_grounded) =
            ctx.world
                .get::<&ae_core::ecs::CharacterController>(ctx.entity)
                .map(|c| {
                    (
                        c.height,
                        c.radius,
                        c.center_y,
                        c.max_slope_climb_angle,
                        c.step_height,
                        c.is_grounded,
                    )
                })
                .unwrap_or((1.80, 0.40, 0.0, 45.0, 0.30, false));

        scope.card_panel("CharacterControllerCard", |card| {
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

            let mut rows = [
                (
                    InspectorNumberInputId::CharacterHeight,
                    "Height:",
                    &mut height,
                    false,
                ),
                (
                    InspectorNumberInputId::CharacterRadius,
                    "Radius:",
                    &mut radius,
                    false,
                ),
                (
                    InspectorNumberInputId::CharacterCenterY,
                    "Center Y:",
                    &mut center_y,
                    false,
                ),
                (
                    InspectorNumberInputId::CharacterMaxSlope,
                    "Max Slope Angle:",
                    &mut max_slope,
                    true,
                ),
                (
                    InspectorNumberInputId::CharacterStepHeight,
                    "Step Height:",
                    &mut step_height,
                    false,
                ),
            ];

            for (id, lbl, val, is_angle) in &mut rows {
                let tag = encode_inspector_number_input_tag(*id);
                let edit = ctx.edit_state(*id);
                let current_val = **val;
                let mut drag = NumericDrag::new(lbl, val)
                    .tag(tag)
                    .label_width(110.0)
                    .box_width(44.0)
                    .edit_state(edit);
                let formatted;
                if *is_angle {
                    formatted = format!("{:.0}°", current_val);
                    drag = drag.custom_text(&formatted);
                }
                if card.add(drag).changed() {
                    ctx.set_number(*id, **val);
                }
            }

            // Grounded Status Text (e.g. 🛡 In Air / 🛡 Grounded)
            let status_row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(18.0);

            card.container_named("GroundedStatusPill", status_row_style, |status_scope| {
                let (text_str, text_col) = if is_grounded {
                    ("🛡 Grounded", status_scope.theme().success)
                } else {
                    ("🛡 In Air", status_scope.theme().warning)
                };
                status_scope.label_styled_passive(
                    "GroundedStatusText",
                    text_str,
                    10.5,
                    text_col,
                    TextAlign::Left,
                    Style::new().height(18.0),
                );
            });
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let controller = ae_core::ecs::CharacterController {
            height: 1.80,
            radius: 0.40,
            center_y: 0.0,
            max_slope_climb_angle: 45.0,
            step_height: 0.30,
            is_grounded: true,
        };
        let _ = world.insert_one(entity, controller);
    }
}

/// Inspector handler for `🎮 PlayerTag`.
pub struct PlayerTagHandler;

impl ComponentInspectorHandler for PlayerTagHandler {
    fn component_name(&self) -> &'static str {
        "PlayerTag"
    }

    fn display_title(&self) -> &'static str {
        "PlayerTag"
    }

    fn icon(&self) -> &'static str {
        "🎮"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.96, 0.62, 0.15, 1.0) // Amber / Orange (#f59e0b)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Gameplay
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_core::ecs::PlayerTag>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        scope.card_panel("PlayerTagCard", |card| {
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

            card.label_styled_passive(
                "PlayerTagDesc",
                "Designates this entity as the active Player target",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new().margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
            );
            card.label_styled_passive(
                "PlayerTagDescSub",
                "for gameplay logic and camera tracking.",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new(),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_core::ecs::PlayerTag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_player_tag_declarative_render() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::PlayerTag,));
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

        let handler = PlayerTagHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("PlayerTag");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "PlayerTag card must tag its delete button with semantic delete tag"
        );
    }
}