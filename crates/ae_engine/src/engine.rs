// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use ae_editor::editor_state::EditorState;
use ae_editor::undo_redo::Command;
use ae_input::InputManager;
use ae_physics::PhysicsWorld;
use ae_renderer::render::RenderState;
use ae_time::Time;
use std::sync::Arc;
use winit::window::Window;

/// Defines the current operational mode of the Aeon Engine.
pub use ae_core::modules::EngineMode;

/// The main application state and core controller of the Aeon Engine.
///
/// Owns all subsystems: render pipeline, ECS world, UI, camera, gizmo, input,
/// plugin host, asset manager, profiler, event bus, spatial grid, and undo history.
/// Orchestrates the per-frame update cycle (physics → ECS → plugins → render → UI).
pub struct AeEngine {
    pub render_state: RenderState,
    pub ui: ae_editor_ui::ui::EngineUi,
    pub camera: ae_camera::Camera,
    pub transform_gizmo: ae_gizmos::TransformGizmo,
    pub gizmo_renderer: ae_gizmos_render::GizmoRenderer,
    pub billboard_system: ae_gizmos_render::BillboardSystem,
    pub time: Time,
    pub input: InputManager,
    pub world: ae_ecs::World,
    pub mode: EngineMode,
    pub previous_mode: EngineMode,
    pub editor: EditorState,
    /// Unified asynchronous import receivers (native file dialogs and background `rayon` parsing).
    pub import_receivers: Vec<std::sync::mpsc::Receiver<ae_assets::AsyncImportResult>>,
    /// Tracks active mouse cursor grab status to prevent duplicate Win32 `ShowCursor` calls.
    pub is_cursor_grabbed: bool,
    /// Channel receiver for holding background scene parsing thread results.
    ///
    /// Once the background thread finishes parallel loading/parsing of all models
    /// and textures, it sends the full `PendingSceneData` package here.
    pub scene_rx: Option<std::sync::mpsc::Receiver<Result<ae_assets::PendingSceneData, String>>>,
    /// Dynamic plugin manager for hot-reloadable game logic.
    pub plugin_manager: ae_plugin_host::PluginManager,
    pub asset_manager: ae_renderer::asset::AssetManager,
    /// Debug wireframe renderer for collider visualization.
    pub debug_renderer: ae_gizmos_render::DebugRenderer,
    pub resources: ae_core::Resources,
    pub event_bus: ae_core::events::DynamicEventBus,
    pub spatial_grid: ae_transform::SpatialGrid,
    pub profiler: ae_diagnostic::Profiler,
    /// Decoupled Rapier3D physics simulation engine world.
    pub physics_world: PhysicsWorld,
    /// Hardware-accelerated 3D spatial audio engine manager.
    pub audio_manager: ae_audio::AudioManager,
    /// Live texture file modification watcher for hot-reloading.
    pub texture_watcher: ae_texture::TextureFileWatcher,
    /// Dirty flag to throttle `sync_ecs_to_physics` in Edit mode.
    /// Set to `true` whenever the ECS scene changes (spawn, delete, transform edit).
    /// Cleared after the sync runs. Prevents O(N) ECS scans every frame when nothing changed.
    pub physics_sync_dirty: bool,
    /// Stack-based Game State Machine for high-level flow and mode transitions.
    pub state_manager: ae_state::StateManager,
    /// In-Game HUD subsystem manager (Health Bar, Score Counter, Reticle).
    pub in_game_hud: ae_ui::InGameHudState,
    /// Active dimension mode governing engine operational state and memory isolation.
    pub dimension_mode: ae_2d::mode::ActiveDimensionMode,
    /// 2D Sprite batching and rendering pipeline system (allocated ONLY when in 2D mode).
    pub sprite_2d_system: Option<ae_2d::Sprite2DSystem>,
    /// Window event and pointer capture arbitration state machine.
    pub input_arbiter: crate::events::InputArbiter,
    /// Window event state and HighDPI coordinate translation coordinator.
    pub winit_state: irisui::winit::WinitState,
    /// Optional localhost MCP inspection session (`--inspect`) isolated in [`ae_mcp`].
    #[cfg(feature = "mcp")]
    pub mcp: Option<ae_mcp::McpSession>,
    /// Active partitioned world directory tracking dirty-only `.aechunk` differential saves.
    pub saved_world_dir: Option<std::path::PathBuf>,
    /// Persistent [`ae_world::EntityGuid`] identifying the active authored world.
    pub active_world_guid: ae_world::EntityGuid,
    /// Per-cell CRC32 checksums from the most recent world load or partitioned directory save.
    pub last_saved_cell_crcs: std::collections::HashMap<ae_world::SpatialCellCoord, u32>,
    /// Periodic background Edit-mode auto-save state writing to `Saved/Autosaves/`.
    pub autosave: ae_scene::EditorAutosaveState,
    /// Active Virtual File System provider (`DiskVfsProvider` in editor or `Pak*VfsProvider`).
    pub active_vfs: Option<Arc<dyn ae_world::VfsProvider>>,
    /// Runtime spatial chunk streaming coordinator and cross-cell GUID resolver.
    pub streaming: Option<ae_scene::RuntimeStreamingManager>,
    /// Asynchronous project export progress and completion receiver.
    pub export_rx: Option<std::sync::mpsc::Receiver<ae_export::ExportPipelineEvent>>,
}

