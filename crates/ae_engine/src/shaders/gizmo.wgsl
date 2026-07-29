// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
// src/gizmo.wgsl
// The entire gizmo is drawn with a single MVP matrix.
// - Constant colors come from the Vertex.
// - Axis scaling is bound to the model matrix on the Rust side.
struct GizmoUniform {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> gizmo_uniform: GizmoUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // MVP transform (from model space to clip space).
    out.clip_position = gizmo_uniform.mvp * vec4<f32>(in.position, 1.0);
    // Axis color remains constant throughout rendering.
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Gizmo is unlit/flat-shaded (readability is prioritized for the transform tool).
    return vec4<f32>(in.color, 1.0);
}
