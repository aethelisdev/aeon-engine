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
    time: f32,
    cloud_coverage: f32,
    cloud_speed: f32,
    _pad: f32,
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
    let x = f32((in_vertex_index & 1u) << 2u);
    let y = f32((in_vertex_index & 2u) << 1u);
    let pos_ndc = vec4<f32>(x - 1.0, 1.0 - y, 1.0, 1.0);

    out.clip_position = pos_ndc;

    var view_rot_inv = camera.view_inv;
    view_rot_inv[3][0] = 0.0;
    view_rot_inv[3][1] = 0.0;
    view_rot_inv[3][2] = 0.0;
    
    let unprojected = camera.proj_inv * pos_ndc;
    out.view_direction = (view_rot_inv * vec4<f32>(unprojected.xyz, 0.0)).xyz;

    return out;
}

// ── Procedural Value Noise and Cloud Synthesis ──
fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash12(i);
    let b = hash12(i + vec2<f32>(1.0, 0.0));
    let c = hash12(i + vec2<f32>(0.0, 1.0));
    let d = hash12(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm_octaves(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var pos = p;
    for (var i = 0u; i < 5u; i++) {
        v += a * value_noise(pos);
        pos = pos * 2.08 + vec2<f32>(1.7, 9.2);
        a *= 0.5;
    }
    return v;
}

fn sample_clouds(p: vec2<f32>) -> f32 {
    let warp_x = fbm_octaves(p + vec2<f32>(0.0, 0.0));
    let warp_y = fbm_octaves(p + vec2<f32>(4.3, 1.7));
    let warped_pos = p + vec2<f32>(warp_x, warp_y) * 0.40;
    return fbm_octaves(warped_pos);
}

fn render_clouds(dir: vec3<f32>, sun_dir: vec3<f32>, sunset_factor: f32) -> vec4<f32> {
    if dir.y <= 0.02 {
        return vec4<f32>(0.0);
    }
    
    // High-altitude atmospheric dome projection
    let dome_dist = 1.0 / (dir.y + 0.12 * (1.0 - dir.y));
    let world_pos = dir.xz * dome_dist;
    
    // Continuous wind animation
    let wind = vec2<f32>(sky.time * sky.cloud_speed * 0.04, sky.time * sky.cloud_speed * 0.015);
    let uv = world_pos * 1.75 + wind;
    
    // Multi-frequency natural cumulus structure
    let cluster_mask = smoothstep(0.35, 0.70, fbm_octaves(uv * 0.55));
    let billowy_detail = sample_clouds(uv * 1.35);
    let raw_cloud = billowy_detail * (cluster_mask * 0.65 + 0.35);
    
    // Continuous smooth density curve without hard ceiling plateau
    let d = max(0.0, raw_cloud - 0.40);
    let density = d * (1.0 + d * 2.0);
    
    if density <= 0.001 {
        return vec4<f32>(0.0);
    }
    
    // Directional shadow calculation
    let sun_offset = sun_dir.xz * 0.08;
    let sun_sample = sample_clouds((uv + sun_offset) * 1.35) * (cluster_mask * 0.65 + 0.35);
    let sun_d = max(0.0, sun_sample - 0.40);
    let sun_density = sun_d * (1.0 + sun_d * 2.0);
    let shadow = clamp(1.0 - (sun_density - density) * 1.8, 0.55, 1.0);
    
    // Internal volumetric light absorption (darkens interior to create 3D volume instead of flat white plate)
    let internal_absorption = exp(-density * 1.2);
    let ambient_shade = mix(vec3<f32>(0.62, 0.72, 0.85), vec3<f32>(1.08, 1.05, 1.02), shadow * internal_absorption);
    
    // Soft perimeter light scatter
    let sun_dot = max(0.0, dot(dir, sun_dir));
    let silver_lining = pow(sun_dot, 16.0) * exp(-density * 2.0) * density * 0.4;
    
    let sunset_tint = vec3<f32>(1.0, 0.56, 0.26);
    let cloud_color = mix(ambient_shade, sunset_tint, sunset_factor * 0.85) + sky.sun_color.rgb * silver_lining;
    
    // Atmospheric horizon fade and smooth transmission alpha
    let horizon_fade = smoothstep(0.02, 0.22, dir.y);
    let alpha = (1.0 - exp(-density * 2.2)) * horizon_fade * 0.78;
    
    return vec4<f32>(cloud_color, alpha);
}

// ── Quality 0: Low Tier ──
fn calculate_low_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let altitude = max(0.0, dir.y);
    let horizon_factor = pow(1.0 - altitude, 4.0);
    var color = mix(sky.zenith_color.rgb, sky.horizon_color.rgb, horizon_factor);
    
    let sun_dot = max(0.0, dot(dir, sun_dir));
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let sun_radius = sky.sun_disc_size * 0.009;
    
    let disc = exp(-pow(angular_dist / max(1e-5, sun_radius), 6.0)) * step(0.0, dir.y);
    let corona = exp(-pow(angular_dist / max(1e-5, sun_radius * 2.5), 2.0)) * 0.3 * step(0.0, dir.y);
    color += sky.sun_color.rgb * (disc * 5.0 + corona);
    
    return color;
}

