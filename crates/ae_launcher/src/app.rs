// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::path::PathBuf;
use std::sync::Arc;

use irisui::core::geometry::{Point, Size};
use irisui::core::tree::UiTree;
use irisui::layout::LayoutEngine;
use irisui::text::{TextRenderer, TextSection, TextSystem};
use irisui::wgpu_backend::{DrawCommandList, IrisRenderer};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::project::ProjectRegistry;
use crate::spawner::launch_engine_and_exit;
use crate::ui::{LauncherAction, LauncherUiState, build_launcher_ui};

/// Main application controller managing Winit event loop, WGPU surface, and Iris UI rendering for the launcher.
pub struct LauncherApp {
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,

    tree: UiTree,
    layout_engine: LayoutEngine,
    renderer: Option<IrisRenderer>,
    text_system: TextSystem,
    text_renderer: Option<TextRenderer>,
    command_list: DrawCommandList,

    cursor_pos: Point,
    screen_size: glam::Vec2,

    registry: ProjectRegistry,
    state: LauncherUiState,
    start_time: std::time::Instant,
    winit_state: irisui::winit::state::WinitState,
}

impl Default for LauncherApp {
    fn default() -> Self {
        Self {
            window: None,
            device: None,
            queue: None,
            surface: None,
            surface_config: None,

            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            renderer: None,
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::new(),

            cursor_pos: Point::new(-1000.0, -1000.0),
            screen_size: glam::Vec2::new(920.0, 580.0),

            registry: ProjectRegistry::load_from_disk(),
            state: LauncherUiState::default(),
            start_time: std::time::Instant::now(),
            winit_state: irisui::winit::state::WinitState::new(1.0),
        }
    }
}

