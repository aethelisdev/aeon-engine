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
    grid_scale: f32,       // viewport grid scale multiplier (offset 12..16)
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
    // Grid quad always follows the camera — infinite grid illusion (elevated 0.001 to prevent Z-fighting)
    let world_p = vec3<f32>(model.position.x + camera.camera_pos.x, 0.001, model.position.z + camera.camera_pos.z);
    
    out.clip_position = camera.view_proj * vec4<f32>(world_p, 1.0);
    out.world_pos = world_p;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Dynamic grid scale from editor snap size (e.g., 0.1, 0.5, 1.0, 5.0, 10.0)
    let scale = max(light.grid_scale, 0.05);
    let coord = in.world_pos.xz / scale;
    
    // Geometric derivatives per pixel for screen-space anti-aliasing
    let deriv1 = max(fwidth(coord), vec2<f32>(0.00001, 0.00001));
    
    // 1. Minor grid line calculation with Moiré-suppression LOD (cell size = scale)
    let grid1 = abs(fract(coord - 0.5) - 0.5) / deriv1;
    let line1 = min(grid1.x, grid1.y);
    let raw_alpha1 = 1.0 - clamp(line1, 0.0, 1.0);
    // Smoothly fade out minor grid when screen lines get closer than 8 pixels
    let lod1 = clamp(1.0 - max(deriv1.x, deriv1.y) * 8.0, 0.0, 1.0);
    let minor_alpha = raw_alpha1 * lod1 * 0.22; // Subtle 22% peak opacity for fine lines

    // 2. Major 10x grid line calculation for structural scale (cell size = scale * 10.0)
    let coord10 = coord * 0.1;
    let deriv10 = max(fwidth(coord10), vec2<f32>(0.00001, 0.00001));
    let grid10 = abs(fract(coord10 - 0.5) - 0.5) / deriv10;
    let line10 = min(grid10.x, grid10.y);
    let raw_alpha10 = 1.0 - clamp(line10, 0.0, 1.0);
    // Major grid persists further, fading when lines get closer than 4 pixels
    let lod10 = clamp(1.0 - max(deriv10.x, deriv10.y) * 4.0, 0.0, 1.0);
    let major_alpha = raw_alpha10 * lod10 * 0.42; // Defined 42% peak opacity for major lines

    // 3. Neutral center origin axes (X = 0 and Z = 0) in screen pixels (fixed 1-pixel screen width)
    let world_deriv = deriv1 * scale;
    let axis_pixel_z = abs(in.world_pos.z) / world_deriv.y; // Distance to X-axis in screen pixels
    let axis_pixel_x = abs(in.world_pos.x) / world_deriv.x; // Distance to Z-axis in screen pixels
    let axis_dist = min(axis_pixel_z, axis_pixel_x);
    let raw_axis_alpha = 1.0 - clamp(axis_dist, 0.0, 1.0);
    let axis_alpha = raw_axis_alpha * 0.65; // 65% peak opacity for central origin axes

    // Neutral color palette: distraction-free studio standards
    // Minor: dark slate | Major: steel slate | Axes: refined neutral silver-slate
    var color = vec3<f32>(0.36, 0.38, 0.42);
    var combined_alpha = minor_alpha;

    if (major_alpha > combined_alpha) {
        let t = clamp((major_alpha - combined_alpha) / max(major_alpha, 0.001), 0.0, 1.0);
        color = mix(color, vec3<f32>(0.48, 0.51, 0.56), t);
        combined_alpha = max(combined_alpha, major_alpha);
    }

    if (axis_alpha > combined_alpha) {
        let t = clamp((axis_alpha - combined_alpha) / max(axis_alpha, 0.001), 0.0, 1.0);
        color = mix(color, vec3<f32>(0.62, 0.65, 0.72), t);
        combined_alpha = max(combined_alpha, axis_alpha);
    }

    // Altitude and scale aware spherical distance falloff (smoothstep)
    let dist = length(camera.camera_pos.xyz - in.world_pos);
    let altitude = max(abs(camera.camera_pos.y), 2.0);
    var max_dist = max(75.0 * sqrt(scale), altitude * 3.5);

    // If scene fog is configured with a nearer limit, synchronize with fog
    if (light.fog_params.w > 0.0 && light.fog_params.w < max_dist) {
        max_dist = light.fog_params.w;
    }

    // Smooth Hermite falloff: fully visible within 25% of radius, smoothly vanishing at boundary
    let fade = smoothstep(max_dist, max_dist * 0.25, dist);
    combined_alpha = combined_alpha * fade;

    if (combined_alpha <= 0.005) {
        discard;
    }

    return vec4<f32>(color, combined_alpha);
}
