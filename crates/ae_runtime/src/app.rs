// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone editor-free desktop player shell executing compiled worlds from `.aepak`
//! or workspace folders using the shared [`crate::simulation`] runtime loop.

use crate::behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
use crate::hud::{InGameHudState, RuntimeHudOverlay, RuntimePauseAction};
use crate::simulation::{
    SaveLoadDispatchContext, process_save_load_requests, step_play_state_and_pause,
    update_play_camera, update_skeletal_animations,
};
use crate::static_plugin::attach_static_game_logic;
use crate::world_loader::{
    LoadedRuntimeWorld, RuntimeLaunchArgs, discover_vfs_and_entry, load_world_into_runtime,
};
use ae_2d::mode::ActiveDimensionMode;
use ae_audio::AudioManager;
use ae_core::camera::{Camera, ProjectionMode};
use ae_core::events::DynamicEventBus;
use ae_core::input::{InputManager, KeyCode};
use ae_core::modules::EngineModule;
use ae_core::spatial::SpatialGrid;
use ae_core::state::StateManager;
use ae_core::time::Time;
use ae_core::transform::update_hierarchy_transforms;
use ae_physics::PhysicsWorld;
use ae_plugin_api::{Position, Scale};
use ae_plugin_host::PluginManager;
use ae_renderer::asset::{AssetHandle, AssetManager, AssetStorage};
use ae_renderer::render::types::{LightUniform, RenderScene};
use ae_renderer::render::{
    OverlayRenderer, RenderFrameParams, RenderOptions, RenderState, TextureAsset, ViewportRect,
};
use ae_scene::{RuntimeStreamingManager, bind_entity_assets_from_binding};
use std::collections::HashSet;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

/// Complete editor-free runtime game session state.
pub struct RuntimeSession {
    render_state: RenderState,
    window: Arc<Window>,
    camera: Camera,
    time: Time,
    input: InputManager,
    mouse_delta: (f32, f32),
    mouse_sensitivity: f32,
    world: ae_ecs::World,
    asset_manager: AssetManager,
    physics_world: PhysicsWorld,
    audio_manager: AudioManager,
    spatial_grid: SpatialGrid,
    resources: ae_core::Resources,
    event_bus: DynamicEventBus,
    state_manager: StateManager,
    hud: InGameHudState,
    hud_overlay: RuntimeHudOverlay,
    plugin_manager: PluginManager,
    dimension_mode: ActiveDimensionMode,
    sprite_2d: Option<(
        ae_2d::renderer::Sprite2DPipeline,
        ae_2d::renderer::SpriteBatcher,
    )>,
    streaming: RuntimeStreamingManager,
    world_info: LoadedRuntimeWorld,
    physics_sync_dirty: bool,
    start_instant: std::time::Instant,
    is_cursor_grabbed: bool,
    should_exit: bool,
}

