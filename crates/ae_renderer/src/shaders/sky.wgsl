// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2.5D Atmospheric Sky, Optical Sun Disc & Procedural Cloud Shader.
//!
//! Technical Pipeline:
//! 1. Barometric Rayleigh & Ozone attenuation model for sky gradient.
//! 2. Optical solar disc rendering with polynomial limb darkening (u=0.60) and Henyey-Greenstein Mie aureole.
//! 3. Curved tropospheric dome projection for 2.5D procedural FBM cloud generation.
//! 4. Tiered quality scaling: Low (0), Medium (1), High (2).

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
    position: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct SkyUniform {
    sun_direction: vec4<f32>, // xyz: direction, w: padding
    sun_color: vec4<f32>,     // rgb: sun color, w: HDR sun intensity
    horizon_color: vec4<f32>, // rgb: horizon color, w: padding
    zenith_color: vec4<f32>,  // rgb: zenith color, w: padding
    atmosphere_density: f32,
    ozone_density: f32,
    sun_disc_size: f32,
    sun_glow_strength: f32,
    cloud_coverage: f32,
    cloud_density: f32,
    cloud_speed: f32,
    cloud_evolution: f32,
    cloud_altitude: f32,
    cloud_thickness: f32,
    time: f32,
    sky_quality_mode: u32,   // 0=Low, 1=Medium, 2=High
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

// ══════════════════════════════════════════════════════════════════════════════
// 🌌 1. PROCEDURAL NOISE & MULTI-SCALE SYNTHESIS
// ══════════════════════════════════════════════════════════════════════════════

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Perlin's Improved Quintic Smoothstep (6t^5 - 15t^4 + 10t^3) guarantees C2 continuity
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);

    let a = hash12(i);
    let b = hash12(i + vec2<f32>(1.0, 0.0));
    let c = hash12(i + vec2<f32>(0.0, 1.0));
    let d = hash12(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm_octaves(p: vec2<f32>, octaves: u32) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var pos = p;
    for (var i = 0u; i < octaves; i++) {
        v += a * value_noise(pos);
        pos = pos * 2.06 + vec2<f32>(1.7, 9.2);
        a *= 0.5;
    }
    return v;
}

fn sample_clouds_animated(p: vec2<f32>, evolution_phase: vec2<f32>, is_high: bool) -> f32 {
    if !is_high {
        return fbm_octaves(p + evolution_phase, 3u);
    }
    let warp_x = fbm_octaves(p + evolution_phase, 4u);
    let warp_y = fbm_octaves(p + vec2<f32>(4.3, 1.7) - evolution_phase, 4u);
    let warped_pos = p + vec2<f32>(warp_x, warp_y) * 0.22;
    return fbm_octaves(warped_pos, 5u);
}

// ══════════════════════════════════════════════════════════════════════════════
// ☁️ 2. TROPOSPHERIC DOME PROCEDURAL CLOUDS (Tiered Quality)
// ══════════════════════════════════════════════════════════════════════════════

struct CloudResult {
    color: vec3<f32>,
    alpha: f32,
    density: f32,
};

