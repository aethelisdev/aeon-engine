// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone editor-free desktop player shell executing compiled worlds from `.aepak`
//! or workspace folders over shared [`ae_app::RenderHost`] and [`ae_app::SimulationHost`].

use crate::static_plugin::attach_static_game_logic;
use crate::world_loader::{
    LoadedRuntimeWorld, RuntimeLaunchArgs, discover_vfs_and_entry, load_world_into_runtime,
};
use ae_2d::ActiveDimensionMode;
use ae_app::{
    CoreWorld, PauseTransition, PlayFrameParams, RenderHost, SimulationHost, prepare_2d_overlay,
};
use ae_audio::AudioManager;
use ae_camera::ViewportRect;
use ae_input::{ActionMap, InputManager, KeyCode};
use ae_physics::PhysicsWorld;
use ae_plugin_host::PluginManager;
use ae_renderer::asset::AssetManager;
use ae_renderer::render::{OverlayRenderer, RenderFrameParams, RenderOptions, RenderState};
use ae_time::FramePacer;
use ae_ui::{RuntimeHudOverlay, RuntimePauseAction};
use std::collections::HashSet;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

/// Complete editor-free runtime game session composed from [`ae_app`] modular hosts.
pub struct RuntimeSession {
    core: CoreWorld,
    render: RenderHost,
    sim: SimulationHost,
    window: Arc<Window>,
    hud_overlay: RuntimeHudOverlay,
    world_info: LoadedRuntimeWorld,
    mouse_sensitivity: f32,
    start_instant: std::time::Instant,
    is_cursor_grabbed: bool,
    should_exit: bool,
}

impl RuntimeSession {
    /// Initializes WGPU rendering, mounts VFS/PAK resources, spawns the entry world,
    /// and attaches statically linked gameplay logic.
    ///
    /// # Errors
    /// Returns an error string if graphics adapter initialization or world decoding fails.
    pub async fn new(
        window: Arc<Window>,
        args: &RuntimeLaunchArgs,
        embedded_pak: Option<&'static [u8]>,
    ) -> Result<Self, String> {
        let win_size = window.inner_size();
        let (mut render_state, camera) =
            RenderState::new(Arc::clone(&window), win_size.width, win_size.height).await?;
        let scale = window.scale_factor() as f32;
        render_state.graphics_settings.fps_limit =
            ae_renderer::graphics_settings::FpsLimit::Limit60;
        let hud_overlay = RuntimeHudOverlay::new(
            &render_state.device,
            render_state.config.format,
            (win_size.width, win_size.height),
            scale,
        );

        let (vfs, entry_world, save_games_dir) = discover_vfs_and_entry(args, embedded_pak)?;
        let mut world = ae_ecs::World::new();
        let mut asset_manager = AssetManager::new();
        let mut physics_world = PhysicsWorld::new();

        let (world_info, streaming) = load_world_into_runtime(
            vfs,
            &entry_world,
            save_games_dir,
            &render_state,
            &mut asset_manager,
            &mut world,
            &mut physics_world,
        )?;

        let dimension_mode = if world_info.is_2d {
            ActiveDimensionMode::Mode2D
        } else {
            ActiveDimensionMode::Mode3D
        };

        let render = RenderHost::new(
            render_state,
            asset_manager,
            camera,
            world_info.cell_size,
            dimension_mode,
        );

        let mut plugin_manager = PluginManager::new();
        attach_static_game_logic(&mut plugin_manager);

        let mut input = InputManager::new();
        if let Ok(raw_input) = streaming.vfs().read_bytes("Config/input.json") {
            match std::str::from_utf8(&raw_input)
                .map_err(|e| e.to_string())
                .and_then(ActionMap::from_json)
            {
                Ok(action_map) => {
                    log::info!("Loaded custom ActionMap '{}'", action_map.name);
                    input.action_map = action_map;
                }
                Err(err) => log::warn!("Ignoring invalid Config/input.json: {err}"),
            }
        }

        let mut audio_manager = AudioManager::new();
        audio_manager.set_vfs_provider(Some(Arc::clone(streaming.vfs())));

        let mut sim = SimulationHost::new(audio_manager, plugin_manager, Some(streaming));
        sim.physics_world = physics_world;

        let core = CoreWorld::with_loaded(world, input, world_info.world_guid);

        Ok(Self {
            core,
            render,
            sim,
            window,
            hud_overlay,
            world_info,
            mouse_sensitivity: 0.005,
            start_instant: std::time::Instant::now(),
            is_cursor_grabbed: false,
            should_exit: false,
        })
    }

