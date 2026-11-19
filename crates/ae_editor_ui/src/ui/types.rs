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
        let data = postcard::to_stdvec(component).unwrap_or_default();
        Self::ModifyComponent(entity, type_name, data)
    }
}

/// Lightweight snapshot of a single log entry – owned, no Mutex dependency.
#[derive(Clone)]
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