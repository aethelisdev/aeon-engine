// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Procedural atmosphere, solar direction, sunset color interpolation, ambient fill, and fog evaluation.
//!
//! Serves as the single authoritative CPU model producing synchronized [`SkyUniform`] and
//! [`LightUniform`] state from environment configuration without touching GPU handles.

use crate::settings::SkyQuality;
use crate::uniforms::{LightUniform, SkyUniform};

/// High-level environment, solar, procedural cloud, and fog configuration.
///
/// Consumed by [`evaluate_environment`] to produce synchronized sky and directional lighting uniforms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentLightConfig {
    /// Base atmospheric sRGB sky tint (`[r, g, b]`).
    pub environment_color: [f32; 3],
    /// Active sky shader complexity tier.
    pub sky_quality: SkyQuality,
    /// Solar elevation angle in radians relative to the horizon.
    pub sun_pitch: f32,
    /// Solar compass azimuth in radians.
    pub sun_yaw: f32,
    /// Atmospheric Rayleigh/Mie scattering density multiplier.
    pub atmosphere_density: f32,
    /// Physical Chappuis ozone absorption multiplier.
    pub ozone_density: f32,
    /// Angular sun disc size multiplier.
    pub sun_disc_size: f32,
    /// Solar glow halo strength multiplier.
    pub sun_glow_strength: f32,
    /// Procedural cloud coverage (`0.0..=1.0`).
    pub cloud_coverage: f32,
    /// Procedural cloud optical density multiplier.
    pub cloud_density: f32,
    /// Procedural cloud drift speed multiplier.
    pub cloud_speed: f32,
    /// Procedural cloud noise evolution rate multiplier.
    pub cloud_evolution: f32,
    /// Cloud deck base altitude in meters.
    pub cloud_altitude: f32,
    /// Cloud deck thickness in meters.
    pub cloud_thickness: f32,
    /// Enables atmospheric distance fog blended into horizon sky color.
    pub fog_enabled: bool,
    /// Distance at which fog reaches full occlusion when enabled.
    pub fog_distance: f32,
}

impl Default for EnvironmentLightConfig {
    fn default() -> Self {
        Self {
            environment_color: [0.12, 0.35, 0.65],
            sky_quality: SkyQuality::High,
            sun_pitch: 0.5,
            sun_yaw: 0.5,
            atmosphere_density: 1.0,
            ozone_density: 1.0,
            sun_disc_size: 1.0,
            sun_glow_strength: 1.0,
            cloud_coverage: 0.45,
            cloud_density: 1.0,
            cloud_speed: 1.0,
            cloud_evolution: 1.0,
            cloud_altitude: 1500.0,
            cloud_thickness: 3000.0,
            fog_enabled: true,
            fog_distance: 800.0,
        }
    }
}

/// Synchronized sky and directional lighting uniforms derived from [`EnvironmentLightConfig`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentEvaluation {
    /// Procedural sky shader uniform buffer data.
    pub sky: SkyUniform,
    /// Scene PBR directional light, ambient fill, and fog uniform data.
    pub light: LightUniform,
}

/// Computes a finite, normalized vector pointing towards the sun from pitch/yaw angles (radians).
///
/// Non-finite inputs (`NaN` or infinity) fall back cleanly to zenith `[0.0, 1.0, 0.0]`.
#[must_use]
pub fn sun_direction_from_angles(pitch: f32, yaw: f32) -> [f32; 3] {
    if !pitch.is_finite() || !yaw.is_finite() {
        return [0.0, 1.0, 0.0];
    }
    let cos_p = pitch.cos();
    let raw = [cos_p * yaw.sin(), pitch.sin(), cos_p * yaw.cos()];
    let len_sq = raw[0] * raw[0] + raw[1] * raw[1] + raw[2] * raw[2];
    if !len_sq.is_finite() || len_sq < 1e-12 {
        return [0.0, 1.0, 0.0];
    }
    let inv_len = len_sq.sqrt().recip();
    [raw[0] * inv_len, raw[1] * inv_len, raw[2] * inv_len]
}

/// Converts a perceptual sRGB environment color `[r, g, b]` into linear `[f64; 3]` clear channels.
///
/// Clamps negative and non-finite channels to `0.0` before applying the gamma `2.2` curve.
#[must_use]
pub fn linear_clear_rgb(environment_color: [f32; 3]) -> [f64; 3] {
    let to_linear = |c: f32| -> f64 {
        if c.is_finite() && c > 0.0 {
            (c as f64).powf(2.2)
        } else {
            0.0
        }
    };
    [
        to_linear(environment_color[0]),
        to_linear(environment_color[1]),
        to_linear(environment_color[2]),
    ]
}

fn sanitize_non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