impl RuntimeSession {
    /// Initializes WGPU rendering, mounts VFS/PAK resources, spawns the entry world,
    /// and attaches statically linked gameplay logic (or optional dynamic library fallback).
    ///
    /// # Errors
    /// Returns an error string if graphics adapter initialization or world decoding fails.
    pub async fn new(
        window: Arc<Window>,
        args: &RuntimeLaunchArgs,
        embedded_pak: Option<&'static [u8]>,
    ) -> Result<Self, String> {
        let (mut render_state, mut camera) = RenderState::new(Arc::clone(&window)).await?;
        render_state.graphics_settings.fps_limit =
            ae_renderer::graphics_settings::FpsLimit::Limit60;
        render_state.apply_settings_changes();
        let (vfs, entry_world, save_games_dir) = discover_vfs_and_entry(args, embedded_pak)?;

        let mut audio_manager = AudioManager::new();
        audio_manager.set_vfs_provider(Some(Arc::clone(&vfs)));

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

        let sprite_2d = if world_info.is_2d {
            let pipeline = ae_2d::renderer::Sprite2DPipeline::new(
                &render_state.device,
                render_state.config.format,
                None,
                1,
            );
            let batcher = ae_2d::renderer::SpriteBatcher::new(
                &render_state.device,
                &render_state.queue,
                &pipeline,
            );
            camera.position = cgmath::Point3::new(0.0, 0.0, 10.0);
            camera.target = cgmath::Point3::new(0.0, 0.0, 0.0);
            camera.yaw = cgmath::Deg(-90.0).into();
            camera.pitch = cgmath::Deg(0.0).into();
            camera.mode = ProjectionMode::Orthographic;
            camera.ortho_scale = 10.0;
            Some((pipeline, batcher))
        } else {
            None
        };

        let mut plugin_manager = PluginManager::new();
        attach_static_game_logic(&mut plugin_manager);

        let mut input = InputManager::new();
        if let Ok(raw_input) = streaming.vfs().read_bytes("Config/input.json") {
            match std::str::from_utf8(&raw_input)
                .map_err(|e| e.to_string())
                .and_then(ae_core::action_map::ActionMap::from_json)
            {
                Ok(action_map) => {
                    log::info!(
                        "Loaded custom ActionMap '{}' from Config/input.json",
                        action_map.name
                    );
                    input.action_map = action_map;
                }
                Err(err) => {
                    log::warn!("Invalid Config/input.json in VFS, using default ActionMap: {err}");
                }
            }
        }

        let hud_overlay = RuntimeHudOverlay::new(
            &render_state.device,
            render_state.config.format,
            (render_state.size.width, render_state.size.height),
            window.scale_factor() as f32,
        );

        Ok(Self {
            render_state,
            window,
            camera,
            time: Time::new(),
            input,
            mouse_delta: (0.0, 0.0),
            mouse_sensitivity: 0.005,
            world,
            asset_manager,
            physics_world,
            audio_manager,
            spatial_grid: SpatialGrid::new(world_info.cell_size.max(128.0)),
            resources: ae_core::Resources::new(),
            event_bus: DynamicEventBus::new(),
            state_manager: StateManager::new(),
            hud: InGameHudState::new(),
            hud_overlay,
            plugin_manager,
            dimension_mode,
            sprite_2d,
            streaming,
            world_info,
            physics_sync_dirty: false,
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
        if self.render_state.size.width == 0 || self.render_state.size.height == 0 {
            return;
        }
        self.time.tick();

        let (was_paused, is_paused) = step_play_state_and_pause(
            &mut self.state_manager,
            &mut self.world,
            &mut self.event_bus,
            &self.input,
            self.time.delta_time,
        );
        if is_paused && !was_paused {
            self.set_cursor_grab(false);
            self.mouse_delta = (0.0, 0.0);
            self.input.clear_pressed_keys();
        } else if !is_paused && was_paused {
            if !self.world_info.is_2d {
                self.set_cursor_grab(true);
            }
            self.mouse_delta = (0.0, 0.0);
        }

        if self.event_bus.is_module_enabled(EngineModule::Physics) {
            while self.time.consume_fixed_step() {
                if !self.state_manager.is_paused() {
                    ae_core::transform::integrate_unconstrained_velocities(
                        &mut self.world,
                        self.time.fixed_time_step,
                    );
                    crate::simulation::step_player_movement(
                        &self.input,
                        &mut self.world,
                        &mut self.physics_world,
                        &self.camera,
                        self.world_info.is_2d,
                        self.time.fixed_time_step,
                    );
                    self.physics_world.step(
                        &mut self.world,
                        |handle| {
                            self.asset_manager
                                .get_physics_mesh_data(handle)
                                .map(|(v, i)| (v.as_slice(), i.as_slice()))
                        },
                        self.time.fixed_time_step,
                        &mut self.event_bus,
                    );
                }
            }
        } else {
            while self.time.consume_fixed_step() {}
        }

        if self.physics_sync_dirty {
            self.physics_sync_dirty = false;
            self.physics_world
                .sync_ecs_to_physics(&mut self.world, |handle| {
                    self.asset_manager
                        .get_physics_mesh_data(handle)
                        .map(|(v, i)| (v.as_slice(), i.as_slice()))
                });
        }

        if !self.state_manager.is_paused() {
            update_play_camera(
                &mut self.world,
                &mut self.camera,
                &mut self.mouse_delta,
                self.mouse_sensitivity,
            );
            self.spatial_grid.sync(&self.world);

            let active_vfs = Arc::clone(self.streaming.vfs());
            let render_state = &self.render_state;
            let report = self.streaming.step(
                &mut self.world,
                &mut self.asset_manager,
                [
                    self.camera.position.x,
                    self.camera.position.y,
                    self.camera.position.z,
                ],
                4,
                |world, asset_manager, entity, binding| {
                    bind_entity_assets_from_binding(
                        world,
                        render_state,
                        asset_manager,
                        Some(active_vfs.as_ref()),
                        entity,
                        binding,
                    );
                },
            );
            if report.spawned_entities > 0 || report.despawned_entities > 0 {
                update_hierarchy_transforms(&mut self.world);
                self.physics_sync_dirty = true;
            }

            let cam_fwd = self.camera.get_forward();
            update_gameplay_behaviors(BehaviorRunnerParams {
                world: &mut self.world,
                physics_world: &mut self.physics_world,
                input: &self.input,
                event_bus: &mut self.event_bus,
                camera_forward: cam_fwd,
                delta_time: self.time.delta_time,
            });

            self.plugin_manager.tick(
                &mut self.world,
                &mut self.resources,
                &mut self.event_bus,
                self.time.delta_time,
            );
        }

        let play_secs = self.start_instant.elapsed().as_secs();
        if process_save_load_requests(&mut SaveLoadDispatchContext {
            save_games_dir: &self.world_info.save_games_dir,
            world_guid: self.world_info.world_guid,
            cell_size: self.world_info.cell_size,
            play_time_secs: play_secs,
            world: &mut self.world,
            event_bus: &mut self.event_bus,
            streaming: Some(&mut self.streaming),
        }) {
            self.physics_sync_dirty = true;
        }

        self.hud
            .update_from_events(&mut self.world, &mut self.event_bus);

        let audio_enabled = self.event_bus.is_module_enabled(EngineModule::Audio);
        let cam_pos = ae_audio::Vec3::new(
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        );
        let right = self.camera.get_right();
        let cam_right = ae_audio::Vec3::new(right.x, right.y, right.z);
        self.audio_manager
            .update(&self.world, cam_pos, cam_right, audio_enabled);

        update_skeletal_animations(
            &mut self.world,
            &self.render_state,
            &mut self.asset_manager,
            self.time.delta_time,
        );

        self.render_frame();
        self.input.end_frame();
    }

    fn render_frame(&mut self) {
        let is_2d = self.dimension_mode.is_2d();
        let render_enabled = if is_2d {
            self.event_bus.is_module_enabled(EngineModule::Render2D)
        } else {
            self.event_bus.is_module_enabled(EngineModule::Render)
        };
        if render_enabled && !is_2d {
            self.spatial_grid.sync(&self.world);
        }
        update_hierarchy_transforms(&mut self.world);

        let empty_selected = HashSet::new();
        let mut scene = if render_enabled && !is_2d {
            RenderScene::extract(
                &self.world,
                &self.camera,
                &self.asset_manager,
                &empty_selected,
                None,
                &self.spatial_grid,
            )
        } else {
            RenderScene {
                light_uniform: LightUniform {
                    direction: [0.0, 1.0, 0.0],
                    grid_scale: 1.0,
                    color: [1.0, 1.0, 1.0],
                    _padding2: 0,
                    ambient_color: [0.1, 0.1, 0.15],
                    _padding3: 0,
                    fog_params: [0.0; 4],
                },
                triangle_instances: Vec::new(),
                cube_instances: Vec::new(),
                sphere_instances: Vec::new(),
                cylinder_instances: Vec::new(),
                capsule_instances: Vec::new(),
                torus_instances: Vec::new(),
                transparent_objs: Vec::new(),
                model_instance_data: std::collections::HashMap::new(),
                selected_primitive_instances: Vec::new(),
                selected_model_instances: Vec::new(),
                visible_entities: Vec::new(),
                culled_entities_count: 0,
            }
        };

        if render_enabled {
            self.resources.insert(ae_plugin_api::VisibleEntities {
                entities: std::mem::take(&mut scene.visible_entities),
            });
        }

        let mut sprite_overlay = None;
        if render_enabled
            && is_2d
            && let Some((pipeline, batcher)) = &mut self.sprite_2d
        {
            batcher.begin_frame();
            let view_proj = self.camera.build_view_projection_matrix();
            batcher.update_camera(&self.render_state.queue, &view_proj);
            for (pos, scale, sprite, hidden_opt) in self
                .world
                .query::<(
                    &Position,
                    &Scale,
                    &ae_2d::components::SpriteRenderer,
                    Option<&ae_plugin_api::Hidden>,
                )>()
                .iter()
            {
                if hidden_opt.is_some() {
                    continue;
                }
                let tex_id = sprite
                    .texture
                    .map(|t| {
                        use slotmap::Key;
                        t.data().as_ffi() as u32
                    })
                    .unwrap_or(0);
                let key = ae_2d::components::SpriteSortKey::from_layer_order_texture(
                    sprite.sorting_layer,
                    sprite.order_in_layer,
                    tex_id,
                );
                let model =
                    cgmath::Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, pos.z))
                        * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, 1.0);
                let instance = ae_2d::renderer::SpriteInstance {
                    model_matrix: model.into(),
                    uv_rect: sprite.uv_rect,
                    tint: sprite.tint,
                };
                batcher.push_instance(key, instance);
            }
            batcher.finish_and_upload(&self.render_state.device, &self.render_state.queue);
            sprite_overlay = Some(RuntimeSprite2DOverlay {
                pipeline,
                batcher,
                textures: &self.asset_manager.textures,
            });
        }

