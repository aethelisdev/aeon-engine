// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod cli;
pub mod debug_renderer;
pub mod engine;
pub mod events;
pub mod icon;
#[cfg(feature = "mcp")]
pub mod mcp_host;

/// Aeon Engine — Main Entry Point
///
/// Initializes the winit event loop, creates the application window with embedded icon,
/// and delegates all event processing to `AeEngine`. Implements `ApplicationHandler` for
/// the modern winit 0.30+ callback-based architecture.
mod importer;
pub mod profiler;
pub mod render_pass;
pub mod update;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{WindowAttributes, WindowId},
};

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
use winit::platform::{wayland::WindowAttributesExtWayland, x11::WindowAttributesExtX11};

use engine::AeEngine;

/// Global memory allocator configured for Linux to optimize performance native-wide.
///
/// Substitutes the standard allocator with `Jemalloc` under Linux native execution,
/// while leaving Windows and macOS defaults completely untouched.
#[cfg(all(target_os = "linux", not(target_os = "android")))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Application shell that owns the engine and manages the winit event loop.
///
/// Uses `Option<AeEngine>` to support deferred initialization (engine is created
/// in `resumed()` when the window is ready). Also tracks frame timing for FPS limiting.
struct AeApp {
    cli_args: cli::CliArgs,
    engine: Option<AeEngine>,
    last_frame_time: std::time::Instant,
}

impl AeApp {
    fn new(cli_args: cli::CliArgs) -> Self {
        Self {
            cli_args,
            engine: None,
            last_frame_time: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for AeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            if let Some(ref proj_path) = self.cli_args.project_path {
                let target_dir = if proj_path.is_file() {
                    proj_path.parent().unwrap_or(proj_path)
                } else {
                    proj_path.as_path()
                };
                if let Err(e) = std::env::set_current_dir(target_dir) {
                    log::warn!("Could not set working directory to {:?}: {}", target_dir, e);
                } else {
                    let layout = ae_world::ProjectWorkspaceLayout::new(".");
                    if let Err(err) = layout.ensure_directories() {
                        log::warn!("Could not initialize project folders: {err}");
                    }
                    log::info!("📂 Project root activated: {:?}", target_dir);
                }
            }

            // Embed the icon directly in the binary for cross-platform reliability
            let icon_bytes = include_bytes!("../assets/icon/aeicon.png");
            let icon = icon::load_icon_from_memory(icon_bytes);

            #[cfg(target_os = "linux")]
            icon::ensure_xdg_desktop_integration(icon_bytes);

            let window_attributes = {
                let attr = WindowAttributes::default()
                    .with_title("Aeon Engine")
                    .with_maximized(true)
                    .with_visible(false)
                    .with_decorations(false)
                    .with_window_icon(icon);

                #[cfg(any(
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "openbsd",
                    target_os = "netbsd"
                ))]
                {
                    let attr = WindowAttributesExtWayland::with_name(
                        attr,
                        "com.aeonengine.Editor",
                        "com.aeonengine.Editor",
                    );
                    WindowAttributesExtX11::with_name(
                        attr,
                        "com.aeonengine.Editor",
                        "com.aeonengine.Editor",
                    )
                }

