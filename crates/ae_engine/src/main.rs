// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod debug_renderer;
pub mod engine;
pub mod events;
pub mod icon;
/// Aeon Engine — Main Entry Point
/// Initializes the winit event loop, creates the application window with embedded icon,
/// and delegates all event processing to `AeEngine`. Implements `ApplicationHandler` for
/// the modern winit 0.30+ callback-based architecture.
mod importer;
pub mod profiler;
pub mod render_pass;
pub mod scene;
pub mod update;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
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
/// Substitutes the standard allocator with `Jemalloc` under Linux native execution,
/// while leaving Windows and macOS defaults completely untouched.
#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Application shell that owns the engine and manages the winit event loop.
/// Uses `Option<AeEngine>` to support deferred initialization (engine is created
/// in `resumed()` when the window is ready). Also tracks frame timing for FPS limiting.
struct AeApp {
    engine: Option<AeEngine>,
    last_frame_time: std::time::Instant,
}

impl AeApp {
    fn new() -> Self {
        Self {
            engine: None,
            last_frame_time: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for AeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            // Embed the icon directly in the binary for cross-platform reliability
            let icon_bytes = include_bytes!("../assets/icon/aeicon.png");
            let icon = icon::load_icon_from_memory(icon_bytes);

            let window_attributes = {
                let attr = WindowAttributes::default()
                    .with_title("Aeon Engine")
                    .with_maximized(true)
                    .with_visible(false)
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
                        "com.aeengine.Editor",
                        "ae_engine",
                    );
                    WindowAttributesExtX11::with_name(attr, "com.aeengine.Editor", "ae_engine")
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

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            // Initialize Core Engine State
            let engine = pollster::block_on(AeEngine::new(window.clone()));
            window.set_visible(true);
            self.engine = Some(engine);
            self.last_frame_time = std::time::Instant::now();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let engine = match self.engine.as_mut() {
            Some(e) => e,
            None => return,
        };

        let window = &engine.render_state.window;

        // Event processing
        let consumed = engine.ui.handle_event(window, &event);

        match &event {
            WindowEvent::CloseRequested => {
                // Drop the engine BEFORE exiting to release WGPU surface/device.
                // On Linux/Vulkan, winit sends extra redraws after exit() which
                // would crash if the GPU resources are still partially alive.
                self.engine = None;
                event_loop.exit();
                return;
            }
            WindowEvent::Resized(physical_size) => engine.resize(*physical_size),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                // Professional Focus Check: Always process Escape key so Play mode ESC pause is never blocked by Egui
                if *key == winit::keyboard::KeyCode::Escape || !consumed {
                    engine.input.process_key_event(*key, *state);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                // Get cursor position and check if it's over any UI area
                let (cursor_x, cursor_y) = engine.editor.last_cursor_pos;
                let scale = engine.render_state.window.scale_factor() as f32;
                let logical_pos = egui::pos2(cursor_x as f32 / scale, cursor_y as f32 / scale);

                // Only scroll 3D camera if cursor is NOT over any UI rect
                if !engine.ui.is_point_over_ui_rects(logical_pos) {
                    engine.handle_mouse_scroll(delta, cursor_x, cursor_y);
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                // Background Interaction: Pass all mouse input to the engine.
                // The engine uses internal geometric checks (viewport_rect / is_pointer_over_area) to safely separate UI vs 3D.
                engine.handle_mouse_click(*button, *state);
            }

            WindowEvent::Focused(focused) => {
                engine.handle_focus_change(*focused);
            }

            WindowEvent::RedrawRequested => {
                // Handles OS-initiated draw events (e.g. during window resizing or dragging on Win32 modal loops).
                // Ensures perfect window repainting and immediate Win32 client area validation, preventing UI freezing.
                self.update_and_render(event_loop);
            }

            // All other events pass through normally
            other_event => {
                engine.handle_window_event(&other_event, consumed);
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

                    // Measure elapsed time since the previous frame
                    let elapsed = self.last_frame_time.elapsed();
                    if elapsed < target_duration {
                        spin_sleep::sleep(target_duration - elapsed);
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

        // Scene IO triggers
        engine.ui.process_scene_dialogs();

        if engine.ui.should_save_scene {
            engine.ui.should_save_scene = false;
            let path = engine
                .ui
                .pending_save_path
                .take()
                .unwrap_or_else(|| std::path::PathBuf::from(&engine.ui.active_scene_path));
            let path_str = path.to_string_lossy();
            if let Err(e) = crate::scene::save_scene(engine, &path_str) {
                engine.ui.status_message = Some((
                    vec![(format!("Save Error: {}", e), egui::Color32::RED)],
                    std::time::Instant::now(),
                ));
            } else {
                engine.ui.status_message = Some((
                    vec![(
                        format!("Scene saved to {}", path_str),
                        egui::Color32::LIGHT_BLUE,
                    )],
                    std::time::Instant::now(),
                ));
            }
        }
        if engine.ui.should_load_scene {
            engine.ui.should_load_scene = false;
            let path = engine
                .ui
                .pending_load_path
                .take()
                .unwrap_or_else(|| std::path::PathBuf::from(&engine.ui.active_scene_path));
            let path_str = path.to_string_lossy();
            if let Err(e) = crate::scene::load_scene(engine, &path_str) {
                engine.ui.status_message = Some((
                    vec![(format!("Load Error: {}", e), egui::Color32::RED)],
                    std::time::Instant::now(),
                ));
                engine.ui.is_loading_assets = false;
            }
        }

        // Exit trigger
        if engine.ui.should_exit {
            self.engine = None;
            event_loop.exit();
            return;
        }

        crate::scene::process_async_scene_load(engine);
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
                eprintln!("Render error: {:?}", e);
            }
        }
        engine.input.end_frame();
        log::trace!("Render frame end");
    }
}

/// Entry point: initializes the dual-output logger, creates the winit event loop
/// with `ControlFlow::Poll` (continuous rendering), and runs the application.
fn main() {
    ae_editor::editor_logger::init().unwrap();
    log::info!("Aeon Engine started.");

    let event_loop = EventLoop::new().expect("Failed to create Event Loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AeApp::new();
    let _ = event_loop.run_app(&mut app);
}