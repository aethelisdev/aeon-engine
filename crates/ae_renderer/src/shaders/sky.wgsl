// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
    position: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct SkyUniform {
    sun_direction: vec4<f32>, // w unused
    sun_color: vec4<f32>,     // rgb, w=intensity
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    atmosphere_density: f32,
    sun_disc_size: f32,
    sun_glow_strength: f32,
    sky_quality_mode: u32,
};
@group(1) @binding(0)
var<uniform> sky: SkyUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) view_direction: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate a full screen triangle
    let x = f32((in_vertex_index & 1u) << 2u);
    let y = f32((in_vertex_index & 2u) << 1u);
    let pos_ndc = vec4<f32>(x - 1.0, 1.0 - y, 1.0, 1.0); // Z=1.0 for farthest depth

    out.clip_position = pos_ndc;

    // Remove translation from view matrix so sky doesn't move with camera
    var view_rot_inv = camera.view_inv;
    view_rot_inv[3][0] = 0.0;
    view_rot_inv[3][1] = 0.0;
    view_rot_inv[3][2] = 0.0;
    
    let unprojected = camera.proj_inv * pos_ndc;
    out.view_direction = (view_rot_inv * vec4<f32>(unprojected.xyz, 0.0)).xyz;

    return out;
}

// ── Quality 0: Low (Artistic) ──
fn calculate_low_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let zenith_angle = max(0.0, dir.y);
    let t = clamp(pow(zenith_angle, 0.7), 0.0, 1.0);
    var color = mix(sky.horizon_color.rgb, sky.zenith_color.rgb, t);
    
    // Sun disc — matches High/Medium visual size: tight core + corona shell
    let sun_dot = dot(dir, sun_dir);
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let core_r   = sky.sun_disc_size * 0.022; // 2x of High's 0.011 core
    let corona_r = sky.sun_disc_size * 0.05;  // 2x of High's 0.025 corona
    let sun_core   = exp(-pow(angular_dist / max(1e-5, core_r),   4.0));
    let sun_corona = exp(-pow(angular_dist / max(1e-5, corona_r), 2.0)) * 0.6;
    let disc = clamp(sun_core + sun_corona, 0.0, 1.0) * step(0.0, dir.y);
    if disc > 0.0 {
        let limb_dark = pow(clamp(1.0 - angular_dist / max(1e-5, core_r), 0.0, 1.0), 0.35);
        let sun_c = mix(vec3<f32>(1.0, 0.5, 0.1), sky.sun_color.rgb, limb_dark);
        color += sun_c * disc;
    }
    
    return color;
}

// ── Quality 1: Medium (Enhanced) ──
fn calculate_medium_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let zenith_angle = abs(dir.y);
    // Smooth S-curve: spread gradient across 0° to ~30° for natural feel
    let t = smoothstep(0.0, 0.5, zenith_angle);
    var color = mix(sky.horizon_color.rgb, sky.zenith_color.rgb, t);
    
    let sun_dot = max(dot(dir, sun_dir), 0.0);
    
    // Sunset attenuation (sunset fade)
    let sunset_fade = smoothstep(-0.05, 0.15, sun_dir.y);
    
    // Horizon haze blending with sun color if sun is low, ONLY above the horizon
    let horizon_haze = pow(1.0 - zenith_angle, 16.0) * step(0.0, dir.y);
    let sun_haze = pow(sun_dot, 16.0) * horizon_haze; 
    color = mix(color, sky.sun_color.rgb, sun_haze * 0.08 * sky.atmosphere_density * sunset_fade);
    
    // Soft sun halo ONLY above horizon (scaled down to prevent over-exposure dome)
    let glow_power = 256.0 / max(0.01, sky.sun_glow_strength); 
    let glow = pow(sun_dot, glow_power);
    color += sky.sun_color.rgb * glow * 0.04 * sky.atmosphere_density * step(0.0, dir.y) * sunset_fade;

    // Smooth sun disc with C-infinity exponential falloff
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let sun_core = exp(-pow(angular_dist / max(1e-5, sky.sun_disc_size * 0.011), 4.0));
    color += sky.sun_color.rgb * (sky.sun_color.w * 8.0 * sun_core) * step(0.0, dir.y);
    
    return color;
}

