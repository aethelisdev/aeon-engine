// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::types::LightUniform;
use wgpu::util::DeviceExt;

/// Central uniform buffer manager for camera, lighting, sky, and texture bind groups.
pub struct SceneUniforms {
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_uniform: crate::camera::CameraUniform,

    pub light_bind_group: wgpu::BindGroup,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_uniform: LightUniform,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    pub sky_bind_group: wgpu::BindGroup,
    pub sky_buffer: wgpu::Buffer,
    pub sky_bind_group_layout: wgpu::BindGroupLayout,
    pub sky_uniform: crate::render::types::SkyUniform,
}

impl SceneUniforms {
    /// Initializes all uniform buffers and bind group layouts for camera, light, sky, and textures.
    pub fn new(device: &wgpu::Device, camera: &crate::camera::Camera) -> Self {
        let mut camera_uniform = crate::camera::CameraUniform::new();
        camera_uniform.update_view_proj(camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // --- LIGHTING SETUP ---
        let light_uniform = LightUniform {
            direction: [0.0, 1.0, 0.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
            ambient_color: [0.1, 0.1, 0.1],
            _padding3: 0,
            fog_params: [0.0; 4],
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("light_bind_group_layout"),
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });

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

        let sky_uniform = crate::render::types::SkyUniform {
            sun_direction: [0.0, 1.0, 0.0, 0.0],
            sun_color: [1.0, 1.0, 1.0, 1.0],
            horizon_color: [0.5, 0.6, 0.7, 0.0],
            zenith_color: [0.1, 0.2, 0.4, 0.0],
            atmosphere_density: 1.0,
            sun_disc_size: 0.02,
            sun_glow_strength: 0.1,
            sky_quality_mode: 2,
        };

        let sky_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Buffer"),
            contents: bytemuck::cast_slice(&[sky_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sky_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("sky_bind_group_layout"),
            });

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sky_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sky_buffer.as_entire_binding(),
            }],
            label: Some("sky_bind_group"),
        });

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
        }
    }

    /// Updates the camera uniform buffer with the current view-projection matrix.
    pub fn update(&mut self, queue: &wgpu::Queue, camera: &crate::camera::Camera) {
        self.camera_uniform.update_view_proj(camera);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Synchronizes both the Sky Shader and the Scene PBR Lighting from the single Global Settings.
    /// Computes sun direction from pitch/yaw, derives horizon/zenith colors,
    /// sunset color transitions, ambient fill, and atmospheric fog parameters.
    pub fn update_environment(
        &mut self,
        queue: &wgpu::Queue,
        settings: &crate::graphics_settings::GraphicsSettings,
    ) {
        // --- 1. Compute Shared Vector (Source of Truth) ---
        // Pitch/Yaw converted to a normalized vector pointing TOWARDS the sun.
        let pitch = settings.sun_pitch;
        let yaw = settings.sun_yaw;
        let sun_x = pitch.cos() * yaw.sin();
        let sun_y = pitch.sin();
        let sun_z = pitch.cos() * yaw.cos();
        let sun_dir = [sun_x, sun_y, sun_z];

        // --- 2. Update Sky Uniform ---
        self.sky_uniform.sun_direction = [sun_x, sun_y, sun_z, 0.0];

        let env_r = settings.environment_color[0];
        let env_g = settings.environment_color[1];
        let env_b = settings.environment_color[2];

        // Horizon and zenith are close in tone — gradient should be subtle, not a visible band
        self.sky_uniform.horizon_color = [env_r * 0.2, env_g * 0.4, env_b * 0.75, 1.0];
        self.sky_uniform.zenith_color = [env_r * 0.1, env_g * 0.25, env_b * 0.9, 1.0];
        self.sky_uniform.sun_color = [1.0, 0.95, 0.8, 100.0]; // HDR intensity in w

        self.sky_uniform.atmosphere_density = settings.atmosphere_density;
        self.sky_uniform.sun_disc_size = settings.sun_disc_size;
        self.sky_uniform.sun_glow_strength = settings.sun_glow_strength;
        self.sky_uniform.sky_quality_mode = settings.sky_quality as u32;

        queue.write_buffer(
            &self.sky_buffer,
            0,
            bytemuck::cast_slice(&[self.sky_uniform]),
        );

        // --- 3. Update PBR Scene Light Uniform ---
        self.light_uniform.direction = sun_dir;

        // Calculate physics-esque sunset transition
        let elevation = sun_y.max(0.0);
        let noon_color = [1.0, 0.95, 0.9];
        let sunset_color = [1.0, 0.4, 0.1];

        let blend = elevation.powf(0.5); // Fast color transition near horizon

        // Direct sun color (Balanced 1.05 multiplier for natural non-overexposed lighting)
        self.light_uniform.color = [
            (noon_color[0] * blend + sunset_color[0] * (1.0 - blend)) * 1.05,
            (noon_color[1] * blend + sunset_color[1] * (1.0 - blend)) * 1.05,
            (noon_color[2] * blend + sunset_color[2] * (1.0 - blend)) * 1.05,
        ];

        // Soft ambient environment light (derived from sky mood, but bright enough for clear PBR visibility)
        let ambient_intensity = 0.2 + elevation * 0.4; // Bright enough to fill in deep shadows cleanly
        self.light_uniform.ambient_color = [
            env_r * ambient_intensity,
            env_g * ambient_intensity,
            (env_b * 1.2) * ambient_intensity,
        ];

        // --- 4. Atmospheric Fog ---
        let fog_dist = if settings.fog_enabled {
            settings.fog_distance
        } else {
            0.0
        };
        self.light_uniform.fog_params = [
            self.sky_uniform.horizon_color[0],
            self.sky_uniform.horizon_color[1],
            self.sky_uniform.horizon_color[2],
            fog_dist,
        ];

        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}