fn render_procedural_clouds(
    dir: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_radiance: vec3<f32>,
    ambient_sky: vec3<f32>,
    sunset_factor: f32,
    is_high: bool
) -> CloudResult {
    var cr: CloudResult;
    cr.color = vec3<f32>(0.0);
    cr.alpha = 0.0;
    cr.density = 0.0;

    if dir.y <= 0.02 || sky.cloud_coverage <= 0.01 {
        return cr;
    }

    // Curved atmospheric dome projection
    let dome_dist = 1.0 / (dir.y + 0.12 * (1.0 - dir.y));
    let world_pos = dir.xz * dome_dist;

    // Fluid wind drift
    let wind_rate = sky.cloud_speed * 0.08;
    let wind = vec2<f32>(sky.time * wind_rate, sky.time * wind_rate * 0.35);

    // Turbulent boiling evolution
    let evo_rate = sky.cloud_evolution * 0.06;
    let evolution_phase = vec2<f32>(
        sin(sky.time * evo_rate * 0.7),
        cos(sky.time * evo_rate * 0.5)
    ) * 0.25;

    let uv = world_pos * 1.50 + wind;

    // Macro cluster mask
    let cluster_octaves = select(2u, 4u, is_high);
    let cluster = fbm_octaves(uv * 0.40 + evolution_phase * 0.40, cluster_octaves);
    let cluster_mask = smoothstep(0.28, 0.70, cluster);

    let detail = sample_clouds_animated(uv * 1.15, evolution_phase, is_high);
    let raw_cloud = detail * (cluster_mask * 0.65 + 0.35);

    // Smoothstep coverage curve
    let coverage_threshold = 1.0 - clamp(sky.cloud_coverage, 0.0, 1.0);
    let threshold = coverage_threshold * 0.65;
    let d = smoothstep(threshold - 0.08, threshold + 0.28, raw_cloud);
    let density = d * (1.0 + d * 1.8) * sky.cloud_density;

    if density <= 0.0005 {
        return cr;
    }

    // Directional light sampling
    var direct_sun_illum = 0.5;
    var silver_lining = 0.0;

    if is_high {
        let sun_step = sun_dir.xz * 0.12;
        let sun_sample_val = sample_clouds_animated((uv + sun_step) * 1.15, evolution_phase, true) * (cluster_mask * 0.65 + 0.35);
        let sun_d = smoothstep(threshold - 0.08, threshold + 0.28, sun_sample_val);
        let sun_density = sun_d * (1.0 + sun_d * 1.8) * sky.cloud_density;

        let optical_depth_sun = max(0.0, sun_density - density * 0.35);
        let sun_transmission = exp(-optical_depth_sun * 2.0);
        let powder_sugar = 1.0 - exp(-density * 3.0);
        direct_sun_illum = clamp(sun_transmission * powder_sugar, 0.0, 1.0);

        let cos_theta = max(0.0, dot(dir, sun_dir));
        silver_lining = pow(cos_theta, 16.0) * exp(-density * 1.4) * density * 0.85;
    } else {
        // Fast analytical illumination for Medium Quality
        direct_sun_illum = clamp(1.0 - exp(-density * 2.0), 0.0, 1.0);
    }

    // Shaded base + sunlit highlights
    let base_white = vec3<f32>(0.75, 0.78, 0.82);
    let shade_ambient = mix(sky.zenith_color.rgb * 0.85, base_white, 0.68);
    let cloud_base_tone = mix(shade_ambient, base_white, 0.30 + 0.70 * direct_sun_illum);
    let direct_light = (sky.sun_color.rgb * 0.70) * (direct_sun_illum * 0.40 + silver_lining * 0.50);
    let illuminated_color = cloud_base_tone + direct_light;

    let sunset_tint = vec3<f32>(1.0, 0.48, 0.18);
    let final_color = mix(illuminated_color, sunset_tint * illuminated_color, sunset_factor * 0.75);

    // Horizon blend & alpha
    let horizon_fade = smoothstep(0.02, 0.22, dir.y);
    let alpha = (1.0 - exp(-density * 2.4)) * horizon_fade * 0.85;

    cr.color = final_color;
    cr.alpha = alpha;
    cr.density = density * horizon_fade;
    return cr;
}

// ══════════════════════════════════════════════════════════════════════════════
// ☀️ 3. OPTICAL SOLAR DISC & MIE AUREOLE MODEL
// ══════════════════════════════════════════════════════════════════════════════

struct SunAtmosphereContribution {
    direct_sun_disc: vec3<f32>,
    mie_aureole: vec3<f32>,
};

fn calculate_optical_sun(
    dir: vec3<f32>,
    sun_dir: vec3<f32>,
    sunset_factor: f32,
    horizon_blend: f32
) -> SunAtmosphereContribution {
    var sc: SunAtmosphereContribution;
    sc.direct_sun_disc = vec3<f32>(0.0);
    sc.mie_aureole = vec3<f32>(0.0);

    let sun_dot = dot(dir, sun_dir);
    if sun_dot <= -0.1 {
        return sc;
    }

    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let sun_radius = sky.sun_disc_size * 0.015;

    let sunset_tint = vec3<f32>(1.0, 0.38, 0.10);
    let core_tint = mix(vec3<f32>(1.0, 0.98, 0.94), sunset_tint, sunset_factor);

    // Optical Solar Disc with Polynomial Limb Darkening (u = 0.60)
    let r_rel = clamp(angular_dist / max(1e-5, sun_radius), 0.0, 1.0);
    let on_disc = smoothstep(sun_radius * 1.02, sun_radius * 0.98, angular_dist);
    let mu = sqrt(max(0.0, 1.0 - r_rel * r_rel));
    let limb_darkening = 1.0 - 0.60 * (1.0 - mu);

    let disc_hdr_intensity = 38.0 * (sky.sun_color.w * 0.10);
    sc.direct_sun_disc = (core_tint * disc_hdr_intensity * limb_darkening * on_disc) * horizon_blend * step(0.0, dir.y);

    // Dual-Layer Henyey-Greenstein Atmospheric Mie Aureole
    let inner_corona = exp(-angular_dist * (80.0 / max(0.3, sky.sun_disc_size))) * 3.5;
    
    let g = 0.86;
    let denom = 1.0 + g * g - 2.0 * g * max(0.0, sun_dot);
    let hg_wide = (1.0 - g * g) / max(1e-4, denom * sqrt(denom));
    let wide_aureole = hg_wide * 0.0008 * sky.atmosphere_density;

    let total_glare = (inner_corona + wide_aureole) * sky.sun_glow_strength * horizon_blend;
    sc.mie_aureole = mix(vec3<f32>(1.0, 0.96, 0.88), sunset_tint, sunset_factor) * total_glare;

    return sc;
}

// ══════════════════════════════════════════════════════════════════════════════
// 🌌 4. ATMOSPHERIC SKY INTEGRATOR
// ══════════════════════════════════════════════════════════════════════════════