                #[cfg(not(any(
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "openbsd",
                    target_os = "netbsd"
                )))]
                {
                    attr
                }
            };

            let window = match event_loop.create_window(window_attributes) {
                Ok(win) => Arc::new(win),
                Err(err) => {
                    log::error!("[AeonEngine] Failed to create OS window: {err}");
                    let _ = std::io::Write::write_fmt(
                        &mut std::io::stderr(),
                        format_args!(
                            "\n==================================================================\n\
                             ❌ [Aeon Engine Critical Startup Error]\n\
                             Failed to create OS application window:\n\n\
                             {err}\n\n\
                             Please verify that an active display server (X11 / Wayland / Win32 / Cocoa) is available.\n\
                             ==================================================================\n\n"
                        ),
                    );
                    std::process::exit(1);
                }
            };

            // Initialize Core Engine State
            let engine = match pollster::block_on(AeEngine::new(
                Arc::clone(&window),
                self.cli_args.clone(),
            )) {
                Ok(eng) => eng,
                Err(err) => {
                    log::error!("[AeonEngine] Critical initialization failure: {err}");
                    let _ = std::io::Write::write_fmt(
                        &mut std::io::stderr(),
                        format_args!(
                            "\n==================================================================\n\
                             ❌ [Aeon Engine Critical Startup Error]\n\
                             Failed to initialize GPU or graphics backend:\n\n\
                             {err}\n\n\
                             Please verify your graphics driver (Vulkan / DX12 / Metal) and reboot if needed.\n\
                             ==================================================================\n\n"
                        ),
                    );
                    std::process::exit(1);
                }
            };
            window.set_visible(true);
            self.engine = Some(engine);
            self.last_frame_time = std::time::Instant::now();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // Drop the engine BEFORE exiting to release WGPU surface/device.
                // On Linux/Vulkan, winit sends extra redraws after exit() which
                // would crash if the GPU resources are still partially alive.
                self.engine = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Handles OS-initiated draw events (e.g. during window resizing or dragging on Win32 modal loops).
                // Ensures reliable window repainting and immediate Win32 client area validation, preventing UI freezing.
                self.update_and_render(event_loop);
            }
            ref other_event => {
                if let Some(engine) = self.engine.as_mut() {
                    engine.process_window_event(other_event);
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            engine.handle_device_event(&event);
        }
    }

    /// Continuous event loop and rendering process driver.
    /// Drives the continuous event loop rendering cycle in combination with `ControlFlow::Poll`.
    ///
    /// In Uncapped mode, the draw operation is triggered directly within this function to completely bypass
    /// the winit/Win32 event queue (`WM_PAINT` / `request_redraw`) limitations (1000Hz cap) and run at GPU swapchain speed.
    /// In limited modes (60/120 FPS), a high-precision spin-sleep timing pacer is applied before rendering.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(engine) = self.engine.as_mut() {
            let fps_limit = engine.render_state.graphics_settings.fps_limit;

            match fps_limit {
                ae_renderer::graphics_settings::FpsLimit::Limit60
                | ae_renderer::graphics_settings::FpsLimit::Limit120 => {
                    let target_fps = match fps_limit {
                        ae_renderer::graphics_settings::FpsLimit::Limit60 => 60.0,
                        ae_renderer::graphics_settings::FpsLimit::Limit120 => 120.0,
                        _ => unreachable!(),
                    };
                    let target_duration = std::time::Duration::from_secs_f32(1.0 / target_fps);

                    // Enforce frame pacing unconditionally (including during OS file drag hover)
                    // so ControlFlow::Poll never spins uncapped when hovering files over the window
                    if let Some(remaining) =
                        compute_frame_pacing_sleep(self.last_frame_time.elapsed(), target_duration)
                    {
                        spin_sleep::sleep(remaining);
                    }

                    // New frame cycle begins exactly at this point (after sleep)
                    self.last_frame_time = std::time::Instant::now();
                    self.update_and_render(event_loop);
                }
                ae_renderer::graphics_settings::FpsLimit::Uncapped => {
                    // In uncapped mode, update timer immediately and render to prevent input/render latency.
                    self.last_frame_time = std::time::Instant::now();
                    self.update_and_render(event_loop);
                }
            }
        }
    }
}

