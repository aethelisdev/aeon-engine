// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::render::types::{SpriteVertex, Vertex};

/// Default 3D Pyramid (Triangle primitive) mesh vertices (18 vertices, 4 side faces + 1 bottom quad).
pub const VERTICES: &[Vertex] = &[
    // Front face (Z+)
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.4472136, 0.8944272],
        uv: [0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.4472136, 0.8944272],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.4472136, 0.8944272],
        uv: [1.0, 1.0],
    },
    // Right face (X+)
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 1.0, 1.0],
        normal: [0.8944272, 0.4472136, 0.0],
        uv: [0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.8944272, 0.4472136, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.8944272, 0.4472136, 0.0],
        uv: [1.0, 1.0],
    },
    // Back face (Z-)
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.4472136, -0.8944272],
        uv: [0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.4472136, -0.8944272],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, 0.4472136, -0.8944272],
        uv: [1.0, 1.0],
    },
    // Left face (X-)
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 1.0, 1.0],
        normal: [-0.8944272, 0.4472136, 0.0],
        uv: [0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-0.8944272, 0.4472136, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [-0.8944272, 0.4472136, 0.0],
        uv: [1.0, 1.0],
    },
    // Bottom face (Y-) Triangle 1
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 0.0],
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
    // Bottom face (Y-) Triangle 2
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 0.0],
    },
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