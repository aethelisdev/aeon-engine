// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Isolated Iris UI render pass bridge for the Aeon Engine rendering pipeline.
//!
//! Provides a non-intrusive, isolated render pass execution entry point
//! (`iris_render_pass`) overlaying Iris UI SDF elements, text, and dock viewports
//! directly onto the target framebuffer surface without modifying core 3D passes.

use irisui::text::TextRenderer;
use irisui::wgpu_backend::{DrawCommandList, IrisRenderer};

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
    /// Optional GPU text atlas and typography renderer.
    pub text_renderer: Option<&'a TextRenderer>,
    /// Target window resolution in physical pixels.
    pub screen_size: (u32, u32),
}

/// Executes an isolated Iris UI render pass on the target surface view.
/// Overlays prepared SDF quads, text, and texture viewports on top of the
/// existing framebuffer (using `wgpu::LoadOp::Load` to preserve underlying 3D scene content).
pub fn iris_render_pass(params: IrisRenderPassParams<'_>) {
    let IrisRenderPassParams {
        device,
        queue,
        encoder,
        target_view,
        renderer,
        command_list,
        text_renderer,
        screen_size,
    } = params;

    let has_quads = !command_list.commands.is_empty() || !command_list.quads.is_empty();
    if !has_quads && text_renderer.is_none() {
        return;
    }

    if has_quads {
        // Prepare instance and uniform GPU buffers
        renderer.prepare_command_list(
            device,
            queue,
            [screen_size.0 as f32, screen_size.1 as f32],
            command_list,
        );
    }

    // Record isolated UI render pass
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
}