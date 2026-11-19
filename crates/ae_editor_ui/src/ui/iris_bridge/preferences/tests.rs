// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Panel Integration & Two-Way Binding Suite
//!
//! Validates declarative sidebar tabs, autonomous collapsible sections (`UiTree::collapsing_state`),
//! two-way settings bindings, scroll translation, typed numeric inputs, and direct
//! [`ActionBus<EngineUiAction>`] dispatches.

use super::builder::{SIDEBAR_TABS, SIDEBAR_WIDTH, build_preferences_view, tab_virtual_height};
use super::types::{
    PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLL_CONTAINER, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PreferencesDialogState, PreferencesParams, preferences_scroll_style,
};
use crate::ui::iris_bridge::theme::ThemePreset;
use crate::ui::types::EngineUiAction;
use ae_core::modules::EngineModule;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::{FpsLimit, GraphicsSettings};
use irisui::prelude::*;
use irisui::widgets::declarative::{combine_seeds, hash_label, hash_label_with_seed};
use irisui::workbench::ActionBus;
use std::collections::HashSet;

const TEST_PANEL_RECT: Rect = Rect::new(0.0, 0.0, 760.0, 540.0);

fn build_test_view(
    tree: &mut UiTree,
    params: &mut PreferencesParams<'_>,
    events: &[(u64, InteractionEvent)],
) -> (f32, Vec<EngineUiAction>) {
    let root = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .expect("Root creation must succeed");
    let mut bus = ActionBus::new();
    let content_w = TEST_PANEL_RECT.width - SIDEBAR_WIDTH - 1.0;

    let mut scope = UiScope::with_tagged_interactions(tree, root, events, None);
    scope.container_named(
        "PrefTestHost",
        Style::new()
            .position_absolute()
            .left(TEST_PANEL_RECT.x)
            .top(TEST_PANEL_RECT.y)
            .width(TEST_PANEL_RECT.width)
            .height(TEST_PANEL_RECT.height)
            .flex_row(),
        |panel| {
            build_preferences_view(panel, params, &mut bus, content_w, TEST_PANEL_RECT.height);
        },
    );
    scope.finish_layout(TEST_PANEL_RECT);
    let max_scroll = scope.sync_and_render_vertical_scrollbar(
        VerticalScrollbarSpec::new(
            PREF_TAG_CONTENT_VIEW,
            PREF_TAG_SCROLL_CONTAINER,
            PREF_TAG_SCROLLBAR_TRACK,
            PREF_TAG_SCROLLBAR_THUMB,
            Rect::new(SIDEBAR_WIDTH + 1.0, 0.0, content_w, TEST_PANEL_RECT.height),
        )
        .min_virtual_height(tab_virtual_height(*params.active_tab))
        .style(preferences_scroll_style()),
    );
    (max_scroll, bus.drain().collect())
}

