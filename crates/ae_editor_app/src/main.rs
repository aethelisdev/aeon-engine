// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Aeon Studio IDE Desktop Executable Entry Point.
//!
//! Initializes logging, creates the frameless winit window, and drives [`EditorSession`]
//! with [`ae_time::FramePacer`].

pub mod actions;
pub mod cli;
pub mod events;
pub mod icon;
#[cfg(feature = "mcp")]
pub mod mcp_host;
pub mod render_pass;
pub mod session;
pub mod update;

use session::EditorSession;
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

#[cfg(all(target_os = "linux", not(target_os = "android")))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

struct EditorAppRunner {
    cli_args: cli::CliArgs,
    session: Option<EditorSession>,
    frame_pacer: ae_time::FramePacer,
}

impl EditorAppRunner {
    fn new(cli_args: cli::CliArgs) -> Self {
        Self {
            cli_args,
            session: None,
            frame_pacer: ae_time::FramePacer::new(),
        }
    }

    fn update_and_render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if session.render.render_state.size.width == 0
            || session.render.render_state.size.height == 0
        {
            return;
        }

        log::trace!("Render frame start");
        session.editor.poll_scene_dialogs();
        actions::scene_io::process_pending_scene_io(session);

        if session.editor.should_exit {
            self.session = None;
            event_loop.exit();
            return;
        }

        session.update();
        match session.render() {
            Ok(()) => {}
            Err(ae_renderer::render::RenderError::SurfaceLost) => {
                std::hint::cold_path();
                let size = session.render.render_state.size;
                session.resize(winit::dpi::PhysicalSize::new(size.width, size.height));
            }
            Err(ae_renderer::render::RenderError::OutOfMemory) => {
                std::hint::cold_path();
                event_loop.exit();
            }
            Err(e) => {
                std::hint::cold_path();
                let _ = std::io::Write::write_fmt(
                    &mut std::io::stderr(),
                    format_args!("Render error: {e:?}\n"),
                );
            }
        }
        session.core.input.end_frame();
        session.poll_inspect_requests();
        log::trace!("Render frame end");
    }
}

impl ApplicationHandler for EditorAppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.session.is_some() {
            return;
        }
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
                log::error!("[AeEditorApp] Failed to create OS window: {err}");
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

        let session = match pollster::block_on(EditorSession::new(
            Arc::clone(&window),
            self.cli_args.clone(),
        )) {
            Ok(s) => s,
            Err(err) => {
                log::error!("[AeEditorApp] Critical initialization failure: {err}");
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
        self.session = Some(session);
        self.frame_pacer.reset();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // Drop session BEFORE exiting so GPU surface/device tears down before Wayland/X11 cleanup
                self.session = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.update_and_render(event_loop);
            }
            ref other_event => {
                if let Some(session) = self.session.as_mut() {
                    session.process_window_event(other_event);
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
        if let Some(session) = self.session.as_mut() {
            session.handle_device_event(&event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(session) = self.session.as_ref() {
            if session.editor.should_exit {
                self.session = None;
                event_loop.exit();
                return;
            }
            let target_fps = session
                .render
                .render_state
                .graphics_settings
                .fps_limit
                .target_hz();
            if let Some(remaining) = self.frame_pacer.remaining_sleep_for_fps(target_fps) {
                spin_sleep::sleep(remaining);
            }
            self.frame_pacer.reset();
            session.window.window.request_redraw();
        }
    }
}

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

fn main() {
    if let Err(err) = ae_editor::editor_logger::init() {
        let _ = std::io::Write::write_fmt(
            &mut std::io::stderr(),
            format_args!("[AeEditorApp] Logger already initialized or failed to bind: {err}\n"),
        );
    }
    let cli_args = cli::CliArgs::from_env();
    log::info!(
        "Aeon Editor App started. Active mode: {:?}, project: {:?}",
        cli_args.mode,
        cli_args.project_path
    );

    let event_loop = match build_event_loop() {
        Ok(el) => el,
        Err(err) => {
            log::error!("[AeEditorApp] Failed to build window event loop: {err}");
            let _ = std::io::Write::write_fmt(
                &mut std::io::stderr(),
                format_args!(
                    "\n==================================================================\n\
                     ❌ [Aeon Engine Critical Startup Error]\n\
                     Failed to create window event loop:\n\n\
                     {err}\n\
                     ==================================================================\n\n"
                ),
            );
            std::process::exit(1);
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = EditorAppRunner::new(cli_args);
    if let Err(err) = event_loop.run_app(&mut runner) {
        log::error!("[AeEditorApp] Event loop exited with error: {err}");
        std::process::exit(1);
    }
}