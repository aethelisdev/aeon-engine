// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,   // Grid shader doesn't use these but must declare them
    proj_inv: mat4x4<f32>,   // to keep the correct memory layout (208 bytes total)
    camera_pos: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Must match the CPU-side LightUniform struct exactly
struct Light {
    direction: vec3<f32>,  // normalized vector towards the sun
    color: vec3<f32>,
    ambient_color: vec3<f32>,
    fog_params: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(6) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Grid quad always follows the camera — infinite grid illusion
    let world_p = vec3<f32>(model.position.x + camera.camera_pos.x, 0.0, model.position.z + camera.camera_pos.z);
    
    out.clip_position = camera.view_proj * vec4<f32>(world_p, 1.0);
    out.world_pos = world_p;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coord = in.world_pos.xz;
    
    // Anti-Aliasing: geometric derivative per pixel
    let deriv = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / deriv;
    let line_val = min(grid.x, grid.y);
    
    var color = vec3<f32>(0.25, 0.25, 0.25); // Neutral grid line color
    let axis_width = max(deriv.x, deriv.y) * 1.5;
    
    // Axis colors
    if (abs(in.world_pos.z) < axis_width) {
        color = vec3<f32>(0.9, 0.15, 0.15); // X Axis — Red
    }
    if (abs(in.world_pos.x) < axis_width) {
        color = vec3<f32>(0.15, 0.5, 0.9);  // Z Axis — Blue
    }

    // Line alpha
    var alpha = 1.0 - min(line_val, 1.0);
    
    // Moiré prevention: fade out lines only in extreme cases
    let moire_fade = clamp(1.0 - max(deriv.x, deriv.y) * 0.3, 0.0, 1.0);
    alpha = alpha * moire_fade;

    // Distance fade: fades out completely up to Fog distance or default 300.0 units
    var target_dist = 300.0;
    if (light.fog_params.w > 0.0) {
        target_dist = light.fog_params.w;
    }
    
    let dist = length(camera.camera_pos.xyz - in.world_pos);
    let fade = 1.0 - clamp(dist / target_dist, 0.0, 1.0);
    
    alpha = alpha * fade;
    
    if (alpha <= 0.01) {
        discard;
    }
    
    return vec4<f32>(color, alpha);
}
