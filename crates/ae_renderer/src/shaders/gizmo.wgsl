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
    out.clip_position = gizmo_uniform.mvp * vec4<f32>(in.position, 1.0);
    // Axis color remains constant throughout drawing.
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Mode 1: Analytical SDF Anti-Aliased Circle for Center O-Ring (in.uv in [-1.0, 1.0])
    let uv_len = length(in.uv);
    if (uv_len > 0.0) {
        let ring_radius = 0.85;
        let ring_half_width = 0.055;
        let dist = abs(uv_len - ring_radius);
        
        // 1-cycle hardware screen-space derivative for exact sub-pixel smoothing
        let delta = max(fwidth(dist), 0.001);
        let alpha = 1.0 - smoothstep(ring_half_width - delta, ring_half_width + delta, dist);
        
        if (alpha <= 0.01) {
            discard;
        }
        return vec4<f32>(in.color, alpha);
    }

    // Mode 2: Standard 3D geometry
    return vec4<f32>(in.color, 1.0);
}
