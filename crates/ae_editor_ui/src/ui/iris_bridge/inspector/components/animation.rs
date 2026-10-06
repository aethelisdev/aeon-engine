// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Skeletal Animation Inspector Card
//!
//! Provides inspection and playback feedback for AnimationPlayer components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use super::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};

use irisui::prelude::*;

/// Inspector handler for AnimationPlayer component.
pub struct AnimationPlayerHandler;

impl ComponentInspectorHandler for AnimationPlayerHandler {
    fn component_name(&self) -> &'static str {
        "AnimationPlayer"
    }

    fn display_title(&self) -> &'static str {
        "Animation Player"
    }

    fn icon(&self) -> &'static str {
        "🎬"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.95, 0.45, 0.70, 1.0) // Vibrant Rose / Magenta
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Animation
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_animation::AnimationPlayer>(entity).is_ok()
    }

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let (state_text, state_col, clip_title, clip_duration, speed, looping) =
            if let Ok(player) = ctx.world.get::<&ae_animation::AnimationPlayer>(ctx.entity) {
                let (st_txt, st_col) = match player.state {
                    ae_animation::AnimationState::Playing => {
                        ("▶ PLAYING", Color::rgba(0.20, 0.85, 0.40, 1.0))
                    }
                    ae_animation::AnimationState::Paused => {
                        ("⏸ PAUSED", Color::rgba(0.95, 0.75, 0.15, 1.0))
                    }
                    ae_animation::AnimationState::Stopped => {
                        ("⏹ STOPPED", Color::rgba(0.60, 0.62, 0.68, 1.0))
                    }
                };
                let clip_name = player
                    .current_clip
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "No Clip Selected".to_string());
                let duration = player.current_clip.as_ref().map_or(0.0, |c| c.duration);

                (
                    st_txt,
                    st_col,
                    clip_name,
                    duration,
                    player.speed,
                    player.looping,
                )
            } else {
                (
                    "⏹ STOPPED",
                    Color::rgba(0.60, 0.62, 0.68, 1.0),
                    "None".to_string(),
                    0.0,
                    1.0,
                    true,
                )
            };

        let has_skeleton = ctx
            .world
            .get::<&ae_animation::Skeleton>(ctx.entity)
            .map(|s| s.joints.len())
            .ok();

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("AnimationPlayerCard", card_style, |card| {
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

            // Row 1: Status Row (Horizontal: "Status:" + state badge)
            let status_row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(20.0)
                .gap(6.0);

            card.container_named("AnimStatusRow", status_row_style, |row| {
                row.label_styled_passive(
                    "AnimStatusLbl",
                    "Status:",
                    11.0,
                    Color::rgba(0.620, 0.635, 0.678, 1.0),
                    TextAlign::Left,
                    Style::new().width(48.0).height(20.0),
                );
                row.label_styled_passive(
                    "AnimStatusVal",
                    state_text,
                    11.0,
                    state_col,
                    TextAlign::Left,
                    Style::new().flex_grow(1.0).height(20.0),
                );
            });

            // Row 2: Active Clip
            let clip_str = format!("Clip: {}", clip_title);
            card.label_styled_passive(
                "AnimClipLbl",
                &clip_str,
                11.0,
                Color::rgba(0.886, 0.894, 0.918, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 3: Skeleton Info / Static Warning
            let (info_text, info_col) = if let Some(joints) = has_skeleton {
                (
                    format!("🦴 Joints: {} | ⏱ Duration: {:.2}s", joints, clip_duration),
                    Color::rgba(0.38, 0.74, 0.97, 1.0),
                )
            } else {
                (
                    "ℹ Static 3D Mesh (No Armature found)".to_string(),
                    Color::rgba(0.95, 0.75, 0.15, 0.90),
                )
            };
            card.label_styled_passive(
                "AnimSkeletonInfo",
                &info_text,
                10.0,
                info_col,
                TextAlign::Left,
                Style::new().height(20.0),
            );

            // Row 4: Speed & Looping Indicator
            let loop_str = if looping { "Loop: Yes" } else { "Loop: No" };
            let speed_loop_str = format!("Speed: {:.2}x  |  {}", speed, loop_str);
            card.label_styled_passive(
                "AnimSpeedLoop",
                &speed_loop_str,
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_animation::AnimationPlayer::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::registry::ComponentRenderContext;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    #[test]
    fn test_animation_player_card_render_tag() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_animation::AnimationPlayer::default(),));
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

        let handler = AnimationPlayerHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("AnimationPlayer");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "AnimationPlayer card must tag its delete button"
        );
    }
}