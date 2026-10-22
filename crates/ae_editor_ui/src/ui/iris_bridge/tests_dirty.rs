// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Retained UI Decentralized Dirty Checking Unit Tests
//!
//! Verifies localized `is_dirty`, `sync_dirty`, and `check_and_sync_dirty` state
//! transitions for individual editor panel state structures (Phase 3.2).
//!

use super::types::*;
use crate::assets::AssetBrowserState;
use crate::ui::panel_layout::PanelLayoutState;
use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer, GpuPassTimings, VramStats,
};
use ae_editor::editor_state::EditorConfig;
use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use ae_editor::snapping::SnapSettings;
use ae_renderer::asset::AssetStorage;
use ae_renderer::camera::{Camera, ProjectionMode};
use ae_renderer::graphics_settings::GraphicsSettings;
use ae_uidesign::UiDesignerState;
use hecs::World;
use irisui::prelude::Rect;
use std::collections::HashSet;

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

struct TestHarness {
    layout_state: PanelLayoutState,
    world: World,
    camera: Camera,
    graphics_settings: GraphicsSettings,
    snapping_settings: SnapSettings,
    editor_config: EditorConfig,
    enabled_modules: HashSet<ae_core::modules::EngineModule>,
    frame_pacing: FrameRingBuffer<240>,
    frame_pacing_stats: FramePacingStats,
    cpu_timings: CpuSyncTimings,
    gpu_pass_timings: GpuPassTimings,
    draw_call_stats: DrawCallBreakdown,
    vram_stats: VramStats,
    ui_designer_state: UiDesignerState,
    asset_browser: AssetBrowserState,
    textures: AssetStorage<ae_renderer::render::TextureAsset>,
    models: AssetStorage<ae_renderer::render::ModelAsset>,
    inspector_euler: [f32; 3],
    console_entries: Vec<crate::ui::types::ConsoleEntry>,
}

impl Default for TestHarness {
    fn default() -> Self {
        Self {
            layout_state: PanelLayoutState::default(),
            world: World::new(),
            camera: make_test_camera(),
            graphics_settings: GraphicsSettings::default(),
            snapping_settings: SnapSettings::default(),
            editor_config: EditorConfig::default(),
            enabled_modules: HashSet::new(),
            frame_pacing: FrameRingBuffer::new(),
            frame_pacing_stats: FramePacingStats::default(),
            cpu_timings: CpuSyncTimings::default(),
            gpu_pass_timings: GpuPassTimings::default(),
            draw_call_stats: DrawCallBreakdown::default(),
            vram_stats: VramStats::default(),
            ui_designer_state: UiDesignerState::default(),
            asset_browser: AssetBrowserState::default(),
            textures: AssetStorage::new(),
            models: AssetStorage::new(),
            inspector_euler: [0.0; 3],
            console_entries: Vec::new(),
        }
    }
}

impl TestHarness {
    fn make_params<'a>(
        &'a mut self,
        selected_entity: Option<hecs::Entity>,
    ) -> OverlayUpdateParams<'a> {
        let active_entities_count = self.world.len() as usize;
        OverlayUpdateParams {
            context: EditorContextParams {
                dimensions: (1920.0, 1080.0),
                zoom_factor: 1.0,
                is_editing: true,
                is_2d_mode: false,
                layout_state: &self.layout_state,
                can_undo: false,
                can_redo: false,
                enable_live_updates: true,
                status_spans: None,
            },
            viewport: ViewportParams {
                has_viewport_texture: true,
                viewport_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
                camera: &self.camera,
                wireframe_enabled: false,
                grid_enabled: true,
                gizmo_mode: GizmoMode::Translate,
                gizmo_space: GizmoSpace::World,
            },
            scene: SceneParams {
                selected_entity,
                world: &self.world,
                active_entities_count,
            },
            dialogs: DialogParams {
                show_about: false,
                show_preferences: false,
                delete_target: None,
                new_folder_parent: None,
                rename_target: None,
                is_loading_assets: false,
            },
            preferences: OverlayPreferencesParams {
                graphics_settings: &mut self.graphics_settings,
                snapping_settings: &mut self.snapping_settings,
                editor_config: &mut self.editor_config,
                enabled_modules: &self.enabled_modules,
            },
            telemetry: TelemetryParams {
                fps: 60.0,
                frame_pacing: &self.frame_pacing,
                frame_pacing_stats: &self.frame_pacing_stats,
                cpu_timings: &self.cpu_timings,
                gpu_pass_timings: &self.gpu_pass_timings,
                draw_call_stats: &self.draw_call_stats,
                vram_stats: &self.vram_stats,
                render_triangles: 100,
                render_vertices: 300,
                gpu_adapter_name: "Mock Adapter",
                gpu_backend: "Vulkan",
            },
            panel_data: OverlayPanelData {
                ui_designer_state: &self.ui_designer_state,
                asset_browser: &self.asset_browser,
                console_entries: &self.console_entries,
                textures: &self.textures,
                models: &self.models,
                inspector_euler: &self.inspector_euler,
                inspector_color_hex: "#ffffff",
                saved_swatches: &[],
            },
        }
    }
}

#[test]
fn test_material_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::material::MaterialPanelState::default();

    let entity1 = harness.world.spawn(());
    let entity2 = harness.world.spawn(());

    // Initially with entity1, is_dirty must be true (last_selected_entity is None)
    let params = harness.make_params(Some(entity1));
    assert!(state.is_dirty(&params));

    // check_and_sync_dirty synchronizes and returns true
    assert!(state.check_and_sync_dirty(&params));
    assert_eq!(state.last_selected_entity, Some(entity1));

    // Calling again with same parameters must be clean
    assert!(!state.is_dirty(&params));
    assert!(!state.check_and_sync_dirty(&params));

    // Selecting entity2 must make it dirty again
    let params2 = harness.make_params(Some(entity2));
    assert!(state.is_dirty(&params2));
    assert!(state.check_and_sync_dirty(&params2));
    assert_eq!(state.last_selected_entity, Some(entity2));
    assert!(!state.is_dirty(&params2));
}

