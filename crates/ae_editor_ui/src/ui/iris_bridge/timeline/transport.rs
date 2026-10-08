// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Transport Toolbar Bridge
//!
//! Connects engine animation playback state (Play/Pause, Stop, Step, Loop, Speed, Clip Name)
//! to pure declarative [`UiScope`] timeline primitives in Iris UI.
//!

use super::types::{
    MediaTransportStyle, TIMELINE_TAG_LOOP, TIMELINE_TAG_PLAY_PAUSE, TIMELINE_TAG_SPEED_BASE,
    TIMELINE_TAG_STEP_BACK, TIMELINE_TAG_STEP_FWD, TIMELINE_TAG_STOP, TimelinePanelParams,
};
use irisui::prelude::*;

/// Height of the transport controls bar in physical pixels.
pub const TRANSPORT_TOOLBAR_HEIGHT: f32 = 36.0;

/// Available speed preset multipliers.
pub const SPEED_PRESETS: [f32; 4] = [0.25, 0.5, 1.0, 2.0];

/// Builds the transport controls toolbar at the top of the animation timeline panel.
///
/// Emits declarative buttons, pills, dividers, and badges directly into [`UiScope`].
pub fn build_transport_toolbar(
    scope: &mut UiScope<'_>,
    params: &TimelinePanelParams<'_>,
    duration: f32,
) {
    let player = params.animation_player;
    let is_playing = player.is_some_and(|p| p.state == ae_animation::AnimationState::Playing);
    let is_looping = player.is_some_and(|p| p.looping);
    let current_speed = player.map_or(1.0, |p| p.speed);
    let current_time = player.map_or(0.0, |p| p.current_time);
    let clip_name = player
        .and_then(|p| p.current_clip.as_ref())
        .map(|c| c.name.as_str());

    let style = MediaTransportStyle::dark_default();

    let toolbar_style = Style::new()
        .height(TRANSPORT_TOOLBAR_HEIGHT)
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(6.0)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .background(style.bg)
        .border(style.border_width, style.border_color)
        .clip_children(true);

    scope.container_named("TimelineTransportToolbar", toolbar_style, |row| {
        // 1. Step Back Button (26px)
        let _ = render_transport_button(
            row,
            "TimelineStepBackBtn",
            "⏮",
            TIMELINE_TAG_STEP_BACK,
            false,
            false,
            &style,
        );

        // 2. Play / Pause Button (32px accent)
        let _ = render_transport_button(
            row,
            "TimelinePlayPauseBtn",
            if is_playing { "⏸" } else { "▶" },
            TIMELINE_TAG_PLAY_PAUSE,
            is_playing,
            true,
            &style,
        );

        // 3. Stop Button (26px)
        let _ = render_transport_button(
            row,
            "TimelineStopBtn",
            "⏹",
            TIMELINE_TAG_STOP,
            false,
            false,
            &style,
        );

        // 4. Step Forward Button (26px)
        let _ = render_transport_button(
            row,
            "TimelineStepFwdBtn",
            "⏭",
            TIMELINE_TAG_STEP_FWD,
            false,
            false,
            &style,
        );

        // 5. Divider
        row.vertical_divider(18.0, style.divider_color);

        // 6. Loop Toggle Pill (26px)
        let _ = render_loop_pill(row, is_looping, TIMELINE_TAG_LOOP, &style);

        // 7. Divider
        row.vertical_divider(18.0, style.divider_color);

        // 8. Speed Presets Pills
        for (idx, &preset) in SPEED_PRESETS.iter().enumerate() {
            let tag = TIMELINE_TAG_SPEED_BASE + idx as u64;
            let is_selected = (preset - current_speed).abs() < 0.05;
            let _ = render_speed_pill(row, preset, is_selected, tag, &style);
        }

        // 9. Spacer
        row.spacer();

        // 10. Clip Title Badge
        render_clip_badge(row, clip_name, &style);

        // 11. Time & Frame Readout Display
        render_time_readout(row, current_time, duration, 30.0, &style);
    });
}