// ── Quality 1: Medium Tier ──
fn calculate_medium_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let altitude = max(0.0, dir.y);
    let sun_angle = max(0.0, sun_dir.y);
    let horizon_factor = pow(1.0 - altitude, 5.0);
    var color = mix(sky.zenith_color.rgb, sky.horizon_color.rgb, horizon_factor);
    
    let sun_dot = max(0.0, dot(dir, sun_dir));
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let sun_radius = sky.sun_disc_size * 0.009;
    
    let sunset_factor = 1.0 - clamp(sun_angle * 3.5, 0.0, 1.0);
    
    // Sun disc and corona
    let sun_core = exp(-pow(angular_dist / max(1e-5, sun_radius), 6.0));
    let limb_darkening = pow(clamp(1.0 - (angular_dist / max(1e-5, sun_radius)), 0.0, 1.0), 0.5);
    let sun_corona = exp(-pow(angular_dist / max(1e-5, sun_radius * 2.6), 2.0)) * 0.4;
    
    let g = 0.80;
    let mie = (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * sun_dot, 1.5);
    let mie_glow = mie * 0.0004 * sky.sun_glow_strength * sky.atmosphere_density;
    
    color += sky.sun_color.rgb * (sun_core * (5.0 + 3.0 * limb_darkening) + sun_corona + mie_glow) * step(0.0, dir.y);
    
    // Clouds
    let clouds = render_clouds(dir, sun_dir, sunset_factor);
    color = mix(color, clouds.rgb, clouds.a);
    
    return color;
}

// ── Quality 2: High Tier (Atmospheric Scattering Simulation) ──
fn calculate_high_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let altitude = max(0.0, dir.y);
    let sun_angle = max(0.0, sun_dir.y);
    let sun_dot = max(0.0, dot(dir, sun_dir));
    
    // 1. Barometric Optical Depth Gradient
    let horizon_factor = pow(1.0 - altitude, 5.0);
    var base_sky = mix(sky.zenith_color.rgb, sky.horizon_color.rgb, horizon_factor);
    
    // 2. Sunset Factor
    let sunset_factor = 1.0 - clamp(sun_angle * 3.5, 0.0, 1.0);
    let sunset_tint = vec3<f32>(1.0, 0.40, 0.10);
    
    // 3. Directional Rayleigh In-Scattering
    let rayleigh_phase = 0.75 * (1.0 + sun_dot * sun_dot);
    let sun_warmth = pow(sun_dot, 6.0) * 0.10 * horizon_factor;
    let warm_tone = mix(vec3<f32>(1.0, 0.92, 0.80), sunset_tint, sunset_factor);
    base_sky = mix(base_sky, base_sky * warm_tone, sun_warmth * sky.atmosphere_density);
    
    // 4. Horizon Atmospheric Haze Layer
    let horizon_blend = smoothstep(-0.10, 0.04, dir.y);
    let haze_intensity = pow(1.0 - altitude, 10.0) * 0.25 * sky.atmosphere_density * horizon_blend;
    let haze_color = mix(sky.horizon_color.rgb, sunset_tint, sunset_factor);
    base_sky = mix(base_sky, haze_color, haze_intensity * rayleigh_phase);

    // 5. Below Horizon Smooth Ground Tone
    let ground_factor = smoothstep(0.0, -0.30, dir.y);
    let ground_tone = sky.horizon_color.rgb * 0.45;
    base_sky = mix(base_sky, ground_tone, ground_factor);

    // 6. Henyey-Greenstein Mie Forward Scattering
    let g = 0.82;
    let mie_denom = 1.0 + g * g - 2.0 * g * sun_dot;
    let mie = (1.0 - g * g) / (mie_denom * sqrt(max(1e-5, mie_denom)));
    let mie_glow = mie * 0.0005 * sky.sun_glow_strength * sky.atmosphere_density * horizon_blend;
    base_sky += mix(vec3<f32>(1.0, 0.98, 0.90), sunset_tint, sunset_factor) * mie_glow;

    // 7. Perfect Spherical Solar Disc with Limb Darkening and Corona
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let sun_radius = sky.sun_disc_size * 0.009;
    
    let sun_core = exp(-pow(angular_dist / max(1e-5, sun_radius), 6.0));
    let limb_darkening = pow(clamp(1.0 - (angular_dist / max(1e-5, sun_radius)), 0.0, 1.0), 0.5);
    let sun_corona = exp(-pow(angular_dist / max(1e-5, sun_radius * 2.8), 2.0)) * 0.45;
    let sun_outer_halo = exp(-angular_dist * 40.0) * 0.12;

    let core_color = mix(vec3<f32>(10.0, 9.8, 9.2), vec3<f32>(12.0, 4.0, 0.8), sunset_factor);
    let corona_color = mix(vec3<f32>(2.0, 1.9, 1.6), vec3<f32>(3.0, 1.4, 0.2), sunset_factor);
    let halo_color = mix(vec3<f32>(0.6, 0.55, 0.45), vec3<f32>(1.0, 0.35, 0.05), sunset_factor);

    let hdr_sun = (core_color * sun_core * (1.0 + 0.5 * limb_darkening) * (sky.sun_color.w * 0.1))
                + (corona_color * sun_corona * sky.sun_glow_strength)
                + (halo_color * sun_outer_halo * sky.sun_glow_strength * 0.5);

    base_sky += hdr_sun * horizon_blend;

    // 8. Natural Procedural Clouds Layer (Blended seamlessly over the sun and sky)
    let clouds = render_clouds(dir, sun_dir, sunset_factor);
    base_sky = mix(base_sky, clouds.rgb, clouds.a);

    return base_sky;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.view_direction);
    let sun_dir = normalize(sky.sun_direction.xyz);
    
    var color = vec3<f32>(0.0);
    
    if sky.sky_quality_mode == 0u {
        color = calculate_low_quality(dir, sun_dir);
    } else if sky.sky_quality_mode == 1u {
        color = calculate_medium_quality(dir, sun_dir);
    } else {
        color = calculate_high_quality(dir, sun_dir);
    }
    
    return vec4<f32>(color, 1.0);
}
