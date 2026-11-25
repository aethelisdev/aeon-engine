// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone `winit` + `wgpu` example demonstrating a multi-pane `irisui` workbench shell.

#![forbid(unsafe_code)]

use std::sync::Arc;

use irisui::dock::{DockLayoutNode, DockState};
use irisui::prelude::*;
use irisui::wgpu_backend::IrisSurfaceHost;
use irisui::winit::state::WinitState;
use irisui::workbench::{
    PanelMetadata, ScopedPanelRegistry, Workbench, WorkbenchAppData, WorkbenchAppScope,
    WorkbenchDescriptor, WorkbenchHost, WorkbenchHostFrameParams, WorkbenchPanelContext,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoTab {
    Sidebar,
    Workspace,
}

#[derive(Debug, Clone)]
enum DemoAction {
    IncrementCounter,
    ResetCounter,
}

struct DemoApp;

struct DemoData<'frame> {
    clicks: &'frame u32,
}

impl WorkbenchAppData for DemoApp {
    type Data<'frame> = DemoData<'frame>;
    type Action = DemoAction;
    type TabId = DemoTab;
}

#[derive(Default)]
struct SidebarState {
    enabled: bool,
    speed: f32,
    tint: Color,
}

impl SidebarState {
    fn draw(&mut self, scope: &mut UiScope<'_>, ctx: &mut WorkbenchPanelContext<'_, DemoApp>) {
        if self.speed == 0.0 {
            self.speed = 1.0;
            self.tint = Color::rgb(0.25, 0.70, 0.95);
        }
        scope.column(|col| {
            col.label("Controls");
            let _ = col.prop_bool("Active", &mut self.enabled);
            let _ = col.prop_f32("Speed", &mut self.speed, 0.0..=10.0, 0.1);
            let _ = col.prop_color("Tint", &mut self.tint);
            col.row(|row| {
                if row.button("Increment (+1)").clicked() {
                    ctx.action_bus.emit(DemoAction::IncrementCounter);
                }
                if row.button("Reset Counter").clicked() {
                    ctx.action_bus.emit(DemoAction::ResetCounter);
                }
            });
        });
    }
}

#[derive(Default)]
struct WorkspaceState;

impl WorkspaceState {
    fn draw(&mut self, scope: &mut UiScope<'_>, ctx: &mut WorkbenchPanelContext<'_, DemoApp>) {
        scope.centered(|col| {
            col.label("Iris UI Standalone Workbench");
            col.label(format!("Counter: {}", ctx.data.clicks));
        });
    }
}

struct GpuState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    winit_state: WinitState,
    host: WorkbenchHost<DemoApp>,
    clicks: u32,
    domain_revision: u64,
}

impl GpuState {
    fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;

        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("Unsupported swapchain surface configuration")?;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let dock_state = DockState::from_layout(DockLayoutNode::split_horizontal(
            0.30,
            DockLayoutNode::tabs([DemoTab::Sidebar]),
            DockLayoutNode::tabs([DemoTab::Workspace]),
        ));
        let registry = ScopedPanelRegistry::<DemoTab, WorkbenchAppScope<DemoApp>>::new()
            .with_stateful_panel(
                PanelMetadata::new(DemoTab::Sidebar, "Sidebar"),
                SidebarState::draw,
            )
            .with_stateful_panel(
                PanelMetadata::new(DemoTab::Workspace, "Workspace").with_domain_bound(true),
                WorkspaceState::draw,
            );
        let mut host = WorkbenchHost::new(
            Workbench::new(WorkbenchDescriptor::new("Iris UI Demo", dock_state)),
            registry,
            IrisSurfaceHost::new(&device, config.format),
        );
        host.resize(config.width, config.height);

        Ok(Self {
            winit_state: WinitState::new(window.scale_factor()),
            window,
            surface,
            device,
            queue,
            config,
            host,
            clicks: 0,
            domain_revision: 0,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.host.resize(width, height);
    }

    fn render(&mut self) {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Demo Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.09,
                            b: 0.11,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
        }

        let clicks = &self.clicks;
        let _ = self.host.render_frame_with(
            WorkbenchHostFrameParams {
                device: &self.device,
                queue: &self.queue,
                encoder: &mut encoder,
                target_view: &view,
                window: &mut self.window,
                physical_size: (self.config.width, self.config.height),
                selection_id: None,
                domain_revision: self.domain_revision,
                drag_active: false,
                surface_revision: 0,
            },
            |pass| pass.execute(DemoData { clicks }),
        );
        let mut dirty_domain = false;
        self.host.dispatch_actions(|_, action| match action {
            DemoAction::IncrementCounter => {
                self.clicks += 1;
                dirty_domain = true;
            }
            DemoAction::ResetCounter => {
                if self.clicks != 0 {
                    self.clicks = 0;
                    dirty_domain = true;
                }
            }
        });
        if dirty_domain {
            self.domain_revision += 1;
            self.window.request_redraw();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);
    }
}

#[derive(Default)]
struct DemoAppRunner {
    state: Option<GpuState>,
}

impl ApplicationHandler for DemoAppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("Iris UI Workbench Example");
        if let Ok(window) = event_loop.create_window(attrs) {
            self.state = GpuState::new(Arc::new(window)).ok();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
            return;
        }
        if let WindowEvent::Resized(size) = event {
            state.resize(size.width, size.height);
        } else if matches!(event, WindowEvent::RedrawRequested) {
            state.render();
            return;
        }
        if let Some(ui_event) = state.winit_state.on_window_event(&event) {
            let res = state.host.handle_event(
                &mut state.window,
                (state.config.width, state.config.height),
                &ui_event,
            );
            if res.close_requested {
                event_loop.exit();
                return;
            }
            if res.consumed {
                state.window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut runner = DemoAppRunner::default();
    event_loop.run_app(&mut runner)?;
    Ok(())
}