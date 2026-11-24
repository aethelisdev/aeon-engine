// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Declarative Iris UI builders translating gameplay ECS UI commands and pause overlays into
//! `UiScope` trees shared between the editor Play viewport and standalone runtime player.

use crate::layout::{UiDrawCommand, UiLayoutResolver};
use ae_plugin_api::UiTextAlignment;
use irisui_core::{AlignItems, Color, Insets, JustifyContent, Point, Rect, Style, TextAlign};
use irisui_widgets::{Button, UiScope};

/// Semantic tag assigned to the In-Game Pause Menu Resume button.
pub const TAG_PLAY_RESUME: u64 = 2040;

/// Semantic tag assigned to the In-Game Pause Menu Exit button.
pub const TAG_PLAY_EXIT: u64 = 2041;

/// Builds active in-game ECS UI components (health bars, score labels, panels) into declarative
/// [`UiScope`] nodes.
pub fn build_ingame_ecs_ui(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    viewport_rect: Rect,
    cursor_pos: Point,
) {
    let screen_w = viewport_rect.width;
    let screen_h = viewport_rect.height;
    if !screen_w.is_finite() || !screen_h.is_finite() || screen_w <= 10.0 || screen_h <= 10.0 {
        return;
    }

    let mouse_pos = Some([
        cursor_pos.x - viewport_rect.x,
        cursor_pos.y - viewport_rect.y,
    ]);
    let draw_commands =
        UiLayoutResolver::resolve_draw_commands(world, screen_w, screen_h, mouse_pos, false);

    for cmd in &draw_commands {
        emit_iris_draw_command(scope, cmd);
    }
}

fn emit_iris_draw_command(scope: &mut UiScope<'_>, cmd: &UiDrawCommand) {
    match cmd {
        UiDrawCommand::Rect {
            rect,
            fill_color,
            border_color,
            border_width,
            border_radius,
            ..
        } => {
            let mut style = Style::new()
                .position_absolute()
                .left(rect.min_x)
                .top(rect.min_y)
                .width(rect.width())
                .height(rect.height())
                .background(Color::rgba(
                    fill_color[0],
                    fill_color[1],
                    fill_color[2],
                    fill_color[3],
                ));
            if border_color[3] > 0.01 && *border_width > 0.0 {
                style = style.border(
                    *border_width,
                    Color::rgba(
                        border_color[0],
                        border_color[1],
                        border_color[2],
                        border_color[3],
                    ),
                );
            }
            if *border_radius > 0.0 {
                style = style.border_radius(*border_radius);
            }
            scope.empty_box(style);
        }
        UiDrawCommand::Text {
            pos,
            text,
            font_size,
            color,
            alignment,
            ..
        } => {
            let approx_w = (text.len() as f32) * (*font_size * 0.65);
            let approx_h = *font_size * 1.3;
            let text_x = match alignment {
                UiTextAlignment::Left => pos[0],
                UiTextAlignment::Center => pos[0] - approx_w * 0.5,
                UiTextAlignment::Right => pos[0] - approx_w,
            };
            let text_y = pos[1] - approx_h * 0.5;
            let iris_align = match alignment {
                UiTextAlignment::Left => TextAlign::Left,
                UiTextAlignment::Center => TextAlign::Center,
                UiTextAlignment::Right => TextAlign::Right,
            };
            let style = Style::new()
                .position_absolute()
                .left(text_x)
                .top(text_y)
                .width(approx_w)
                .height(approx_h);
            scope.container(style, |tc| {
                tc.label_custom(
                    text,
                    *font_size,
                    Color::rgba(color[0], color[1], color[2], color[3]),
                    iris_align,
                );
            });
        }
        UiDrawCommand::Image { rect, tint, .. } => {
            let style = Style::new()
                .position_absolute()
                .left(rect.min_x)
                .top(rect.min_y)
                .width(rect.width())
                .height(rect.height())
                .background(Color::rgba(tint[0], tint[1], tint[2], tint[3]));
            scope.empty_box(style);
        }
    }
}