impl AeEngine {
    /// Advances any active multi-frame MCP sequences and drains queued RPC calls.
    pub fn poll_inspect_requests(&mut self) {
        #[cfg(feature = "mcp")]
        if let Some(mut session) = self.mcp.take() {
            session.tick(self);
            self.mcp = Some(session);
        }
    }
    /// Pushes a new command to the undo stack, dropping the oldest if the limit is exceeded.
    pub fn push_undo(&mut self, cmd: Command) {
        ae_editor::history::push_undo(&mut self.editor, cmd);
    }

    /// Asynchronously initializes the engine: creates WGPU surface, default scene,
    /// plugin host, debug renderer, checks Python dependency, and isolates 2D/3D subsystems.
    ///
    /// # Errors
    /// Returns an error string if WGPU fails to locate a compatible graphics adapter or initialize the surface.
    pub async fn new(window: Arc<Window>, cli_args: crate::cli::CliArgs) -> Result<Self, String> {
        let (render_state, camera) = RenderState::new(Arc::clone(&window)).await?;
        let mut ui = ae_editor_ui::ui::EngineUi::new(
            &render_state.device,
            render_state.config.format,
            &window,
        );
        let init_size = window.inner_size();
        ui.on_window_resized(init_size.width, init_size.height);
        let transform_gizmo = ae_gizmos::TransformGizmo::new();
        let gizmo_renderer = ae_gizmos_render::GizmoRenderer::new(
            &render_state.device,
            render_state.config.format,
            render_state.post_process.msaa_samples,
        );
        let billboard_system = ae_gizmos_render::BillboardSystem::new(
            &render_state.device,
            &render_state.queue,
            render_state.config.format,
            render_state.post_process.msaa_samples,
        );

        let dimension_mode = cli_args.mode;
        let sprite_2d_system =
            (dimension_mode == ae_2d::mode::ActiveDimensionMode::Mode2D).then(|| {
                ae_2d::Sprite2DSystem::new(
                    &render_state.device,
                    &render_state.queue,
                    render_state.config.format,
                )
            });

        let mut world = ae_ecs::World::new();
        if dimension_mode == ae_2d::mode::ActiveDimensionMode::Mode2D {
            ae_scene::spawn_default_2d_scene(&mut world);
        } else {
            ae_scene::spawn_default_3d_scene(&mut world);
        }

        let debug_renderer = ae_gizmos_render::DebugRenderer::new(
            &render_state.device,
            render_state.config.format,
            render_state.post_process.msaa_samples,
        );
        #[cfg(feature = "mcp")]
        let mcp = cli_args
            .inspect_addr
            .as_deref()
            .map(|addr| {
                ae_mcp::McpSession::start(addr, &render_state.device, render_state.config.format)
            })
            .transpose()?;
        #[cfg(feature = "mcp")]
        if let Some(ref session) = mcp {
            log::info!(
                "🔌 Live inspection server listening on http://{}",
                session.bound_addr()
            );
        }
        #[cfg(not(feature = "mcp"))]
        if cli_args.inspect_addr.is_some() {
            return Err("ae_engine was compiled without the 'mcp' feature".to_string());
        }

        let active_vfs: Arc<dyn ae_world::VfsProvider> =
            Arc::new(ae_world::DiskVfsProvider::new("."));
        let mut audio_manager = ae_audio::AudioManager::new();
        audio_manager.set_vfs_provider(Some(Arc::clone(&active_vfs)));

        let mut engine = Self {
            render_state,
            ui,
            camera,
            transform_gizmo,
            gizmo_renderer,
            billboard_system,
            time: Time::new(),
            input: InputManager::new(),
            world,
            mode: EngineMode::Edit,
            previous_mode: EngineMode::Edit,
            editor: EditorState::default(),
            import_receivers: Vec::new(),
            is_cursor_grabbed: false,
            scene_rx: None,
            plugin_manager: ae_plugin_host::PluginManager::new(),
            asset_manager: ae_renderer::asset::AssetManager::new(),
            debug_renderer,
            resources: ae_core::Resources::new(),
            event_bus: ae_core::events::DynamicEventBus::new(),
            spatial_grid: ae_transform::SpatialGrid::new(200.0), // 200-unit default cell size for spatial grid culling
            profiler: ae_diagnostic::Profiler::new(),
            physics_world: PhysicsWorld::new(),
            audio_manager,
            texture_watcher: ae_texture::TextureFileWatcher::new(),
            physics_sync_dirty: true, // Run a full sync on the first frame to initialize Rapier bodies
            state_manager: ae_state::StateManager::new(),
            in_game_hud: ae_ui::InGameHudState::new(),
            dimension_mode,
            sprite_2d_system,
            input_arbiter: crate::events::InputArbiter::new(),
            winit_state: irisui::winit::WinitState::new(window.scale_factor()),
            #[cfg(feature = "mcp")]
            mcp,
            saved_world_dir: None,
            active_world_guid: ae_world::GuidGenerator::new().next_entity_guid(),
            last_saved_cell_crcs: std::collections::HashMap::new(),
            autosave: ae_scene::EditorAutosaveState::default(),
            active_vfs: Some(active_vfs),
            streaming: None,
            export_rx: None,
        };

        let is_2d = dimension_mode.is_2d();
        engine.editor.is_2d = is_2d;
        if is_2d {
            ae_2d::configure_2d_camera(&mut engine.camera);
        }

        // Align physics fixed time step with EditorConfig frequency
        engine.time.set_fixed_hz(engine.editor.config.physics_hz);

        // Discover and load native game logic plugin from executable/target directory
        engine
            .plugin_manager
            .load_workspace_native_plugin("game_logic");

        Ok(engine)
    }
}