// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::types::LightSpaceUniform;
use cgmath::SquareMatrix;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Cascaded Shadow Map (CSM) system managing up to 4 shadow cascades.
/// Owns the shadow depth texture array, per-cascade views, light-space matrices,
/// and the shadow rendering pipeline.
pub struct ShadowSystem {
    pub shadow_pipeline: wgpu::RenderPipeline,

    pub shadow_depth_texture: wgpu::Texture,
    pub shadow_cascade_views: [wgpu::TextureView; 4],

    pub shadow_bind_group_layout: wgpu::BindGroupLayout,
    pub shadow_bind_group: wgpu::BindGroup,

    pub light_space_bgl_vs: wgpu::BindGroupLayout,
    pub light_space_uniform: LightSpaceUniform,
    pub light_space_buffer: wgpu::Buffer,

    pub shadow_matrix_buffers: [wgpu::Buffer; 4],
    pub shadow_pass_bind_groups: [wgpu::BindGroup; 4],
}

impl ShadowSystem {
    /// Initializes the shadow system with the configured resolution and cascade count.
    pub fn new(
        device: &wgpu::Device,
        graphics_settings: &crate::graphics_settings::GraphicsSettings,
    ) -> Self {
        let shadow_res = graphics_settings.shadow_resolution.as_u32();

        let shadow_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Depth Texture"),
            size: wgpu::Extent3d {
                width: shadow_res,
                height: shadow_res,
                depth_or_array_layers: 4,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_cascade_views = [
            shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("CSM0"),
                base_array_layer: 0,
                array_layer_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
                ..Default::default()
            }),
            shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("CSM1"),
                base_array_layer: 1,
                array_layer_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
                ..Default::default()
            }),
            shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("CSM2"),
                base_array_layer: 2,
                array_layer_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
                ..Default::default()
            }),
            shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("CSM3"),
                base_array_layer: 3,
                array_layer_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
                ..Default::default()
            }),
        ];

        let shadow_depth_view = shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let shadow_depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let light_space_uniform = LightSpaceUniform {
            matrices: [[[0.0; 4]; 4]; 4],
            cascade_splits: [0.0; 4],
            shadow_bias: graphics_settings.shadow_bias,
            pcf_radius: graphics_settings.shadow_pcf.radius(),
            shadow_enabled: if graphics_settings.shadow_enabled {
                1
            } else {
                0
            },
            _pad: 0,
        };

        let light_space_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Space Buffer"),
            contents: bytemuck::cast_slice(&[light_space_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_depth_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_space_buffer.as_entire_binding(),
                },
            ],
        });

        let light_space_bgl_vs =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow VS BGL"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let mut shadow_matrix_buffers_vec = Vec::new();
        let mut shadow_pass_bind_groups_vec = Vec::new();
        for i in 0..4 {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Cascade Matrix Buffer {}", i)),
                size: 64, // mat4x4
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Cascade Bind Group {}", i)),
                layout: &light_space_bgl_vs,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf.as_entire_binding(),
                }],
            });
            shadow_matrix_buffers_vec.push(buf);
            shadow_pass_bind_groups_vec.push(bg);
        }

        let shadow_pass_bind_groups: [wgpu::BindGroup; 4] =
            shadow_pass_bind_groups_vec.try_into().unwrap();
        let shadow_matrix_buffers: [wgpu::Buffer; 4] =
            shadow_matrix_buffers_vec.try_into().unwrap();

        let shadow_pipeline =
            crate::render::pipelines::shadow::create_shadow_pipeline(device, &light_space_bgl_vs);

        Self {
            shadow_pipeline,
            shadow_depth_texture,
            shadow_cascade_views,
            shadow_bind_group_layout,
            shadow_bind_group,
            light_space_bgl_vs,
            light_space_uniform,
            light_space_buffer,
            shadow_matrix_buffers,
            shadow_pass_bind_groups,
        }
    }

    /// Recreates shadow depth textures and bind groups when shadow resolution changes.
    pub fn resize_targets(
        &mut self,
        device: &wgpu::Device,
        graphics_settings: &crate::graphics_settings::GraphicsSettings,
    ) {
        let shadow_res = graphics_settings.shadow_resolution.as_u32();

        self.shadow_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Depth Texture"),
            size: wgpu::Extent3d {
                width: shadow_res,
                height: shadow_res,
                depth_or_array_layers: 4,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.shadow_cascade_views = [
            self.shadow_depth_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("CSM0"),
                    base_array_layer: 0,
                    array_layer_count: Some(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                }),
            self.shadow_depth_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("CSM1"),
                    base_array_layer: 1,
                    array_layer_count: Some(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                }),
            self.shadow_depth_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("CSM2"),
                    base_array_layer: 2,
                    array_layer_count: Some(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                }),
            self.shadow_depth_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("CSM3"),
                    base_array_layer: 3,
                    array_layer_count: Some(1),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                }),
        ];

        let shadow_depth_view =
            self.shadow_depth_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Shadow Array View"),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                });

        let shadow_depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        self.shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &self.shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_depth_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.light_space_buffer.as_entire_binding(),
                },
            ],
        });
    }

    /// Computes light-space projection matrices for each cascade based on camera
    /// position and directional light direction. Uploads to GPU buffers.
    pub fn update_cascades(
        &mut self,
        queue: &wgpu::Queue,
        graphics_settings: &crate::graphics_settings::GraphicsSettings,
        camera: &crate::camera::Camera,
        light_dir: cgmath::Vector3<f32>,
    ) {
        use cgmath::InnerSpace;
        let cascades = graphics_settings.shadow_cascades.clamp(1, 4) as usize;
        let splits = graphics_settings.shadow_cascade_splits; // Configurable View-Z distances

        let cam_pos = cgmath::Vector3::new(camera.position.x, camera.position.y, camera.position.z);
        let cam_forward = cgmath::Vector3::new(
            camera.yaw.0.cos() * camera.pitch.0.cos(),
            camera.pitch.0.sin(),
            camera.yaw.0.sin() * camera.pitch.0.cos(),
        )
        .normalize();

        // WGPU depth correction
        let corr = cgmath::Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
        );

        for i in 0..4 {
            if i >= cascades {
                self.light_space_uniform.matrices[i] = cgmath::Matrix4::identity().into();
                self.light_space_uniform.cascade_splits[i] = 10000.0;
                continue;
            }

            let center_dist = if i == 0 {
                splits[i] * 0.5
            } else {
                (splits[i] + splits[i - 1]) * 0.5
            };
            let cascade_center = cam_pos + cam_forward * center_dist;

            let radius = if i == 0 {
                splits[i]
            } else {
                splits[i] - splits[i - 1] * 0.5
            } * 1.5;

            let light_pos_target = cascade_center + light_dir * radius * 2.0;
            let up_vector = if light_dir.y.abs() > 0.99 {
                cgmath::Vector3::unit_z()
            } else {
                cgmath::Vector3::unit_y()
            };
            let light_view = cgmath::Matrix4::look_at_rh(
                cgmath::Point3::new(light_pos_target.x, light_pos_target.y, light_pos_target.z),
                cgmath::Point3::new(cascade_center.x, cascade_center.y, cascade_center.z),
                up_vector,
            );

            let light_proj = cgmath::ortho(-radius, radius, -radius, radius, 0.1, radius * 5.0);

            let m: [[f32; 4]; 4] = (corr * light_proj * light_view).into();
            self.light_space_uniform.matrices[i] = m;
            self.light_space_uniform.cascade_splits[i] = splits[i];

            queue.write_buffer(
                &self.shadow_matrix_buffers[i],
                0,
                bytemuck::cast_slice(&[m]),
            );
        }

        self.light_space_uniform.shadow_enabled = if graphics_settings.shadow_enabled {
            1
        } else {
            0
        };
        self.light_space_uniform.shadow_bias = graphics_settings.shadow_bias;
        self.light_space_uniform.pcf_radius = graphics_settings.shadow_pcf.radius();
        queue.write_buffer(
            &self.light_space_buffer,
            0,
            bytemuck::cast_slice(&[self.light_space_uniform]),
        );
    }

    /// Executes depth-only shadow passes for all active cascades, rendering
    /// triangles, cubes, sphere, cylinder, capsule, torus, and imported models into the shadow map array.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        graphics_settings: &crate::graphics_settings::GraphicsSettings,
        primitives: &crate::render::primitives::GeometrySystem,
        instance_buffer: &wgpu::Buffer,
        all_instances_count: usize,
        triangle_instances_count: usize,
        tri_start: usize,
        cube_instances_count: usize,
        cube_start: usize,
        sphere_instances_count: usize,
        sphere_start: usize,
        cylinder_instances_count: usize,
        cylinder_start: usize,
        capsule_instances_count: usize,
        capsule_start: usize,
        torus_instances_count: usize,
        torus_start: usize,
        asset_manager: &crate::asset::AssetManager,
        model_instance_data: &HashMap<
            crate::asset::AssetHandle,
            Vec<crate::render::types::Instance>,
        >,
        model_starts: &HashMap<crate::asset::AssetHandle, usize>,
    ) {
        if !graphics_settings.shadow_enabled || all_instances_count == 0 {
            return;
        }

        let cascades = graphics_settings.shadow_cascades.clamp(1, 4) as usize;
        for cascade_idx in 0..cascades {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Shadow Pass {}", cascade_idx)),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_cascade_views[cascade_idx],
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_pass_bind_groups[cascade_idx], &[]);

            if triangle_instances_count > 0 {
                pass.set_vertex_buffer(0, primitives.vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    instance_buffer
                        .slice((tri_start * crate::render::types::INSTANCE_SIZE) as u64..),
                );
                pass.draw(0..3, 0..triangle_instances_count as u32);
            }
            if cube_instances_count > 0 {
                pass.set_vertex_buffer(0, primitives.cube_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    instance_buffer
                        .slice((cube_start * crate::render::types::INSTANCE_SIZE) as u64..),
                );
                pass.draw(0..36, 0..cube_instances_count as u32);
            }
            if sphere_instances_count > 0 {
                pass.set_vertex_buffer(0, primitives.sphere_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    instance_buffer
                        .slice((sphere_start * crate::render::types::INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..primitives.sphere_num_vertices,
                    0..sphere_instances_count as u32,
                );
            }
            if cylinder_instances_count > 0 {
                pass.set_vertex_buffer(0, primitives.cylinder_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    instance_buffer
                        .slice((cylinder_start * crate::render::types::INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..primitives.cylinder_num_vertices,
                    0..cylinder_instances_count as u32,
                );
            }
            if capsule_instances_count > 0 {
                pass.set_vertex_buffer(0, primitives.capsule_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    instance_buffer
                        .slice((capsule_start * crate::render::types::INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..primitives.capsule_num_vertices,
                    0..capsule_instances_count as u32,
                );
            }
            if torus_instances_count > 0 {
                pass.set_vertex_buffer(0, primitives.torus_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    instance_buffer
                        .slice((torus_start * crate::render::types::INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..primitives.torus_num_vertices,
                    0..torus_instances_count as u32,
                );
            }
            for (handle, m) in asset_manager.models.iter() {
                let c = model_instance_data
                    .get(&handle)
                    .map(|v| v.len())
                    .unwrap_or(0) as u32;
                if c > 0 {
                    pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        instance_buffer.slice(
                            ((*model_starts.get(&handle).unwrap_or(&0))
                                * crate::render::types::INSTANCE_SIZE)
                                as u64..,
                        ),
                    );
                    pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..m.num_indices, 0, 0..c);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// Tests that computing shadow cascades with a vertical light vector does not generate NaN values.
    #[test]
    fn test_shadow_cascade_vertical_light_no_nan() {
        let light_dir: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
        let up_vector = if light_dir.y.abs() > 0.99 {
            cgmath::Vector3::unit_z()
        } else {
            cgmath::Vector3::unit_y()
        };

        let cascade_center = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let radius = 10.0_f32;
        let light_pos_target = cascade_center + light_dir * radius * 2.0;

        let light_view = cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(light_pos_target.x, light_pos_target.y, light_pos_target.z),
            cgmath::Point3::new(cascade_center.x, cascade_center.y, cascade_center.z),
            up_vector,
        );

        let mat_arr: [[f32; 4]; 4] = light_view.into();
        for row in &mat_arr {
            for val in row {
                assert!(!val.is_nan(), "Light view matrix element is NaN!");
            }
        }
    }
}