/// Builds the centered, interactive In-Game Pause Menu overlay when gameplay is paused.
///
/// Button click events are dispatched via semantic tags ([`TAG_PLAY_RESUME`], [`TAG_PLAY_EXIT`]).
pub fn build_in_game_pause_menu(scope: &mut UiScope<'_>, viewport_rect: Rect, exit_label: &str) {
    let card_w = 320.0;
    let card_h = 220.0;

    let backdrop_style = Style::new()
        .position_absolute()
        .left(0.0)
        .top(0.0)
        .width(viewport_rect.width)
        .height(viewport_rect.height)
        .background(Color::rgba(0.04, 0.05, 0.08, 0.70));

    let card_rect = Rect {
        x: (viewport_rect.width - card_w) * 0.5,
        y: (viewport_rect.height - card_h) * 0.5,
        width: card_w,
        height: card_h,
    };

    scope.container_named("InGamePauseBackdrop", backdrop_style, |card_scope| {
        build_pause_card(card_scope, card_rect, exit_label);
    });
}

fn build_pause_card(scope: &mut UiScope<'_>, card_rect: Rect, exit_label: &str) {
    let card_style = Style::new()
        .position_absolute()
        .left(card_rect.x)
        .top(card_rect.y)
        .flex_col()
        .align_items(AlignItems::Center)
        .padding_insets(Insets::new(24.0, 20.0, 20.0, 20.0))
        .gap(10.0)
        .width(card_rect.width)
        .height(card_rect.height)
        .background(Color::rgba(0.09, 0.11, 0.15, 0.96))
        .border(1.5, Color::rgba(0.0, 0.80, 1.0, 0.60))
        .border_radius(8.0)
        .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.80));

    scope.container_named("InGamePauseCard", card_style, |card| {
        card.container_named("InGamePauseTitle", Style::new(), |title| {
            title.label_custom(
                "GAME PAUSED",
                20.0,
                Color::rgba(1.0, 1.0, 1.0, 1.0),
                TextAlign::Center,
            );
        });

        card.label_custom(
            "Press P or choose an option below",
            11.0,
            Color::rgba(0.60, 0.68, 0.78, 1.0),
            TextAlign::Center,
        );

        let btn_group_style = Style::new()
            .flex_col()
            .align_items(AlignItems::Center)
            .gap(12.0)
            .width(230.0);

        card.container(btn_group_style, |btns| {
            let btn_style = Style::new()
                .width(180.0)
                .height(26.0)
                .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
                .background(Color::rgba(0.0, 0.30, 0.42, 0.60))
                .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.60))
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            btns.add(
                Button::new("Resume Game")
                    .name("InGamePauseResumeButton")
                    .style(btn_style)
                    .tag(TAG_PLAY_RESUME),
            );
            btns.add(
                Button::new(exit_label)
                    .name("InGamePauseExitButton")
                    .style(btn_style)
                    .tag(TAG_PLAY_EXIT),
            );
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::{UiAnchor, UiElement, UiPanel, UiText, UiTextAlignment};
    use irisui_core::{UiTree, WidgetId};

    fn collect_text_values(tree: &UiTree, current: WidgetId, out: &mut Vec<String>) {
        if let Some(node) = tree.get(current) {
            if let Some(t) = &node.text {
                out.push(t.clone());
            }
            for &child in &node.children {
                collect_text_values(tree, child, out);
            }
        }
    }

    #[test]
    fn test_shared_ingame_ecs_ui_and_pause_menu_build_nodes() {
        let mut world = ae_ecs::World::new();
        world.spawn((
            UiElement::new(UiAnchor::TopLeft, [16.0, 16.0], [200.0, 48.0]),
            UiPanel::default(),
            UiText {
                text: "HP: 100 | Score: 250".to_string(),
                font_size: 16.0,
                color: [1.0, 1.0, 1.0, 1.0],
                alignment: UiTextAlignment::Left,
                ..Default::default()
            },
        ));

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let vp = Rect::new(0.0, 0.0, 1280.0, 720.0);
        {
            let mut scope = UiScope::new(&mut tree, root);
            build_ingame_ecs_ui(&mut scope, &world, vp, Point::new(0.0, 0.0));
            build_in_game_pause_menu(&mut scope, vp, "Exit Game");
        }

        let mut texts = Vec::new();
        collect_text_values(&tree, root, &mut texts);
        assert!(texts.iter().any(|t| t == "HP: 100 | Score: 250"));
        assert!(texts.iter().any(|t| t == "GAME PAUSED"));
        assert!(tree.find_node_by_tag(TAG_PLAY_RESUME).is_some());
        assert!(tree.find_node_by_tag(TAG_PLAY_EXIT).is_some());
    }
}