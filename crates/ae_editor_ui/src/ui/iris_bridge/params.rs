// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Parameter descriptors and domain-specific groupings for Iris UI overlay resolution.
//!
//! Encapsulates editor context, 3D viewport canvas bounds, ECS scene graph, modal dialogues,
//! configuration preferences, performance telemetry, and panel subsystem caches.

use crate::ui::panel_layout::PanelLayoutState;
use ae_core::modules::EngineModule;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::core::{Color, Rect};
use std::collections::HashSet;
use std::path::Path;

/// Global editor context, window dimensions, and interaction flags.
pub struct EditorContextParams<'a> {
    /// Screen dimensions (width, height) in physical pixels.
    pub dimensions: (f32, f32),
    /// Current display/UI zoom factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
    /// Whether the editor is currently in Edit mode.
    pub is_editing: bool,
    /// Whether the editor and project are running in 2D dimension mode.
    pub is_2d_mode: bool,
    /// Active panel layout state reference.
    pub layout_state: &'a PanelLayoutState,
    /// Whether undo is available.
    pub can_undo: bool,
    /// Whether redo is available.
    pub can_redo: bool,
    /// Whether live hot-reload editor updates are active.
    pub enable_live_updates: bool,
    /// Optional status notification message spans with text color.
    pub status_spans: Option<&'a [(String, Color)]>,
}

/// 3D Viewport canvas bounds, camera, and navigation state.
pub struct ViewportParams<'a> {
    /// Whether the resolved 3D viewport render target texture is present.
    pub has_viewport_texture: bool,
    /// Screen rectangle bounding the 3D viewport canvas.
    pub viewport_rect: Rect,
    /// Reference to the active 3D camera.
    pub camera: &'a ae_renderer::camera::Camera,
    /// Whether wireframe rendering is currently enabled.
    pub wireframe_enabled: bool,
    /// Whether the viewport coordinate grid is enabled.
    pub grid_enabled: bool,
    /// Currently active gizmo manipulation mode (Translate, Rotate, Scale).
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    /// Currently active gizmo coordinate space (World, Local).
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
}

/// Active ECS scene, selection handle, and entity population telemetry.
pub struct SceneParams<'a> {
    /// Active ECS world reference for entity queries.
    pub world: &'a hecs::World,
    /// Currently selected entity in the scene, if any.
    pub selected_entity: Option<hecs::Entity>,
    /// Count of active entities in the ECS world.
    pub active_entities_count: usize,
}

impl<'a> SceneParams<'a> {
    /// Returns the live entity count computed directly from the active ECS world.
    #[inline]
    #[must_use]
    pub fn live_entity_count(&self) -> usize {
        self.world.len() as usize
    }
}

/// Modal dialog display flags and file system operation targets.
pub struct DialogParams<'a> {
    /// Whether the About Aeon Engine modal dialogue is currently visible.
    pub show_about: bool,
    /// Whether the Preferences modal dialogue is currently visible.
    pub show_preferences: bool,
    /// Optional target path pending delete confirmation.
    pub delete_target: Option<&'a Path>,
    /// Optional new folder parent path.
    pub new_folder_parent: Option<&'a Path>,
    /// Optional rename target path and is_folder flag.
    pub rename_target: Option<(&'a Path, bool)>,
    /// Whether background assets are currently being loaded.
    pub is_loading_assets: bool,
}

/// Engine configuration, snapping, and preferences settings references.
pub struct OverlayPreferencesParams<'a> {
    /// Mutable reference to graphics settings for Preferences rendering and two-way data binding.
    pub graphics_settings: &'a mut GraphicsSettings,
    /// Mutable reference to snapping settings for Preferences rendering and two-way data binding.
    pub snapping_settings: &'a mut SnapSettings,
    /// Mutable reference to editor configuration for Preferences rendering and two-way data binding.
    pub editor_config: &'a mut EditorConfig,
    /// Set of enabled engine core modules for Preferences rendering.
    pub enabled_modules: &'a HashSet<EngineModule>,
}

