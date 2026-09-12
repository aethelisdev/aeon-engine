// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport 3D Billboard Shader
//!
//! Renders camera-facing, screen-space size-invariant billboard icon badges
//! for scene entities (Light, Camera, Audio) directly inside the 3D viewport forward pass.

struct CameraUniform {
    view_proj: mat4x4<f32>,
    viewport_size: vec2<f32>,
    icon_size_px: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_atlas: texture_2d_array<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

struct VertexInput {
    @location(0) world_pos: vec3<f32>,
    @location(1) quad_offset: vec2<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) layer: f32,
    @location(4) bg_color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) icon_tint: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) quad_offset: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) layer: u32,
    @location(3) bg_color: vec4<f32>,
    @location(4) border_color: vec4<f32>,
    @location(5) icon_tint: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let clip_center = camera.view_proj * vec4<f32>(in.world_pos, 1.0);

    let pixel_scale = (camera.icon_size_px * 0.5) / camera.viewport_size * 2.0;
    let ndc_offset = in.quad_offset * pixel_scale;

    out.clip_position = vec4<f32>(
        clip_center.xy + ndc_offset * clip_center.w,
        clip_center.z,
        clip_center.w
    );
    out.quad_offset = in.quad_offset;
    out.uv = in.uv;
    out.layer = u32(round(in.layer));
    out.bg_color = in.bg_color;
    out.border_color = in.border_color;
    out.icon_tint = in.icon_tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.quad_offset);
    if (dist > 1.0) {
        discard;
    }

    let delta = max(fwidth(dist), 0.002);
    let outer_alpha = 1.0 - smoothstep(1.0 - delta, 1.0, dist);

    let border_inner = 0.85;
    let border_factor = smoothstep(border_inner - delta, border_inner + delta, dist);
    let badge_rgb = mix(in.bg_color.rgb, in.border_color.rgb, border_factor);
    let badge_a = mix(in.bg_color.a, in.border_color.a, border_factor) * outer_alpha;

    var final_color = vec4<f32>(badge_rgb, badge_a);

    let icon_min = 0.18;
    let icon_max = 0.82;
    let icon_size = icon_max - icon_min;

    if (in.uv.x >= icon_min && in.uv.x <= icon_max && in.uv.y >= icon_min && in.uv.y <= icon_max) {
        let inner_uv = (in.uv - vec2<f32>(icon_min, icon_min)) / icon_size;
        let icon_sample = textureSample(t_atlas, s_atlas, inner_uv, in.layer);
        let icon_col = icon_sample * in.icon_tint;

        let out_a = icon_col.a + final_color.a * (1.0 - icon_col.a);
        let out_rgb = (icon_col.rgb * icon_col.a + final_color.rgb * final_color.a * (1.0 - icon_col.a)) / max(out_a, 0.0001);
        final_color = vec4<f32>(out_rgb, out_a);
    }

    if (final_color.a <= 0.01) {
        discard;
    }
    return final_color;
}
