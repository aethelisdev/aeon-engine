// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Command enum representing all possible UI-driven actions.
///
/// Each variant is dispatched from UI panels (Inspector, Hierarchy, Menu)
/// and consumed by the engine loop to modify ECS, scene, or editor state.
#[derive(Clone, Debug)]
pub enum EngineUiAction {
    SpawnModel(ae_renderer::asset::AssetHandle),
    SpawnModelAt(ae_renderer::asset::AssetHandle, [f32; 3]),
    SpawnModelPathAt(std::path::PathBuf, [f32; 3]),
    SpawnSprite(ae_renderer::asset::AssetHandle),
    SpawnSpriteAt(ae_renderer::asset::AssetHandle, [f32; 3]),
    SpawnSpritePathAt(std::path::PathBuf, [f32; 3]),
    ChangeMode(ae_core::modules::EngineMode),
    ResumeGame,

    /// Triggers an ECS-wide asset garbage collection sweep.
    ///
    /// Identifies all unused CPU/GPU resources that are no longer referenced in the
    /// active ECS scene and frees their memory.
    GarbageCollect,

    // --- HIERARCHY ACTIONS ---
    SpawnShape(ae_core::ecs::Shape),
    SpawnDefaultSprite,
    SpawnPlayerSprite,
    SpawnEmpty2D,
    SpawnUiElement(UiElementType),
    DeleteSelected,
    StressTest(usize),
    AaaOpenWorldTest,
    Explode,
    ParentEntity(hecs::Entity, hecs::Entity),
    UnparentEntity(hecs::Entity),
    ToggleVisibility(hecs::Entity),

    // --- UI/EDITOR STATE ACTIONS ---
    OpenPanel(crate::ui::panel_layout::PanelId),
    /// Toggles the visibility or active tab selection of the specified dock panel.
    TogglePanel(crate::ui::panel_layout::PanelId),
    /// Resets the dock layout hierarchy and floating window positions to the default layout preset.
    ResetLayout,
    /// Opens the editor global preferences and configuration modal window.
    OpenPreferences,
    SetUiScale(f32),
    /// Updates the workspace color elevation theme preset cleanly via the workbench action bus.
    SetThemePreset(crate::ui::iris_bridge::theme::ThemePreset),
    SelectEntity(Option<hecs::Entity>),
    SetWireframe(bool),
    SetGridEnabled(bool),
    SetGizmoMode(ae_editor::gizmo::GizmoMode),
    ToggleGizmoSpace,
    SetCameraMode(ae_renderer::camera::ProjectionMode),
    ToggleCameraProjection,
    SetCameraTransform {
        pitch: cgmath::Rad<f32>,
        yaw: cgmath::Rad<f32>,
        position: cgmath::Point3<f32>,
        mode: Option<ae_renderer::camera::ProjectionMode>,
    },

    // --- INSPECTOR MODIFICATION ACTIONS ---
    ModifyName(hecs::Entity, String, String), // (Entity, OldName, NewName)
    LiveUpdatePosition(hecs::Entity, ae_core::ecs::Position),
    ModifyPosition(hecs::Entity, ae_core::ecs::Position, ae_core::ecs::Position),
    LiveUpdateRotation(hecs::Entity, ae_core::ecs::Rotation),
    ModifyRotation(hecs::Entity, ae_core::ecs::Rotation, ae_core::ecs::Rotation),
    LiveUpdateScale(hecs::Entity, ae_core::ecs::Scale),
    ModifyScale(hecs::Entity, ae_core::ecs::Scale, ae_core::ecs::Scale),
    ModifyColor(hecs::Entity, ae_core::ecs::Color, ae_core::ecs::Color),
    ModifyLightColor(hecs::Entity, [f32; 3], [f32; 3]),
    AssignTextureToEntity(hecs::Entity, String),
    RemoveTextureFromEntity(hecs::Entity),
    /// Opens the OS file dialog to pick and assign a texture to a 2D sprite entity.
    PickAndAssignEntityTexture(hecs::Entity),
    SetModelSubmeshAlphaMode(
        ae_renderer::asset::AssetHandle,
        usize,
        ae_renderer::render::types::SubmeshAlphaMode,
    ),
    SetModelSubmeshTexture(ae_renderer::asset::AssetHandle, usize, String),
    /// Opens the OS file dialog to pick and assign a texture for a 3D model submesh slot.
    PickAndSetSubmeshTexture(ae_renderer::asset::AssetHandle, usize),

