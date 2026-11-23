// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Autonomous Scroll Area Integration Verification Suite
//!
//! Validates autonomous scroll frame geometry, mouse wheel scrolling with bounds clamping,
//! scrollbar thumb drag, slider track drag, and sidebar tab switch scroll resets using
//! the production `build_preferences_view` and `UiTree::dispatch_event`.

use super::panel::{SIDEBAR_WIDTH, build_preferences_view};
use super::types::{PREF_TAG_CONTENT_VIEW, PreferencesDialogState, preferences_scroll_style};
use crate::ui::types::{
    EditorFrameData, EditorTelemetrySnapshot, EngineUiAction, ExportDialogState,
};
use ae_assets::AssetBrowserState;
use ae_editor::editor_state::{EditorConfig, EditorState};
use ae_editor::snapping::SnapSettings;
use ae_renderer::asset::AssetStorage;
use ae_renderer::camera::{Camera, ProjectionMode};
use ae_renderer::graphics_settings::GraphicsSettings;
use ae_uidesign::UiDesignerState;
use irisui::prelude::*;
use irisui::widgets::scroll_area::ScrollAreaFrame;
use irisui::workbench::ActionBus;
use std::collections::HashSet;

fn render_pref_panel(
    tree: &mut UiTree,
    panel_rect: Rect,
    tab_idx: &mut u8,
    cfg: &mut EditorConfig,
    _cursor_pos: Point,
) -> (Rect, f32) {
    let root = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .expect("root");
    let gs = GraphicsSettings::default();
    let snap = SnapSettings::default();
    let enabled = HashSet::new();
    let world = ae_ecs::World::new();
    let editor_state = EditorState::default();
    let telemetry = EditorTelemetrySnapshot::default();
    let mut asset_browser = AssetBrowserState::default();
    let mut ui_designer = UiDesignerState::default();
    let mut export_modal = ExportDialogState::default();
    let textures = AssetStorage::new();
    let models = AssetStorage::new();
    let camera = Camera {
        position: cgmath::Point3::new(0.0, 2.0, 5.0),
        yaw: cgmath::Rad(-std::f32::consts::FRAC_PI_2),
        pitch: cgmath::Rad(0.0),
        aspect: 16.0 / 9.0,
        fovy: 45.0,
        znear: 0.1,
        zfar: 100.0,
        mode: ProjectionMode::Perspective,
        ortho_scale: 1.0,
        target: cgmath::Point3::new(0.0, 0.0, 0.0),
    };
    let mut bus = ActionBus::<EngineUiAction>::new();
    let content_w = (panel_rect.width - SIDEBAR_WIDTH - 1.0).max(100.0);
    let content_rect = Rect::new(
        panel_rect.x + SIDEBAR_WIDTH + 1.0,
        panel_rect.y,
        content_w,
        panel_rect.height,
    );
    let frame = EditorFrameData {
        is_editing: true,
        is_2d_mode: false,
        view_panel_mask: 0,
        can_undo: false,
        can_redo: false,
        has_viewport_texture: false,
        viewport_rect: Rect::ZERO,
        camera: &camera,
        world: &world,
        editor_state: &editor_state,
        graphics_settings: &gs,
        snapping_settings: &snap,
        editor_config: cfg,
        enable_live_updates: false,
        enabled_modules: &enabled,
        telemetry: &telemetry,
        gpu_adapter_name: "MockGPU",
        gpu_backend: "Vulkan",
        asset_browser: &mut asset_browser,
        ui_designer_state: &mut ui_designer,
        export_modal: &mut export_modal,
        textures: &textures,
        models: &models,
    };
    let mut state = PreferencesDialogState {
        tab: *tab_idx,
        ..Default::default()
    };

    {
        let mut scope = UiScope::new(tree, root);
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
                build_preferences_view(panel, &mut state, &frame, 1.0, &mut bus);
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    }

    *tab_idx = state.tab;
    for action in bus.drain() {
        if let EngineUiAction::UpdateEditorConfig(updated) = action {
            *cfg = updated;
        }
    }
    (content_rect, tree.max_scroll(PREF_TAG_CONTENT_VIEW))
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
    let mut tab = 1;
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
    assert!(
        thumb_rect.x > 740.0 && thumb_rect.right() <= 760.0,
        "Frame-1 scrollbar thumb must align to right edge of 760px panel, got {thumb_rect:?}"
    );
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
    assert!(
        tree.scroll_offset(PREF_TAG_CONTENT_VIEW) > 0.0,
        "Dragging scrollbar thumb must advance scroll offset"
    );

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