/// Emits a compact styled playback transport control button with semantic tagging and interaction evaluation.
///
/// Uses pure declarative [`UiScope::button_named_styled_tagged`] with editor-side [`Style`].
pub fn render_transport_button(
    scope: &mut UiScope<'_>,
    name: &'static str,
    glyph: &str,
    tag: u64,
    is_active: bool,
    is_accent: bool,
    style: &MediaTransportStyle,
) -> (WidgetId, WidgetResponse) {
    let width = if is_accent { 32.0 } else { 26.0 };
    let (bg, border_color) = if is_active {
        (style.play_bg_active, style.play_border_active)
    } else {
        (style.btn_bg_idle, style.btn_border)
    };
    let btn_style = Style::new()
        .width(width)
        .height(24.0)
        .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
        .background(bg)
        .border(1.0, border_color)
        .border_radius(4.0)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center);
    let resp = scope.button_named_styled_tagged(name, glyph, btn_style, tag);
    (resp.id, resp)
}

/// Emits a playback speed multiplier pill button via declarative [`UiScope::toggle_pill_tagged`].
///
/// Formats exact labels without redundant unit characters (e.g. "0.25x", "0.5x", "1x", "2x").
pub fn render_speed_pill(
    scope: &mut UiScope<'_>,
    speed: f32,
    is_selected: bool,
    tag: u64,
    _style: &MediaTransportStyle,
) -> (WidgetId, WidgetResponse) {
    let label = if (speed - 1.0).abs() < 1e-3 {
        "1x".to_string()
    } else if (speed - 0.5).abs() < 1e-3 {
        "0.5x".to_string()
    } else if (speed - 0.25).abs() < 1e-3 {
        "0.25x".to_string()
    } else if (speed - 2.0).abs() < 1e-3 {
        "2x".to_string()
    } else {
        let s = format!("{:.2}", speed);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        format!("{}x", s)
    };
    let resp = scope.toggle_pill_tagged(&label, is_selected, tag);
    (resp.id, resp)
}

/// Emits a playback looping toggle button via declarative [`UiScope::button_named_styled_tagged`].
///
/// Uses the universal clockwise loop glyph (`↻`) for crisp rendering across all font engines.
pub fn render_loop_pill(
    scope: &mut UiScope<'_>,
    is_looping: bool,
    tag: u64,
    style: &MediaTransportStyle,
) -> (WidgetId, WidgetResponse) {
    let (bg, border_color) = if is_looping {
        (style.loop_active_bg, style.loop_active_color)
    } else {
        (style.btn_bg_idle, style.btn_border)
    };
    let loop_style = Style::new()
        .width(26.0)
        .height(24.0)
        .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
        .background(bg)
        .border(1.0, border_color)
        .border_radius(4.0)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center);
    let resp = scope.button_named_styled_tagged("TimelineLoopBtn", "↻", loop_style, tag);
    (resp.id, resp)
}

/// Emits a badge pill displaying the title of the active animation clip using declarative scope nesting.
pub fn render_clip_badge(
    scope: &mut UiScope<'_>,
    clip_name: Option<&str>,
    style: &MediaTransportStyle,
) -> WidgetId {
    let text = clip_name
        .map(|name| format!("🎬 {}", name))
        .unwrap_or_else(|| "🎬 No Clip".to_string());

    let text_w = (text.chars().count() as f32 * 6.8).ceil() + 18.0;

    let badge_style = Style::new()
        .width(text_w)
        .height(24.0)
        .background(style.clip_badge_bg)
        .border(1.0, style.clip_badge_border)
        .border_radius(4.0)
        .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .flex_shrink(0.0);

    scope.container_named("TimelineClipBadge", badge_style, |b| {
        b.label(text, 10.0, style.clip_badge_text, TextAlign::Center);
    })
}