fn calculate_physical_sky(
    dir: vec3<f32>,
    sun_dir: vec3<f32>,
    is_high: bool
) -> vec3<f32> {
    let altitude = max(0.0, dir.y);
    let sun_angle = max(0.0, sun_dir.y);
    let sun_dot = max(0.0, dot(dir, sun_dir));

    // 1. Barometric Rayleigh Optical Depth Gradient
    let horizon_factor = pow(1.0 - altitude, 4.8);
    var base_sky = mix(sky.zenith_color.rgb, sky.horizon_color.rgb, horizon_factor);

    // 2. Sunset Factor & Ozone Chappuis Band Absorption
    let sunset_factor = 1.0 - clamp(sun_angle * 3.5, 0.0, 1.0);
    let sunset_tint = vec3<f32>(1.0, 0.38, 0.10);
    let ozone_twilight_tint = vec3<f32>(0.20, 0.28, 0.75) * sky.ozone_density;

    let twilight_boost = pow(sunset_factor, 2.0) * (1.0 - horizon_factor) * 0.45;
    base_sky = mix(base_sky, base_sky * ozone_twilight_tint + base_sky, twilight_boost);

    // 3. Directional Rayleigh In-Scattering
    let rayleigh_phase = 0.75 * (1.0 + sun_dot * sun_dot);
    let sun_warmth = pow(sun_dot, 5.0) * 0.12 * horizon_factor;
    let warm_tone = mix(vec3<f32>(1.0, 0.94, 0.82), sunset_tint, sunset_factor);
    base_sky = mix(base_sky, base_sky * warm_tone, sun_warmth * sky.atmosphere_density);

    // 4. Horizon Atmospheric Haze Layer
    let horizon_blend = smoothstep(-0.08, 0.04, dir.y);
    let haze_intensity = pow(1.0 - altitude, 8.0) * 0.28 * sky.atmosphere_density * horizon_blend;
    let haze_color = mix(sky.horizon_color.rgb, sunset_tint, sunset_factor);
    base_sky = mix(base_sky, haze_color, haze_intensity * rayleigh_phase);

    // 5. Below Horizon Smooth Ground Tone
    let ground_factor = smoothstep(0.0, -0.30, dir.y);
    let ground_tone = sky.horizon_color.rgb * 0.40;
    base_sky = mix(base_sky, ground_tone, ground_factor);

    // 6. Calculate Optical Sun & Atmospheric Aureole
    let sun_res = calculate_optical_sun(dir, sun_dir, sunset_factor, horizon_blend);
    base_sky += sun_res.mie_aureole;

    // 7. Dynamic Procedural Clouds Layer
    let clouds = render_procedural_clouds(
        dir,
        sun_dir,
        sky.sun_color.rgb,
        mix(sky.zenith_color.rgb, sky.horizon_color.rgb, 0.5),
        sunset_factor,
        is_high
    );

    // Solar Disc Extinction Through Clouds & Forward Scatter
    let direct_disc_visibility = 1.0 - smoothstep(0.06, 0.65, clouds.alpha);
    let visible_sun_disc = sun_res.direct_sun_disc * direct_disc_visibility;

    // Focused forward scatter strictly around the solar disc
    let forward_cloud_scatter = (sky.sun_color.rgb * 0.50) * pow(sun_dot, 96.0) * clouds.alpha * exp(-clouds.density * 0.70);

    let final_sky = mix(base_sky + visible_sun_disc, clouds.color + forward_cloud_scatter, clouds.alpha);

    return final_sky;
}

// ══════════════════════════════════════════════════════════════════════════════
// 🎮 5. QUALITY TIERS & MAIN ENTRY
// ══════════════════════════════════════════════════════════════════════════════

fn calculate_low_quality(dir: vec3<f32>, sun_dir: vec3<f32>) -> vec3<f32> {
    let altitude = max(0.0, dir.y);
    let horizon_factor = pow(1.0 - altitude, 4.0);
    var color = mix(sky.zenith_color.rgb, sky.horizon_color.rgb, horizon_factor);

    let sun_dot = max(0.0, dot(dir, sun_dir));
    let angular_dist = acos(clamp(sun_dot, -1.0, 1.0));
    let sun_radius = sky.sun_disc_size * 0.015;

    let disc = exp(-pow(angular_dist / max(1e-5, sun_radius), 6.0)) * step(0.0, dir.y);
    let corona = exp(-pow(angular_dist / max(1e-5, sun_radius * 2.5), 2.0)) * 0.3 * step(0.0, dir.y);
    color += sky.sun_color.rgb * (disc * 5.0 + corona);

    return color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.view_direction);
    let sun_dir = normalize(sky.sun_direction.xyz);

    var color = vec3<f32>(0.0);

    if sky.sky_quality_mode == 0u {
        color = calculate_low_quality(dir, sun_dir);
    } else if sky.sky_quality_mode == 1u {
        color = calculate_physical_sky(dir, sun_dir, false);
    } else {
        color = calculate_physical_sky(dir, sun_dir, true);
    }

    return vec4<f32>(color, 1.0);
}
