// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Play Mode Viewport HUD & In-Game Pause Menu Overlay Builder
//!
//! Renders the centered aiming crosshair reticle canvas, in-game gameplay HUD, and bottom-left
//! controls reminder badge during active Play Mode. When gameplay is paused, cleanly suppresses
//! the reticle and draws an interactive In-Game Pause Menu modal with Resume and Exit buttons.
//!
//! All positioning is 100% declarative via Taffy absolute layout ([`Style::position_absolute`]).
//! Crosshair reticle rendering is delegated directly to hardware SDF command lists without dummy layout nodes.
//! Button interactions are 100% semantic tag-driven ([`TAG_PLAY_RESUME`], [`TAG_PLAY_EXIT`]).
//!

use super::types::{TAG_PLAY_CROSSHAIR, TAG_PLAY_EXIT, TAG_PLAY_RESUME, ViewportHudParams};
use irisui::prelude::*;
use irisui::wgpu_backend::{DrawCommandList, QuadInstance};

/// Builds the Play Mode HUD overlay (reticle canvas, quick control badge, in-game UI, and pause menu).
///
/// Constructed entirely using declarative [`UiScope`] containers and semantic tags.
pub fn build_play_hud(tree: &mut UiTree, parent_id: WidgetId, params: &ViewportHudParams<'_>) {
    let is_paused = params
        .world
        .query::<&ae_core::ui::PauseMenuUiTag>()
        .iter()
        .next()
        .is_some();

    if is_paused {
        build_in_game_pause_menu(tree, parent_id, params);
        return;
    }

    let mut scope = UiScope::new(tree, parent_id);

    // 1. Render in-game ECS UI elements (health bar, score text, custom HUD) if present
    render_ingame_ecs_ui(&mut scope, params);

    // Check whether the active player character has `CharacterAction` (aiming/shooting ability)
    let has_action = if let Some(sel) = params.selected_entity
        && (params.world.get::<&ae_core::ecs::PlayerTag>(sel).is_ok()
            || params
                .world
                .get::<&ae_core::ecs::CharacterController>(sel)
                .is_ok())
    {
        params
            .world
            .get::<&ae_core::ecs::CharacterAction>(sel)
            .is_ok()
    } else {
        params
            .world
            .query::<(&ae_core::ecs::PlayerTag, &ae_core::ecs::CharacterAction)>()
            .iter()
            .next()
            .is_some()
            || params
                .world
                .query::<(
                    &ae_core::ecs::CharacterController,
                    &ae_core::ecs::CharacterAction,
                )>()
                .iter()
                .next()
                .is_some()
    };

    // 2. Center Crosshair Reticle Canvas (Only rendered when not paused and character has CharacterAction)
    // Emits a single declarative canvas widget without allocating dummy tree nodes for lines/dots.
    if has_action {
        let ch_size = 32.0;
        let ch_style = Style::new()
            .position_absolute()
            .left((params.viewport_rect.width - ch_size) * 0.5)
            .top((params.viewport_rect.height - ch_size) * 0.5)
            .width(ch_size)
            .height(ch_size);

        scope.canvas_named(
            "PlayReticleCanvas",
            ch_style,
            WidgetRole::OscilloscopeCanvas,
            TAG_PLAY_CROSSHAIR,
            None,
        );
    }

    // 3. Bottom-left quick controls badge (Anchored via Style::position_absolute, left(16.0), bottom(16.0))
    let guide_text = if has_action {
        "🔫 Left Click: Shoot   |   🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    } else {
        "🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    };

    let badge_w = if has_action { 420.0 } else { 290.0 };
    let badge_h = 24.0;

    let badge_style = Style::new()
        .position_absolute()
        .left(16.0)
        .bottom(16.0)
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(badge_w)
        .height(badge_h)
        .background(Color::rgba(0.06, 0.07, 0.10, 0.85))
        .border(1.0, Color::rgba(0.25, 0.30, 0.42, 0.60))
        .border_radius(5.0)
        .box_shadow(0.0, 4.0, 12.0, Color::rgba(0.0, 0.0, 0.0, 0.60));

    scope.container_named("PlayGuideBadge", badge_style, |badge| {
        badge.label(
            guide_text,
            11.0,
            Color::rgba(0.90, 0.93, 0.98, 1.0),
            TextAlign::Center,
        );
    });
}

/// Appends hardware SDF quad commands for the center reticle crosshair in Play Mode.
///
/// Draws 1 center aim point and 4 cardinal crosshair ticks (North, South, West, East)
/// directly into the GPU command batch with zero intermediate layout nodes.
pub fn append_crosshair_quads(command_list: &mut DrawCommandList, rect: Rect) {
    let center_x = rect.x + rect.width * 0.5;
    let center_y = rect.y + rect.height * 0.5;
    let dot_size = 4.0;

    // 1. Center Aim Dot
    command_list.push_quad(QuadInstance {
        rect: [
            center_x - dot_size * 0.5,
            center_y - dot_size * 0.5,
            dot_size,
            dot_size,
        ],
        color: [1.0, 1.0, 1.0, 0.90],
        border_color: [0.0, 0.0, 0.0, 0.60],
        border_width: [1.0, 1.0, 1.0, 1.0],
        corner_radii: [
            dot_size * 0.5,
            dot_size * 0.5,
            dot_size * 0.5,
            dot_size * 0.5,
        ],
        ..Default::default()
    });

    // 2. Reticle Crosshairs (North, South, West, East)
    let len = 8.0;
    let gap = 4.0;
    let thick = 2.0;

    let lines = [
        // North
        [center_x - thick * 0.5, center_y - gap - len, thick, len],
        // South
        [center_x - thick * 0.5, center_y + gap, thick, len],
        // West
        [center_x - gap - len, center_y - thick * 0.5, len, thick],
        // East
        [center_x + gap, center_y - thick * 0.5, len, thick],
    ];

    for line_rect in lines {
        command_list.push_quad(QuadInstance {
            rect: line_rect,
            color: [1.0, 1.0, 1.0, 0.85],
            border_color: [0.0, 0.0, 0.0, 0.50],
            border_width: [0.5, 0.5, 0.5, 0.5],
            corner_radii: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        });
    }
}

/// Builds the centered, interactive In-Game Pause Menu overlay when gameplay is paused.
///
/// Constructed entirely using declarative [`UiScope`] modal primitives:
/// [`UiScope::container`], [`UiScope::modal_confirm_button`], and [`UiScope::modal_danger_button`].
/// Button click events are dispatched 100% via semantic tags ([`TAG_PLAY_RESUME`], [`TAG_PLAY_EXIT`]).
fn build_in_game_pause_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
) {
    let card_w = 320.0;
    let card_h = 220.0;

    let mut scope = UiScope::new(tree, parent_id);

    // 1. Full-viewport darkened translucent backdrop
    let backdrop_style = Style::new()
        .position_absolute()
        .left(0.0)
        .top(0.0)
        .width(params.viewport_rect.width)
        .height(params.viewport_rect.height)
        .background(Color::rgba(0.04, 0.05, 0.08, 0.70));

    let card_rect = Rect {
        x: (params.viewport_rect.width - card_w) * 0.5,
        y: (params.viewport_rect.height - card_h) * 0.5,
        width: card_w,
        height: card_h,
    };

    scope.container_named("InGamePauseBackdrop", backdrop_style, |card_scope| {
        build_pause_card(card_scope, card_rect);
    });
}

