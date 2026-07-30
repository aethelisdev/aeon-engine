// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, 1.0 - (y + 1.0) * 0.5);
    return out;
}

@group(0) @binding(0) var mask_texture: texture_2d<f32>;
@group(0) @binding(1) var mask_sampler: sampler;

struct OutlineUniforms {
    viewport_size: vec2<f32>,
    outline_color: vec4<f32>,
};
@group(0) @binding(2) var<uniform> uniforms: OutlineUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = vec2<f32>(1.0 / uniforms.viewport_size.x, 1.0 / uniforms.viewport_size.y);
    let center_mask = textureSample(mask_texture, mask_sampler, in.uv).r;

    let step = 1.5;
    let n0 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(0.0, -step) * texel_size).r;
    let n1 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(0.0, step) * texel_size).r;
    let n2 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(-step, 0.0) * texel_size).r;
    let n3 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(step, 0.0) * texel_size).r;

    let n4 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(-step, -step) * texel_size).r;
    let n5 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(step, -step) * texel_size).r;
    let n6 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(-step, step) * texel_size).r;
    let n7 = textureSample(mask_texture, mask_sampler, in.uv + vec2<f32>(step, step) * texel_size).r;

    let max_neighbor = max(max(max(n0, n1), max(n2, n3)), max(max(n4, n5), max(n6, n7)));
    let min_neighbor = min(min(min(n0, n1), min(n2, n3)), min(min(n4, n5), min(n6, n7)));

    let edge = max(max_neighbor - center_mask, max_neighbor - min_neighbor);

    if (edge > 0.01) {
        let alpha = clamp(edge * 2.5, 0.0, 1.0);
        return vec4<f32>(uniforms.outline_color.rgb, alpha * uniforms.outline_color.a);
    }

    discard;
}