/// Emits a timestamp and frame index readout display badge using declarative scope nesting.
pub fn render_time_readout(
    scope: &mut UiScope<'_>,
    current_time: f32,
    duration: f32,
    fps: f32,
    style: &MediaTransportStyle,
) -> WidgetId {
    let cur_m = (current_time / 60.0).floor() as u32;
    let cur_s = current_time % 60.0;
    let dur_m = (duration / 60.0).floor() as u32;
    let dur_s = duration % 60.0;

    let cur_f = (current_time * fps).round() as u32;
    let total_f = (duration * fps).round() as u32;

    let text = format!(
        "{:02}:{:05.2} / {:02}:{:05.2} (F{} / F{})",
        cur_m, cur_s, dur_m, dur_s, cur_f, total_f
    );

    let text_w = (text.chars().count() as f32 * 6.2).ceil() + 18.0;

    let readout_style = Style::new()
        .width(text_w)
        .height(24.0)
        .background(Color::rgba(0.06, 0.08, 0.11, 0.85))
        .border(1.0, Color::rgba(0.18, 0.22, 0.28, 0.60))
        .border_radius(4.0)
        .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0))
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .flex_shrink(0.0);

    scope.container_named("TimelineTimeReadout", readout_style, |r| {
        r.label(text, 9.5, style.time_readout_color, TextAlign::Center);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_toolbar_declarative_build() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 200.0),
            entity: None,
            animation_player: None,
            cursor_pos: Point::new(10.0, 10.0),
            is_dragging_scrubber: false,
            events: &[],
        };

        {
            let mut scope = UiScope::new(&mut tree, root);
            build_transport_toolbar(&mut scope, &params, 5.0);
        }

        // Verify root has child nodes emitted declaratively
        assert!(tree.get(root).is_some());
        assert!(!tree.get(root).unwrap().children.is_empty());
    }

    #[test]
    fn test_transport_helpers_pure_declarative() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let style = MediaTransportStyle::dark_default();

        let (btn_id, pill_id, loop_id, badge_id, readout_id) = {
            let mut scope = UiScope::new(&mut tree, root);

            let (btn_id, btn_resp) = render_transport_button(
                &mut scope,
                "PlayBtn",
                "▶",
                TIMELINE_TAG_PLAY_PAUSE,
                false,
                true,
                &style,
            );
            assert_eq!(btn_resp.id, btn_id);

            let (pill_id, pill_resp) =
                render_speed_pill(&mut scope, 1.5, true, TIMELINE_TAG_SPEED_BASE, &style);
            assert_eq!(pill_resp.id, pill_id);

            let (loop_id, loop_resp) =
                render_loop_pill(&mut scope, true, TIMELINE_TAG_LOOP, &style);
            assert_eq!(loop_resp.id, loop_id);

            let badge_id = render_clip_badge(&mut scope, Some("WalkCycle"), &style);
            let readout_id = render_time_readout(&mut scope, 1.25, 4.0, 30.0, &style);

            (btn_id, pill_id, loop_id, badge_id, readout_id)
        };

        assert!(tree.get(btn_id).is_some());
        assert!(tree.get(pill_id).is_some());
        assert!(tree.get(loop_id).is_some());
        assert!(tree.get(badge_id).is_some());
        assert!(tree.get(readout_id).is_some());
    }

    #[test]
    fn test_transport_toolbar_bounds_and_no_overflow() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let params = TimelinePanelParams {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 200.0),
            entity: None,
            animation_player: None,
            cursor_pos: Point::new(10.0, 10.0),
            is_dragging_scrubber: false,
            events: &[],
        };

        {
            let mut scope = UiScope::new(&mut tree, root);
            build_transport_toolbar(&mut scope, &params, 5.0);
        }

        let bounds = Rect::new(0.0, 0.0, 800.0, TRANSPORT_TOOLBAR_HEIGHT);
        layout_subtree(&mut tree, root, bounds);

        let root_node = tree.get(root).expect("Root exists");
        let toolbar_id = root_node.children[0];
        let toolbar_node = tree.get(toolbar_id).expect("Toolbar exists");

        // Verify that all children stay within the toolbar horizontal bounds
        for &cid in &toolbar_node.children {
            let child = tree.get(cid).expect("Child exists");
            assert!(
                child.computed_rect.x >= 0.0,
                "Widget {:?} x ({}) is negative",
                child.name,
                child.computed_rect.x
            );
            assert!(
                child.computed_rect.x + child.computed_rect.width <= 800.0 + 1.0,
                "Widget {:?} overflows toolbar right boundary: x ({}) + w ({}) = {} > 800.0",
                child.name,
                child.computed_rect.x,
                child.computed_rect.width,
                child.computed_rect.x + child.computed_rect.width
            );
        }
    }
}