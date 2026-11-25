// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular desktop studio session composing [`ae_app`] core/render/simulation hosts with
//! cohesive editor-only gizmo, I/O, and window state groups.

use ae_app::{CoreWorld, RenderHost, SimulationHost};
use ae_core::modules::EngineMode;
use ae_editor::editor_state::EditorState;
use ae_editor::undo_redo::Command;
use ae_renderer::render::RenderState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, mpsc::Receiver};
use winit::window::Window;

/// Re-exported operational mode shared with editor UI panels.
pub use ae_core::modules::EngineMode as ActiveEngineMode;

/// Cohesive container for viewport-only 3D gizmos, entity icons, and collider wireframes.
pub struct EditorGizmos {
    /// Interactive translation/rotation/scale gizmo state machine.
    pub transform: ae_gizmos::TransformGizmo,
    /// WGPU forward-pass gizmo handle renderer.
    pub renderer: ae_gizmos_render::GizmoRenderer,
    /// Editor sprite/light/camera icon billboard system.
    pub billboards: ae_gizmos_render::BillboardSystem,
    /// Wireframe debug collider line renderer.
    pub debug: ae_gizmos_render::DebugRenderer,
}

impl EditorGizmos {
    /// Allocates GPU gizmo, billboard, and debug wireframe renderers matching `format`/`msaa_samples`.
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> Self {
        Self {
            transform: ae_gizmos::TransformGizmo::new(),
            renderer: ae_gizmos_render::GizmoRenderer::new(device, format, msaa_samples),
            billboards: ae_gizmos_render::BillboardSystem::new(device, queue, format, msaa_samples),
            debug: ae_gizmos_render::DebugRenderer::new(device, format, msaa_samples),
        }
    }

    /// Rebuilds MSAA-dependent editor overlay pipelines after graphics setting changes.
    pub fn rebuild_for_msaa(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        msaa_samples: u32,
        editor: &EditorState,
    ) {
        self.renderer = ae_gizmos_render::GizmoRenderer::new(device, format, msaa_samples);
        self.transform.mode = editor.gizmo_mode;
        self.transform.space = editor.gizmo_space;
        self.debug.rebuild_pipeline(device, format, msaa_samples);
        self.billboards
            .rebuild_pipeline(device, format, msaa_samples);
    }
}

/// Asynchronous asset import, scene load/export channels, VFS mount, autosave, and texture watcher.
pub struct EditorHostIo {
    /// Unified background dialog picker and `rayon` model/FBX import channels.
    pub import_receivers: Vec<Receiver<ae_assets::AsyncImportResult>>,
    /// Async partitioned world loading receiver.
    pub scene_rx: Option<Receiver<Result<ae_assets::PendingSceneData, String>>>,
    /// Async standalone project export event receiver.
    pub export_rx: Option<Receiver<ae_export::ExportPipelineEvent>>,
    /// Live texture file modification watcher for Edit-mode hot reload.
    pub texture_watcher: ae_texture::TextureFileWatcher,
    /// Active partitioned `.aeworld` directory for differential `.aechunk` saves.
    pub saved_world_dir: Option<PathBuf>,
    /// Last persisted per-cell CRC32 hashes for differential save skipping.
    pub last_saved_cell_crcs: HashMap<ae_world::SpatialCellCoord, u32>,
    /// Background periodic Edit-mode autosave state.
    pub autosave: ae_scene::EditorAutosaveState,
    /// Active Virtual File System provider.
    pub active_vfs: Option<Arc<dyn ae_world::VfsProvider>>,
}

/// Platform window handle, input arbitration, telemetry profiling, mode transitions, and optional MCP inspection.
pub struct EditorWindowState {
    /// Host OS window managed directly by the editor application shell.
    pub window: Arc<Window>,
    /// Current Edit / Play / Pause execution mode.
    pub mode: EngineMode,
    /// Previous frame mode for detecting clean Edit ↔ Play transitions.
    pub previous_mode: EngineMode,
    /// Pointer capture arbiter routing window events between UI overlays and the 3D viewport.
    pub input_arbiter: crate::events::InputArbiter,
    /// Winit-to-IrisUI coordinate and DPI event translator.
    pub winit_state: irisui::winit::WinitState,
    /// Frame timing and CPU/GPU telemetry profiler.
    pub profiler: ae_diagnostic::Profiler,
    /// Cached hardware cursor lock state preventing duplicate OS grab/visibility calls.
    pub is_cursor_grabbed: bool,
    /// Optional localhost JSON-RPC UI/world inspection server (`--inspect`).
    #[cfg(feature = "mcp")]
    pub mcp: Option<ae_mcp::McpSession>,
}

