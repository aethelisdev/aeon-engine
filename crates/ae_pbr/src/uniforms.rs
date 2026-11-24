// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GPU uniform buffer and bind-group manager for camera, lighting, sky, and textures.

use ae_light::{EnvironmentLightConfig, LightUniform, SkyUniform, evaluate_environment};
use wgpu::util::DeviceExt;

/// Central GPU uniform manager owning camera, scene light, procedural sky, and diffuse texture layouts.
pub struct SceneUniforms {
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_uniform: ae_camera::CameraUniform,

    pub light_bind_group: wgpu::BindGroup,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_uniform: LightUniform,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    pub sky_bind_group: wgpu::BindGroup,
    pub sky_buffer: wgpu::Buffer,
    pub sky_bind_group_layout: wgpu::BindGroupLayout,
    pub sky_uniform: SkyUniform,
    pub start_time: std::time::Instant,
}

fn create_single_uniform_buffer_group<T: bytemuck::Pod>(
    device: &wgpu::Device,
    buffer_label: &str,
    bgl_label: &str,
    bg_label: &str,
    visibility: wgpu::ShaderStages,
    initial_data: &T,
) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(buffer_label),
        contents: bytemuck::bytes_of(initial_data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(bgl_label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(bg_label),
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (buffer, bgl, bg)
}

impl SceneUniforms {
    /// Initializes GPU uniform buffers and bind group layouts for camera, light, sky, and textures.
    pub fn new(device: &wgpu::Device, camera: &ae_camera::Camera) -> Self {
        let mut camera_uniform = ae_camera::CameraUniform::new();
        camera_uniform.update_view_proj(camera);
        let (camera_buffer, camera_bind_group_layout, camera_bind_group) =
            create_single_uniform_buffer_group(
                device,
                "Camera Buffer",
                "camera_bind_group_layout",
                "camera_bind_group",
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                &camera_uniform,
            );

        let light_uniform = LightUniform::default();
        let (light_buffer, light_bind_group_layout, light_bind_group) =
            create_single_uniform_buffer_group(
                device,
                "Light Buffer",
                "light_bind_group_layout",
                "light_bind_group",
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                &light_uniform,
            );

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let sky_uniform = SkyUniform::default();
        let (sky_buffer, sky_bind_group_layout, sky_bind_group) =
            create_single_uniform_buffer_group(
                device,
                "Sky Buffer",
                "sky_bind_group_layout",
                "sky_bind_group",
                wgpu::ShaderStages::FRAGMENT,
                &sky_uniform,
            );

        Self {
            camera_bind_group,
            camera_buffer,
            camera_bind_group_layout,
            camera_uniform,
            light_bind_group,
            light_buffer,
            light_bind_group_layout,
            light_uniform,
            texture_bind_group_layout,
            sky_bind_group,
            sky_buffer,
            sky_bind_group_layout,
            sky_uniform,
            start_time: std::time::Instant::now(),
        }
    }

    /// Updates and uploads the camera view-projection uniform buffer.
    pub fn update(&mut self, queue: &wgpu::Queue, camera: &ae_camera::Camera) {
        self.camera_uniform.update_view_proj(camera);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Synchronizes procedural sky and directional light uniforms via `ae_light::evaluate_environment`.
    pub fn update_environment(&mut self, queue: &wgpu::Queue, config: &EnvironmentLightConfig) {
        let eval = evaluate_environment(
            config,
            self.start_time.elapsed().as_secs_f32(),
            self.light_uniform.grid_scale,
        );
        self.sky_uniform = eval.sky;
        self.light_uniform = eval.light;

        queue.write_buffer(
            &self.sky_buffer,
            0,
            bytemuck::cast_slice(&[self.sky_uniform]),
        );
        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}