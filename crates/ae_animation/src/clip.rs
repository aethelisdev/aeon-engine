// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Animation clip, keyframe tracks, and interpolation logic for Aeon Engine.
//!

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Interpolation mode between animation keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Interpolation {
    /// Linear interpolation between keyframes.
    #[default]
    Linear,
    /// Instantaneous step transition at keyframe boundary.
    Step,
    /// Cubic spline smooth interpolation.
    CubicSpline,
}

/// A keyframe containing a time stamp and value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    /// Time in seconds from the start of the clip.
    pub time: f32,
    /// Value at this keyframe timestamp.
    pub value: T,
}

/// A keyframe track for 3D vector properties (Position or Scale).
/// Uses `Vec3::lerp` for smooth linear interpolation between keyframes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorTrack {
    /// Keyframe list ordered by timestamp.
    pub keyframes: Vec<Keyframe<Vec3>>,
    /// Interpolation type.
    pub interpolation: Interpolation,
}

impl VectorTrack {
    /// Evaluates the vector track value at the given timestamp.
    #[must_use]
    pub fn sample(&self, time: f32) -> Vec3 {
        if self.keyframes.is_empty() {
            return Vec3::ZERO;
        }
        if self.keyframes.len() == 1 || time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        let last_idx = self.keyframes.len() - 1;
        if time >= self.keyframes[last_idx].time {
            return self.keyframes[last_idx].value;
        }

        // Find keyframe interval
        for i in 0..last_idx {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            if time >= k0.time && time <= k1.time {
                let duration = k1.time - k0.time;
                if duration <= 1e-6 {
                    return k0.value;
                }
                let t = (time - k0.time) / duration;
                return match self.interpolation {
                    Interpolation::Step => k0.value,
                    Interpolation::Linear | Interpolation::CubicSpline => {
                        k0.value.lerp(k1.value, t)
                    }
                };
            }
        }

        self.keyframes[last_idx].value
    }
}

/// A keyframe track for Quaternion rotations.
/// Uses `Quat::slerp` (Spherical Linear Interpolation) to prevent candy-wrapper mesh distortion
/// when joints undergo large rotations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotationTrack {
    /// Keyframe list ordered by timestamp.
    pub keyframes: Vec<Keyframe<Quat>>,
    /// Interpolation type.
    pub interpolation: Interpolation,
}

impl RotationTrack {
    /// Evaluates the quaternion rotation track value at the given timestamp using Slerp.
    #[must_use]
    pub fn sample(&self, time: f32) -> Quat {
        if self.keyframes.is_empty() {
            return Quat::IDENTITY;
        }
        if self.keyframes.len() == 1 || time <= self.keyframes[0].time {
            return self.keyframes[0].value.normalize();
        }
        let last_idx = self.keyframes.len() - 1;
        if time >= self.keyframes[last_idx].time {
            return self.keyframes[last_idx].value.normalize();
        }

        // Find keyframe interval
        for i in 0..last_idx {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            if time >= k0.time && time <= k1.time {
                let duration = k1.time - k0.time;
                if duration <= 1e-6 {
                    return k0.value.normalize();
                }
                let t = (time - k0.time) / duration;
                return match self.interpolation {
                    Interpolation::Step => k0.value.normalize(),
                    Interpolation::Linear | Interpolation::CubicSpline => {
                        k0.value.slerp(k1.value, t).normalize()
                    }
                };
            }
        }

        self.keyframes[last_idx].value.normalize()
    }
}

/// Target property for a joint animation channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetProperty {
    /// Translation (Position).
    Translation,
    /// Rotation (Quaternion).
    Rotation,
    /// Scale.
    Scale,
}

/// Animation channel targeting a specific joint and property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Index of the joint being animated.
    pub joint_index: usize,
    /// Target property (Translation, Rotation, or Scale).
    pub target_property: TargetProperty,
    /// Vector track for Translation / Scale (if applicable).
    pub vector_track: Option<VectorTrack>,
    /// Rotation track for Quaternions (if applicable).
    pub rotation_track: Option<RotationTrack>,
}

/// A named animation clip containing channels and playback duration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationClip {
    /// Name of the clip (e.g., "Idle", "Run", "Walk", "Attack").
    pub name: String,
    /// Total duration of the clip in seconds.
    pub duration: f32,
    /// Collection of animation channels.
    pub channels: Vec<Channel>,
}

impl AnimationClip {
    /// Creates a new animation clip with the given name and duration.
    #[must_use]
    pub fn new(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            channels: Vec::new(),
        }
    }
}