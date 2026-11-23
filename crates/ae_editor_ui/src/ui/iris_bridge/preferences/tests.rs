// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Panel Integration & ActionBus Binding Suite
//!
//! Validates declarative sidebar tabs, autonomous collapsible sections (`UiTree::collapsing_state`),
//! `UiScope::edit_with_state` action emission, scroll translation, typed numeric inputs, and
//! [`ActionBus<EngineUiAction>`] dispatches.

use super::panel::{SIDEBAR_TABS, SIDEBAR_WIDTH, build_preferences_view};
use super::types::{PREF_TAG_CONTENT_VIEW, PreferencesDialogState};
use crate::ui::types::{
    EditorFrameData, EditorTelemetrySnapshot, EngineUiAction, ExportDialogState,
};
use ae_assets::AssetBrowserState;
use ae_core::modules::EngineModule;
use ae_editor::editor_state::{EditorConfig, EditorState};
use ae_editor::snapping::SnapSettings;
use ae_renderer::asset::AssetStorage;
use ae_renderer::camera::{Camera, ProjectionMode};
use ae_renderer::graphics_settings::{FpsLimit, GraphicsSettings};
use ae_uidesign::UiDesignerState;
use irisui::prelude::*;
use irisui::widgets::declarative::{combine_seeds, hash_label, hash_label_with_seed};
use irisui::workbench::ActionBus;
use std::collections::HashSet;

const TEST_PANEL_RECT: Rect = Rect::new(0.0, 0.0, 760.0, 540.0);

fn mock_camera() -> Camera {
    Camera {
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
    }
}

fn apply_test_actions(
    actions: &[EngineUiAction],
    gs: &mut GraphicsSettings,
    snap: &mut SnapSettings,
    cfg: &mut EditorConfig,
    live: &mut bool,
) {
    for action in actions {
        match action {
            EngineUiAction::UpdateGraphicsSettings(updated) => *gs = *updated,
            EngineUiAction::UpdateSnapSettings(updated) => *snap = *updated,
            EngineUiAction::UpdateEditorConfig(updated) => *cfg = *updated,
            EngineUiAction::SetLiveEditorUpdates(updated) => *live = *updated,
            _ => {}
        }
    }
}

struct TestHarnessInputs<'a> {
    tab: &'a mut u8,
    gs: &'a mut GraphicsSettings,
    snap: &'a mut SnapSettings,
    cfg: &'a mut EditorConfig,
    live: &'a mut bool,
    enabled_modules: &'a HashSet<EngineModule>,
}

