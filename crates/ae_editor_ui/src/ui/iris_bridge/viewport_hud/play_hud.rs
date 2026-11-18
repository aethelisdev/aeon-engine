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

use super::types::{TAG_PLAY_CROSSHAIR, TAG_PLAY_GUIDE_BADGE, ViewportHudParams};
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;

/// Builds the Play Mode HUD overlay (reticle canvas, quick control badge, in-game UI, and pause menu).
///
/// Constructed entirely using declarative [`UiScope`] containers and semantic tags.
pub fn build_play_hud(scope: &mut UiScope<'_>, params: &ViewportHudParams<'_>) {
    let is_paused = params
        .world
        .query::<&ae_core::ui::PauseMenuUiTag>()
        .iter()
        .next()
        .is_some();

    if is_paused {
        ae_runtime::hud::build_in_game_pause_menu(scope, params.viewport_rect, "⏹  Exit to Editor");
        return;
    }

    // 1. Render in-game ECS UI elements (health bar, score text, custom HUD) if present
    ae_runtime::hud::build_ingame_ecs_ui(
        scope,
        params.world,
        params.viewport_rect,
        params.cursor_pos,
    );

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
    // Emits centered retained CanvasPrimitives resolved relative to node center on first layout pass.
    if has_action {
        scope.immediate_canvas_with(
            ImmediateCanvasSpec::new(
                "PlayReticleCanvas",
                TAG_PLAY_CROSSHAIR,
                Style::new()
                    .position_absolute()
                    .left((params.viewport_rect.width - 32.0) * 0.5)
                    .top((params.viewport_rect.height - 32.0) * 0.5)
                    .width(32.0)
                    .height(32.0),
            )
            .origin(CanvasOrigin::NodeCenter)
            .interactive(false),
            draw_crosshair_primitives,
        );
    }

    // 3. Bottom-left quick controls badge (Anchored via Style::position_absolute, left(16.0), bottom(16.0))
    let guide_text = if has_action {
        "🔫 Left Click: Shoot   |   🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    } else {
        "🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    };

    let text_w = irisui::text::measure_text_advance(guide_text, 11.0).ceil();
    let pad_h = 12.0;
    let badge_w = text_w + pad_h * 2.0;
    let badge_h = 24.0;
    let badge_margin = 16.0;

    let badge_style = Style::new()
        .position_absolute()
        .left(badge_margin)
        .bottom(badge_margin)
        .flex_row()
        .align_items(AlignItems::Center)
        .padding_insets(Insets::new(0.0, pad_h, 0.0, pad_h))
        .width(badge_w)
        .height(badge_h)
        .background(Color::rgba(0.06, 0.07, 0.10, 0.85))
        .border(1.0, Color::rgba(0.25, 0.30, 0.42, 0.60))
        .border_radius(5.0);

    scope.container_tagged(
        "PlayGuideBadge",
        badge_style,
        WidgetRole::Default,
        TAG_PLAY_GUIDE_BADGE,
        |badge| {
            badge.label_with_width(
                guide_text,
                text_w,
                11.0,
                Color::rgba(0.90, 0.93, 0.98, 1.0),
                TextAlign::Left,
            );
        },
    );
}

/// Populates node-centered crosshair primitives directly inside the reticle [`ImmediateCanvas`].
///
/// Draws 1 center aim point and 4 cardinal crosshair ticks (North, South, West, East)
/// using [`CanvasOrigin::NodeCenter`] so coordinates automatically track the resolved canvas center.
fn draw_crosshair_primitives(canvas: &mut ImmediateCanvas<'_>) {
    canvas.fill_styled_rect(
        Rect::new(-2.0, -2.0, 4.0, 4.0),
        Color::rgba(1.0, 1.0, 1.0, 0.90),
        Color::rgba(0.0, 0.0, 0.0, 0.60),
        1.0,
        2.0,
    );

    [
        Rect::new(-1.0, -12.0, 2.0, 8.0), // North
        Rect::new(-1.0, 4.0, 2.0, 8.0),   // South
        Rect::new(-12.0, -1.0, 8.0, 2.0), // West
        Rect::new(4.0, -1.0, 8.0, 2.0),   // East
    ]
    .into_iter()
    .for_each(|rect| {
        canvas.fill_styled_rect(
            rect,
            Color::rgba(1.0, 1.0, 1.0, 0.85),
            Color::rgba(0.0, 0.0, 0.0, 0.50),
            0.5,
            1.0,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::super::types::{TAG_PLAY_EXIT, TAG_PLAY_RESUME};
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
            selected_entity: None,
            world: &world,
            is_editing: false,
            is_2d: false,
        };

        build_play_hud(&mut UiScope::new(&mut tree, root), &params);

        let mut names = Vec::new();
        collect_node_names(&tree, root, &mut names);

        // Crosshair reticle canvas and guide badge must be present
        assert!(names.iter().any(|n| n == "PlayReticleCanvas"));
        assert!(names.iter().any(|n| n == "PlayGuideBadge"));

        let badge_id = tree
            .find_node_by_tag(TAG_PLAY_GUIDE_BADGE)
            .expect("Play guide badge must have semantic tag");
        let badge_node = tree.get(badge_id).expect("Badge node must exist");
        assert_eq!(badge_node.name.as_deref(), Some("PlayGuideBadge"));
        assert!(
            !badge_node.children.is_empty(),
            "Badge must contain label child node"
        );
        let label_id = badge_node.children[0];
        let label_node = tree.get(label_id).expect("Label node must exist");
        assert!(
            label_node
                .text
                .as_deref()
                .is_some_and(|t| t.contains("WASD: Move")),
            "Badge label must contain quick controls text"
        );

        // Verify hardware SDF crosshair quad compilation from retained CanvasOrigin::NodeCenter primitives
        let mut layout_engine = LayoutEngine::new();
        let _ = layout_engine.compute_layout(&mut tree, Size::new(800.0, 600.0));
        let reticle_id = tree
            .find_node_by_tag(TAG_PLAY_CROSSHAIR)
            .expect("Play reticle canvas node must exist");
        let reticle_node = tree.get(reticle_id).expect("Reticle node must exist");
        assert!(
            !reticle_node.interactive,
            "Reticle must pass pointer clicks through"
        );
        let mut opts = irisui::wgpu_backend::TreeCompilerOptions::new();
        let cmd_list =
            irisui::wgpu_backend::compile_tree_draw_commands(&tree, reticle_id, &mut opts);
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
            selected_entity: None,
            world: &world,
            is_editing: false,
            is_2d: false,
        };

        build_play_hud(&mut UiScope::new(&mut tree, root), &params);

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
        let (_, r_node) = resume_node.expect("resume button should exist");
        assert!(r_node.computed_rect.width > 0.0 && r_node.computed_rect.height > 0.0);

        let exit_node = tree.iter().find(|(_, n)| n.tag == TAG_PLAY_EXIT);
        assert!(
            exit_node.is_some(),
            "TAG_PLAY_EXIT must be assigned to Exit button"
        );
        let (_, e_node) = exit_node.expect("exit button should exist");
        assert!(e_node.computed_rect.width > 0.0 && e_node.computed_rect.height > 0.0);
    }
}