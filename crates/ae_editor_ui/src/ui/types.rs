// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Command enum representing all possible UI-driven actions.
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
    /// Identifies all unused CPU/GPU resources that are no longer referenced in the
    /// active ECS scene and frees their memory.
    GarbageCollect,

    // --- HIERARCHY ACTIONS ---
    SpawnShape(ae_core::ecs::Shape),
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
    SetUiScale(f32),
    SelectEntity(Option<hecs::Entity>),
    SetCameraMode(ae_renderer::camera::ProjectionMode),
    SetCameraTransform {
        pitch: cgmath::Rad<f32>,
        yaw: cgmath::Rad<f32>,
        position: cgmath::Point3<f32>,
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
    SetModelSubmeshAlphaMode(
        ae_renderer::asset::AssetHandle,
        usize,
        ae_renderer::render::types::SubmeshAlphaMode,
    ),
    SetModelSubmeshTexture(ae_renderer::asset::AssetHandle, usize, String),

    // --- DYNAMIC COMPONENT ACTIONS (Generic ComponentRegistry Pattern) ---
    AddComponent(hecs::Entity, &'static str),
    RemoveComponent(hecs::Entity, &'static str),
    ModifyComponent(hecs::Entity, &'static str, Vec<u8>),

    // --- SCENE/SYSTEM ACTIONS ---
    OpenModelDialog,
    OpenSaveSceneDialog,
    OpenLoadSceneDialog,
    SaveScene,
    LoadScene,
    SaveSceneToPath(std::path::PathBuf),
    LoadSceneFromPath(std::path::PathBuf),
    SaveEntityAsPrefab(hecs::Entity, std::path::PathBuf),
    InstantiatePrefab(std::path::PathBuf),
    Undo,
    Redo,
    UndoBatch(Vec<ae_editor::undo_redo::Command>), // For large spawn batches like stress test

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
}

impl EngineUiAction {
    /// Creates a generic component modification action by serializing `component` data to JSON bytes.
    pub fn modify_component<T: serde::Serialize>(
        entity: hecs::Entity,
        type_name: &'static str,
        component: &T,
    ) -> Self {
        let data = serde_json::to_vec(component).unwrap_or_default();
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

/// Canonical UI element type variants for spawning from the editor hierarchy.
pub use ae_uidesign::UiElementType;