    fn set_cursor_grab(&mut self, grabbed: bool) {
        if self.is_cursor_grabbed == grabbed {
            return;
        }
        let mode = if grabbed {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        if self.window.set_cursor_grab(mode).is_err() && grabbed {
            let _ = self.window.set_cursor_grab(CursorGrabMode::Confined);
        }
        self.window.set_cursor_visible(!grabbed);
        self.is_cursor_grabbed = grabbed;
    }

    fn step_frame(&mut self) {
        if self.render.render_state.size.width == 0 || self.render.render_state.size.height == 0 {
            return;
        }
        self.core.time.tick();

        self.sim
            .step_fixed_physics(&mut self.core, &self.render, true);
        self.sim
            .sync_dirty_physics(&mut self.core.world, &self.render.asset_manager);

        let play_secs = self.start_instant.elapsed().as_secs();
        let pause_transition = self.sim.step_play_frame(
            &mut self.core,
            &mut self.render,
            &PlayFrameParams {
                mouse_sensitivity: self.mouse_sensitivity,
                save_games_dir: &self.world_info.save_games_dir,
                cell_size: self.world_info.cell_size,
                play_time_secs: play_secs,
                streaming_budget_per_frame: 4,
                tick_plugins: true,
            },
        );

        match pause_transition {
            PauseTransition::EnteredPause => self.set_cursor_grab(false),
            PauseTransition::ExitedPause if !self.world_info.is_2d => self.set_cursor_grab(true),
            PauseTransition::ExitedPause | PauseTransition::Unchanged { .. } => {}
        }

        self.render_frame();
        self.core.input.end_frame();
    }

    fn render_frame(&mut self) {
        let live_size = self.window.inner_size();
        if live_size.width > 0
            && live_size.height > 0
            && (live_size.width != self.render.render_state.size.width
                || live_size.height != self.render.render_state.size.height)
        {
            self.render
                .resize_dimensions(live_size.width, live_size.height);
        }
        let empty_selected = HashSet::new();
        let prepared = self.render.prepare_scene(
            &mut self.core.world,
            &mut self.core.resources,
            &self.core.event_bus,
            &empty_selected,
            None,
        );

        let is_paused = self.sim.state_manager.is_paused();
        let pause_action = self
            .hud_overlay
            .prepare_and_poll_action(&self.core.world, is_paused);

        let sprite_overlay = prepare_2d_overlay(
            self.render.sprite_2d.as_mut(),
            &self.render.render_state,
            &self.core.world,
            &self.render.camera,
            &self.render.asset_manager,
            prepared.render_enabled,
            prepared.is_2d,
        );

        let mut overlays: Vec<&dyn OverlayRenderer> = Vec::new();
        if let Some(ref ov) = sprite_overlay {
            overlays.push(ov);
        }

        let options = RenderOptions {
            grid_enabled: false,
            grid_scale: 1.0,
            wireframe_enabled: false,
            scale_factor: self.window.scale_factor() as f32,
            is_2d_mode: prepared.is_2d,
        };

        let hud_overlay = &mut self.hud_overlay;
        let mut ui_cb = |device: &wgpu::Device,
                         queue: &wgpu::Queue,
                         encoder: &mut wgpu::CommandEncoder,
                         surface_view: &wgpu::TextureView,
                         _viewport_view: Option<&wgpu::TextureView>,
                         _surface_size: (u32, u32)| {
            hud_overlay.render(device, queue, encoder, surface_view);
            // Zero rect keeps rendering directly onto surface_view without offscreen viewport allocation
            ViewportRect::default()
        };

        let _ = self.render.render_state.render(RenderFrameParams {
            scene: prepared.scene,
            camera: &self.render.camera,
            overlays: &overlays,
            editor_overlays: &[],
            asset_manager: &self.render.asset_manager,
            enabled_modules: &self.core.event_bus.enabled_modules,
            options: &options,
            ui_renderer: Some(&mut ui_cb),
        });
        drop(overlays);

        match pause_action {
            RuntimePauseAction::Resume => {
                self.sim.state_manager.pop();
                let mut cmd = ae_ecs::EntityCommandBuffer::new();
                self.sim.state_manager.update(
                    &mut self.core.world,
                    &mut self.core.event_bus,
                    &mut cmd,
                    0.0,
                );
                cmd.apply(&mut self.core.world);
                if !prepared.is_2d {
                    self.set_cursor_grab(true);
                }
                self.core.input.clear_mouse_delta();
            }
            RuntimePauseAction::Exit => {
                self.should_exit = true;
            }
            RuntimePauseAction::None => {}
        }
    }
}

struct DesktopPlayerApp {
    args: RuntimeLaunchArgs,
    embedded_pak: Option<&'static [u8]>,
    session: Option<RuntimeSession>,
    frame_pacer: FramePacer,
}

impl ApplicationHandler for DesktopPlayerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.session.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("Aeon Engine Runtime")
            .with_maximized(true);
        let Ok(win) = event_loop.create_window(attrs) else {
            event_loop.exit();
            return;
        };
        let window = Arc::new(win);
        match pollster::block_on(RuntimeSession::new(window, &self.args, self.embedded_pak)) {
            Ok(session) => {
                self.session = Some(session);
                self.frame_pacer.reset();
            }
            Err(err) => {
                log::error!("Runtime initialization failed: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let is_paused = session.sim.state_manager.is_paused();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                session.render.resize_dimensions(size.width, size.height);
                session.hud_overlay.resize(
                    (size.width, size.height),
                    session.window.scale_factor() as f32,
                );
            }
            WindowEvent::Focused(false) => session.core.input.clear_pressed_keys(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape && event.state == ElementState::Pressed {
                        session.set_cursor_grab(false);
                    }
                    session.core.input.process_key_event(code, event.state);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                session
                    .hud_overlay
                    .handle_pointer_moved(position.x, position.y, is_paused);
                session
                    .core
                    .input
                    .process_cursor_moved(position.x, position.y);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        (pos.x as f32 * 0.02, pos.y as f32 * 0.02)
                    }
                };
                session.core.input.process_mouse_wheel(dx, dy);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    session
                        .hud_overlay
                        .handle_left_mouse_input(state == ElementState::Pressed, is_paused);
                }
                if button == winit::event::MouseButton::Left
                    && state == ElementState::Pressed
                    && !session.is_cursor_grabbed
                    && !session.world_info.is_2d
                    && !is_paused
                {
                    session.set_cursor_grab(true);
                }
                session.core.input.process_mouse_button_event(button, state);
            }
            WindowEvent::RedrawRequested => {
                session.step_frame();
                if session.should_exit {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(session) = self.session.as_mut()
            && let DeviceEvent::MouseMotion { delta } = event
        {
            session
                .core
                .input
                .add_mouse_delta(delta.0 as f32, delta.1 as f32);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(session) = self.session.as_ref() {
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
            session.window.request_redraw();
        }
    }
}

/// Runs the standalone editor-free desktop player window event loop.
///
/// # Errors
/// Returns a diagnostic string if `winit` event loop creation or execution fails.
pub fn run_desktop_player(
    args: RuntimeLaunchArgs,
    embedded_pak: Option<&'static [u8]>,
) -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|e| format!("EventLoop init failed: {e}"))?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = DesktopPlayerApp {
        args,
        embedded_pak,
        session: None,
        frame_pacer: FramePacer::new(),
    };
    event_loop
        .run_app(&mut app)
        .map_err(|e| format!("EventLoop error: {e}"))
}