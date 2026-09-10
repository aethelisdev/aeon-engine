// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Play Mode Viewport HUD & In-Game Pause Menu Overlay Builder
//!
//! Renders the centered aiming crosshair reticle, in-game gameplay HUD, and bottom-left
//! controls reminder badge during active Play Mode. When gameplay is paused, cleanly suppresses
//! the reticle and draws an interactive In-Game Pause Menu modal with Resume and Exit buttons.
//!

use super::types::{ViewportHudAction, ViewportHudParams, ViewportHudTargets};
use irisui::prelude::*;

/// Builds the Play Mode HUD overlay (reticle, quick control badge, in-game UI, and pause menu).
pub fn build_play_hud(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let is_paused = params
        .world
        .query::<&ae_core::ui::PauseMenuUiTag>()
        .iter()
        .next()
        .is_some();

    if is_paused {
        build_in_game_pause_menu(tree, parent_id, params, targets);
        return;
    }

    // 1. Render in-game ECS UI elements (health bar, score text, custom HUD) if present
    render_ingame_ecs_ui(tree, parent_id, params);

    let center_x = params.viewport_rect.x + params.viewport_rect.width * 0.5;
    let center_y = params.viewport_rect.y + params.viewport_rect.height * 0.5;

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

    // 2. Center Crosshair Reticle (Only rendered when not paused and character has CharacterAction capability)
    if has_action {
        let dot_size = 4.0;
        let dot_rect = Rect::new(
            center_x - dot_size * 0.5,
            center_y - dot_size * 0.5,
            dot_size,
            dot_size,
        );
        let dot_id = tree.create_node();
        if let Some(node) = tree.get_mut(dot_id) {
            node.set_name("PlayReticleCenter");
            node.computed_rect = dot_rect;
            node.style = Style::new()
                .background(Color::rgba(1.0, 1.0, 1.0, 0.90))
                .border(1.0, Color::rgba(0.0, 0.0, 0.0, 0.60))
                .border_radius(dot_size * 0.5);
        }
        let _ = tree.add_child(parent_id, dot_id);

        // Reticle Crosshairs (North, South, East, West)
        let len = 8.0;
        let gap = 4.0;
        let thick = 2.0;

        let crosshair_lines = [
            // North
            Rect::new(center_x - thick * 0.5, center_y - gap - len, thick, len),
            // South
            Rect::new(center_x - thick * 0.5, center_y + gap, thick, len),
            // West
            Rect::new(center_x - gap - len, center_y - thick * 0.5, len, thick),
            // East
            Rect::new(center_x + gap, center_y - thick * 0.5, len, thick),
        ];

        for line_rect in crosshair_lines {
            let line_id = tree.create_node();
            if let Some(node) = tree.get_mut(line_id) {
                node.set_name("PlayReticleLine");
                node.computed_rect = line_rect;
                node.style = Style::new()
                    .background(Color::rgba(1.0, 1.0, 1.0, 0.85))
                    .border(0.5, Color::rgba(0.0, 0.0, 0.0, 0.50))
                    .border_radius(1.0);
            }
            let _ = tree.add_child(parent_id, line_id);
        }
    }

    // 3. Bottom-left quick controls badge
    let guide_text = if has_action {
        "🔫 Left Click: Shoot   |   🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    } else {
        "🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    };

    let badge_w = if has_action { 420.0 } else { 290.0 };
    let badge_h = 24.0;
    let badge_x = params.viewport_rect.x + 16.0;
    let badge_y = params.viewport_rect.y + params.viewport_rect.height - badge_h - 16.0;
    let badge_rect = Rect::new(badge_x, badge_y, badge_w, badge_h);

    let badge_id = tree.create_node();
    if let Some(node) = tree.get_mut(badge_id) {
        node.set_name("PlayGuideBadge");
        node.computed_rect = badge_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.10, 0.85))
            .border(1.0, Color::rgba(0.25, 0.30, 0.42, 0.60))
            .border_radius(5.0)
            .box_shadow(0.0, 4.0, 12.0, Color::rgba(0.0, 0.0, 0.0, 0.60));
    }
    let _ = tree.add_child(parent_id, badge_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name("PlayGuideText");
        node.set_text(guide_text);
        node.font_size = 11.0;
        node.line_height = badge_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.90, 0.93, 0.98, 1.0);
        node.computed_rect = badge_rect;
    }
    let _ = tree.add_child(badge_id, txt_id);
}

