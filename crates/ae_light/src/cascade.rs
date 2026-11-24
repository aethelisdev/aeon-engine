// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cascaded Shadow Map (CSM) frustum split projection and light-space matrix math.
//!
//! Pure CPU mathematical kernel separated from GPU shadow depth textures and WGPU passes.

use crate::settings::{PcfQuality, ShadowResolution};
use crate::uniforms::LightSpaceUniform;
use ae_camera::{Camera, OPENGL_TO_WGPU_MATRIX};
use cgmath::{InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, ortho};

/// CPU-side Cascaded Shadow Map configuration decoupled from WGPU pipeline handles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CascadeShadowConfig {
    /// Toggles directional cascaded shadows (`shadow_enabled` flag).
    pub enabled: bool,
    /// Depth array resolution tier.
    pub resolution: ShadowResolution,
    /// PCF filtering kernel tier.
    pub pcf: PcfQuality,
    /// Depth bias suppressing surface shadow acne.
    pub bias: f32,
    /// Active cascade count (`1..=4`).
    pub cascade_count: u32,
    /// View-Z split boundaries for up to 4 cascades.
    pub cascade_splits: [f32; 4],
}

impl Default for CascadeShadowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resolution: ShadowResolution::High,
            pcf: PcfQuality::Soft,
            bias: 0.005,
            cascade_count: 4,
            cascade_splits: [4.0, 15.0, 50.0, 150.0],
        }
    }
}

/// Computes WGPU-corrected orthographic light-space matrices and split depths for 1..=4 cascades.
///
/// Sanitizes degenerate or non-finite light vectors (falling back to `+Y`), guards near-vertical
/// view alignment against `look_at_rh` collinearity (`unit_z` up vector), clamps `cascade_count`
/// into `1..=4`, and fills inactive tail slots with identity matrices and `10000.0` split depth.
#[must_use]
pub fn compute_cascade_light_space(
    config: &CascadeShadowConfig,
    camera: &Camera,
    light_dir: Vector3<f32>,
) -> LightSpaceUniform {
    let dir_is_finite =
        light_dir.x.is_finite() && light_dir.y.is_finite() && light_dir.z.is_finite();
    let safe_light_dir = if !dir_is_finite || light_dir.magnitude2() < 1e-6 {
        Vector3::unit_y()
    } else {
        light_dir.normalize()
    };

    let cascades = config.cascade_count.clamp(1, 4) as usize;
    let splits = config.cascade_splits;
    let cam_pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z);
    let cam_forward = camera.get_forward();

    let mut matrices = [Matrix4::<f32>::identity().into(); 4];
    let mut cascade_splits = [10_000.0_f32; 4];

    for i in 0..cascades {
        let split_end = if splits[i].is_finite() && splits[i] > 0.0 {
            splits[i]
        } else {
            150.0
        };
        let split_prev = if i == 0 {
            0.0
        } else if splits[i - 1].is_finite() && splits[i - 1] >= 0.0 {
            splits[i - 1]
        } else {
            0.0
        };

        let center_dist = (split_end + split_prev) * 0.5;
        let cascade_center = cam_pos + cam_forward * center_dist;

        // Apply a 1.5x margin around half the cascade depth range to avoid edge clipping on rotation.
        let half_depth = if i == 0 {
            split_end
        } else {
            (split_end - split_prev).abs() * 0.5
        };
        let radius = (half_depth * 1.5).max(0.5);

        let light_pos_target = cascade_center + safe_light_dir * radius * 2.0;
        let up_vector = if safe_light_dir.y.abs() > 0.99 {
            Vector3::unit_z()
        } else {
            Vector3::unit_y()
        };

        let light_view = Matrix4::look_at_rh(
            Point3::new(light_pos_target.x, light_pos_target.y, light_pos_target.z),
            Point3::new(cascade_center.x, cascade_center.y, cascade_center.z),
            up_vector,
        );
        let light_proj = ortho(-radius, radius, -radius, radius, 0.1, radius * 5.0);

        matrices[i] = (OPENGL_TO_WGPU_MATRIX * light_proj * light_view).into();
        cascade_splits[i] = split_end;
    }

    LightSpaceUniform {
        matrices,
        cascade_splits,
        shadow_bias: if config.bias.is_finite() && config.bias >= 0.0 {
            config.bias
        } else {
            0.005
        },
        pcf_radius: config.pcf.radius(),
        shadow_enabled: u32::from(config.enabled),
        cascade_count: cascades as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_camera::ProjectionMode;

    fn test_camera() -> Camera {
        Camera {
            position: Point3::new(0.0, 2.0, 5.0),
            yaw: cgmath::Rad(-std::f32::consts::FRAC_PI_2),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 1.0,
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    fn assert_all_matrices_finite(uniform: &LightSpaceUniform) {
        for mat in &uniform.matrices {
            for row in mat {
                for val in row {
                    assert!(val.is_finite(), "Cascade matrix contains non-finite value");
                }
            }
        }
    }

    #[test]
    fn test_vertical_and_degenerate_light_vectors_produce_finite_matrices() {
        let camera = test_camera();
        let cfg = CascadeShadowConfig::default();

        for light_dir in [
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(f32::NAN, 0.0, 0.0),
            Vector3::new(0.0, f32::INFINITY, 0.0),
        ] {
            let uniform = compute_cascade_light_space(&cfg, &camera, light_dir);
            assert_all_matrices_finite(&uniform);
            assert_eq!(uniform.cascade_count, 4);
            assert_eq!(uniform.shadow_enabled, 1);
        }
    }

    #[test]
    fn test_cascade_count_clamping_and_inactive_tail_slots() {
        let camera = test_camera();
        let cfg_two = CascadeShadowConfig {
            cascade_count: 2,
            cascade_splits: [5.0, 20.0, 60.0, 180.0],
            pcf: PcfQuality::UltraSoft,
            ..Default::default()
        };
        let uniform = compute_cascade_light_space(&cfg_two, &camera, Vector3::new(0.3, 0.9, 0.2));

        assert_eq!(uniform.cascade_count, 2);
        assert_eq!(uniform.pcf_radius, 2);
        assert_eq!(uniform.cascade_splits, [5.0, 20.0, 10_000.0, 10_000.0]);
        let identity: [[f32; 4]; 4] = Matrix4::<f32>::identity().into();
        assert_eq!(uniform.matrices[2], identity);
        assert_eq!(uniform.matrices[3], identity);

        // Extreme counts clamp into 1..=4 safely without panic.
        let cfg_zero = CascadeShadowConfig {
            cascade_count: 0,
            enabled: false,
            bias: f32::NAN,
            ..Default::default()
        };
        let clamped_low = compute_cascade_light_space(&cfg_zero, &camera, Vector3::unit_y());
        assert_eq!(clamped_low.cascade_count, 1);
        assert_eq!(clamped_low.shadow_enabled, 0);
        assert_eq!(clamped_low.shadow_bias, 0.005);

        let cfg_huge = CascadeShadowConfig {
            cascade_count: 99,
            ..Default::default()
        };
        let clamped_high = compute_cascade_light_space(&cfg_huge, &camera, Vector3::unit_y());
        assert_eq!(clamped_high.cascade_count, 4);
    }
}