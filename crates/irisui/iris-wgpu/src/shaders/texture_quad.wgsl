// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

// Iris UI GPU Textured Quad Shader
// Renders 2D texture array views (editor icon layers, textures) inside UI bounds.

struct Uniforms {
    screen_size: vec2<f32>,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_globals: Uniforms;

@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>, // Unit quad [0.0 .. 1.0]
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,        // [x, y, width, height]
    @location(2) uv_rect: vec4<f32>,     // [min_u, min_v, max_u, layer_index]
    @location(3) tint: vec4<f32>,        // Tint RGBA
    @location(4) clip_rect: vec4<f32>,   // [min_x, min_y, max_x, max_y]
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) screen_pos: vec2<f32>,
    @location(3) clip_bounds: vec4<f32>,
    @location(4) @interpolate(flat) layer_index: u32,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let pixel_pos = instance.rect.xy + vertex.position * instance.rect.zw;
    let ndc_x = (pixel_pos.x / u_globals.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / u_globals.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    let uv_max = vec2<f32>(instance.uv_rect.z, instance.uv_rect.z);
    out.uv = mix(instance.uv_rect.xy, uv_max, vertex.position);
    out.tint = instance.tint;
    out.screen_pos = pixel_pos;
    out.clip_bounds = instance.clip_rect;
    out.layer_index = u32(round(instance.uv_rect.w));

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Scissor / clipping rectangle test
    if (in.clip_bounds.z > 0.0 && in.clip_bounds.w > 0.0) {
        if (in.screen_pos.x < in.clip_bounds.x || in.screen_pos.x > in.clip_bounds.z ||
            in.screen_pos.y < in.clip_bounds.y || in.screen_pos.y > in.clip_bounds.w) {
            discard;
        }
    }

    let sampled = textureSample(t_diffuse, s_diffuse, in.uv, in.layer_index);
    return sampled * in.tint;
}
