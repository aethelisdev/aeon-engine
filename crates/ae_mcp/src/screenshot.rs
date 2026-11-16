// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Offscreen WGPU framebuffer screenshot capture with optional visual overlay compositing.

use crate::host::McpHost;
use crate::overlay::InspectVisualOverlay;
use std::path::Path;
use std::sync::mpsc;

/// Captures the current composed host UI (+ active [`InspectVisualOverlay`]) into a PNG file.
///
/// # Errors
/// Returns an error string when surface dimensions are zero, GPU readback mapping fails,
/// or PNG encoding/writing fails.
pub fn capture_ui_screenshot<H: McpHost + ?Sized>(
    host: &mut H,
    overlay: &mut InspectVisualOverlay,
    path: &Path,
) -> Result<(u32, u32), String> {
    let gpu = host.gpu_handles();
    let (width, height) = gpu.physical_size;
    if width == 0 || height == 0 {
        return Err("Window surface has zero dimensions".to_string());
    }
    let format = gpu.format;

    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Aeon MCP Screenshot Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Aeon MCP Screenshot Encoder"),
        });

    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Aeon MCP Screenshot Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.06,
                        b: 0.08,
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
    }

    let zoom = host.health_snapshot().scale_factor;
    host.render_ui_to_view(&mut encoder, &view, (width, height), zoom);

    let gpu = host.gpu_handles();
    overlay.render_overlay(
        gpu.device,
        gpu.queue,
        &mut encoder,
        &view,
        (width, height),
        zoom,
    );

    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
    let buffer_size = u64::from(padded_bytes_per_row) * u64::from(height);

    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Aeon MCP Screenshot Readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    gpu.queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv()
        .map_err(|e| format!("GPU readback channel failed: {e}"))?
        .map_err(|e| format!("GPU buffer map failed: {e:?}"))?;

    let is_bgra = matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    let mapped = slice
        .get_mapped_range()
        .map_err(|e| format!("GPU mapped range failed: {e:?}"))?;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height as usize {
        let row_start = row * padded_bytes_per_row as usize;
        let row_bytes = &mapped[row_start..row_start + unpadded_bytes_per_row as usize];
        if is_bgra {
            for px in row_bytes.as_chunks::<4>().0 {
                rgba.push(px[2]);
                rgba.push(px[1]);
                rgba.push(px[0]);
                rgba.push(px[3]);
            }
        } else {
            rgba.extend_from_slice(row_bytes);
        }
    }
    drop(mapped);
    staging.unmap();

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed creating screenshot parent dir: {e}"))?;
    }
    image::save_buffer(path, &rgba, width, height, image::ColorType::Rgba8)
        .map_err(|e| format!("Failed encoding PNG screenshot: {e}"))?;

    Ok((width, height))
}