        let mut overlays: Vec<&dyn OverlayRenderer> = Vec::new();
        if let Some(ref ov) = sprite_overlay {
            overlays.push(ov);
        }

        let options = RenderOptions {
            grid_enabled: false,
            grid_scale: 1.0,
            wireframe_enabled: false,
            scale_factor: self.window.scale_factor() as f32,
            is_2d_mode: is_2d,
        };

        let is_paused = self.state_manager.is_paused();
        let pause_action = self.hud_overlay.prepare_and_poll_action(
            &self.world,
            is_paused,
            &self.render_state.device,
            &self.render_state.queue,
        );

        let hud_overlay = &mut self.hud_overlay;
        let mut ui_cb = |device: &wgpu::Device,
                         queue: &wgpu::Queue,
                         encoder: &mut wgpu::CommandEncoder,
                         _window: &Window,
                         surface_view: &wgpu::TextureView,
                         _viewport_view: Option<&wgpu::TextureView>,
                         _surface_size: (u32, u32)| {
            hud_overlay.render(device, queue, encoder, surface_view);
            // Return zero rect so RenderState does not allocate an offscreen viewport_texture
            // and renders the 3D/2D scene directly onto surface_view behind the HUD.
            ViewportRect::default()
        };

        let _ = self.render_state.render(RenderFrameParams {
            scene,
            camera: &self.camera,
            overlays: &overlays,
            editor_overlays: &[],
            asset_manager: &self.asset_manager,
            enabled_modules: &self.event_bus.enabled_modules,
            options: &options,
            ui_renderer: Some(&mut ui_cb),
        });