#[test]
fn test_timeline_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::timeline::TimelinePanelState::default();

    let params = harness.make_params(None);
    assert!(!state.is_dirty(&params));

    // Dragging scrubber makes it dirty
    state.is_dragging = true;
    assert!(state.is_dirty(&params));
    assert!(state.check_and_sync_dirty(&params));
    assert!(!state.is_dirty(&params));

    // Releasing scrubber makes it dirty again
    state.is_dragging = false;
    assert!(state.is_dirty(&params));
    assert!(state.check_and_sync_dirty(&params));
    assert!(!state.is_dirty(&params));
}

#[test]
fn test_viewport_hud_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::viewport_hud::ViewportHudState::default();

    // Initial state with default camera has delta from (0,0,0)
    let params = harness.make_params(None);
    assert!(state.check_and_sync_dirty(&params));
    assert!(!state.is_dirty(&params));

    // Camera movement below 0.05 threshold is not dirty
    harness.camera.position.x += 0.01;
    let params_sub = harness.make_params(None);
    assert!(!state.is_dirty(&params_sub));

    // Camera movement above 0.05 threshold triggers dirty
    harness.camera.position.x += 0.1;
    let params_moved = harness.make_params(None);
    assert!(state.is_dirty(&params_moved));
    assert!(state.check_and_sync_dirty(&params_moved));
    assert!(!state.is_dirty(&params_moved));
}

#[test]
fn test_hierarchy_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::hierarchy::HierarchyPanelState::default();

    let params = harness.make_params(None);
    assert!(!state.is_dirty(&params));

    // Spawning an entity changes world.len(), triggering dirty
    let ent = harness.world.spawn(());
    let params_spawned = harness.make_params(None);
    assert!(state.is_dirty(&params_spawned));
    assert!(state.check_and_sync_dirty(&params_spawned));
    assert!(!state.is_dirty(&params_spawned));

    // Selecting the entity triggers dirty
    let params_selected = harness.make_params(Some(ent));
    assert!(state.is_dirty(&params_selected));
    assert!(state.check_and_sync_dirty(&params_selected));
    assert!(!state.is_dirty(&params_selected));
}

#[test]
fn test_console_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::console::ConsolePanelState::default();

    let params = harness.make_params(None);
    assert!(!state.is_dirty(&params));

    // New console logs arrive
    harness
        .console_entries
        .push(crate::ui::types::ConsoleEntry {
            level: log::Level::Info,
            target: "engine".to_string(),
            msg: "Test message".to_string(),
            timestamp: "00:00:00".to_string(),
        });
    let params_new_log = harness.make_params(None);
    assert!(state.is_dirty(&params_new_log));
    assert!(state.check_and_sync_dirty(&params_new_log));
    assert!(!state.is_dirty(&params_new_log));

    // Changing log level filter triggers dirty
    state.filter = super::console::ConsoleFilterLevel::Error;
    assert!(state.is_dirty(&params_new_log));
    assert!(state.check_and_sync_dirty(&params_new_log));
    assert!(!state.is_dirty(&params_new_log));
}

#[test]
fn test_assets_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::assets::AssetsPanelState::default();

    let params = harness.make_params(None);
    assert!(!state.is_dirty(&params));

    // Changing folder triggers dirty
    harness.asset_browser.current_folder = std::path::PathBuf::from("assets/textures");
    let params_folder = harness.make_params(None);
    assert!(state.is_dirty(&params_folder));
    assert!(state.check_and_sync_dirty(&params_folder));
    assert!(!state.is_dirty(&params_folder));
}

#[test]
fn test_stats_state_dirty_detection() {
    let mut harness = TestHarness::default();
    let mut state = super::stats::StatsPanelState::default();

    let params = harness.make_params(None);

    // Initial state immediately after creation has elapsed < 0.25s and no actions
    assert!(!state.is_dirty(&params));

    // Action dispatch triggers dirty
    state
        .interactions
        .push_action(super::stats::StatsPanelAction::ToggleWireframe);
    assert!(state.is_dirty(&params));
    let _ = state.interactions.take_actions();

    // Elapsed >= 250ms triggers dirty
    state.last_fps_refresh = std::time::Instant::now() - std::time::Duration::from_millis(300);
    assert!(state.is_dirty(&params));
    assert!(state.check_and_sync_dirty(&params));
    assert!(!state.is_dirty(&params));
}

#[test]
fn test_ui_notifier_panel_targeted_redraw() {
    use crate::ui::panel_layout::PanelId;
    use irisui::prelude::UiNotifier;

    let mut notifier: UiNotifier<PanelId> = UiNotifier::clean();

    // Initially clean notifier has no dirty panels
    assert!(!notifier.is_any_dirty());

    // Mark Hierarchy and Console dirty
    notifier.tag_redraw(PanelId::Hierarchy);
    notifier.tag_redraw(PanelId::Console);

    assert!(notifier.is_any_dirty());
    assert!(!notifier.is_global_dirty());
    assert!(notifier.is_dirty(PanelId::Hierarchy));
    assert!(notifier.is_dirty(PanelId::Console));
    assert!(!notifier.is_dirty(PanelId::Inspector));
    assert!(!notifier.is_dirty(PanelId::Assets));
    assert!(!notifier.is_dirty(PanelId::Stats));

    // Clear and verify
    notifier.clear_all();
    assert!(!notifier.is_any_dirty());
}