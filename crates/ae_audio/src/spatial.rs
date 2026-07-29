// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec3;

/// 3D Spatial Audio mathematics for distance attenuation and stereo channel panning.
/// Implements logarithmic falloff attenuation and vector dot-product stereo panning
/// based on 3D spatial orientation of the listener and audio source.
pub struct SpatialAudioMath;

impl SpatialAudioMath {
    /// Computes inverse-distance logarithmic attenuation factor `[0.0, 1.0]`.
    /// # Parameters
    /// - `emitter_pos`: 3D position of the sound source entity.
    /// - `listener_pos`: 3D position of the listener (camera/ear).
    /// - `min_dist`: Distance below which volume is 100% (unattenuated).
    /// - `max_dist`: Distance at or beyond which volume drops to 0% (silent).
    pub fn compute_distance_attenuation(
        emitter_pos: Vec3,
        listener_pos: Vec3,
        min_dist: f32,
        max_dist: f32,
    ) -> f32 {
        let dist = (emitter_pos - listener_pos).length();
        if dist <= min_dist {
            return 1.0;
        }
        if dist >= max_dist {
            return 0.0;
        }
        let safe_min = min_dist.max(0.1);
        let safe_max = max_dist.max(safe_min + 0.1);

        // OpenAL/ standard inverse-distance roll-off with smooth max_distance cutoff fade
        let inv_dist_falloff = safe_min / dist;
        let cutoff_factor = (safe_max - dist) / (safe_max - safe_min);
        (inv_dist_falloff * cutoff_factor).clamp(0.0, 1.0)
    }

    /// Computes left and right channel stereo volume gain factors `(left_gain, right_gain)`
    /// based on 3D spatial angle relative to the listener.
    /// # Parameters
    /// - `emitter_pos`: 3D position of the sound source entity.
    /// - `listener_pos`: 3D position of the listener.
    /// - `listener_right`: Normalized right-vector of the listener orientation (`glam::Vec3`).
    pub fn compute_stereo_panning(
        emitter_pos: Vec3,
        listener_pos: Vec3,
        listener_right: Vec3,
    ) -> (f32, f32) {
        let to_emitter = (emitter_pos - listener_pos).normalize_or_zero();
        if to_emitter == Vec3::ZERO {
            return (1.0, 1.0); // Center if overlapping
        }

        // Dot product with right vector: +1.0 = full right, -1.0 = full left
        let pan = to_emitter.dot(listener_right).clamp(-1.0, 1.0);

        // Constant-power stereo panning curve
        let left_gain = (0.5 * (1.0 - pan)).sqrt();
        let right_gain = (0.5 * (1.0 + pan)).sqrt();

        (left_gain, right_gain)
    }
}