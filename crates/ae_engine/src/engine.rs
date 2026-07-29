// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use ae_core::ecs::{
    EcsManager, Light, Name, PlayerTag, Position, Rotation, Scale, Shape, Velocity,
};
use ae_core::time::Time;
use ae_editor::editor_state::EditorState;
use ae_editor::input::InputManager;
use ae_editor::undo_redo::Command;
use ae_physics::PhysicsWorld;
use ae_renderer::render::RenderState;
use std::sync::Arc;
use winit::window::Window;

/// Defines the current operational mode of the Aeon Engine.
pub use ae_core::modules::EngineMode;

/// The main application state and core controller of the Aeon Engine.
/// Owns all subsystems: render pipeline, ECS world, UI, camera, gizmo, input,
/// plugin host, asset manager, profiler, event bus, spatial grid, and undo history.
/// Orchestrates the per-frame update cycle (physics → ECS → plugins → render → UI).
pub struct AeEngine {
    pub render_state: RenderState,
    pub ui: ae_editor_ui::ui::EngineUi,
    pub camera: ae_renderer::camera::Camera,
    pub gizmo_system: ae_editor::gizmo::GizmoSystem,
    pub time: Time,
    pub input: InputManager,
    pub ecs: EcsManager,
    pub mode: EngineMode,
    pub previous_mode: EngineMode,
    pub editor: EditorState,
    pub asset_receivers: Vec<std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>>,
    pub model_receivers:
        Vec<std::sync::mpsc::Receiver<Result<ae_renderer::asset::ParsedModelData, String>>>,
    pub dialog_receivers: Vec<std::sync::mpsc::Receiver<std::path::PathBuf>>,
    /// Tracks active mouse cursor grab status to prevent duplicate Win32 `ShowCursor` calls.
    pub is_cursor_grabbed: bool,
    /// Channel receiver for holding background scene parsing thread results.
    /// Once the background thread finishes parallel loading/parsing of all models
    /// and textures, it sends the full `PendingSceneData` package here.
    pub scene_rx: Option<std::sync::mpsc::Receiver<Result<crate::scene::PendingSceneData, String>>>,
    /// Dynamic plugin manager for hot-reloadable game logic.
    pub plugin_manager: ae_plugin_host::PluginManager,
    pub asset_manager: ae_renderer::asset::AssetManager,
    /// Debug wireframe renderer for collider visualization.
    pub debug_renderer: crate::debug_renderer::DebugRenderer,
    pub resources: ae_core::Resources,
    pub event_bus: ae_core::events::DynamicEventBus,
    pub spatial_grid: ae_core::spatial::SpatialGrid,
    pub profiler: crate::profiler::Profiler,
    /// Decoupled Rapier3D physics simulation engine world.
    pub physics_world: PhysicsWorld,
    /// Hardware-accelerated 3D spatial audio engine manager.
    pub audio_manager: ae_audio::AudioManager,
}

impl AeEngine {
    /// Pushes a new command to the undo stack, dropping the oldest if the limit is exceeded.
    pub fn push_undo(&mut self, cmd: Command) {
        ae_editor::history::push_undo(&mut self.editor, cmd);
    }