// ── Quality 2: High (Atmospheric Simulation Approx) ──
fn calculate_high_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let zenith_angle = abs(dir.y); // Mirrors cleanly
    let sun_angle = max(0.0, sun_dir.y);
    let sun_dot = max(dot(dir, sun_dir), 0.0);
    
    // Smooth S-curve: spread gradient across 0° to ~30° for natural atmospheric feel
    let t = smoothstep(0.0, 0.5, zenith_angle);
    var base_sky = mix(sky.horizon_color.rgb, sky.zenith_color.rgb, t);
    
    // Sun-influenced sky warmth: subtly warm the sky in the sun's hemisphere
    let sun_influence = pow(sun_dot, 3.0) * 0.03;
    let warm_tint = vec3<f32>(1.0, 0.85, 0.6);
    base_sky = mix(base_sky, base_sky * warm_tint, sun_influence);
    
    let rayleigh = 0.5 * (1.0 + sun_dot * sun_dot);
    let sunset_tint = vec3<f32>(1.0, 0.4, 0.1); 
    let sunset_factor = 1.0 - clamp(sun_angle * 4.0, 0.0, 1.0);
    
    let horizon_blend = smoothstep(-0.15, 0.05, dir.y);
    
    // Softer horizon haze band — reads as atmosphere, not a sharp line
    let horizon_haze = pow(1.0 - zenith_angle, 10.0) * horizon_blend;
    let haze_color = mix(vec3<f32>(0.55, 0.7, 0.85), sunset_tint, sunset_factor);
    
    let density = sky.atmosphere_density * 0.15; // Natural atmospheric horizon haze
    base_sky = mix(base_sky, haze_color, horizon_haze * density * rayleigh);

    // Mie scattering — tighter and dimmer
    let g = 0.999 - (sky.sun_glow_strength * 0.05); 
    let mie_denom = 1.0 + g * g - 2.0 * g * sun_dot;
    let mie = (1.0 - g * g) / (mie_denom * sqrt(mie_denom));
    
    let halo_intensity = mie * 0.0002 * sky.atmosphere_density * horizon_blend;
    base_sky += mix(vec3<f32>(1.0), sunset_tint, sunset_factor) * halo_intensity;

    // HDR Sun Disc & Solar Corona (Natural Photorealistic Sunlight)
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    
    let sun_core = exp(-pow(angular_dist / max(1e-5, sky.sun_disc_size * 0.011), 4.0));
    let sun_corona = exp(-pow(angular_dist / max(1e-5, sky.sun_disc_size * 0.025), 2.0));
    let sun_halo = exp(-angular_dist * (200.0 / max(0.01, sky.sun_disc_size)));
    
    // Natural sunlight colors (prevents blue+red=magenta neon artifact)
    let core_color = mix(vec3<f32>(10.0, 9.8, 9.2), vec3<f32>(12.0, 4.5, 1.0), sunset_factor);
    let corona_color = mix(vec3<f32>(2.2, 2.1, 1.8), vec3<f32>(3.0, 1.5, 0.3), sunset_factor);
    let halo_color = mix(vec3<f32>(0.5, 0.48, 0.42), vec3<f32>(1.0, 0.4, 0.1), sunset_factor);

    let hdr_sun = (core_color * sun_core * sky.sun_color.w) 
                + (corona_color * sun_corona * sky.sun_glow_strength * 0.3)
                + (halo_color * sun_halo * sky.sun_glow_strength * 0.15);
                
    base_sky += hdr_sun * horizon_blend;

    return base_sky;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.view_direction);
    let sun_dir = normalize(sky.sun_direction.xyz);
    
    var color = vec3<f32>(0.0);
    
    // Switch on quality tier
    if sky.sky_quality_mode == 0u {
        color = calculate_low_quality(dir, sun_dir);
    } else if sky.sky_quality_mode == 1u {
        color = calculate_medium_quality(dir, sun_dir);
    } else {
        color = calculate_high_quality(dir, sun_dir);
    }
    
    // Radial sun glow for warmer sunsets/atmospheres with sunset fade to prevent keyhole artifact
    let sun_dist = distance(dir, sun_dir);
    let sunset_fade = smoothstep(-0.05, 0.15, sun_dir.y);
    let glow = exp(-sun_dist * 8.0) * sky.sun_glow_strength * 0.35 * sunset_fade;
    color += vec3<f32>(1.0, 0.8, 0.4) * glow * smoothstep(-0.15, 0.05, dir.y);
    
    return vec4<f32>(color, 1.0);
}
