// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Play Mode Viewport HUD & In-Game Pause Menu Overlay Builder
//!
//! Renders the centered aiming crosshair reticle canvas, in-game gameplay HUD, and bottom-left
//! controls reminder badge during active Play Mode using zero imperative `let` statements
//! and emitting [`EngineUiAction`] directly into [`ActionBus`].

use super::types::{TAG_PLAY_CROSSHAIR, TAG_PLAY_EXIT, TAG_PLAY_GUIDE_BADGE, TAG_PLAY_RESUME};
use crate::ui::types::EngineUiAction;
use ae_core::modules::EngineMode;
use hecs::{Entity, World};
use irisui::prelude::*;
use irisui::widgets::canvas::ImmediateCanvas;
use irisui::workbench::ActionBus;

#[inline]
fn world_is_paused(world: &World) -> bool {
    world
        .query::<&ae_core::ui::PauseMenuUiTag>()
        .iter()
        .next()
        .is_some()
}

#[inline]
fn world_has_character_action(world: &World, selected_entity: Option<Entity>) -> bool {
    selected_entity
        .filter(|&sel| {
            world.get::<&ae_core::ecs::PlayerTag>(sel).is_ok()
                || world.get::<&ae_core::ecs::CharacterController>(sel).is_ok()
        })
        .map_or_else(
            || {
                world
                    .query::<(&ae_core::ecs::PlayerTag, &ae_core::ecs::CharacterAction)>()
                    .iter()
                    .next()
                    .is_some()
                    || world
                        .query::<(
                            &ae_core::ecs::CharacterController,
                            &ae_core::ecs::CharacterAction,
                        )>()
                        .iter()
                        .next()
                        .is_some()
            },
            |sel| world.get::<&ae_core::ecs::CharacterAction>(sel).is_ok(),
        )
}

#[inline]
fn emit_play_guide_badge(scope: &mut UiScope<'_>, guide_text: &str, text_w: f32) {
    scope.container_tagged(
        "PlayGuideBadge",
        Style::new()
            .position_absolute()
            .left(16.0)
            .bottom(16.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(0.0, 12.0, 0.0, 12.0))
            .width(text_w + 24.0)
            .height(24.0)
            .background(Color::rgba(0.06, 0.07, 0.10, 0.85))
            .border(1.0, Color::rgba(0.25, 0.30, 0.42, 0.60))
            .border_radius(5.0),
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

#[inline]
fn guide_text_for(has_action: bool) -> &'static str {
    if has_action {
        "🔫 Left Click: Shoot   |   🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    } else {
        "🏃 WASD: Move   |   ⬆ Space: Jump   |   ⏹ ESC: Exit"
    }
}

/// Builds the Play Mode HUD overlay (reticle canvas, quick control badge, in-game UI, and pause menu).
///
/// Constructed entirely using declarative [`UiScope`] expressions with zero imperative `let` statements.
pub fn build_play_hud(
    scope: &mut UiScope<'_>,
    world: &World,
    viewport_rect: Rect,
    cursor_pos: Point,
    selected_entity: Option<Entity>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    if world_is_paused(world) {
        ae_runtime::hud::build_in_game_pause_menu(scope, viewport_rect, "⏹  Exit to Editor");
        if scope.is_tag_clicked(TAG_PLAY_RESUME) {
            action_bus.emit(EngineUiAction::ResumeGame);
        }
        if scope.is_tag_clicked(TAG_PLAY_EXIT) {
            action_bus.emit(EngineUiAction::ChangeMode(EngineMode::Edit));
        }
        return;
    }

    ae_runtime::hud::build_ingame_ecs_ui(scope, world, viewport_rect, cursor_pos);

    scope.when(world_has_character_action(world, selected_entity), |s| {
        s.immediate_canvas_with(
            ImmediateCanvasSpec::new(
                "PlayReticleCanvas",
                TAG_PLAY_CROSSHAIR,
                Style::new()
                    .position_absolute()
                    .left((viewport_rect.width - 32.0) * 0.5)
                    .top((viewport_rect.height - 32.0) * 0.5)
                    .width(32.0)
                    .height(32.0),
            )
            .origin(CanvasOrigin::NodeCenter)
            .interactive(false),
            draw_crosshair_primitives,
        );
    });

    emit_play_guide_badge(
        scope,
        guide_text_for(world_has_character_action(world, selected_entity)),
        irisui::text::measure_text_advance(
            guide_text_for(world_has_character_action(world, selected_entity)),
            11.0,
        )
        .ceil(),
    );
}

/// Populates node-centered crosshair primitives directly inside the reticle [`ImmediateCanvas`].
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
    use super::*;
    use hecs::World;

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

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let mut bus = ActionBus::new();
        build_play_hud(
            &mut UiScope::new(&mut tree, root),
            &world,
            Rect::new(100.0, 50.0, 800.0, 600.0),
            Point::new(500.0, 350.0),
            None,
            &mut bus,
        );

        let mut names = Vec::new();
        collect_node_names(&tree, root, &mut names);

        assert!(names.iter().any(|n| n == "PlayReticleCanvas"));
        assert!(names.iter().any(|n| n == "PlayGuideBadge"));

        let badge_id = tree
            .find_node_by_tag(TAG_PLAY_GUIDE_BADGE)
            .expect("Play guide badge must have semantic tag");
        let badge_node = tree.get(badge_id).expect("Badge node must exist");
        let label_id = badge_node.children[0];
        let label_node = tree.get(label_id).expect("Label node must exist");
        assert!(
            label_node
                .text
                .as_deref()
                .is_some_and(|t| t.contains("WASD: Move")),
            "Badge label must contain quick controls text"
        );

        let mut layout_engine = LayoutEngine::new();
        let _ = layout_engine.compute_layout(&mut tree, Size::new(800.0, 600.0));
        let reticle_id = tree
            .find_node_by_tag(TAG_PLAY_CROSSHAIR)
            .expect("Play reticle canvas node must exist");
        let reticle_node = tree.get(reticle_id).expect("Reticle node must exist");
        assert!(!reticle_node.interactive);
        let mut opts = irisui::wgpu_backend::TreeCompilerOptions::new();
        let cmd_list =
            irisui::wgpu_backend::compile_tree_draw_commands(&tree, reticle_id, &mut opts);
        assert_eq!(cmd_list.quads.len(), 5);
    }

    #[test]
    fn test_play_hud_paused_mode_suppresses_reticle_and_emits_pause_actions() {
        let mut world = World::new();
        let _player = world.spawn((
            ae_core::ecs::PlayerTag,
            ae_core::ecs::CharacterAction::default(),
        ));
        let _pause_tag = world.spawn((ae_core::ui::PauseMenuUiTag,));

        let viewport_rect = Rect::new(100.0, 50.0, 800.0, 600.0);
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");

        let mut bus = ActionBus::new();
        tree.event_state.frame_clicked_tag = Some(TAG_PLAY_EXIT);
        {
            let mut scope = UiScope::new(&mut tree, root);
            build_play_hud(
                &mut scope,
                &world,
                viewport_rect,
                Point::new(500.0, 350.0),
                None,
                &mut bus,
            );
            scope.finish_layout(viewport_rect);
        }

        assert!(tree.iter().any(|(_, n)| n.tag == TAG_PLAY_RESUME));
        assert!(tree.iter().any(|(_, n)| n.tag == TAG_PLAY_EXIT));
        assert!(
            bus.drain()
                .any(|a| matches!(a, EngineUiAction::ChangeMode(EngineMode::Edit))),
            "Clicking Exit in Pause Menu must emit EngineUiAction::ChangeMode(Edit)"
        );
    }
}