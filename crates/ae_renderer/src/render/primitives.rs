// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::types::{SpriteVertex, Vertex};
use wgpu::util::DeviceExt;

/// Default debug triangle mesh vertices.
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.2],
        normal: [0.0, 0.0, 1.0],
        uv: [0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.5],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.2, 0.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    },
];

/// Unit cube mesh (36 vertices, 6 faces, no index buffer).
pub const CUBE_VERTICES: &[Vertex] = &[
    // Front (Z+)
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    },
    // Back (Z-)
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 1.0],
    },
    // Top (Y+)
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    // Bottom (Y-)
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 0.0],
    },
    // Right (X+)
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    // Left (X-)
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
    },
];

/// Billboard quad vertices for sprite rendering.
pub const QUAD_VERTICES: &[SpriteVertex] = &[
    SpriteVertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    SpriteVertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    SpriteVertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    SpriteVertex {
        position: [0.5, 0.5, 0.0],
        uv: [1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
];

/// Large ground-plane quad for the infinite grid shader.
pub const GRID_QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-2000.0, 0.0, -2000.0],
        color: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [2000.0, 0.0, -2000.0],
        color: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-2000.0, 0.0, 2000.0],
        color: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [2000.0, 0.0, -2000.0],
        color: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [2000.0, 0.0, 2000.0],
        color: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-2000.0, 0.0, 2000.0],
        color: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 1.0],
    },
];

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

/// Generates a parametric UV Sphere mesh with smooth normals.
/// latitude_bands and longitude_bands determine resolution (smoothness).
pub fn generate_sphere(latitude_bands: u32, longitude_bands: u32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let mut raw_vertices = Vec::new();

    for lat in 0..=latitude_bands {
        let theta = lat as f32 * std::f32::consts::PI / latitude_bands as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=longitude_bands {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / longitude_bands as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            let u = lon as f32 / longitude_bands as f32;
            let v = lat as f32 / latitude_bands as f32;

            // Sphere radius is 0.5 (to fit a unit bounding box of size 1x1x1)
            raw_vertices.push(Vertex {
                position: [x * 0.5, y * 0.5, z * 0.5],
                color: [1.0, 1.0, 1.0],
                normal: [x, y, z],
                uv: [u, v],
            });
        }
    }

    for lat in 0..latitude_bands {
        for lon in 0..longitude_bands {
            let first = (lat * (longitude_bands + 1) + lon) as usize;
            let second = first + (longitude_bands + 1) as usize;

            // Triangle 1
            vertices.push(raw_vertices[first].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[first + 1].clone());

            // Triangle 2
            vertices.push(raw_vertices[first + 1].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[second + 1].clone());
        }
    }
    vertices
}