/// Builds the centered, interactive In-Game Pause Menu overlay when gameplay is paused.
fn build_in_game_pause_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let center_x = params.viewport_rect.x + params.viewport_rect.width * 0.5;
    let center_y = params.viewport_rect.y + params.viewport_rect.height * 0.5;

    // 1. Full-viewport darkened translucent backdrop
    let backdrop_id = tree.create_node();
    if let Some(node) = tree.get_mut(backdrop_id) {
        node.set_name("InGamePauseBackdrop");
        node.computed_rect = params.viewport_rect;
        node.style = Style::new().background(Color::rgba(0.04, 0.05, 0.08, 0.70));
    }
    let _ = tree.add_child(parent_id, backdrop_id);

    // 2. Centered Pause Modal Card
    let card_w = 320.0;
    let card_h = 220.0;
    let card_x = center_x - card_w * 0.5;
    let card_y = center_y - card_h * 0.5;
    let card_rect = Rect::new(card_x, card_y, card_w, card_h);

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("InGamePauseCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.09, 0.11, 0.15, 0.96))
            .border(1.5, Color::rgba(0.0, 0.80, 1.0, 0.60))
            .border_radius(8.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.80));
    }
    let _ = tree.add_child(parent_id, card_id);

    // Title: "GAME PAUSED"
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("InGamePauseTitle");
        node.computed_rect = Rect::new(card_x, card_y + 24.0, card_w, 28.0);
        node.set_text("GAME PAUSED");
        node.set_text_properties(
            20.0,
            28.0,
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            TextAlign::Center,
        );
    }
    let _ = tree.add_child(card_id, title_id);

    // Subtitle: "Press P or choose an option below"
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("InGamePauseSubtitle");
        node.computed_rect = Rect::new(card_x, card_y + 54.0, card_w, 18.0);
        node.set_text("Press P or choose an option below");
        node.set_text_properties(
            11.0,
            18.0,
            Color::rgba(0.60, 0.68, 0.78, 1.0),
            TextAlign::Center,
        );
    }
    let _ = tree.add_child(card_id, sub_id);

    let btn_w = 230.0;
    let btn_h = 38.0;
    let btn_x = center_x - btn_w * 0.5;

    // 3. Resume Button
    let resume_y = card_y + 88.0;
    let resume_rect = Rect::new(btn_x, resume_y, btn_w, btn_h);
    let resume_hovered = resume_rect.contains_point(params.cursor_pos);

    let resume_id = tree.create_node();
    if let Some(node) = tree.get_mut(resume_id) {
        node.set_name("InGamePauseResumeButton");
        node.computed_rect = resume_rect;
        let mut style = Style::new();
        if resume_hovered {
            style = style
                .background(Color::rgba(0.0, 0.55, 0.78, 1.0))
                .border(1.5, Color::rgba(0.0, 0.90, 1.0, 1.0))
                .border_radius(5.0)
                .box_shadow(0.0, 2.0, 10.0, Color::rgba(0.0, 0.80, 1.0, 0.45));
        } else {
            style = style
                .background(Color::rgba(0.12, 0.16, 0.23, 1.0))
                .border(1.0, Color::rgba(0.0, 0.70, 0.90, 0.50))
                .border_radius(5.0);
        }
        node.style = style;
    }
    let _ = tree.add_child(card_id, resume_id);

    let resume_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(resume_txt_id) {
        node.set_name("InGamePauseResumeText");
        node.computed_rect = resume_rect;
        node.set_text("▶  Resume Game");
        node.set_text_properties(
            13.0,
            btn_h,
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            TextAlign::Center,
        );
    }
    let _ = tree.add_child(resume_id, resume_txt_id);
    targets
        .buttons
        .push((ViewportHudAction::ResumeGame, resume_rect));

    // 4. Exit to Editor Button
    let exit_y = resume_y + btn_h + 14.0;
    let exit_rect = Rect::new(btn_x, exit_y, btn_w, btn_h);
    let exit_hovered = exit_rect.contains_point(params.cursor_pos);

    let exit_id = tree.create_node();
    if let Some(node) = tree.get_mut(exit_id) {
        node.set_name("InGamePauseExitButton");
        node.computed_rect = exit_rect;
        let mut style = Style::new();
        if exit_hovered {
            style = style
                .background(Color::rgba(0.75, 0.18, 0.18, 1.0))
                .border(1.5, Color::rgba(0.95, 0.30, 0.30, 1.0))
                .border_radius(5.0)
                .box_shadow(0.0, 2.0, 10.0, Color::rgba(0.90, 0.20, 0.20, 0.45));
        } else {
            style = style
                .background(Color::rgba(0.15, 0.16, 0.20, 1.0))
                .border(1.0, Color::rgba(0.80, 0.30, 0.30, 0.50))
                .border_radius(5.0);
        }
        node.style = style;
    }
    let _ = tree.add_child(card_id, exit_id);

    let exit_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(exit_txt_id) {
        node.set_name("InGamePauseExitText");
        node.computed_rect = exit_rect;
        node.set_text("⏹  Exit to Editor");
        node.set_text_properties(
            13.0,
            btn_h,
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            TextAlign::Center,
        );
    }
    let _ = tree.add_child(exit_id, exit_txt_id);
    targets
        .buttons
        .push((ViewportHudAction::ExitToEditor, exit_rect));
}

