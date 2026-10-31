// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
// src/gizmo.wgsl
// The entire gizmo is drawn with a single MVP matrix.
// - Flat colors are provided per-vertex.
// - Screen-Space Distance Field (SDF) provides sub-pixel anti-aliasing for rings and circular handles.
struct GizmoUniform {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> gizmo_uniform: GizmoUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // MVP transform (from model space to clip space).