/// Engine performance telemetry, frame pacing, and hardware statistics.
pub struct TelemetryParams<'a> {
    /// Real-time engine frames per second (FPS) rate.
    pub fps: f32,
    /// Historical frame pacing ring buffer.
    pub frame_pacing: &'a ae_core::telemetry::FrameRingBuffer<240>,
    /// Calculated frametime variance, 1% low, and 0.1% low stats.
    pub frame_pacing_stats: &'a ae_core::telemetry::FramePacingStats,
    /// CPU thread synchronization timings breakdown.
    pub cpu_timings: &'a ae_core::telemetry::CpuSyncTimings,
    /// CPU command buffer recording and pass orchestration timings breakdown.
    pub pass_recording_timings: &'a ae_core::telemetry::RenderPassRecordingTimings,
    /// GPU draw calls, pipeline binds, and primitive counts.
    pub draw_call_stats: &'a ae_core::telemetry::DrawCallBreakdown,
    /// Video memory (VRAM) budget allocation breakdown.
    pub vram_stats: &'a ae_core::telemetry::VramStats,
    /// Total rendered triangles count.
    pub render_triangles: usize,
    /// Total rendered vertices count.
    pub render_vertices: usize,
    /// Name of active GPU hardware adapter.
    pub gpu_adapter_name: &'a str,
    /// Active rendering backend identifier (e.g. Vulkan, DX12, Metal).
    pub gpu_backend: &'a str,
}

/// Bounding rectangles computed by dock system for active overlay panels.
/// Subsystem data repositories and live editor caches consumed by panel builders.
pub struct OverlayPanelData<'a> {
    /// Reference to persistent UI Designer state for canvas and toolbar rendering.
    pub ui_designer_state: &'a ae_uidesign::UiDesignerState,
    /// Reference to persistent asset browser state for Content Browser rendering.
    pub asset_browser: &'a crate::assets::AssetBrowserState,
    /// Slice of active in-memory log entries for Developer Console rendering.
    pub console_entries: &'a [crate::ui::types::ConsoleEntry],
    /// GPU texture asset repository for material panel inspection.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// GPU 3D model asset repository for material panel inspection.
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Euler angle cache for rotation editing: `[yaw, pitch, roll]` in degrees.
    pub inspector_euler: &'a [f32; 3],
    /// Hex color string cache for object appearance editing (e.g. `"#6699cc"`).
    pub inspector_color_hex: &'a str,
    /// Saved swatches palette: list of RGBA float arrays `[r, g, b, a]`.
    pub saved_swatches: &'a [[f32; 4]],
}

/// Parameters required for reconstructing and resolving all Iris UI editor overlays.
///
/// Composed of domain-specific parameter sub-structures to eliminate tight coupling.
pub struct OverlayUpdateParams<'a> {
    /// Global editor context and window properties.
    pub context: EditorContextParams<'a>,
    /// 3D Viewport canvas bounds, camera, and navigation state.
    pub viewport: ViewportParams<'a>,
    /// Scene graph, active entity, and ECS world access.
    pub scene: SceneParams<'a>,
    /// Modal dialog flags and file operations.
    pub dialogs: DialogParams<'a>,
    /// Engine configuration and preferences data.
    pub preferences: OverlayPreferencesParams<'a>,
    /// Performance statistics, frame pacing, and hardware telemetry.
    pub telemetry: TelemetryParams<'a>,
    /// Subsystem data repositories and caches consumed by panel builders.
    pub panel_data: OverlayPanelData<'a>,
}

impl<'a> OverlayUpdateParams<'a> {
    /// Returns the active panel docking layout state reference.
    #[inline]
    #[must_use]
    pub fn layout_state(&self) -> &PanelLayoutState {
        self.context.layout_state
    }

    /// Returns the logical screen dimensions `(width, height)`.
    #[inline]
    #[must_use]
    pub fn dimensions(&self) -> (f32, f32) {
        self.context.dimensions
    }

    /// Returns the current display scaling zoom factor.
    #[inline]
    #[must_use]
    pub fn zoom_factor(&self) -> f32 {
        self.context.zoom_factor
    }

    /// Returns the active ECS scene world reference.
    #[inline]
    #[must_use]
    pub fn world(&self) -> &hecs::World {
        self.scene.world
    }

    /// Returns the currently selected entity in the active scene, if any.
    #[inline]
    #[must_use]
    pub fn selected_entity(&self) -> Option<hecs::Entity> {
        self.scene.selected_entity
    }

    /// Evaluates whether any modal dialog or preferences card is actively open.
    #[inline]
    #[must_use]
    pub fn is_modal_active(&self) -> bool {
        self.dialogs.show_about
            || self.dialogs.delete_target.is_some()
            || self.dialogs.new_folder_parent.is_some()
            || self.dialogs.rename_target.is_some()
            || self.dialogs.is_loading_assets
    }
}