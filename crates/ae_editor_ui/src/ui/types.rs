// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Command enum representing all possible UI-driven actions.
/// Each variant is dispatched from UI panels (Inspector, Hierarchy, Menu)
/// and consumed by the engine loop to modify ECS, scene, or editor state.
pub enum EngineUiAction {
    SpawnModel(ae_renderer::asset::AssetHandle),
    SpawnSprite(ae_renderer::asset::AssetHandle),
    ChangeMode(ae_core::modules::EngineMode),

    /// Triggers an ECS-wide asset garbage collection sweep.
    /// Identifies all unused CPU/GPU resources that are no longer referenced in the
    /// active ECS scene and frees their memory.
    GarbageCollect,

    // --- HIERARCHY ACTIONS ---
    SpawnShape(ae_core::ecs::Shape),
    DeleteSelected,
    StressTest(usize),
    AaaOpenWorldTest,
    Explode,
    ParentEntity(hecs::Entity, hecs::Entity),
    UnparentEntity(hecs::Entity),

    // --- UI/EDITOR STATE ACTIONS ---
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

    // --- DYNAMIC COMPONENT ACTIONS (Physics) ---
    AddRigidBody(hecs::Entity, ae_core::ecs::RigidBody),
    RemoveRigidBody(hecs::Entity),
    ModifyRigidBody(hecs::Entity, ae_core::ecs::RigidBody),
    AddCollider(hecs::Entity, ae_core::ecs::Collider),
    RemoveCollider(hecs::Entity),
    ModifyCollider(hecs::Entity, ae_core::ecs::Collider),
    AddCharacterController(hecs::Entity, ae_core::ecs::CharacterController),
    RemoveCharacterController(hecs::Entity),
    ModifyCharacterController(hecs::Entity, ae_core::ecs::CharacterController),

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
    AddLodGroup(hecs::Entity),
    RemoveLodGroup(hecs::Entity),
    ModifyLodThresholds(hecs::Entity, f32, f32),
    ModifyLodModel(hecs::Entity, u8, Option<ae_renderer::asset::AssetHandle>),
    AddAudioSource(hecs::Entity),
    RemoveAudioSource(hecs::Entity),
    AddAudioListener(hecs::Entity),
    RemoveAudioListener(hecs::Entity),
    ModifyAudioSource(hecs::Entity, ae_audio::AudioSource),
}

/// Lightweight snapshot of a single log entry – owned, no Mutex dependency.
#[derive(Clone)]
pub struct ConsoleEntry {
    pub level: log::Level,
    pub target: String,
    pub msg: String,
    pub timestamp: String,
}