impl LauncherApp {
    /// Runs the launcher application with an explicit Winit event loop.
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let mut app = LauncherApp::default();
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    fn init_wgpu(&mut self, window: Arc<Window>) {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create WGPU surface for launcher window");

        let (adapter, device, queue) = pollster::block_on(
            crate::adapter::request_launcher_adapter_and_device(&instance, &surface),
        )
        .unwrap_or_else(|err| {
            panic!("{err}");
        });

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::AutoVsync) {
            wgpu::PresentMode::AutoVsync
        } else if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            caps.present_modes
                .first()
                .copied()
                .unwrap_or(wgpu::PresentMode::Fifo)
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Srgb,
        };
        surface.configure(&device, &config);

        let mut renderer = IrisRenderer::new(&device, format);

        // Upload canonical 16-layer editor icon texture array (`editor_atlas.png`)
        const ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/icons/editor_atlas.png");
        if let Ok(img) = image::load_from_memory(ATLAS_BYTES) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let raw = rgba.as_raw();
            let tile_size = 64u32;
            let cols = w / tile_size;
            let rows = h / tile_size;
            let layer_count = cols * rows;

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Launcher Icon Texture Array"),
                size: wgpu::Extent3d {
                    width: tile_size,
                    height: tile_size,
                    depth_or_array_layers: layer_count,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            for layer in 0..layer_count {
                let r = layer / cols;
                let c = layer % cols;
                let mut tile = Vec::with_capacity((tile_size * tile_size * 4) as usize);
                for y in 0..tile_size {
                    let sy = r * tile_size + y;
                    let sx = c * tile_size;
                    let start = ((sy * w + sx) * 4) as usize;
                    let end = start + (tile_size * 4) as usize;
                    tile.extend_from_slice(&raw[start..end]);
                }

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &tile,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * tile_size),
                        rows_per_image: Some(tile_size),
                    },
                    wgpu::Extent3d {
                        width: tile_size,
                        height: tile_size,
                        depth_or_array_layers: 1,
                    },
                );
            }

            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Launcher Icon Texture Array View"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });

            let bind_group = renderer
                .texture_pipeline
                .create_texture_bind_group(&device, &view);
            renderer.set_texture_bind_group(Some(bind_group));
        }

        let text_renderer = TextRenderer::new(&device, &queue, format);

        self.screen_size = glam::Vec2::new(width as f32, height as f32);
        self.window = Some(window);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.renderer = Some(renderer);
        self.text_renderer = Some(text_renderer);
    }

    /// Calculates the appropriate mouse cursor icon based on current cursor position and active UI tab.
    ///
    /// Returns `CursorIcon::Text` when hovering over editable text inputs, `CursorIcon::Pointer` when
    /// hovering over clickable cards/buttons, and `CursorIcon::Default` elsewhere.
    fn compute_cursor_icon(&self) -> winit::window::CursorIcon {
        if let Some(target) = self.tree.hit_test_target(self.cursor_pos) {
            match target.role {
                irisui::core::WidgetRole::Button => winit::window::CursorIcon::Pointer,
                irisui::core::WidgetRole::TextInput => winit::window::CursorIcon::Text,
                _ => winit::window::CursorIcon::Default,
            }
        } else {
            winit::window::CursorIcon::Default
        }
    }

    fn handle_click(&mut self) {
        if let Some(target) = self.tree.hit_test_target(self.cursor_pos) {
            self.tree.event_state.frame_clicked_tag = Some(target.tag);
        }
    }

    fn handle_action(&mut self, action: LauncherAction) {
        match action {
            LauncherAction::None => {}
            LauncherAction::SwitchTab(tab) => {
                self.state.active_tab = tab;
                self.state.is_name_focused = false;
            }
            LauncherAction::BrowseDirectory => {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.state.new_project_dir = folder.to_string_lossy().to_string();
                }
            }
            LauncherAction::OpenProject(path, mode) => {
                self.registry.add_or_touch(crate::project::ProjectConfig {
                    name: path
                        .file_name()
                        .map_or("Project".into(), |n| n.to_string_lossy().into()),
                    path: path.to_string_lossy().into(),
                    dimension_mode: mode.clone(),
                    ..Default::default()
                });
                let _ = launch_engine_and_exit(&path, &mode);
            }
            LauncherAction::CreateAndLaunch {
                name,
                parent_dir,
                mode,
            } => match ProjectRegistry::create_project(&name, &parent_dir, &mode) {
                Ok(config) => {
                    self.registry.add_or_touch(config.clone());
                    let path = PathBuf::from(&config.path);
                    let _ = launch_engine_and_exit(&path, &config.dimension_mode);
                }
                Err(e) => {
                    self.state.status_message = Some(format!("Error: {}", e));
                }
            },
            LauncherAction::FocusNameInput(focused) => {
                self.state.is_name_focused = focused;
                self.state.cursor_blink_visible = true;
            }
            LauncherAction::SelectDimensionMode {
                mode,
                status_message,
            } => {
                self.state.selected_mode = mode;
                self.state.status_message = status_message;
            }
            LauncherAction::RemoveRecent(path) => {
                self.registry.remove(&path);
            }
        }
    }

    fn render(&mut self) {
        // Compute cursor blink phase: 530ms visible, 530ms hidden (~1Hz pulse)
        self.state.cursor_blink_visible =
            (self.start_time.elapsed().as_millis() / 530).is_multiple_of(2);

        self.tree.clear();
        let action = build_launcher_ui(
            &mut self.tree,
            &mut self.state,
            &self.registry,
            self.screen_size,
            self.cursor_pos,
        );

        let _ = self.layout_engine.compute_layout(
            &mut self.tree,
            Size::new(self.screen_size.x, self.screen_size.y),
        );

        if action != LauncherAction::None {
            self.handle_action(action);
            self.tree.clear();
            let _ = build_launcher_ui(
                &mut self.tree,
                &mut self.state,
                &self.registry,
                self.screen_size,
                self.cursor_pos,
            );
            let _ = self.layout_engine.compute_layout(
                &mut self.tree,
                Size::new(self.screen_size.x, self.screen_size.y),
            );
            if let Some(ref win) = self.window {
                win.request_redraw();
            }
        }

        self.tree.clear_frame_clicks();

        let (Some(device), Some(queue), Some(surface), Some(renderer), Some(text_renderer)) = (
            self.device.as_ref(),
            self.queue.as_ref(),
            self.surface.as_ref(),
            self.renderer.as_mut(),
            self.text_renderer.as_mut(),
        ) else {
            return;
        };

        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            wgpu::CurrentSurfaceTexture::Lost => {
                if let (Some(device), Some(config)) =
                    (self.device.as_ref(), self.surface_config.as_ref())
                {
                    surface.configure(device, config);
                }
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Timeout => return,
            _ => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Collect quads and text sections
        self.command_list.clear();
        let mut sections = Vec::new();
        if let Some(root_id) = self.tree.root() {
            populate_commands_and_text(&self.tree, root_id, &mut self.command_list, &mut sections);
        }

        let screen_dim = [self.screen_size.x, self.screen_size.y];
        renderer.prepare_command_list(device, queue, screen_dim, &self.command_list);
        text_renderer.prepare(
            device,
            queue,
            &mut self.text_system,
            (self.screen_size.x as u32, self.screen_size.y as u32),
            1.0,
            &sections,
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Launcher Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Launcher Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.055,
                            g: 0.063,
                            b: 0.086,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            renderer.render_command_list(
                &mut rpass,
                &self.command_list,
                (self.screen_size.x as u32, self.screen_size.y as u32),
            );
            text_renderer.render(&mut rpass);
        }

        queue.submit(Some(encoder.finish()));
        queue.present(output);
    }
}