        match pause_action {
            RuntimePauseAction::Resume => {
                self.state_manager.pop();
                let mut cmd = ae_ecs::EntityCommandBuffer::new();
                self.state_manager
                    .update(&mut self.world, &mut self.event_bus, &mut cmd, 0.0);
                cmd.apply(&mut self.world);
                if !is_2d {
                    self.set_cursor_grab(true);
                }
                self.mouse_delta = (0.0, 0.0);
            }
            RuntimePauseAction::Exit => {
                self.should_exit = true;
            }
            RuntimePauseAction::None => {}
        }
    }
}

struct RuntimeSprite2DOverlay<'a> {
    pipeline: &'a ae_2d::renderer::Sprite2DPipeline,
    batcher: &'a ae_2d::renderer::SpriteBatcher,
    textures: &'a AssetStorage<TextureAsset>,
}

impl OverlayRenderer for RuntimeSprite2DOverlay<'_> {
    fn draw_overlay<'b>(&'b self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'b>) {
        self.batcher.render(pass, self.pipeline, |id| {
            let handle = AssetHandle::from(slotmap::KeyData::from_ffi(id as u64));
            self.textures.get(handle).map(|t| &t.bind_group)
        });
    }
}

struct DesktopPlayerApp {
    args: RuntimeLaunchArgs,
    embedded_pak: Option<&'static [u8]>,
    session: Option<RuntimeSession>,
    last_frame_time: std::time::Instant,
}