impl AeApp {
    /// Per-frame update + render cycle with error recovery.
    ///
    /// Executes the engine's update (`engine.update()`) and render (`engine.render()`) loops.
    /// Manages async scene loading, file saving/loading triggers, and handles window resizing
    /// in the event of a WGPU SurfaceLost error.
    fn update_and_render(&mut self, event_loop: &ActiveEventLoop) {
        let engine = match self.engine.as_mut() {
            Some(e) => e,
            None => return,
        };

        // Skip render if the window has a size of zero
        if engine.render_state.size.width == 0 || engine.render_state.size.height == 0 {
            return;
        }

        log::trace!("Render frame start");

        engine.editor.poll_scene_dialogs();

        if engine.editor.should_save_scene {
            engine.editor.should_save_scene = false;
            let path = engine
                .editor
                .pending_save_path
                .take()
                .or_else(|| engine.editor.active_scene_path.clone())
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(
                        ae_world::ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL,
                    )
                });
            let path_str = path.to_string_lossy();
            match ae_scene::save_partitioned_world(
                &mut engine.world,
                &engine.asset_manager,
                &mut engine.active_world_guid,
                ae_scene::WorldSaveRequest {
                    filepath: &path,
                    is_2d: engine.dimension_mode.is_2d(),
                    prev_saved_world_dir: engine.saved_world_dir.as_deref(),
                    prev_cell_crcs: &engine.last_saved_cell_crcs,
                },
            ) {
                Ok(saved) => {
                    engine.saved_world_dir = Some(saved.saved_world_dir);
                    engine.last_saved_cell_crcs = saved.cell_crcs;
                    engine.editor.active_scene_path = Some(saved.manifest_path);
                    engine.autosave.reset_timer();
                    engine.ui.set_status_message(
                        format!("Scene saved to {}", path_str),
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                Err(e) => {
                    engine.ui.set_status_message(
                        format!("Save Error: {}", e),
                        irisui::prelude::Color::RED,
                    );
                }
            }
        }
        if engine.editor.should_load_scene {
            engine.editor.should_load_scene = false;
            let path = engine
                .editor
                .pending_load_path
                .take()
                .or_else(|| engine.editor.active_scene_path.clone())
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(
                        ae_world::ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL,
                    )
                });
            match ae_scene::begin_async_scene_load(&path, engine.dimension_mode.is_2d()) {
                Ok(rx) => {
                    engine.editor.is_loading_assets = true;
                    engine.scene_rx = Some(rx);
                    engine.ui.set_status_message(
                        "Loading world: parsing assets in background...",
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        log::error!("{e}");
                        let ui_msg = if engine.dimension_mode.is_2d() {
                            "Cannot load 3D world in 2D mode! Switch to 3D mode."
                        } else {
                            "Cannot load 2D world in 3D mode! Switch to 2D mode."
                        };
                        engine
                            .ui
                            .set_status_message(ui_msg, irisui::prelude::Color::RED);
                    } else {
                        engine.ui.set_status_message(
                            format!("Load Error: {}", e),
                            irisui::prelude::Color::RED,
                        );
                    }
                    engine.editor.is_loading_assets = false;
                }
            }
        }

        // Exit trigger
        if engine.editor.should_exit {
            self.engine = None;
            event_loop.exit();
            return;
        }

        if let Some(rx) = engine.scene_rx.as_ref()
            && let Ok(result) = rx.try_recv()
        {
            engine.scene_rx = None;
            match result {
                Ok(scene_data) => {
                    engine.physics_world.clear();
                    engine.editor.selected_entities.clear();
                    engine.editor.undo_stack.clear();
                    engine.editor.redo_stack.clear();
                    engine.autosave.reset_timer();

                    let applied = ae_scene::apply_loaded_scene_data(
                        scene_data,
                        &mut engine.world,
                        &engine.render_state,
                        &mut engine.asset_manager,
                        engine.active_vfs.as_deref(),
                    );
                    engine.saved_world_dir = applied.saved_world_dir;
                    engine.active_world_guid = applied.world_guid;
                    engine.last_saved_cell_crcs = applied.cell_crcs;
                    engine.streaming = applied.streaming;

                    engine
                        .physics_world
                        .sync_ecs_to_physics(&mut engine.world, |handle| {
                            engine
                                .asset_manager
                                .get_physics_mesh_data(handle)
                                .map(|(v, i)| (v.as_slice(), i.as_slice()))
                        });
                    engine.editor.is_loading_assets = false;
                    engine.ui.set_status_message(
                        "World loaded successfully!",
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                    log::info!("Async binary world load processed cleanly.");
                }
                Err(e) => {
                    engine.editor.is_loading_assets = false;
                    engine.ui.set_status_message(
                        format!("Async world load failed: {e}"),
                        irisui::prelude::Color::RED,
                    );
                }
            }
        }
        engine.update();
        match engine.render() {
            Ok(_) => {}
            Err(ae_renderer::render::RenderError::SurfaceLost) => {
                std::hint::cold_path();
                engine.resize(engine.render_state.size);
            }
            Err(ae_renderer::render::RenderError::OutOfMemory) => {
                std::hint::cold_path();
                event_loop.exit();
            }
            Err(e) => {
                std::hint::cold_path();
                let _ = std::io::Write::write_fmt(
                    &mut std::io::stderr(),
                    format_args!("Render error: {:?}\n", e),
                );
            }
        }
        engine.input.end_frame();
        engine.poll_inspect_requests();
        log::trace!("Render frame end");
    }
}

