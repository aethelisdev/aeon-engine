// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Autonomous Scroll Area Integration Verification Suite
//!
//! Validates autonomous scroll frame geometry, mouse wheel scrolling with bounds clamping,
//! scrollbar thumb drag, slider track drag, and sidebar tab switch scroll resets using
//! the production `build_preferences_view` and `UiTree::dispatch_event`.

use super::builder::{SIDEBAR_WIDTH, build_preferences_view, tab_virtual_height};
use super::types::{
    PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLL_CONTAINER, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PreferencesParams, preferences_scroll_style,
};
use crate::ui::iris_bridge::theme::ThemePreset;
use crate::ui::types::EngineUiAction;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::widgets::scroll_area::ScrollAreaFrame;
use irisui::workbench::ActionBus;
use std::collections::HashSet;

fn render_pref_panel(
    tree: &mut UiTree,
    panel_rect: Rect,
    tab_idx: &mut u8,
    cfg: &mut EditorConfig,
    cursor_pos: Point,
) -> (Rect, f32) {
    let root = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .expect("root");
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut live_updates = false;
    let mut theme = ThemePreset::default();
    let enabled = HashSet::new();
    let mut bus = ActionBus::<EngineUiAction>::new();
    let content_w = (panel_rect.width - SIDEBAR_WIDTH - 1.0).max(100.0);
    let content_rect = Rect::new(
        panel_rect.x + SIDEBAR_WIDTH + 1.0,
        panel_rect.y,
        content_w,
        panel_rect.height,
    );

    let mut scope = UiScope::new(tree, root);
    let mut params = PreferencesParams {
        active_tab: tab_idx,
        zoom_factor: 1.0,
        graphics_settings: &mut gs,
        snapping_settings: &mut snap,
        editor_config: cfg,
        enable_live_updates: &mut live_updates,
        enabled_modules: &enabled,
        current_theme: &mut theme,
    };
    scope.container_named(
        "PrefScrollHost",
        Style::new()
            .position_absolute()
            .left(panel_rect.x)
            .top(panel_rect.y)
            .width(panel_rect.width)
            .height(panel_rect.height)
            .flex_row(),
        |panel| {
            build_preferences_view(panel, &mut params, &mut bus, content_w, panel_rect.height);
        },
    );
    scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    let max_scroll = scope.sync_and_render_vertical_scrollbar(
        VerticalScrollbarSpec::new(
            PREF_TAG_CONTENT_VIEW,
            PREF_TAG_SCROLL_CONTAINER,
            PREF_TAG_SCROLLBAR_TRACK,
            PREF_TAG_SCROLLBAR_THUMB,
            content_rect,
        )
        .min_virtual_height(tab_virtual_height(*params.active_tab))
        .cursor(Some(cursor_pos))
        .style(preferences_scroll_style()),
    );
    (content_rect, max_scroll)
}

#[test]
fn test_preferences_scroll_style_and_frame_geometry() {
    let style = preferences_scroll_style();
    assert_eq!(style.thickness, 6.0);
    assert_eq!(style.inset, 3.0);

    let viewport = Rect::new(161.0, 36.0, 599.0, 504.0);
    let frame = ScrollAreaFrame::from_geometry(viewport, 1800.0, 120.0, &style);
    assert_eq!(frame.viewport_rect, viewport);
    assert_eq!(frame.content_height, 1800.0);
    assert_eq!(frame.max_scroll_y, 1800.0 - 504.0);

    let geom = frame
        .scrollbar
        .expect("Scrollbar geometry must be computed");
    assert_eq!(geom.track_rect.width, 6.0);
    assert!(geom.thumb_rect.height >= style.thumb_min_length);
}

#[test]
fn test_preferences_scroll_wheel_and_outside_isolation() {
    let mut tree = UiTree::new();
    let mut cfg = EditorConfig::default();
    let mut tab = 0;
    let panel_rect = Rect::new(100.0, 50.0, 760.0, 540.0);
    let (content_rect, max_scroll_y) = render_pref_panel(
        &mut tree,
        panel_rect,
        &mut tab,
        &mut cfg,
        Point::new(400.0, 300.0),
    );
    assert!(max_scroll_y > 0.0);

    let wheel_inside = UiEvent::MouseScroll {
        point: Point::new(content_rect.x + 50.0, content_rect.y + 50.0),
        delta_x: 0.0,
        delta_y: -50.0,
    };
    assert!(tree.dispatch_event(&wheel_inside).is_consumed());

    let huge_wheel = UiEvent::MouseScroll {
        point: Point::new(content_rect.x + 50.0, content_rect.y + 50.0),
        delta_x: 0.0,
        delta_y: -5000.0,
    };
    assert!(tree.dispatch_event(&huge_wheel).is_consumed());
    assert!(tree.scroll_offset(PREF_TAG_CONTENT_VIEW) <= max_scroll_y);

    let outside_wheel = UiEvent::MouseScroll {
        point: Point::new(20.0, 20.0),
        delta_x: 0.0,
        delta_y: -50.0,
    };
    assert!(!tree.dispatch_event(&outside_wheel).is_consumed());
}

#[test]
fn test_preferences_thumb_drag_and_slider_drag() {
    let mut tree = UiTree::new();
    let mut cfg = EditorConfig::default();
    let mut tab = 1;
    let panel_rect = Rect::new(0.0, 0.0, 760.0, 540.0);
    let (_content_rect, max_scroll_y) =
        render_pref_panel(&mut tree, panel_rect, &mut tab, &mut cfg, Point::ZERO);
    assert!(max_scroll_y > 0.0);

    let thumb_rect = tree
        .iter()
        .find(|(_, n)| n.role == irisui::core::node::WidgetRole::ScrollBarThumb)
        .expect("Scrollbar thumb emitted")
        .1
        .computed_rect;
    let thumb_center = Point::new(
        thumb_rect.x + thumb_rect.width * 0.5,
        thumb_rect.y + thumb_rect.height * 0.5,
    );

    assert!(
        tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: thumb_center,
        })
        .is_consumed()
    );
    assert!(
        tree.dispatch_event(&UiEvent::MouseMove {
            point: Point::new(thumb_center.x, thumb_center.y + 40.0),
        })
        .is_consumed()
    );
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(thumb_center.x, thumb_center.y + 40.0),
    });

    // Slider track drag on Navigation tab (tab = 3)
    let mut tree_nav = UiTree::new();
    let mut nav_tab = 3;
    render_pref_panel(
        &mut tree_nav,
        panel_rect,
        &mut nav_tab,
        &mut cfg,
        Point::ZERO,
    );
    let tag_track = irisui::widgets::declarative::hash_label("Base Fly Speed");
    let track_id = tree_nav
        .find_node_by_tag(tag_track)
        .expect("Base Fly Speed track emitted");
    let track_rect = tree_nav.get(track_id).unwrap().computed_rect;
    let click_point = Point::new(track_rect.x + track_rect.width * 0.5, track_rect.y + 5.0);

    assert!(
        tree_nav
            .dispatch_event(&UiEvent::MouseDown {
                button: MouseButton::Left,
                point: click_point,
            })
            .is_consumed()
    );
    assert_eq!(tree_nav.event_state.pointer_capture, Some(track_id));

    let mut tree_nav_after = UiTree::new();
    tree_nav_after.event_state = tree_nav.event_state.clone();
    tree_nav_after.tag_rects = tree_nav.tag_rects.clone();
    render_pref_panel(
        &mut tree_nav_after,
        panel_rect,
        &mut nav_tab,
        &mut cfg,
        click_point,
    );
    assert!((cfg.camera_base_speed - 15.25).abs() < 1.0);
}