    // --- DYNAMIC COMPONENT ACTIONS (Generic ComponentRegistry Pattern) ---
    AddComponent(hecs::Entity, &'static str),
    RemoveComponent(hecs::Entity, &'static str),
    ModifyComponent(hecs::Entity, &'static str, Vec<u8>),
    CommitComponentModify(hecs::Entity, &'static str, Vec<u8>, Vec<u8>),

    // --- SCENE/SYSTEM ACTIONS ---
    OpenModelDialog,
    OpenSaveSceneDialog,
    OpenLoadSceneDialog,
    SaveScene,
    LoadScene,
    SaveSceneToPath(std::path::PathBuf),
    LoadSceneFromPath(std::path::PathBuf),
    SaveEntityAsPrefab(hecs::Entity, std::path::PathBuf),
    OpenLoadPrefabDialog,
    InstantiatePrefab(std::path::PathBuf),
    Undo,
    Redo,
    UndoBatch(Vec<ae_editor::undo_redo::Command>), // For large spawn batches like stress test
    /// Ingests external files dropped from the host operating system into the engine asset pipeline.
    ImportDroppedFiles {
        /// Absolute or relative file system paths to the dropped assets.
        paths: Vec<std::path::PathBuf>,
        /// Logical screen cursor coordinates where the drop occurred, or `None` if unpositioned.
        screen_pos: Option<[f32; 2]>,
    },
    /// Deletes a file or directory from the host file system.
    DeleteAssetOrFolder(std::path::PathBuf),
    /// Creates a new subfolder under the specified parent directory.
    CreateSubfolder {
        /// Parent directory path where the new folder will be created.
        parent: std::path::PathBuf,
        /// Name of the new folder.
        name: String,
    },
    /// Renames a file or directory on disk to a new name.
    RenameAssetOrFolder {
        /// Target file or directory path to be renamed.
        target: std::path::PathBuf,
        /// New name for the target asset.
        new_name: String,
    },

    // --- SETTINGS ACTIONS ---
    UpdateGraphicsSettings(ae_renderer::graphics_settings::GraphicsSettings),
    UpdateSnapSettings(ae_editor::snapping::SnapSettings),
    UpdateEditorConfig(ae_editor::editor_state::EditorConfig),
    SetLiveEditorUpdates(bool),
    Exit,
    /// Toggle the enable/disable state of a core engine module.
    ToggleModule(ae_core::modules::EngineModule),
    ModifyLodThresholds(hecs::Entity, f32, f32),
    ModifyLodModel(hecs::Entity, u8, Option<ae_renderer::asset::AssetHandle>),
    SpawnPhase1TestSandbox,
    /// Launches standalone project export using the configured request parameters.
    StartProjectExport(ae_export::ExportRequest),
}

/// UI state model backing the declarative `File -> Export Project...` modal dialog.
#[derive(Debug, Clone)]
pub struct ExportDialogState {
    /// Selected target platform (`Linux`, `Windows GNU`, `Windows MSVC`).
    pub target: ae_export::ExportTarget,
    /// Block compression strategy for `game.aepak`.
    pub compression: ae_export::CookCompressionProfile,
    /// Destination directory path entered by the user.
    pub output_dir: String,
    /// Whether to compile `ae_runtime --release` alongside `game.aepak`.
    pub build_binary: bool,
    /// `true` while background export worker is active.
    pub is_running: bool,
    /// Normalized progress `[0.0, 1.0]` displayed by the live progress bar.
    pub progress: f32,
    /// Latest human-readable stage or completion/error diagnostic.
    pub status_message: String,
}

impl Default for ExportDialogState {
    fn default() -> Self {
        Self {
            target: ae_export::ExportTarget::LinuxX86_64,
            compression: ae_export::CookCompressionProfile::FastLz4,
            output_dir: "dist_export/standalone".to_string(),
            build_binary: true,
            is_running: false,
            progress: 0.0,
            status_message: "Ready to package game.aepak and standalone runtime.".to_string(),
        }
    }
}

impl ExportDialogState {
    /// Marks project export active for `output_dir` and resets normalized progress.
    pub fn begin(&mut self, output_dir: &std::path::Path) {
        self.is_running = true;
        self.progress = 0.0;
        self.status_message = format!("Exporting to {}...", output_dir.display());
    }

    /// Updates progress fraction and diagnostic message while an export job runs.
    pub fn set_progress(&mut self, progress: f32, message: impl Into<String>) {
        self.progress = progress;
        self.status_message = message.into();
    }

    /// Records successful project export completion metrics and output path.
    pub fn finish_success(
        &mut self,
        worlds_packed: usize,
        total_assets: usize,
        output_dir: &std::path::Path,
    ) {
        self.is_running = false;
        self.progress = 1.0;
        self.status_message = format!(
            "Exported {} worlds ({} assets) to {}",
            worlds_packed,
            total_assets,
            output_dir.display()
        );
    }

    /// Records export failure diagnostics and clears active running state.
    pub fn finish_error(&mut self, error: &str) {
        self.is_running = false;
        self.status_message = format!("Export failed: {error}");
    }
}

impl EngineUiAction {
    /// Creates a generic component modification action by serializing `component` data to binary `postcard` bytes.
    pub fn modify_component<T: serde::Serialize>(
        entity: hecs::Entity,
        type_name: &'static str,
        component: &T,
    ) -> Self {
        Self::ModifyComponent(
            entity,
            type_name,
            postcard::to_stdvec(component).unwrap_or_default(),
        )
    }
}

/// Lightweight snapshot of a single log entry – owned, no Mutex dependency.
#[derive(Clone, Debug)]
pub struct ConsoleEntry {
    pub level: log::Level,
    pub target: String,
    pub msg: String,
    pub timestamp: String,
}

/// Batched per-frame performance and rendering telemetry passed from `ae_engine` into `EngineUi`.
///
/// Copying one plain-data snapshot replaces scattered per-field telemetry mutations without heap allocation.
#[derive(Debug, Default, Clone, Copy)]
pub struct EditorTelemetrySnapshot {
    /// ECS frame execution time in milliseconds.
    pub profiler_ecs_ms: f32,
    /// Physics simulation frame time in milliseconds.
    pub profiler_physics_ms: f32,
    /// GPU scene rendering duration (excluding present wait and UI pass) in milliseconds.
    pub profiler_render_ms: f32,
    /// VSync / swapchain present blocking wait duration in milliseconds.
    pub profiler_present_ms: f32,
    /// UI pass CPU command recording duration in milliseconds.
    pub profiler_ui_ms: f32,
    /// Total frame duration in milliseconds.
    pub profiler_frame_ms: f32,
    /// Detailed CPU thread synchronization timings breakdown.
    pub cpu_timings: ae_core::telemetry::CpuSyncTimings,
    /// Detailed CPU command recording timings per render pass.
    pub pass_recording_timings: ae_core::telemetry::RenderPassRecordingTimings,
    /// 240-frame rolling frame duration history ring buffer.
    pub frame_pacing: ae_core::telemetry::FrameRingBuffer<240>,
    /// Precalculated statistical pacing metrics (`1% Low`, `0.1% Low`, jitter).
    pub frame_pacing_stats: ae_core::telemetry::FramePacingStats,
    /// Draw call and batching breakdown counters.
    pub draw_call_stats: ae_core::telemetry::DrawCallBreakdown,
    /// Granular Video RAM (VRAM) usage breakdown.
    pub vram_stats: ae_core::telemetry::VramStats,
    /// Resident model memory usage in megabytes.
    pub memory_models_mb: f32,
    /// Resident texture memory usage in megabytes.
    pub memory_textures_mb: f32,
    /// Total rendered draw calls for the latest frame.
    pub render_draw_calls: u32,
    /// Total rendered triangles for the latest frame.
    pub render_triangles: u64,
    /// Total rendered vertices for the latest frame.
    pub render_vertices: u64,
}

/// Canonical UI element type variants for spawning from the editor hierarchy.
pub use ae_uidesign::UiElementType;

/// Complete borrowed frame state descriptor passed into [`crate::ui::EngineUi::render`].
pub struct EditorUiRenderParams<'a> {
    /// Active WGPU logical device handle.
    pub device: &'a wgpu::Device,
    /// Active WGPU command queue handle.
    pub queue: &'a wgpu::Queue,
    /// Frame command encoder for recording UI GPU passes.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// Native OS window reference used for cursor and input queries.
    pub window: &'a winit::window::Window,
    /// Swapchain target texture view for the editor window.
    pub window_surface_view: &'a wgpu::TextureView,
    /// Physical surface dimensions `(width, height)` in pixels.
    pub surface_size: (u32, u32),
    /// Offscreen 3D viewport render target view when available.
    pub viewport_texture_view: Option<&'a wgpu::TextureView>,
    /// Measured engine frames per second for status/telemetry readouts.
    pub fps: f32,
    /// Authoritative ECS world containing all entities and components.
    pub world: &'a hecs::World,
    /// Active engine execution mode (`Edit`, `Play`, `Pause`).
    pub mode: &'a ae_core::modules::EngineMode,
    /// Current command undo stack slice for menu enablement.
    pub undo_stack: &'a [ae_editor::undo_redo::Command],
    /// Current command redo stack slice for menu enablement.
    pub redo_stack: &'a [ae_editor::undo_redo::Command],
    /// Active rendering quality and post-processing settings.
    pub graphics_settings: &'a ae_renderer::graphics_settings::GraphicsSettings,
    /// Active grid/angle snapping configuration.
    pub snapping: &'a ae_editor::snapping::SnapSettings,
    /// Authoritative editor state (selection, tools, overlays, revisions).
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    /// Active editor or gameplay viewport camera.
    pub camera: &'a ae_renderer::camera::Camera,
    /// Resident 3D model storage pool.
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Resident 2D texture storage pool.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// Resident WGSL shader storage pool.
    pub shaders: &'a ae_renderer::asset::AssetStorage<ae_renderer::asset::ShaderAsset>,
    /// Set of currently enabled runtime engine modules.
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    /// Whether the editor viewport is operating in 2D mode.
    pub is_2d_mode: bool,
    /// Output queue collecting UI-triggered engine actions for this frame.
    pub ui_actions: &'a mut Vec<EngineUiAction>,
}

/// Compact borrowed frame context supplied to registered panels via [`irisui::workbench::WorkbenchHost`].
pub struct EditorFrameData<'frame> {
    /// Whether the engine is currently in editor mode (not playing).
    pub is_editing: bool,
    /// Whether the scene is operating in 2D editing mode.
    pub is_2d_mode: bool,
    /// Bitmask tracking open panels for `View` / `Window` menu checkmarks.
    pub view_panel_mask: u64,
    /// Whether undo is currently available.
    pub can_undo: bool,
    /// Whether redo is currently available.
    pub can_redo: bool,
    /// Whether a valid 3D viewport render target texture is bound.
    pub has_viewport_texture: bool,
    /// Computed 3D viewport dock region in logical coordinates.
    pub viewport_rect: irisui::prelude::Rect,
    /// Active camera parameters for HUD and drop raycasting.
    pub camera: &'frame ae_renderer::camera::Camera,
    /// Authoritative ECS world.
    pub world: &'frame hecs::World,
    /// Authoritative editor state (selection, gizmo, wireframe, grid, revision).
    pub editor_state: &'frame ae_editor::editor_state::EditorState,
    /// Read-only graphics settings reference for the active frame.
    pub graphics_settings: &'frame ae_renderer::graphics_settings::GraphicsSettings,
    /// Read-only snap settings reference for the active frame.
    pub snapping_settings: &'frame ae_editor::snapping::SnapSettings,
    /// Read-only general editor configuration reference for the active frame.
    pub editor_config: &'frame ae_editor::editor_state::EditorConfig,
    /// Active live hot-reload editor updates toggle.
    pub enable_live_updates: bool,
    /// Enabled optional engine modules.
    pub enabled_modules: &'frame std::collections::HashSet<ae_core::modules::EngineModule>,
    /// Latest CPU/GPU telemetry snapshot.
    pub telemetry: &'frame EditorTelemetrySnapshot,
    /// Cached physical GPU adapter display name.
    pub gpu_adapter_name: &'frame str,
    /// Cached GPU backend identifier string.
    pub gpu_backend: &'frame str,
    /// Mutable persistent Content / Asset Browser state.
    pub asset_browser: &'frame mut ae_assets::AssetBrowserState,
    /// Mutable persistent 2D UI Designer canvas state.
    pub ui_designer_state: &'frame mut ae_uidesign::UiDesignerState,
    /// Mutable Export Project modal dialog state.
    pub export_modal: &'frame mut ExportDialogState,
    /// Resident 2D texture storage pool.
    pub textures: &'frame ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// Resident 3D model storage pool.
    pub models: &'frame ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
}