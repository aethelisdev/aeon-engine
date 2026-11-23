// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences All Tabs & Two-Way Data Binding Verification Suite
//!
//! Validates that all ten preference tabs (`0..=9`) render robustly without errors,
//! and that Navigation (Tab 3), Editor (Tab 2), and Input (Tab 7) property sliders
//! emit updated [`EditorConfig`] actions via [`UiScope::edit_with_state`].

use super::panel::build_preferences_view;
use super::types::{PREF_TAG_CONTENT_VIEW, PreferencesDialogState};
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
use irisui::widgets::declarative::hash_label;
use irisui::workbench::ActionBus;
use std::collections::HashSet;

const TEST_PANEL_RECT: Rect = Rect::new(0.0, 0.0, 760.0, 540.0);

fn build_tab_harness(
    tree: &mut UiTree,
    tab_idx: u8,
    gs: &mut GraphicsSettings,
    snap: &mut SnapSettings,
    cfg: &mut EditorConfig,
    events: &[(u64, InteractionEvent)],
) -> f32 {
    let root = tree.create_root().expect("Root creation must succeed");
    let enabled_modules = HashSet::new();
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
        graphics_settings: gs,
        snapping_settings: snap,
        editor_config: cfg,
        enable_live_updates: false,
        enabled_modules: &enabled_modules,
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
        tab: tab_idx,
        ..Default::default()
    };

    {
        let mut scope = UiScope::with_tagged_interactions(tree, root, events, None);
        scope.container_named(
            "PrefTabHarness",
            Style::new()
                .position_absolute()
                .left(TEST_PANEL_RECT.x)
                .top(TEST_PANEL_RECT.y)
                .width(TEST_PANEL_RECT.width)
                .height(TEST_PANEL_RECT.height)
                .flex_row(),
            |panel| {
                build_preferences_view(panel, &mut state, &frame, 1.0, &mut bus);
            },
        );
        scope.finish_layout(TEST_PANEL_RECT);
    }

    for action in bus.drain() {
        match action {
            EngineUiAction::UpdateGraphicsSettings(updated) => *gs = updated,
            EngineUiAction::UpdateSnapSettings(updated) => *snap = updated,
            EngineUiAction::UpdateEditorConfig(updated) => *cfg = updated,
            _ => {}
        }
    }
    tree.max_scroll(PREF_TAG_CONTENT_VIEW)
}

#[test]
fn test_preferences_all_ten_tabs_render_cleanly() {
    for tab_idx in 0..=9 {
        let mut tree = UiTree::new();
        let mut gs = GraphicsSettings::default();
        let mut snap = SnapSettings::default();
        let mut cfg = EditorConfig::default();

        let max_scroll_y = build_tab_harness(&mut tree, tab_idx, &mut gs, &mut snap, &mut cfg, &[]);

        assert!(
            max_scroll_y >= 0.0,
            "Tab {tab_idx} max_scroll_y must be non-negative"
        );
        assert!(
            tree.len() >= 10,
            "Tab {} must construct at least 10 UI nodes, got {}",
            tab_idx,
            tree.len()
        );
    }
}

#[test]
fn test_preferences_two_way_data_binding_navigation_input_and_editor_tabs() {
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig {
        camera_base_speed: 5.0,
        camera_shift_multiplier: 3.0,
        camera_scroll_speed: 1.5,
        mouse_sensitivity: 0.005,
        max_undo_history: 100,
        physics_hz: 60.0,
        autosave_enabled: true,
        autosave_interval_secs: 300.0,
    };

    let nav_events = [
        (
            hash_label("Base Fly Speed"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Shift Boost Multiplier"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Scroll Zoom Speed"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
    ];
    build_tab_harness(
        &mut UiTree::new(),
        3,
        &mut gs,
        &mut snap,
        &mut cfg,
        &nav_events,
    );

    assert!((cfg.camera_base_speed - 7.0).abs() < 1e-4);
    assert!((cfg.camera_shift_multiplier - 4.0).abs() < 1e-4);
    assert!((cfg.camera_scroll_speed - 2.0).abs() < 1e-4);

    let input_events = [(
        hash_label("Look Sensitivity"),
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];
    build_tab_harness(
        &mut UiTree::new(),
        7,
        &mut gs,
        &mut snap,
        &mut cfg,
        &input_events,
    );
    assert!((cfg.mouse_sensitivity - 0.007).abs() < 1e-5);

    let editor_drag = [(
        hash_label("Auto-Save Interval"),
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];
    let mut editor_tree = UiTree::new();
    build_tab_harness(
        &mut editor_tree,
        2,
        &mut gs,
        &mut snap,
        &mut cfg,
        &editor_drag,
    );
    assert!((cfg.autosave_interval_secs - 350.0).abs() < 1e-3);

    let editor_click = [(
        hash_label("Enable Periodic Auto-Save"),
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    build_tab_harness(
        &mut UiTree::new(),
        2,
        &mut gs,
        &mut snap,
        &mut cfg,
        &editor_click,
    );
    assert!(!cfg.autosave_enabled);
}

#[test]
fn test_preferences_keymap_scroll_covers_full_content() {
    let mut tree = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();

    let max_scroll_y = build_tab_harness(&mut tree, 4, &mut gs, &mut snap, &mut cfg, &[]);
    assert!(
        max_scroll_y >= 500.0,
        "Keymap tab max_scroll_y must be at least 500px, got {max_scroll_y}"
    );
}