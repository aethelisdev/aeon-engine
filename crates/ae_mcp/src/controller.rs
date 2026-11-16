// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified live inspection session coordinator owning bridge, overlay, drag, and keyboard state.

use crate::drag::{ActiveInspectDrag, start_drag, tick_drag};
use crate::host::McpHost;
use crate::input::{handle_ui_click, handle_ui_mouse_move, handle_ui_scroll};
use crate::overlay::InspectVisualOverlay;
use crate::protocol::{InspectRequest, InspectResponse};
use crate::query::{
    handle_engine_logs, handle_health, handle_inspect_entity, handle_list_entities,
    handle_screenshot, handle_select_entity, handle_ui_tree_dump,
};
use crate::sequence::{
    ActiveInspectTyping, PendingInspectKeyRelease, finish_key_release, start_key_press,
    start_type_text, tick_typing,
};
use crate::server::InspectBridge;

/// Single-owner live inspection runtime holding the HTTP bridge, visual cursor overlay,
/// and any in-flight multi-frame drag or typing sequence.
pub struct McpSession {
    bridge: InspectBridge,
    overlay: InspectVisualOverlay,
    active_drag: Option<ActiveInspectDrag>,
    active_typing: Option<ActiveInspectTyping>,
    pending_key_release: Option<PendingInspectKeyRelease>,
}

impl McpSession {
    /// Starts the localhost HTTP bridge at `addr` and allocates visual overlay GPU pipelines.
    ///
    /// # Errors
    /// Returns an error string when binding the TCP listener or spawning the server thread fails.
    pub fn start(
        addr: &str,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let bridge = InspectBridge::start(addr)?;
        let overlay = InspectVisualOverlay::new(device, target_format);
        Ok(Self {
            bridge,
            overlay,
            active_drag: None,
            active_typing: None,
            pending_key_release: None,
        })
    }

    /// Returns the actual bound `HOST:PORT` string of the localhost inspection listener.
    #[must_use]
    pub fn bound_addr(&self) -> &str {
        self.bridge.bound_addr()
    }

    /// Renders active agent cursor markers, click ripples, and drag trails onto `target_view`.
    pub fn render_overlay(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_size: (u32, u32),
        zoom: f32,
    ) {
        self.overlay
            .render_overlay(device, queue, encoder, target_view, physical_size, zoom);
    }

    /// Advances active multi-frame sequences and drains queued HTTP RPC commands against `host`.
    pub fn tick<H: McpHost + ?Sized>(&mut self, host: &mut H) {
        if let Some(pending) = self.pending_key_release.take() {
            finish_key_release(host, pending);
        }
        if let Some(typing) = self.active_typing.take() {
            self.active_typing = tick_typing(host, &mut self.overlay, typing);
        }
        if let Some(drag) = self.active_drag.take() {
            self.active_drag = tick_drag(host, &mut self.overlay, drag);
        }

        let calls = self.bridge.drain_requests();
        for call in calls {
            match call.request.method.as_str() {
                "ui.drag" => {
                    if self.active_drag.is_some() {
                        let _ = call.responder.send(InspectResponse::err(
                            "Another inspect drag gesture is already in progress",
                        ));
                    } else {
                        self.active_drag = start_drag(
                            host,
                            &mut self.overlay,
                            &call.request.params,
                            call.responder,
                        );
                    }
                }
                "ui.key" => {
                    if self.pending_key_release.is_some() || self.active_typing.is_some() {
                        let _ = call.responder.send(InspectResponse::err(
                            "Another keyboard injection sequence is already in progress",
                        ));
                    } else {
                        self.pending_key_release = start_key_press(
                            host,
                            &mut self.overlay,
                            &call.request.params,
                            call.responder,
                        );
                    }
                }
                "ui.type_text" => {
                    if self.pending_key_release.is_some() || self.active_typing.is_some() {
                        let _ = call.responder.send(InspectResponse::err(
                            "Another keyboard injection sequence is already in progress",
                        ));
                    } else {
                        self.active_typing = start_type_text(
                            host,
                            &mut self.overlay,
                            &call.request.params,
                            call.responder,
                        );
                    }
                }
                _ => {
                    let response = self.dispatch_sync(host, call.request);
                    let _ = call.responder.send(response);
                }
            }
        }

        if self.overlay.has_active_animation() {
            host.request_redraw();
        }
    }

    fn dispatch_sync<H: McpHost + ?Sized>(
        &mut self,
        host: &mut H,
        req: InspectRequest,
    ) -> InspectResponse {
        match req.method.as_str() {
            "health" => handle_health(host, self.active_drag.is_some()),
            "ui.tree_dump" => handle_ui_tree_dump(host, &req.params),
            "ui.click" => handle_ui_click(host, &mut self.overlay, &req.params),
            "ui.mouse_move" => handle_ui_mouse_move(host, &mut self.overlay, &req.params),
            "ui.scroll" => handle_ui_scroll(host, &mut self.overlay, &req.params),
            "scene.list_entities" => handle_list_entities(host),
            "scene.select_entity" => handle_select_entity(host, &req.params),
            "scene.inspect_entity" => handle_inspect_entity(host, &req.params),
            "engine.logs" => handle_engine_logs(host, &req.params),
            "render.screenshot" => handle_screenshot(host, &mut self.overlay, &req.params),
            other => InspectResponse::err(format!("Unknown inspect method: {other}")),
        }
    }
}