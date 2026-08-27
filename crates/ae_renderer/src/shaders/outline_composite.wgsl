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
    _padding: vec2<f32>,
    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,
};
@group(0) @binding(2) var<uniform> uniforms: OutlineUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = vec2<f32>(1.0 / uniforms.viewport_size.x, 1.0 / uniforms.viewport_size.y);
    let center = textureSample(mask_texture, mask_sampler, in.uv);
    let center_sel = center.r;
    let center_id = center.g;

    // 16-Tap Dual-Ring Sub-Pixel Anti-Aliasing Kernel
    // Inner ring (R = 0.75px, 4 taps) + Outer ring (R = 1.40px, 12 taps at 30-degree increments)
    let inner_offsets = array<vec2<f32>, 4>(
        vec2<f32>(0.0, -0.75),
        vec2<f32>(0.0, 0.75),
        vec2<f32>(-0.75, 0.0),
        vec2<f32>(0.75, 0.0),
    );

    let outer_offsets = array<vec2<f32>, 12>(
        vec2<f32>(1.40, 0.0),
        vec2<f32>(1.212, 0.70),
        vec2<f32>(0.70, 1.212),
        vec2<f32>(0.0, 1.40),
        vec2<f32>(-0.70, 1.212),
        vec2<f32>(-1.212, 0.70),
        vec2<f32>(-1.40, 0.0),
        vec2<f32>(-1.212, -0.70),
        vec2<f32>(-0.70, -1.212),
        vec2<f32>(0.0, -1.40),
        vec2<f32>(0.70, -1.212),
        vec2<f32>(1.212, -0.70),
    );

    var max_sel: f32 = center_sel;
    var has_primary: bool = center_sel > 0.75;
    var edge_weight: f32 = 0.0;

    // 1. Evaluate Inner Ring (Higher weight for line core sharpness)
    for (var i = 0u; i < 4u; i++) {
        let n = textureSample(mask_texture, mask_sampler, in.uv + inner_offsets[i] * texel_size);
        if (n.r > max_sel) {
            max_sel = n.r;
        }
        if (n.r > 0.75) {
            has_primary = true;
        }
        if (abs(n.g - center_id) > 0.002) {
            edge_weight += 1.4;
        }
    }

    // 2. Evaluate Outer Ring (Smoother weight for sub-pixel anti-aliased gradient)
    for (var i = 0u; i < 12u; i++) {
        let n = textureSample(mask_texture, mask_sampler, in.uv + outer_offsets[i] * texel_size);
        if (n.r > max_sel) {
            max_sel = n.r;
        }
        if (n.r > 0.75) {
            has_primary = true;
        }
        if (abs(n.g - center_id) > 0.002) {
            edge_weight += 0.8;
        }
    }

    if (edge_weight > 0.8 && max_sel > 0.05) {
        // Continuous smoothstep coverage mapping for silky smooth anti-aliased edges
        let coverage = smoothstep(0.8, 6.5, edge_weight);
        let alpha = clamp(coverage * 1.05, 0.0, 1.0);
        let outline_color = select(uniforms.secondary_color, uniforms.primary_color, has_primary);
        return vec4<f32>(outline_color.rgb, alpha * outline_color.a);
    }

    discard;
}