impl ApplicationHandler for DesktopPlayerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.session.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("Aeon Runtime")
            .with_maximized(true);
        let Ok(window) = event_loop.create_window(attrs) else {
            event_loop.exit();
            return;
        };
        let window = Arc::new(window);
        match pollster::block_on(RuntimeSession::new(window, &self.args, self.embedded_pak)) {
            Ok(session) => self.session = Some(session),
            Err(err) => {
                log::error!("Aeon Runtime boot failed: {err}");
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
        let is_paused = session.state_manager.is_paused();
        session.hud_overlay.handle_window_event(&event, is_paused);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                session.render_state.resize(size);
                session.hud_overlay.resize(
                    (size.width, size.height),
                    session.window.scale_factor() as f32,
                );
                if size.height > 0 {
                    session.camera.aspect = size.width as f32 / size.height as f32;
                }
            }
            WindowEvent::Focused(false) => session.input.clear_pressed_keys(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape && event.state == ElementState::Pressed {
                        session.set_cursor_grab(false);
                    }
                    session.input.process_key_event(code, event.state);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left
                    && state == ElementState::Pressed
                    && !session.is_cursor_grabbed
                    && !session.world_info.is_2d
                    && !is_paused
                {
                    session.set_cursor_grab(true);
                }
                session.input.process_mouse_button_event(button, state);
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
            session.mouse_delta.0 += delta.0 as f32;
            session.mouse_delta.1 += delta.1 as f32;
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(session) = self.session.as_ref() {
            match session.render_state.graphics_settings.fps_limit {
                ae_renderer::graphics_settings::FpsLimit::Limit60 => {
                    let target = std::time::Duration::from_secs_f32(1.0 / 60.0);
                    let elapsed = self.last_frame_time.elapsed();
                    if elapsed < target {
                        spin_sleep::sleep(target - elapsed);
                    }
                }
                ae_renderer::graphics_settings::FpsLimit::Limit120 => {
                    let target = std::time::Duration::from_secs_f32(1.0 / 120.0);
                    let elapsed = self.last_frame_time.elapsed();
                    if elapsed < target {
                        spin_sleep::sleep(target - elapsed);
                    }
                }
                ae_renderer::graphics_settings::FpsLimit::Uncapped => {}
            }
            self.last_frame_time = std::time::Instant::now();
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
        last_frame_time: std::time::Instant::now(),
    };
    event_loop
        .run_app(&mut app)
        .map_err(|e| format!("EventLoop error: {e}"))
}