// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod actions;
pub mod cli;
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
    frame_pacer: ae_time::FramePacer,
}

impl AeApp {
    fn new(cli_args: cli::CliArgs) -> Self {
        Self {
            cli_args,
            engine: None,
            frame_pacer: ae_time::FramePacer::new(),
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
            self.frame_pacer.reset();
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
            let target_fps = engine.render_state.graphics_settings.fps_limit.target_hz();
            if let Some(remaining) = self.frame_pacer.remaining_sleep_for_fps(target_fps) {
                spin_sleep::sleep(remaining);
            }
            self.frame_pacer.reset();
            self.update_and_render(event_loop);
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
        actions::scene_io::process_pending_scene_io(engine);

        // Exit trigger
        if engine.editor.should_exit {
            self.engine = None;
            event_loop.exit();
            return;
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