#[test]
fn test_preferences_sidebar_tabs_and_collapsing_section_persistence() {
    let mut tree = UiTree::new();
    let mut tab = 1;
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();
    let mut live = false;
    let enabled = HashSet::new();
    let mut theme = ThemePreset::default();

    let (max_scroll_y, _) = build_test_view(
        &mut tree,
        &mut PreferencesParams {
            active_tab: &mut tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &[],
    );
    assert!(max_scroll_y > 0.0);

    for &(label, _) in &SIDEBAR_TABS {
        assert!(
            tree.iter()
                .any(|(_, node)| node.text.as_deref() == Some(label)),
            "Sidebar tab {label} must exist in declarative UiTree"
        );
    }

    let accent_count = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("TabAccentBar"))
        .count();
    assert_eq!(accent_count, SIDEBAR_TABS.len());

    // Clicking the Shadows collapsing header toggles its state inside UiTree without any HashSet
    let shadow_section_tag = UiTag::from("graphics_shadows").raw();
    let collapse_events = [(
        shadow_section_tag,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let mut collapsed_tree = UiTree::new();
    build_test_view(
        &mut collapsed_tree,
        &mut PreferencesParams {
            active_tab: &mut tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &collapse_events,
    );
    assert!(
        !collapsed_tree.is_collapsing_open(shadow_section_tag, true),
        "Clicking collapsing_section header must persist collapsed state in UiTree"
    );
}

#[test]
fn test_preferences_combobox_and_text_wrap_purity() {
    let mut tree = UiTree::new();
    let mut tab = 9;
    let mut gs = GraphicsSettings::default();
    assert_eq!(gs.fps_limit, FpsLimit::Limit120);
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();
    let mut live = false;
    let mut enabled = HashSet::new();
    enabled.insert(EngineModule::Physics);
    let mut theme = ThemePreset::default();

    let (max_scroll_y, _) = build_test_view(
        &mut tree,
        &mut PreferencesParams {
            active_tab: &mut tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &[],
    );
    assert!(max_scroll_y >= 0.0);

    let module_desc_nodes: Vec<_> = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("ModuleDesc"))
        .collect();
    assert!(!module_desc_nodes.is_empty());
    for (_, node) in module_desc_nodes {
        assert_eq!(node.text_wrap, TextWrap::Word);
    }
}

#[test]
fn test_preferences_scroll_translates_children_upward() {
    let mut tree_0 = UiTree::new();
    let mut tree_100 = UiTree::new();
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();
    let mut live = false;
    let enabled = HashSet::new();
    let mut theme = ThemePreset::default();
    let mut tab = 1;

    let (max_scroll_y, _) = build_test_view(
        &mut tree_0,
        &mut PreferencesParams {
            active_tab: &mut tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &[],
    );
    tree_100.set_scroll_offset(PREF_TAG_CONTENT_VIEW, 100.0);
    build_test_view(
        &mut tree_100,
        &mut PreferencesParams {
            active_tab: &mut tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &[],
    );

    let tag = UiTag::from("graphics_shadows").raw();
    let y0 = tree_0
        .iter()
        .find(|(_, n)| n.tag == tag)
        .expect("Shadows card")
        .1
        .computed_rect
        .y;
    let y100 = tree_100
        .iter()
        .find(|(_, n)| n.tag == tag)
        .expect("Shadows card")
        .1
        .computed_rect
        .y;
    assert!((y0 - y100 - 100.0).abs() < 1e-3);

    assert!(ScrollArea::sync_scroll_in_place(
        &mut tree_0,
        PREF_TAG_CONTENT_VIEW,
        Some(PREF_TAG_SCROLLBAR_TRACK),
        Some(PREF_TAG_SCROLLBAR_THUMB),
        100.0,
        max_scroll_y,
    ));
    let y_synced = tree_0
        .iter()
        .find(|(_, n)| n.tag == tag)
        .expect("Shadows card")
        .1
        .computed_rect
        .y;
    assert_eq!(y_synced, y0 - 100.0);
}

#[test]
fn test_preferences_two_way_data_binding_graphics_and_editor_tabs() {
    let mut gs = GraphicsSettings {
        shadow_enabled: false,
        bloom_enabled: false,
        fog_enabled: false,
        shadow_bias: 0.001,
        sun_pitch: 0.0,
        cloud_coverage: 0.2,
        ..Default::default()
    };

    let mut snap = SnapSettings {
        grid_size: 1.0,
        ..Default::default()
    };
    let mut cfg = EditorConfig {
        physics_hz: 60.0,
        max_undo_history: 100,
        ..Default::default()
    };
    let mut live = false;
    let enabled = HashSet::new();
    let mut theme = ThemePreset::default();

    let gfx_events = [
        (
            hash_label("Enable Directional Shadows"),
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            hash_label("Enable Bloom"),
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            hash_label("Enable Atmospheric Depth Fog"),
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            hash_label("Depth Bias"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Sun Pitch"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Cloud Coverage"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
    ];
    let mut gfx_tab = 1;
    build_test_view(
        &mut UiTree::new(),
        &mut PreferencesParams {
            active_tab: &mut gfx_tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &gfx_events,
    );

    assert!(gs.shadow_enabled);
    assert!(gs.bloom_enabled);
    assert!(gs.fog_enabled);
    assert!((gs.shadow_bias - (0.001 + 10.0 * 0.0005)).abs() < 1e-4);
    assert!((gs.sun_pitch - 5.0_f32.to_radians()).abs() < 1e-4);
    assert!((gs.cloud_coverage - 0.3).abs() < 1e-4);

    // Typed degree input on Sun Pitch number box
    let num_box_pitch =
        irisui::widgets::declarative::hash_label_with_seed(hash_label("Sun Pitch"), "##num_box");
    let type_events = [(
        num_box_pitch,
        InteractionEvent::TextInput {
            text: "45.0".to_string(),
        },
    )];
    build_test_view(
        &mut UiTree::new(),
        &mut PreferencesParams {
            active_tab: &mut gfx_tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &type_events,
    );
    assert!((gs.sun_pitch - 45.0_f32.to_radians()).abs() < 1e-4);

    // Editor tab bindings + runtime live toggle
    let editor_events = [
        (
            hash_label("Grid Size"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Fixed Update Frequency"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Undo History Limit"),
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            hash_label("Enable Live Editor Updates (Hot Reload)"),
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
    ];
    let mut editor_tab = 2;
    build_test_view(
        &mut UiTree::new(),
        &mut PreferencesParams {
            active_tab: &mut editor_tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &editor_events,
    );
    assert!((snap.grid_size - 1.5).abs() < 1e-4);
    assert!((cfg.physics_hz - 70.0).abs() < 1e-4);
    assert_eq!(cfg.max_undo_history, 200);
    assert!(live);
}

#[test]
fn test_preferences_action_bus_dispatch_for_modules_and_tabs() {
    let mut gs = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();
    let mut live = false;
    let mut enabled = HashSet::new();
    enabled.insert(EngineModule::Physics);
    let mut theme = ThemePreset::default();

    // 1. Sidebar tab click mutates active tab directly and resets scroll offset
    let mut tab_tree = UiTree::new();
    tab_tree.set_scroll_offset(PREF_TAG_CONTENT_VIEW, 120.0);
    let mut active_tab = 0;
    let tab_events = [(
        hash_label("Navigation"),
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    build_test_view(
        &mut tab_tree,
        &mut PreferencesParams {
            active_tab: &mut active_tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &tab_events,
    );
    assert_eq!(active_tab, 3);
    assert_eq!(tab_tree.scroll_offset(PREF_TAG_CONTENT_VIEW), 0.0);

    // 2. Module toggle emits EngineUiAction::ToggleModule directly on ActionBus
    let physics_checkbox_tag = hash_label_with_seed(combine_seeds(0, "Physics".into_seed()), "");
    let mut mod_tab = 9;
    let mod_events = [(
        physics_checkbox_tag,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let (_, mod_actions) = build_test_view(
        &mut UiTree::new(),
        &mut PreferencesParams {
            active_tab: &mut mod_tab,
            zoom_factor: 1.0,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live,
            enabled_modules: &enabled,
            current_theme: &mut theme,
        },
        &mod_events,
    );
    assert!(
        mod_actions
            .iter()
            .any(|a| matches!(a, EngineUiAction::ToggleModule(EngineModule::Physics)))
    );
}

#[test]
fn test_preferences_build_view_uses_outer_panel_rect_for_content_area() {
    let mut tree = UiTree::new();
    let panel_rect = Rect::new(40.0, 28.0, 920.0, 640.0);
    let root = tree.create_root().expect("Root node");
    let panel_id = {
        let mut root_scope = UiScope::new(&mut tree, root);
        root_scope.container_tagged(
            "TestPanelHost",
            Style::new()
                .position_absolute()
                .left(panel_rect.x)
                .top(panel_rect.y)
                .width(panel_rect.width)
                .height(panel_rect.height),
            WidgetRole::Default,
            9999,
            |panel| {
                panel.empty_box_passive_named("PrefHostSeed", Style::new().width(1.0).height(1.0));
            },
        );
        root_scope.finish_layout(Rect::new(0.0, 0.0, 1280.0, 720.0));
        root_scope.node_id_tagged(9999).unwrap()
    };

    let world = hecs::World::new();
    let mut gfx = GraphicsSettings::default();
    let mut snap = SnapSettings::default();
    let mut cfg = EditorConfig::default();
    let mut live_updates = true;
    let enabled_modules = HashSet::new();
    let fp = ae_core::telemetry::FrameRingBuffer::default();
    let fp_stats = ae_core::telemetry::FramePacingStats::default();
    let cpu = ae_core::telemetry::CpuSyncTimings::default();
    let pass = ae_core::telemetry::RenderPassRecordingTimings::default();
    let dc = ae_core::telemetry::DrawCallBreakdown::default();
    let vram = ae_core::telemetry::VramStats::default();
    let ui_designer = ae_uidesign::UiDesignerState::default();
    let mut asset_browser = crate::assets::AssetBrowserState::default();
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let camera = ae_renderer::camera::Camera {
        position: cgmath::Point3::new(0.0, 2.0, 5.0),
        yaw: cgmath::Rad(-std::f32::consts::FRAC_PI_2),
        pitch: cgmath::Rad(0.0),
        aspect: 16.0 / 9.0,
        fovy: 45.0,
        znear: 0.1,
        zfar: 100.0,
        mode: ae_renderer::camera::ProjectionMode::Perspective,
        ortho_scale: 1.0,
        target: cgmath::Point3::new(0.0, 0.0, 0.0),
    };
    let mut params = crate::ui::iris_bridge::types::OverlayUpdateParams {
        context: crate::ui::iris_bridge::params::EditorContextParams {
            dimensions: (1280.0, 720.0),
            zoom_factor: 1.0,
            is_editing: true,
            is_2d_mode: false,
            view_panel_mask: 0,
            can_undo: false,
            can_redo: false,
            enable_live_updates: true,
            status_spans: None,
            history_revision: 0,
        },
        viewport: crate::ui::iris_bridge::params::ViewportParams {
            has_viewport_texture: false,
            viewport_rect: Rect::ZERO,
            camera: &camera,
            wireframe_enabled: false,
            grid_enabled: true,
            gizmo_mode: ae_editor::gizmo::GizmoMode::Translate,
            gizmo_space: ae_editor::gizmo::GizmoSpace::World,
        },
        scene: crate::ui::iris_bridge::params::SceneParams {
            world: &world,
            selected_entity: None,
            active_entities_count: 0,
        },
        preferences: crate::ui::iris_bridge::params::OverlayPreferencesParams {
            graphics_settings: &mut gfx,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: &mut live_updates,
            enabled_modules: &enabled_modules,
        },
        telemetry: crate::ui::iris_bridge::params::TelemetryParams {
            fps: 60.0,
            frame_pacing: &fp,
            frame_pacing_stats: &fp_stats,
            cpu_timings: &cpu,
            pass_recording_timings: &pass,
            draw_call_stats: &dc,
            vram_stats: &vram,
            render_triangles: 0,
            render_vertices: 0,
            gpu_adapter_name: "MockGPU",
            gpu_backend: "Vulkan",
        },
        panel_data: crate::ui::iris_bridge::params::OverlayPanelData {
            ui_designer_state: &ui_designer,
            asset_browser: &mut asset_browser,
            console_entries: &[],
            textures: &textures,
            models: &models,
        },
    };

    let mut state = PreferencesDialogState::default();
    let mut bus = ActionBus::new();
    {
        let mut scope = UiScope::new(&mut tree, panel_id);
        state.build_view(&mut scope, &mut params, (Point::ZERO, false), &mut bus);
        scope.finish_layout(panel_rect);
    }

    let content_id = tree
        .find_node_by_name("PreferencesContentArea")
        .expect("PreferencesContentArea must exist");
    let content_rect = tree.get(content_id).unwrap().computed_rect;
    assert!((content_rect.width - (panel_rect.width - SIDEBAR_WIDTH - 1.0)).abs() < 1.0);
    assert!((content_rect.height - panel_rect.height).abs() < 1.0);
}