/// Evaluates synchronized [`SkyUniform`] and [`LightUniform`] state from [`EnvironmentLightConfig`].
///
/// Derives solar direction, Rayleigh/Mie horizon and zenith tints, sunset color transitions,
/// elevation-scaled ambient fill light, and horizon-matched atmospheric fog.
#[must_use]
pub fn evaluate_environment(
    config: &EnvironmentLightConfig,
    elapsed_secs: f32,
    grid_scale: f32,
) -> EnvironmentEvaluation {
    let sun_dir = sun_direction_from_angles(config.sun_pitch, config.sun_yaw);

    let env_r = sanitize_non_negative(config.environment_color[0], 0.0);
    let env_g = sanitize_non_negative(config.environment_color[1], 0.0);
    let env_b = sanitize_non_negative(config.environment_color[2], 0.0);

    let horizon_color = [env_r * 0.55, env_g * 0.72, env_b * 0.90, 1.0];
    let zenith_color = [env_r * 0.08, env_g * 0.22, env_b * 0.58, 1.0];

    let sky = SkyUniform {
        sun_direction: [sun_dir[0], sun_dir[1], sun_dir[2], 0.0],
        sun_color: [1.0, 0.95, 0.8, 100.0],
        horizon_color,
        zenith_color,
        atmosphere_density: sanitize_non_negative(config.atmosphere_density, 1.0),
        ozone_density: sanitize_non_negative(config.ozone_density, 1.0),
        sun_disc_size: sanitize_non_negative(config.sun_disc_size, 1.0),
        sun_glow_strength: sanitize_non_negative(config.sun_glow_strength, 1.0),
        cloud_coverage: config.cloud_coverage.clamp(0.0, 1.0),
        cloud_density: sanitize_non_negative(config.cloud_density, 1.0),
        cloud_speed: if config.cloud_speed.is_finite() {
            config.cloud_speed
        } else {
            1.0
        },
        cloud_evolution: sanitize_non_negative(config.cloud_evolution, 1.0),
        cloud_altitude: sanitize_non_negative(config.cloud_altitude, 1500.0),
        cloud_thickness: sanitize_non_negative(config.cloud_thickness, 3000.0),
        time: sanitize_non_negative(elapsed_secs, 0.0),
        sky_quality_mode: config.sky_quality as u32,
    };

    // Derive natural sunset-to-noon tint and ambient fill from solar elevation.
    let elevation = sun_dir[1].max(0.0);
    let noon_color = [1.0_f32, 0.95, 0.9];
    let sunset_color = [1.0_f32, 0.4, 0.1];
    let blend = elevation.powf(0.5);

    let light_color = [
        (noon_color[0] * blend + sunset_color[0] * (1.0 - blend)) * 1.05,
        (noon_color[1] * blend + sunset_color[1] * (1.0 - blend)) * 1.05,
        (noon_color[2] * blend + sunset_color[2] * (1.0 - blend)) * 1.05,
    ];

    let ambient_intensity = 0.2 + elevation * 0.4;
    let ambient_color = [
        env_r * ambient_intensity,
        env_g * ambient_intensity,
        (env_b * 1.2) * ambient_intensity,
    ];

    let fog_dist = if config.fog_enabled {
        sanitize_non_negative(config.fog_distance, 0.0)
    } else {
        0.0
    };

    let light = LightUniform {
        direction: sun_dir,
        grid_scale: if grid_scale.is_finite() && grid_scale > 0.001 {
            grid_scale
        } else {
            1.0
        },
        color: light_color,
        _padding2: 0,
        ambient_color,
        _padding3: 0,
        fog_params: [
            horizon_color[0],
            horizon_color[1],
            horizon_color[2],
            fog_dist,
        ],
    };

    EnvironmentEvaluation { sky, light }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_sunset_vs_zenith_lighting_and_fog_invariants() {
        let horizon_cfg = EnvironmentLightConfig {
            sun_pitch: 0.0,
            sun_yaw: 0.0,
            fog_enabled: true,
            fog_distance: 450.0,
            ..Default::default()
        };
        let horizon_eval = evaluate_environment(&horizon_cfg, 12.5, 2.0);
        // At pitch == 0 (sun on horizon), sunset blend yields warm orange tint ([1.05, 0.42, 0.105]).
        assert!((horizon_eval.light.color[0] - 1.05).abs() < 1e-5);
        assert!((horizon_eval.light.color[1] - 0.42).abs() < 1e-5);
        assert!((horizon_eval.light.color[2] - 0.105).abs() < 1e-5);
        assert_eq!(horizon_eval.light.fog_params[3], 450.0);
        assert_eq!(horizon_eval.light.grid_scale, 2.0);
        assert_eq!(horizon_eval.sky.time, 12.5);

        let noon_cfg = EnvironmentLightConfig {
            sun_pitch: FRAC_PI_2,
            sun_yaw: 0.0,
            fog_enabled: false,
            fog_distance: 450.0,
            ..Default::default()
        };
        let noon_eval = evaluate_environment(&noon_cfg, 0.0, 1.0);
        // At zenith, sun points straight up and noon tint dominates; disabled fog forces distance 0.0.
        assert!((noon_eval.light.direction[1] - 1.0).abs() < 1e-5);
        assert!(noon_eval.light.color[2] > horizon_eval.light.color[2]);
        assert!(noon_eval.light.ambient_color[0] > horizon_eval.light.ambient_color[0]);
        assert_eq!(noon_eval.light.fog_params[3], 0.0);
    }

    #[test]
    fn test_non_finite_and_negative_environment_inputs_clamp_safely() {
        assert_eq!(sun_direction_from_angles(f32::NAN, 0.5), [0.0, 1.0, 0.0]);
        assert_eq!(
            sun_direction_from_angles(0.5, f32::INFINITY),
            [0.0, 1.0, 0.0]
        );
        assert_eq!(linear_clear_rgb([-0.5, f32::NAN, 1.0]), [0.0, 0.0, 1.0]);

        let invalid_cfg = EnvironmentLightConfig {
            environment_color: [-1.0, f32::NAN, 0.5],
            sun_pitch: f32::NAN,
            sun_yaw: 0.0,
            fog_enabled: true,
            fog_distance: -250.0,
            ..Default::default()
        };
        let eval = evaluate_environment(&invalid_cfg, -5.0, -3.0);
        assert_eq!(eval.light.direction, [0.0, 1.0, 0.0]);
        assert_eq!(eval.light.grid_scale, 1.0);
        assert_eq!(eval.light.fog_params[3], 0.0);
        assert_eq!(eval.sky.time, 0.0);
        assert!(eval.light.ambient_color.iter().all(|c| c.is_finite()));
    }
}