/// Generates a parametric Cylinder mesh with smooth normals and caps.
/// segments determines circle resolution. Fits in a unit box (radius=0.5, height=1.0).
pub fn generate_cylinder(segments: u32) -> Vec<Vertex> {
    let mut vertices = Vec::new();

    let top_center = [0.0, 0.5, 0.0];
    let bottom_center = [0.0, -0.5, 0.0];

    let mut circle_pts = Vec::new();
    for i in 0..=segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        circle_pts.push((angle.cos() * 0.5, angle.sin() * 0.5));
    }

    for i in 0..segments as usize {
        let (x0, z0) = circle_pts[i];
        let (x1, z1) = circle_pts[i + 1];

        let n0 = [x0 * 2.0, 0.0, z0 * 2.0];
        let n1 = [x1 * 2.0, 0.0, z1 * 2.0];

        let u0 = i as f32 / segments as f32;
        let u1 = (i + 1) as f32 / segments as f32;

        // Side Triangle 1
        vertices.push(Vertex {
            position: [x0, 0.5, z0],
            color: [1.0; 3],
            normal: n0,
            uv: [u0, 0.0],
        });
        vertices.push(Vertex {
            position: [x0, -0.5, z0],
            color: [1.0; 3],
            normal: n0,
            uv: [u0, 1.0],
        });
        vertices.push(Vertex {
            position: [x1, 0.5, z1],
            color: [1.0; 3],
            normal: n1,
            uv: [u1, 0.0],
        });

        // Side Triangle 2
        vertices.push(Vertex {
            position: [x1, 0.5, z1],
            color: [1.0; 3],
            normal: n1,
            uv: [u1, 0.0],
        });
        vertices.push(Vertex {
            position: [x0, -0.5, z0],
            color: [1.0; 3],
            normal: n0,
            uv: [u0, 1.0],
        });
        vertices.push(Vertex {
            position: [x1, -0.5, z1],
            color: [1.0; 3],
            normal: n1,
            uv: [u1, 1.0],
        });

        // Top cap
        vertices.push(Vertex {
            position: top_center,
            color: [1.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
        });
        vertices.push(Vertex {
            position: [x1, 0.5, z1],
            color: [1.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [x1 + 0.5, z1 + 0.5],
        });
        vertices.push(Vertex {
            position: [x0, 0.5, z0],
            color: [1.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [x0 + 0.5, z0 + 0.5],
        });

        // Bottom cap
        vertices.push(Vertex {
            position: bottom_center,
            color: [1.0; 3],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5, 0.5],
        });
        vertices.push(Vertex {
            position: [x0, -0.5, z0],
            color: [1.0; 3],
            normal: [0.0, -1.0, 0.0],
            uv: [x0 + 0.5, z0 + 0.5],
        });
        vertices.push(Vertex {
            position: [x1, -0.5, z1],
            color: [1.0; 3],
            normal: [0.0, -1.0, 0.0],
            uv: [x1 + 0.5, z1 + 0.5],
        });
    }

    vertices
}

/// Generates a parametric Capsule mesh (hemispheres on top and bottom of a cylinder).
pub fn generate_capsule(segments: u32, rings: u32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let mut raw_vertices = Vec::new();

    let radius = 0.35;
    let cylinder_half_height = 0.15; // Total capsule height = 1.0

    // Top hemisphere
    for ring in 0..=rings {
        let theta = (ring as f32 * 0.5 * std::f32::consts::PI) / rings as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for seg in 0..=segments {
            let phi = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * cos_phi * sin_theta;
            let y = cylinder_half_height + radius * cos_theta;
            let z = radius * sin_phi * sin_theta;

            let u = seg as f32 / segments as f32;
            let v = ring as f32 / (rings * 2) as f32;

            raw_vertices.push(Vertex {
                position: [x, y, z],
                color: [1.0; 3],
                normal: [x / radius, (y - cylinder_half_height) / radius, z / radius],
                uv: [u, v],
            });
        }
    }

    // Bottom hemisphere
    let base_idx = raw_vertices.len();
    for ring in 0..=rings {
        let theta =
            std::f32::consts::PI * 0.5 + (ring as f32 * 0.5 * std::f32::consts::PI) / rings as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for seg in 0..=segments {
            let phi = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * cos_phi * sin_theta;
            let y = -cylinder_half_height + radius * cos_theta;
            let z = radius * sin_phi * sin_theta;

            let u = seg as f32 / segments as f32;
            let v = 0.5 + (ring as f32 / (rings * 2) as f32);

            raw_vertices.push(Vertex {
                position: [x, y, z],
                color: [1.0; 3],
                normal: [x / radius, (y + cylinder_half_height) / radius, z / radius],
                uv: [u, v],
            });
        }
    }

    // Top hemisphere triangles
    for ring in 0..rings {
        for seg in 0..segments {
            let first = (ring * (segments + 1) + seg) as usize;
            let second = first + (segments + 1) as usize;

            vertices.push(raw_vertices[first].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[first + 1].clone());

            vertices.push(raw_vertices[first + 1].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[second + 1].clone());
        }
    }

    // Bottom hemisphere triangles
    for ring in 0..rings {
        for seg in 0..segments {
            let first = (base_idx + (ring * (segments + 1) + seg) as usize) as usize;
            let second = first + (segments + 1) as usize;

            vertices.push(raw_vertices[first].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[first + 1].clone());

            vertices.push(raw_vertices[first + 1].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[second + 1].clone());
        }
    }

    // Side cylinder triangles
    let top_rim_start = (rings * (segments + 1)) as usize;
    let bottom_rim_start = base_idx;
    for seg in 0..segments {
        let t0 = top_rim_start + seg as usize;
        let t1 = t0 + 1;
        let b0 = bottom_rim_start + seg as usize;
        let b1 = b0 + 1;

        vertices.push(raw_vertices[t0].clone());
        vertices.push(raw_vertices[b0].clone());
        vertices.push(raw_vertices[t1].clone());

        vertices.push(raw_vertices[t1].clone());
        vertices.push(raw_vertices[b0].clone());
        vertices.push(raw_vertices[b1].clone());
    }

    vertices
}

/// Generates a parametric Torus mesh.
pub fn generate_torus(radial_segments: u32, tubular_segments: u32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let mut raw_vertices = Vec::new();

    let main_radius = 0.35;
    let tube_radius = 0.15;

    for r_seg in 0..=radial_segments {
        let u = r_seg as f32 * 2.0 * std::f32::consts::PI / radial_segments as f32;
        let cos_u = u.cos();
        let sin_u = u.sin();

        for t_seg in 0..=tubular_segments {
            let v = t_seg as f32 * 2.0 * std::f32::consts::PI / tubular_segments as f32;
            let cos_v = v.cos();
            let sin_v = v.sin();

            let x = (main_radius + tube_radius * cos_v) * cos_u;
            let y = tube_radius * sin_v;
            let z = (main_radius + tube_radius * cos_v) * sin_u;

            let nx = cos_v * cos_u;
            let ny = sin_v;
            let nz = cos_v * sin_u;

            let tex_u = r_seg as f32 / radial_segments as f32;
            let tex_v = t_seg as f32 / tubular_segments as f32;

            raw_vertices.push(Vertex {
                position: [x, y, z],
                color: [1.0; 3],
                normal: [nx, ny, nz],
                uv: [tex_u, tex_v],
            });
        }
    }

    for r_seg in 0..radial_segments {
        for t_seg in 0..tubular_segments {
            let first = (r_seg * (tubular_segments + 1) + t_seg) as usize;
            let second = first + (tubular_segments + 1) as usize;

            vertices.push(raw_vertices[first].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[first + 1].clone());

            vertices.push(raw_vertices[first + 1].clone());
            vertices.push(raw_vertices[second].clone());
            vertices.push(raw_vertices[second + 1].clone());
        }
    }

    vertices
}