/// Renders active in-game ECS UI components (e.g. Health Bars, Score text) into Iris UI nodes.
fn render_ingame_ecs_ui(tree: &mut UiTree, parent_id: WidgetId, params: &ViewportHudParams<'_>) {
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

    for (idx, cmd) in draw_commands.iter().enumerate() {
        match cmd {
            ae_core::ui::UiDrawCommand::Rect {
                rect,
                fill_color,
                border_color,
                border_width,
                border_radius,
                ..
            } => {
                let draw_rect = Rect::new(
                    params.viewport_rect.x + rect.min_x,
                    params.viewport_rect.y + rect.min_y,
                    rect.width(),
                    rect.height(),
                );
                let rect_id = tree.create_node();
                if let Some(node) = tree.get_mut(rect_id) {
                    node.set_name(format!("InGameUiRect_{}", idx));
                    node.computed_rect = draw_rect;
                    let mut style = Style::new().background(Color::rgba(
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
                    node.style = style;
                }
                let _ = tree.add_child(parent_id, rect_id);
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
                let (text_x, iris_align) = match alignment {
                    ae_core::ui::UiTextAlignment::Left => {
                        (params.viewport_rect.x + pos[0], TextAlign::Left)
                    }
                    ae_core::ui::UiTextAlignment::Center => (
                        params.viewport_rect.x + pos[0] - approx_w * 0.5,
                        TextAlign::Center,
                    ),
                    ae_core::ui::UiTextAlignment::Right => {
                        (params.viewport_rect.x + pos[0] - approx_w, TextAlign::Right)
                    }
                };
                let text_y = params.viewport_rect.y + pos[1] - approx_h * 0.5;
                let text_id = tree.create_node();
                if let Some(node) = tree.get_mut(text_id) {
                    node.set_name(format!("InGameUiText_{}", idx));
                    node.computed_rect = Rect::new(text_x, text_y, approx_w, approx_h);
                    node.set_text(text.clone());
                    node.font_size = *font_size;
                    node.line_height = approx_h;
                    node.text_align = iris_align;
                    node.text_color = Color::rgba(color[0], color[1], color[2], color[3]);
                }
                let _ = tree.add_child(parent_id, text_id);
            }
            ae_core::ui::UiDrawCommand::Image { rect, tint, .. } => {
                let img_id = tree.create_node();
                if let Some(node) = tree.get_mut(img_id) {
                    node.set_name(format!("InGameUiImage_{}", idx));
                    node.computed_rect = Rect::new(
                        params.viewport_rect.x + rect.min_x,
                        params.viewport_rect.y + rect.min_y,
                        rect.width(),
                        rect.height(),
                    );
                    node.style =
                        Style::new().background(Color::rgba(tint[0], tint[1], tint[2], tint[3]));
                }
                let _ = tree.add_child(parent_id, img_id);
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
        let root = tree.create_node();

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

        let mut targets = ViewportHudTargets::default();
        build_play_hud(&mut tree, root, &params, &mut targets);

        let mut names = Vec::new();
        collect_node_names(&tree, root, &mut names);

        // Crosshair reticle and guide badge must be present
        assert!(names.iter().any(|n| n == "PlayReticleCenter"));
        assert!(names.iter().any(|n| n == "PlayReticleLine"));
        assert!(names.iter().any(|n| n == "PlayGuideBadge"));

        // Pause menu elements must NOT be present
        assert!(!names.iter().any(|n| n == "InGamePauseCard"));
        assert!(!names.iter().any(|n| n == "InGamePauseResumeButton"));
        assert!(targets.buttons.is_empty());
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
        let root = tree.create_node();

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

        let mut targets = ViewportHudTargets::default();
        build_play_hud(&mut tree, root, &params, &mut targets);

        let mut names = Vec::new();
        collect_node_names(&tree, root, &mut names);

        // Reticle and guide badge must be cleanly SUPPRESSED
        assert!(
            !names.iter().any(|n| n == "PlayReticleCenter"),
            "Reticle center must be suppressed during pause"
        );
        assert!(
            !names.iter().any(|n| n == "PlayReticleLine"),
            "Reticle lines must be suppressed during pause"
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

        // Interactive button hit targets must be registered
        assert_eq!(targets.buttons.len(), 2);
        assert_eq!(targets.buttons[0].0, ViewportHudAction::ResumeGame);
        assert_eq!(targets.buttons[1].0, ViewportHudAction::ExitToEditor);

        // Test click hit detection on Resume button
        let resume_rect = targets.buttons[0].1;
        let resume_center = Point::new(
            resume_rect.x + resume_rect.width * 0.5,
            resume_rect.y + resume_rect.height * 0.5,
        );
        assert!(resume_rect.contains_point(resume_center));

        // Test click hit detection on Exit button
        let exit_rect = targets.buttons[1].1;
        let exit_center = Point::new(
            exit_rect.x + exit_rect.width * 0.5,
            exit_rect.y + exit_rect.height * 0.5,
        );
        assert!(exit_rect.contains_point(exit_center));
    }
}