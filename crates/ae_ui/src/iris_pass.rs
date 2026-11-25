// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Isolated Iris UI render pass bridge over [`irisui_wgpu`] and [`irisui_text`].

use irisui_core::WidgetId;
use irisui_text::TextRenderer;
use irisui_wgpu::{DrawCommandList, IrisRenderer};
use std::collections::HashSet;

/// Parameters for executing the isolated Iris UI overlay render pass.
pub struct IrisRenderPassParams<'a> {
    /// Active WGPU logical rendering device.
    pub device: &'a wgpu::Device,
    /// Active WGPU hardware command queue.
    pub queue: &'a wgpu::Queue,
    /// Primary frame command encoder.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// Target framebuffer surface texture view.
    pub target_view: &'a wgpu::TextureView,
    /// Iris GPU SDF quad and geometry renderer.
    pub renderer: &'a mut IrisRenderer,
    /// Active frame drawing command list.
    pub command_list: &'a DrawCommandList,
    /// Optional GPU text atlas and typography renderer for static UI panels.
    pub text_renderer: Option<&'a TextRenderer>,
    /// Optional isolated GPU text renderer for high-frequency volatile/live HUD & stats labels.
    pub volatile_text_renderer: Option<&'a TextRenderer>,
    /// Target surface/logical resolution used by Iris UI scissor/viewport scaling.
    pub screen_size: (u32, u32),
    /// Optional invalidation boundaries that mutated this frame for partial GPU buffer uploads.
    pub dirty_boundaries: Option<&'a HashSet<WidgetId>>,
}

/// Executes an isolated Iris UI render pass on the target surface view.
///
/// Overlays prepared SDF quads and text on top of the existing framebuffer
/// (`wgpu::LoadOp::Load` preserves underlying scene content).
pub fn iris_render_pass(params: IrisRenderPassParams<'_>) {
    let IrisRenderPassParams {
        device,
        queue,
        encoder,
        target_view,
        renderer,
        command_list,
        text_renderer,
        volatile_text_renderer,
        screen_size,
        dirty_boundaries,
    } = params;

    let has_quads = !command_list.commands.is_empty() || !command_list.quads.is_empty();
    if !has_quads && text_renderer.is_none() && volatile_text_renderer.is_none() {
        return;
    }

    if has_quads {
        renderer.prepare_command_list_with_dirty_boundaries(
            device,
            queue,
            [screen_size.0 as f32, screen_size.1 as f32],
            command_list,
            dirty_boundaries,
        );
    }

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Iris UI Overlay Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    if has_quads {
        renderer.render_command_list(&mut render_pass, command_list, screen_size);
    }

    if let Some(txt_renderer) = text_renderer {
        txt_renderer.render(&mut render_pass);
    }
    if let Some(volatile_txt) = volatile_text_renderer {
        volatile_txt.render(&mut render_pass);
    }
}