    /// Asynchronously initializes the engine: creates WGPU surface, default scene,
    /// plugin host, debug renderer, and checks Python dependency.
    pub async fn new(window: Arc<Window>) -> Self {
        let (render_state, camera) = RenderState::new(window.clone()).await.unwrap();
        let ui = ae_editor_ui::ui::EngineUi::new(
            &render_state.device,
            render_state.config.format,
            &window,
        );
        let gizmo_system = ae_editor::gizmo::GizmoSystem::new(
            &render_state.device,
            render_state.config.format,
            render_state.post_process.msaa_samples,
        );
        let mut ecs = EcsManager::new();

        // Default Scene setup: Static Ground Plane + Physics Dynamic Cube + Sun Light
        ecs.world.spawn((
            Name("Ground Plane".to_string()),
            Shape::Cube,
            Position {
                x: 0.0,
                y: -0.5,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 50.0,
                y: 1.0,
                z: 50.0,
            },
            ae_core::ecs::Color {
                r: 0.2,
                g: 0.25,
                b: 0.3,
                a: 1.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            ae_core::ecs::RigidBody {
                body_type: ae_core::ecs::RigidBodyType::Static,
                mass: 0.0,
                gravity_scale: 0.0,
            },
            ae_core::ecs::Collider {
                shape: ae_core::ecs::ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        ecs.world.spawn((
            PlayerTag,
            Name("Dynamic Cube".to_string()),
            Shape::Cube,
            Position {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            ae_core::ecs::Color::soft_blue(),
            Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            ae_core::ecs::RigidBody {
                body_type: ae_core::ecs::RigidBodyType::Dynamic,
                mass: 1.0,
                gravity_scale: 1.0,
            },
            ae_core::ecs::Collider {
                shape: ae_core::ecs::ColliderShape::Box {
                    half_extents: [0.5, 0.5, 0.5],
                },
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            },
        ));

        ecs.world.spawn((
            Name("Sun".to_string()),
            Light {
                position: [5.0, 15.0, 5.0],
                color: [1.0, 1.0, 0.9],
            },
            Position {
                x: 5.0,
                y: 15.0,
                z: 5.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        ));

        let debug_renderer = crate::debug_renderer::DebugRenderer::new(
            &render_state.device,
            render_state.config.format,
            render_state.post_process.msaa_samples,
        );

        let mut engine = Self {
            render_state,
            ui,
            camera,
            gizmo_system,
            time: Time::new(),
            input: InputManager::new(),
            ecs,
            mode: EngineMode::Edit,
            previous_mode: EngineMode::Edit,
            editor: EditorState::default(),
            asset_receivers: Vec::new(),
            model_receivers: Vec::new(),
            dialog_receivers: Vec::new(),
            is_cursor_grabbed: false,
            scene_rx: None,
            plugin_manager: ae_plugin_host::PluginManager::new(),
            asset_manager: ae_renderer::asset::AssetManager::new(),
            debug_renderer,
            resources: ae_core::Resources::new(),
            event_bus: ae_core::events::DynamicEventBus::new(),
            spatial_grid: ae_core::spatial::SpatialGrid::new(200.0), // 200-unit default cell size for spatial grid culling
            profiler: crate::profiler::Profiler::new(),
            physics_world: PhysicsWorld::new(),
            audio_manager: ae_audio::AudioManager::new(),
        };

        // Align physics fixed time step with EditorConfig frequency
        engine.time.fixed_time_step = 1.0 / engine.editor.config.physics_hz;

        // Load the game logic plugin
        let ext = ae_plugin_api::platform_lib_extension();
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };

        let mut plugin_path_opt = None;
        let mut target_base_opt = None;

        // 1. Try current executable's folder (handles custom cargo target-dir redirection)
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                let plugin_exe_dir = exe_dir.join(format!("game_logic.{}", ext));
                if plugin_exe_dir.exists() {
                    plugin_path_opt = std::fs::canonicalize(&plugin_exe_dir).ok();
                    target_base_opt = std::fs::canonicalize(exe_dir).ok();
                }
            }
        }

        // 2. Fallback to target folder relative to CWD
        let plugin_rel = std::path::PathBuf::from(format!("target/{}/game_logic.{}", profile, ext));
        if plugin_path_opt.is_none() && plugin_rel.exists() {
            plugin_path_opt = std::fs::canonicalize(&plugin_rel).ok();
            target_base_opt = std::fs::canonicalize("target").ok();
        }

        match (plugin_path_opt, target_base_opt) {
            (Some(plugin_path), Some(target_base)) => {
                // Verify the canonicalized plugin path is inside the expected target/ base directory
                if !plugin_path.starts_with(&target_base) {
                    core::hint::cold_path();
                    log::error!(
                        "[SECURITY] Plugin path escapes target/ directory: {:?}",
                        plugin_path
                    );
                } else {
                    let staging_dir = target_base.join("plugins");
                    if let Err(e) = engine
                        .plugin_manager
                        .load_native_plugin(plugin_path, staging_dir)
                    {
                        core::hint::cold_path();
                        log::error!("Failed to load game logic plugin: {}", e);
                    }
                }
            }
            _ => {
                log::warn!(
                    "Game logic plugin not found at {:?}. Run `cargo build -p game_logic` first.",
                    plugin_rel
                );
            }
        }

        engine
    }
}