/// Creates the platform winit [`EventLoop`], falling back from X11 to the default backend on Unix.
fn build_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        EventLoop::builder()
            .with_x11()
            .build()
            .or_else(|_| EventLoop::new())
    }
    #[cfg(not(all(unix, not(target_os = "macos"), not(target_os = "android"))))]
    {
        EventLoop::new()
    }
}

/// Entry point: initializes the dual-output logger, creates the winit event loop
/// with `ControlFlow::Poll` (continuous rendering), and runs the application.
fn main() {
    if let Err(err) = ae_editor::editor_logger::init() {
        let _ = std::io::Write::write_fmt(
            &mut std::io::stderr(),
            format_args!("[AeonEngine] Logger already initialized or failed to bind: {err}\n"),
        );
    }
    let cli_args = cli::CliArgs::from_env();
    log::info!(
        "Aeon Engine started. Active mode: {:?}, project: {:?}",
        cli_args.mode,
        cli_args.project_path
    );

    let event_loop = match build_event_loop() {
        Ok(loop_instance) => loop_instance,
        Err(err) => {
            log::error!("[AeonEngine] Failed to initialize event loop: {err}");
            let _ = std::io::Write::write_fmt(
                &mut std::io::stderr(),
                format_args!(
                    "\n==================================================================\n\
                     ❌ [Aeon Engine Critical Startup Error]\n\
                     Failed to initialize platform event loop:\n\n\
                     {err}\n\n\
                     Please verify that an active display server (X11 / Wayland / Win32 / Cocoa) is available.\n\
                     ==================================================================\n\n"
                ),
            );
            std::process::exit(1);
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AeApp::new(cli_args);
    if let Err(err) = event_loop.run_app(&mut app) {
        log::error!("[AeonEngine] Event loop terminated with error: {err}");
    }
}

/// Computes the remaining frame pacing sleep duration for a capped frame rate limit.
///
/// Returns `Some(target_duration - elapsed)` whenever `elapsed < target_duration` and
/// `target_duration` is positive, ensuring capped 60/120 FPS modes do not unlock and
/// spin under continuous polling loops.
#[must_use]
fn compute_frame_pacing_sleep(
    elapsed: std::time::Duration,
    target_duration: std::time::Duration,
) -> Option<std::time::Duration> {
    if !target_duration.is_zero() && elapsed < target_duration {
        Some(target_duration - elapsed)
    } else {
        None
    }
}

#[cfg(test)]
mod frame_pacing_tests {
    use super::compute_frame_pacing_sleep;
    use std::time::Duration;

    #[test]
    fn test_compute_frame_pacing_sleep_returns_remaining_budget_for_120_fps() {
        let target_120 = Duration::from_secs_f32(1.0 / 120.0);
        let elapsed = Duration::from_millis(2);
        let remaining = compute_frame_pacing_sleep(elapsed, target_120)
            .expect("120 FPS budget with 2ms elapsed must sleep for remaining frame budget");
        assert_eq!(remaining, target_120 - elapsed);
    }

    #[test]
    fn test_compute_frame_pacing_sleep_rejects_expired_and_zero_budgets() {
        let target_120 = Duration::from_secs_f32(1.0 / 120.0);
        assert_eq!(compute_frame_pacing_sleep(target_120, target_120), None);
        assert_eq!(
            compute_frame_pacing_sleep(Duration::from_millis(16), target_120),
            None
        );
        assert_eq!(
            compute_frame_pacing_sleep(Duration::from_millis(1), Duration::ZERO),
            None
        );
    }
}