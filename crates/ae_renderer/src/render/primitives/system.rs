// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::data::{CUBE_VERTICES, GRID_QUAD_VERTICES, QUAD_VERTICES, VERTICES};
use super::generators::{generate_capsule, generate_cylinder, generate_sphere, generate_torus};
use wgpu::util::DeviceExt;

/// Encapsulates static geometry buffers and dynamic instance chunks to decouple them from the main RenderState.
pub struct GeometrySystem {
    /// Vertex buffer for flat debug triangles.
    pub vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for unit cubes.
    pub cube_vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for infinite ground grid.
    pub grid_vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for billboard quads.
    pub quad_vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for parametric spheres.
    pub sphere_vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for parametric cylinders.
    pub cylinder_vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for parametric capsules.
    pub capsule_vertex_buffer: wgpu::Buffer,
    /// Vertex buffer for parametric toruses.
    pub torus_vertex_buffer: wgpu::Buffer,
    /// Number of vertices in the sphere model.
    pub sphere_num_vertices: u32,
    /// Number of vertices in the cylinder model.
    pub cylinder_num_vertices: u32,
    /// Number of vertices in the capsule model.
    pub capsule_num_vertices: u32,
    /// Number of vertices in the torus model.
    pub torus_num_vertices: u32,
    /// Dynamic GPU buffer containing instance data for all drawn models.
    pub instance_buffer: wgpu::Buffer,
    /// Capacity of the active instance buffer.
    pub instance_buffer_capacity: usize,
}

impl GeometrySystem {
    /// Creates all static vertex buffers and an initial dynamic instance buffer (25k capacity).
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Buffer"),
            contents: bytemuck::cast_slice(CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Buffer"),
            contents: bytemuck::cast_slice(GRID_QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Generate parametric shapes
        let sphere_vertices = generate_sphere(20, 20);
        let sphere_num_vertices = sphere_vertices.len() as u32;
        let sphere_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Buffer"),
            contents: bytemuck::cast_slice(&sphere_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cylinder_vertices = generate_cylinder(24);
        let cylinder_num_vertices = cylinder_vertices.len() as u32;
        let cylinder_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cylinder Buffer"),
            contents: bytemuck::cast_slice(&cylinder_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let capsule_vertices = generate_capsule(24, 12);
        let capsule_num_vertices = capsule_vertices.len() as u32;
        let capsule_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Capsule Buffer"),
            contents: bytemuck::cast_slice(&capsule_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let torus_vertices = generate_torus(24, 16);
        let torus_num_vertices = torus_vertices.len() as u32;
        let torus_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Torus Buffer"),
            contents: bytemuck::cast_slice(&torus_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer_capacity = 25000;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (instance_buffer_capacity * std::mem::size_of::<crate::render::types::Instance>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            cube_vertex_buffer,
            grid_vertex_buffer,
            quad_vertex_buffer,
            sphere_vertex_buffer,
            cylinder_vertex_buffer,
            capsule_vertex_buffer,
            torus_vertex_buffer,
            sphere_num_vertices,
            cylinder_num_vertices,
            capsule_num_vertices,
            torus_num_vertices,
            instance_buffer,
            instance_buffer_capacity,
        }
    }

    /// Writes instance data to the GPU buffer, auto-scaling capacity when needed.
    /// # Memory Leak Prevention (WGPU Buffer Allocation Lifecycle)
    /// Overwriting `self.instance_buffer` directly causes the old buffer to be dropped on the
    /// Rust side, but the underlying Vulkan/DX12/Metal resource allocation stays alive inside the
    /// WGPU internal resource registry and GPU drivers until an explicit device command or GC sweep.
    /// We call `self.instance_buffer.destroy()` before allocating a new buffer to immediately free
    /// the virtual memory and VRAM allocations, preventing massive multi-gigabyte memory leaks
    /// during rapid stress-test scaling (100k+ instances).
    pub fn update_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        all_instances: &[crate::render::types::Instance],
    ) {
        if all_instances.is_empty() {
            return;
        }

        if all_instances.len() > self.instance_buffer_capacity {
            self.instance_buffer_capacity = all_instances.len().next_power_of_two();

            // Explicitly destroy the old buffer to prevent severe memory leaks in the GPU driver
            self.instance_buffer.destroy();

            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer (Resized Up)"),
                size: (self.instance_buffer_capacity
                    * std::mem::size_of::<crate::render::types::Instance>())
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            log::info!(
                "Upscaled GeometrySystem instance buffer to {} elements",
                self.instance_buffer_capacity
            );
        } else if self.instance_buffer_capacity > 50000
            && all_instances.len() < self.instance_buffer_capacity / 4
        {
            let target_capacity = all_instances.len().next_power_of_two().max(25000);
            if target_capacity < self.instance_buffer_capacity {
                self.instance_buffer_capacity = target_capacity;

                // Explicitly destroy the old buffer to prevent severe memory leaks in the GPU driver
                self.instance_buffer.destroy();

                self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instance Buffer (Shrunken Down)"),
                    size: (self.instance_buffer_capacity
                        * std::mem::size_of::<crate::render::types::Instance>())
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                log::info!(
                    "Shrunk GeometrySystem instance buffer to {} elements",
                    self.instance_buffer_capacity
                );
            }
        }

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(all_instances),
        );
    }
}