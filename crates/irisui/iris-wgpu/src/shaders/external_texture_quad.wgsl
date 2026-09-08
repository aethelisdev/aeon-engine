// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

// Iris UI external Texture2D compositing shader.
// Samples a single-layer resolved render target and applies optional fragment clipping.

struct Uniforms {
    screen_size: vec2<f32>,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_globals: Uniforms;

@group(1) @binding(0)
var t_external: texture_2d<f32>;
@group(1) @binding(1)
var s_external: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) tint: vec4<f32>,
    @location(4) clip_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) screen_pos: vec2<f32>,
    @location(3) clip_bounds: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let pixel_pos = instance.rect.xy + vertex.position * instance.rect.zw;
    let ndc_x = (pixel_pos.x / u_globals.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / u_globals.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = mix(instance.uv_rect.xy, instance.uv_rect.zw, vertex.position);
    out.tint = instance.tint;
    out.screen_pos = pixel_pos;
    out.clip_bounds = instance.clip_rect;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.clip_bounds.x <= in.clip_bounds.z && in.clip_bounds.y <= in.clip_bounds.w) {
        if (in.screen_pos.x < in.clip_bounds.x || in.screen_pos.x > in.clip_bounds.z ||
            in.screen_pos.y < in.clip_bounds.y || in.screen_pos.y > in.clip_bounds.w) {
            discard;
        }
    }

    return textureSample(t_external, s_external, in.uv) * in.tint;
}