fn build_test_view(
    tree: &mut UiTree,
    inputs: TestHarnessInputs<'_>,
    events: &[(u64, InteractionEvent)],
) -> (f32, Vec<EngineUiAction>) {
    let root = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .expect("Root creation must succeed");
    let mut bus = ActionBus::new();
    let world = ae_ecs::World::new();
    let editor_state = EditorState::default();
    let telemetry = EditorTelemetrySnapshot::default();
    let mut asset_browser = AssetBrowserState::default();
    let mut ui_designer = UiDesignerState::default();
    let mut export_modal = ExportDialogState::default();
    let textures = AssetStorage::new();
    let models = AssetStorage::new();
    let camera = mock_camera();

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
        graphics_settings: inputs.gs,
        snapping_settings: inputs.snap,
        editor_config: inputs.cfg,
        enable_live_updates: *inputs.live,
        enabled_modules: inputs.enabled_modules,
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
        tab: *inputs.tab,
        ..Default::default()
    };

    {
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
                build_preferences_view(panel, &mut state, &frame, 1.0, &mut bus);
            },
        );
        scope.finish_layout(TEST_PANEL_RECT);
    }

    *inputs.tab = state.tab;
    let actions: Vec<EngineUiAction> = bus.drain().collect();
    apply_test_actions(&actions, inputs.gs, inputs.snap, inputs.cfg, inputs.live);
    let max_scroll = tree.max_scroll(PREF_TAG_CONTENT_VIEW);
    (max_scroll, actions)
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

    let (max_scroll_y, _) = build_test_view(
        &mut tree,
        TestHarnessInputs {
            tab: &mut tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
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

    let shadow_section_tag = hash_label("graphics_shadows");
    assert!(tree.is_collapsing_open(shadow_section_tag, true));

    let collapse_events = [(
        shadow_section_tag,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let mut collapsed_tree = UiTree::new();
    build_test_view(
        &mut collapsed_tree,
        TestHarnessInputs {
            tab: &mut tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
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

    let (max_scroll_y, _) = build_test_view(
        &mut tree,
        TestHarnessInputs {
            tab: &mut tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
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
    let mut tab = 1;

    let (max_scroll_y, _) = build_test_view(
        &mut tree_0,
        TestHarnessInputs {
            tab: &mut tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
        },
        &[],
    );
    tree_100.set_scroll_offset(PREF_TAG_CONTENT_VIEW, 100.0);
    build_test_view(
        &mut tree_100,
        TestHarnessInputs {
            tab: &mut tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
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
    assert!(max_scroll_y > 100.0);
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
        TestHarnessInputs {
            tab: &mut gfx_tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
        },
        &gfx_events,
    );

    assert!(gs.shadow_enabled);
    assert!(gs.bloom_enabled);
    assert!(gs.fog_enabled);
    assert!((gs.shadow_bias - (0.001 + 10.0 * 0.0005)).abs() < 1e-4);
    assert!((gs.sun_pitch - 5.0_f32.to_radians()).abs() < 1e-4);
    assert!((gs.cloud_coverage - 0.3).abs() < 1e-4);

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
        TestHarnessInputs {
            tab: &mut gfx_tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
        },
        &type_events,
    );
    assert!((gs.sun_pitch - 45.0_f32.to_radians()).abs() < 1e-4);

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
        TestHarnessInputs {
            tab: &mut editor_tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
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
        TestHarnessInputs {
            tab: &mut active_tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
        },
        &tab_events,
    );
    assert_eq!(active_tab, 3);
    assert_eq!(tab_tree.scroll_offset(PREF_TAG_CONTENT_VIEW), 0.0);

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
        TestHarnessInputs {
            tab: &mut mod_tab,
            gs: &mut gs,
            snap: &mut snap,
            cfg: &mut cfg,
            live: &mut live,
            enabled_modules: &enabled,
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
        let resp = root_scope.container_tagged(
            "TestPanelHost",
            Style::new()
                .position_absolute()
                .left(panel_rect.x)
                .top(panel_rect.y)
                .width(panel_rect.width)
                .height(panel_rect.height),
            WidgetRole::Default,
            9999,
            |_panel| {},
        );
        root_scope.finish_layout(Rect::new(0.0, 0.0, 1280.0, 720.0));
        resp.response.id
    };

    let world = ae_ecs::World::new();
    let gfx = GraphicsSettings::default();
    let snap = SnapSettings::default();
    let cfg = EditorConfig::default();
    let enabled_modules = HashSet::new();
    let mut ui_designer = UiDesignerState::default();
    let mut asset_browser = AssetBrowserState::default();
    let textures = AssetStorage::new();
    let models = AssetStorage::new();
    let camera = mock_camera();
    let mut export_modal = ExportDialogState::default();
    let telemetry = EditorTelemetrySnapshot::default();
    let editor_state = EditorState::default();
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
        graphics_settings: &gfx,
        snapping_settings: &snap,
        editor_config: &cfg,
        enable_live_updates: true,
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

    let mut state = PreferencesDialogState::default();
    let mut bus = ActionBus::new();
    {
        let mut scope = UiScope::new(&mut tree, panel_id);
        crate::ui::panel_layout::PanelId::Preferences
            .surface()
            .expect("Preferences surface")
            .show(&mut scope, |panel| {
                super::panel::build_preferences_view(panel, &mut state, &frame, 1.0, &mut bus);
            });
        scope.finish_layout(panel_rect);
    }

    let content_id = tree
        .find_node_by_name("PreferencesContentArea")
        .expect("PreferencesContentArea must exist");
    let content_rect = tree.get(content_id).unwrap().computed_rect;
    assert!((content_rect.width - (panel_rect.width - SIDEBAR_WIDTH - 1.0)).abs() < 1.0);
    assert!((content_rect.height - panel_rect.height).abs() < 1.0);
}