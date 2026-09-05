// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Subsystem (`iris_bridge::timeline`)
//!
//! Provides the 100% GPU SDF hardware-accelerated Animation Timeline Studio panel
//! for Aeon Engine, replacing legacy egui rasterization with decoupled Retained UI
//! trees, responsive transport controls, adaptive time rulers, and interactive scrubbing.
//!

pub mod events;
pub mod panel;
pub mod ruler;
pub mod transport;
pub mod types;

pub use events::{handle_timeline_click, handle_timeline_drag};
pub use panel::build_timeline_panel;
pub use types::{TimelineAction, TimelinePanelParams, TimelinePanelTargets};

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::prelude::{Point, Rect, UiTree};

    #[test]
    fn test_timeline_panel_build_empty_state() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut targets = TimelinePanelTargets::default();

        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
            entity: None,
            animation_player: None,
            cursor_pos: Point::new(100.0, 420.0),
            is_dragging_scrubber: false,
        };

        build_timeline_panel(&mut tree, root, &params, &mut targets);

        assert_eq!(targets.panel_rect, params.panel_rect);
        assert!(targets.play_pause_btn.is_none());
        assert!(targets.scrubber_track_rect.is_none());
    }

    #[test]
    fn test_timeline_panel_build_with_player() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut targets = TimelinePanelTargets::default();

        let mut player = ae_animation::AnimationPlayer::new();
        player.current_clip = Some(ae_animation::AnimationClip {
            name: "TestClip".to_string(),
            duration: 3.5,
            channels: Vec::new(),
        });
        player.current_time = 1.2;

        let dummy_entity = hecs::World::new().spawn(());

        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 400.0, 800.0, 150.0),
            entity: Some(dummy_entity),
            animation_player: Some(&player),
            cursor_pos: Point::new(100.0, 420.0),
            is_dragging_scrubber: false,
        };

        build_timeline_panel(&mut tree, root, &params, &mut targets);

        assert_eq!(targets.panel_rect, params.panel_rect);
        assert!(targets.play_pause_btn.is_some());
        assert!(targets.stop_btn.is_some());
        assert!(targets.step_back_btn.is_some());
        assert!(targets.step_fwd_btn.is_some());
        assert!(targets.loop_toggle.is_some());
        assert_eq!(targets.speed_buttons.len(), 4);
        assert!(targets.scrubber_track_rect.is_some());
        assert!(targets.playhead_needle_rect.is_some());
        assert!((targets.clip_duration - 3.5).abs() < 1e-4);
    }

    #[test]
    fn test_timeline_scrubber_projection_math() {
        let mut targets = TimelinePanelTargets::default();
        let track_rect = Rect::new(100.0, 450.0, 600.0, 36.0);
        targets.scrubber_track_rect = Some(track_rect);
        targets.clip_duration = 5.0;

        // Click at middle of track (x = 400.0) -> should be 50% = 2.5s
        let click_pos = Point::new(400.0, 460.0);
        let res = handle_timeline_click(&targets, click_pos, None);
        assert!(res.is_some());
        let (action, dragging) = res.unwrap();
        assert!(dragging);
        match action {
            TimelineAction::ScrubTo(time) => {
                assert!((time - 2.5).abs() < 1e-3);
            }
            _ => panic!("Expected ScrubTo action"),
        }

        // Drag to 75% of track (x = 550.0) -> should be 3.75s
        let drag_pos = Point::new(550.0, 460.0);
        let drag_res = handle_timeline_drag(&targets, drag_pos);
        assert!(drag_res.is_some());
        match drag_res.unwrap() {
            TimelineAction::ScrubTo(time) => {
                assert!((time - 3.75).abs() < 1e-3);
            }
            _ => panic!("Expected ScrubTo action"),
        }
    }

    #[test]
    fn test_timeline_click_hit_testing() {
        let mut targets = TimelinePanelTargets::default();
        let play_btn = Rect::new(50.0, 405.0, 34.0, 26.0);
        let stop_btn = Rect::new(90.0, 405.0, 28.0, 26.0);
        let loop_btn = Rect::new(160.0, 405.0, 64.0, 26.0);
        targets.play_pause_btn = Some(play_btn);
        targets.stop_btn = Some(stop_btn);
        targets.loop_toggle = Some(loop_btn);

        // Click Play button
        let (act_play, drag_play) =
            handle_timeline_click(&targets, Point::new(60.0, 410.0), None).unwrap();
        assert_eq!(act_play, TimelineAction::TogglePlayPause);
        assert!(!drag_play);

        // Click Stop button
        let (act_stop, drag_stop) =
            handle_timeline_click(&targets, Point::new(95.0, 410.0), None).unwrap();
        assert_eq!(act_stop, TimelineAction::Stop);
        assert!(!drag_stop);

        // Click Loop toggle
        let (act_loop, drag_loop) =
            handle_timeline_click(&targets, Point::new(170.0, 410.0), None).unwrap();
        assert_eq!(act_loop, TimelineAction::ToggleLoop);
        assert!(!drag_loop);
    }
}