/// Top-level desktop studio composition uniting shared [`ae_app`] hosts with editor subsystems.
pub struct EditorSession {
    /// Authoritative ECS world, resources, events, frame clock, input, and world GUID.
    pub core: CoreWorld,
    /// Shared 3D/2D rendering, spatial grid culling, asset cache, and camera host.
    pub render: RenderHost,
    /// Shared physics, spatial audio, state machine, HUD, plugins, and chunk streaming host.
    pub sim: SimulationHost,
    /// Editor selection, snapping, configuration, scene flags, and Undo/Redo history.
    pub editor: EditorState,
    /// Declarative Iris UI editor workbench coordinator.
    pub ui: ae_editor_ui::ui::EngineUi,
    /// Viewport gizmo, entity icon billboard, and debug collider wireframe overlays.
    pub gizmos: EditorGizmos,
    /// Async I/O queues, differential world tracking, VFS, and hot-reload file watchers.
    pub host_io: EditorHostIo,
    /// Window input arbiter, profiling, active mode, cursor grab, and optional MCP server.
    pub window: EditorWindowState,
}

impl EditorSession {
    /// Advances any active multi-frame MCP inspection sequences and drains queued RPC calls.
    pub fn poll_inspect_requests(&mut self) {
        #[cfg(feature = "mcp")]
        if let Some(mut session) = self.window.mcp.take() {
            session.tick(self);
            self.window.mcp = Some(session);
        }
    }

    /// Pushes a command onto editor Undo history.
    pub fn push_undo(&mut self, cmd: Command) {
        ae_editor::history::push_undo(&mut self.editor, cmd);
    }

    /// Asynchronously initializes WGPU rendering, Iris UI, starter 2D/3D scene, editor overlays,
    /// native workspace plugin host, and optional live inspection server.
    ///
    /// # Errors
    /// Returns a diagnostic string when GPU initialization or `--inspect` setup fails.
    pub async fn new(window: Arc<Window>, cli_args: crate::cli::CliArgs) -> Result<Self, String> {
        let init_size = window.inner_size();
        let (render_state, camera) =
            RenderState::new(Arc::clone(&window), init_size.width, init_size.height).await?;
        let mut ui = ae_editor_ui::ui::EngineUi::new(
            &render_state.device,
            render_state.config.format,
            &window,
        );
        ui.on_window_resized(init_size.width, init_size.height);

        let gizmos = EditorGizmos::new(
            &render_state.device,
            &render_state.queue,
            render_state.config.format,
            render_state.post_process.msaa_samples,
        );

        let mut core = CoreWorld::new();
        if cli_args.mode == ae_2d::mode::ActiveDimensionMode::Mode2D {
            ae_scene::spawn_default_2d_scene(&mut core.world);
        } else {
            ae_scene::spawn_default_3d_scene(&mut core.world);
        }

        let editor = EditorState {
            is_2d: cli_args.mode.is_2d(),
            ..EditorState::default()
        };
        core.time.set_fixed_hz(editor.config.physics_hz);

        let render = RenderHost::new(
            render_state,
            ae_renderer::asset::AssetManager::new(),
            camera,
            200.0,
            cli_args.mode,
        );

        #[cfg(feature = "mcp")]
        let mcp = cli_args
            .inspect_addr
            .as_deref()
            .map(|addr| {
                ae_mcp::McpSession::start(
                    addr,
                    &render.render_state.device,
                    render.render_state.config.format,
                )
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
            return Err("ae_editor_app was compiled without the 'mcp' feature".to_string());
        }

        let active_vfs: Arc<dyn ae_world::VfsProvider> =
            Arc::new(ae_world::DiskVfsProvider::new("."));
        let mut audio_manager = ae_audio::AudioManager::new();
        audio_manager.set_vfs_provider(Some(Arc::clone(&active_vfs)));

        let mut plugin_manager = ae_plugin_host::PluginManager::new();
        plugin_manager.load_workspace_native_plugin("game_logic");

        let mut sim = SimulationHost::new(audio_manager, plugin_manager, None);
        sim.physics_sync_dirty = true;

        Ok(Self {
            core,
            render,
            sim,
            editor,
            ui,
            gizmos,
            host_io: EditorHostIo {
                import_receivers: Vec::new(),
                scene_rx: None,
                export_rx: None,
                texture_watcher: ae_texture::TextureFileWatcher::new(),
                saved_world_dir: None,
                last_saved_cell_crcs: HashMap::new(),
                autosave: ae_scene::EditorAutosaveState::default(),
                active_vfs: Some(active_vfs),
            },
            window: EditorWindowState {
                winit_state: irisui::winit::WinitState::new(window.scale_factor()),
                window,
                mode: EngineMode::Edit,
                previous_mode: EngineMode::Edit,
                input_arbiter: crate::events::InputArbiter::new(),
                profiler: ae_diagnostic::Profiler::new(),
                is_cursor_grabbed: false,
                #[cfg(feature = "mcp")]
                mcp,
            },
        })
    }
}