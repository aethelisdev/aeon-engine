// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
// Shadow Depth Pass Shader
// Opaque: depth-only (no fragment output)
// Cutout/Mask: alpha-tested discard fragment shader

struct CascadeUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> cascade: CascadeUniform;

@group(1) @binding(0)
var t_shadow_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_shadow_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(6) normal: vec3<f32>,
    @location(8) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(7) color: vec4<f32>,
}

@vertex
fn vs_shadow(
    @location(0) position: vec3<f32>,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let world_pos = model_matrix * vec4<f32>(position, 1.0);
    return cascade.matrix * world_pos;
}

struct ShadowCutoutVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_shadow_cutout(
    model: VertexInput,
    instance: InstanceInput,
) -> ShadowCutoutVertexOutput {
    var out: ShadowCutoutVertexOutput;
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let world_pos = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = cascade.matrix * world_pos;
    out.uv = model.uv;
    return out;
}

@fragment
fn fs_shadow_cutout(in: ShadowCutoutVertexOutput) {
    let tex_color = textureSample(t_shadow_diffuse, s_shadow_diffuse, in.uv);
    if (tex_color.a < 0.5) {
        discard;
    }
}