/// Builds the pause menu modal card widget.
fn build_pause_card(scope: &mut UiScope, card_rect: Rect) {
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
            title.label(
                "GAME PAUSED",
                20.0,
                Color::rgba(1.0, 1.0, 1.0, 1.0),
                TextAlign::Center,
            );
        });

        card.label(
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
            btns.button_named_tagged("InGamePauseResumeButton", "▶  Resume Game", TAG_PLAY_RESUME);
            btns.button_named_tagged("InGamePauseExitButton", "⏹  Exit to Editor", TAG_PLAY_EXIT);
        });
    });
}

/// Renders active in-game ECS UI components (e.g. Health Bars, Score text) into declarative nodes.
fn render_ingame_ecs_ui(scope: &mut UiScope, params: &ViewportHudParams<'_>) {
    let screen_w = params.viewport_rect.width;
    let screen_h = params.viewport_rect.height;
    if screen_w <= 10.0 || screen_h <= 10.0 {
        return;
    }

    let mouse_pos = Some([
        params.cursor_pos.x - params.viewport_rect.x,
        params.cursor_pos.y - params.viewport_rect.y,
    ]);

    let draw_commands = ae_core::ui::UiLayoutResolver::resolve_draw_commands(
        params.world,
        screen_w,
        screen_h,
        mouse_pos,
        false,
    );

    if draw_commands.is_empty() {
        return;
    }

    for cmd in &draw_commands {
        match cmd {
            ae_core::ui::UiDrawCommand::Rect {
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
            ae_core::ui::UiDrawCommand::Text {
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
                    ae_core::ui::UiTextAlignment::Left => pos[0],
                    ae_core::ui::UiTextAlignment::Center => pos[0] - approx_w * 0.5,
                    ae_core::ui::UiTextAlignment::Right => pos[0] - approx_w,
                };
                let text_y = pos[1] - approx_h * 0.5;
                let iris_align = match alignment {
                    ae_core::ui::UiTextAlignment::Left => TextAlign::Left,
                    ae_core::ui::UiTextAlignment::Center => TextAlign::Center,
                    ae_core::ui::UiTextAlignment::Right => TextAlign::Right,
                };
                let style = Style::new()
                    .position_absolute()
                    .left(text_x)
                    .top(text_y)
                    .width(approx_w)
                    .height(approx_h);
                scope.container(style, |tc| {
                    tc.label(
                        text,
                        *font_size,
                        Color::rgba(color[0], color[1], color[2], color[3]),
                        iris_align,
                    );
                });
            }
            ae_core::ui::UiDrawCommand::Image { rect, tint, .. } => {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_editor::gizmo::{GizmoMode, GizmoSpace};
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};
    use hecs::World;

    fn make_test_camera() -> Camera {
        Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }

    fn collect_node_names(tree: &UiTree, current: WidgetId, names: &mut Vec<String>) {
        if let Some(node) = tree.get(current) {
            if let Some(name) = &node.name {
                names.push(name.clone());
            }
            for &child in &node.children {
                collect_node_names(tree, child, names);
            }
        }
    }

    #[test]
    fn test_play_hud_normal_mode_renders_reticle_and_controls() {
        let mut world = World::new();
        let _player = world.spawn((
            ae_core::ecs::PlayerTag,
            ae_core::ecs::CharacterAction::default(),
        ));

        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(100.0, 50.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(500.0, 350.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: false,
            is_2d: false,
        };

        build_play_hud(&mut tree, root, &params);

        let mut names = Vec::new();
        collect_node_names(&tree, root, &mut names);

        // Crosshair reticle canvas and guide badge must be present
        assert!(names.iter().any(|n| n == "PlayReticleCanvas"));
        assert!(names.iter().any(|n| n == "PlayGuideBadge"));

        // Verify hardware SDF crosshair quad batch generation
        let mut cmd_list = DrawCommandList::new();
        append_crosshair_quads(&mut cmd_list, Rect::new(400.0, 300.0, 32.0, 32.0));
        assert_eq!(cmd_list.quads.len(), 5);

        // Pause menu elements must NOT be present
        assert!(!names.iter().any(|n| n == "InGamePauseCard"));
        assert!(!names.iter().any(|n| n == "InGamePauseResumeButton"));
    }

    #[test]
    fn test_play_hud_paused_mode_suppresses_reticle_and_renders_interactive_pause_menu() {
        let mut world = World::new();
        let _player = world.spawn((
            ae_core::ecs::PlayerTag,
            ae_core::ecs::CharacterAction::default(),
        ));
        // Spawn pause state marker
        let _pause_tag = world.spawn((ae_core::ui::PauseMenuUiTag,));

        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(100.0, 50.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(500.0, 350.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: false,
            is_2d: false,
        };

        build_play_hud(&mut tree, root, &params);

        let mut names = Vec::new();
        collect_node_names(&tree, root, &mut names);

        // Reticle and guide badge must be cleanly SUPPRESSED
        assert!(
            !names.iter().any(|n| n == "PlayReticleCanvas"),
            "Reticle canvas must be suppressed during pause"
        );
        assert!(
            !names.iter().any(|n| n == "PlayGuideBadge"),
            "Play guide badge must be suppressed during pause"
        );

        // Pause menu elements MUST be present
        assert!(names.iter().any(|n| n == "InGamePauseBackdrop"));
        assert!(names.iter().any(|n| n == "InGamePauseCard"));
        assert!(names.iter().any(|n| n == "InGamePauseTitle"));
        assert!(names.iter().any(|n| n == "InGamePauseResumeButton"));
        assert!(names.iter().any(|n| n == "InGamePauseExitButton"));

        // Resolve layout and verify semantic tags on Resume and Exit buttons
        let mut scope = UiScope::new(&mut tree, root);
        scope.finish_layout(params.viewport_rect);

        let resume_node = tree.iter().find(|(_, n)| n.tag == TAG_PLAY_RESUME);
        assert!(
            resume_node.is_some(),
            "TAG_PLAY_RESUME must be assigned to Resume button"
        );
        let (_, r_node) = resume_node.unwrap();
        assert!(r_node.computed_rect.width > 0.0 && r_node.computed_rect.height > 0.0);

        let exit_node = tree.iter().find(|(_, n)| n.tag == TAG_PLAY_EXIT);
        assert!(
            exit_node.is_some(),
            "TAG_PLAY_EXIT must be assigned to Exit button"
        );
        let (_, e_node) = exit_node.unwrap();
        assert!(e_node.computed_rect.width > 0.0 && e_node.computed_rect.height > 0.0);
    }
}