impl ApplicationHandler for LauncherApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        const AEON_ICON_BYTES: &[u8] = include_bytes!("../../ae_engine/assets/icon/aeicon.png");
        let icon = crate::icon::load_icon_from_memory(AEON_ICON_BYTES);

        #[cfg(target_os = "linux")]
        crate::icon::ensure_xdg_desktop_integration(AEON_ICON_BYTES);

        let attributes = Window::default_attributes()
            .with_title("Aeon Engine Hub")
            .with_inner_size(LogicalSize::new(920.0, 580.0))
            .with_min_inner_size(LogicalSize::new(800.0, 500.0))
            .with_window_icon(icon);

        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd"
        ))]
        let attributes = {
            use winit::platform::wayland::WindowAttributesExtWayland;
            use winit::platform::x11::WindowAttributesExtX11;

            let attr = WindowAttributesExtWayland::with_name(
                attributes,
                "com.aeonengine.Launcher",
                "com.aeonengine.Launcher",
            );
            WindowAttributesExtX11::with_name(
                attr,
                "com.aeonengine.Launcher",
                "com.aeonengine.Launcher",
            )
        };

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create launcher window"),
        );
        self.init_wgpu(window.clone());
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ui_event = self.winit_state.on_window_event(&event);
        if let Some(ref ui_ev) = ui_event {
            let dispatch = self.tree.dispatch_event(ui_ev);
            if (dispatch.is_consumed() || self.tree.event_state.active_text_edit.is_some())
                && let Some(ref win) = self.window
            {
                win.request_redraw();
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    let w = physical_size.width;
                    let h = physical_size.height;
                    self.screen_size = glam::Vec2::new(w as f32, h as f32);
                    if let (Some(surface), Some(device), Some(config)) = (
                        self.surface.as_ref(),
                        self.device.as_ref(),
                        self.surface_config.as_mut(),
                    ) && (config.width != w || config.height != h)
                    {
                        config.width = w;
                        config.height = h;
                        surface.configure(device, config);
                    }
                    if let Some(ref win) = self.window {
                        win.request_redraw();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point::new(position.x as f32, position.y as f32);
                let cursor_icon = self.compute_cursor_icon();
                if let Some(ref win) = self.window {
                    win.set_cursor(cursor_icon);
                    win.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.handle_click();
                let cursor_icon = self.compute_cursor_icon();
                if let Some(ref win) = self.window {
                    win.set_cursor(cursor_icon);
                    win.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    /// Event loop pacing and idle control.
    ///
    /// When the text input box is focused, wakes up every 100ms to drive the smooth ~1Hz text cursor
    /// blink animation. When unfocused, drops to `ControlFlow::Wait` for zero CPU consumption.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.tree.event_state.active_text_edit.is_some() || self.state.is_name_focused {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(100),
            ));
            if let Some(ref win) = self.window {
                win.request_redraw();
            }
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }
    }
}

fn populate_commands_and_text<'a>(
    tree: &'a UiTree,
    node_id: irisui::core::id::WidgetId,
    cmd_list: &mut DrawCommandList,
    sections: &mut Vec<TextSection<'a>>,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };

    let rect = node.computed_rect;
    if rect.width > 0.0 && rect.height > 0.0 {
        let has_border = (node.style.border.width.top > 0.0
            || node.style.border.width.bottom > 0.0
            || node.style.border.width.left > 0.0
            || node.style.border.width.right > 0.0)
            && node.style.border.color.a > 0.0;

        if node.style.background_color.a > 0.0 || has_border || node.style.box_shadow.is_some() {
            let quad = irisui::wgpu_backend::QuadInstance::from_style(rect, &node.style, None);
            cmd_list.push_quad(quad);
        }

        if let Some(uv) = node.texture_uv {
            let tint = node
                .texture_tint
                .unwrap_or(irisui::core::color::Color::WHITE);
            cmd_list.push_texture_quad(irisui::wgpu_backend::TextureQuadInstance {
                rect: [rect.x, rect.y, rect.width, rect.height],
                uv_rect: uv,
                tint: [tint.r, tint.g, tint.b, tint.a],
                clip_rect: [0.0, 0.0, 0.0, 0.0],
            });
        }

        if let Some(ref text) = node.text {
            sections.push(
                TextSection::new(text, rect)
                    .with_font_size(node.font_size, node.line_height.max(node.font_size * 1.2))
                    .with_color(node.text_color),
            );
        }
    }

    for &child_id in &node.children {
        populate_commands_and_text(tree, child_id, cmd_list, sections);
    }
}