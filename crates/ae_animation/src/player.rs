// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Animation player ECS component and crossfading / blending manager for Aeon Engine.
//!

use crate::clip::{AnimationClip, TargetProperty};
use crate::skeleton::Skeleton;
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Playback state of an animation player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationState {
    /// Animation is actively updating.
    #[default]
    Playing,
    /// Animation is paused at current timestamp.
    Paused,
    /// Animation is stopped.
    Stopped,
}

/// ECS component that manages animation playback, time accumulation, and crossfade blending.
/// Supports crossfading (`crossfade()`) between `current_clip` and `target_clip` using a smooth
/// `blend_factor` (0.0 = current clip, 1.0 = target clip) over `blend_duration` seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPlayer {
    /// Active primary animation clip.
    pub current_clip: Option<AnimationClip>,
    /// Elapsed playback time in seconds for the primary clip.
    pub current_time: f32,
    /// Target clip being blended into during a crossfade.
    pub target_clip: Option<AnimationClip>,
    /// Elapsed playback time in seconds for the target clip.
    pub target_time: f32,
    /// Current blend factor (0.0 to 1.0) between primary and target clips.
    pub blend_factor: f32,
    /// Total duration of active crossfade transition in seconds.
    pub blend_duration: f32,
    /// Current playback state (Playing, Paused, Stopped).
    pub state: AnimationState,
    /// Playback speed multiplier (1.0 = normal speed).
    pub speed: f32,
    /// Whether the animation automatically loops when reaching clip duration.
    pub looping: bool,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self {
            current_clip: None,
            current_time: 0.0,
            target_clip: None,
            target_time: 0.0,
            blend_factor: 0.0,
            blend_duration: 0.0,
            state: AnimationState::Playing,
            speed: 1.0,
            looping: true,
        }
    }
}

impl AnimationPlayer {
    /// Creates a new animation player with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Plays the given clip immediately, clearing any active crossfades.
    pub fn play(&mut self, clip: AnimationClip) {
        self.current_clip = Some(clip);
        self.current_time = 0.0;
        self.target_clip = None;
        self.target_time = 0.0;
        self.blend_factor = 0.0;
        self.blend_duration = 0.0;
        self.state = AnimationState::Playing;
    }

    /// Initiates a smooth crossfade transition from the current clip to a new target clip.
    /// # Arguments
    /// * `new_clip` - The target animation clip to blend into.
    /// * `duration` - Transition duration in seconds (e.g., 0.25s).
    pub fn crossfade(&mut self, new_clip: AnimationClip, duration: f32) {
        if self.current_clip.is_none() {
            self.play(new_clip);
            return;
        }

        self.target_clip = Some(new_clip);
        self.target_time = 0.0;
        self.blend_factor = 0.0;
        self.blend_duration = duration.max(1e-4);
        self.state = AnimationState::Playing;
    }

    /// Advances animation playback timers by `delta_time` seconds.
    pub fn update(&mut self, delta_time: f32) {
        if self.state != AnimationState::Playing {
            return;
        }

        let dt = delta_time * self.speed;

        // Advance primary clip
        if let Some(ref clip) = self.current_clip {
            self.current_time += dt;
            if clip.duration > 0.0 {
                if self.looping {
                    self.current_time %= clip.duration;
                } else if self.current_time > clip.duration {
                    self.current_time = clip.duration;
                }
            }
        }

        // Advance target clip & blend factor if crossfading
        if let Some(ref target) = self.target_clip {
            self.target_time += dt;
            if target.duration > 0.0 {
                if self.looping {
                    self.target_time %= target.duration;
                } else if self.target_time > target.duration {
                    self.target_time = target.duration;
                }
            }

            if self.blend_duration > 0.0 {
                self.blend_factor += dt / self.blend_duration;
                if self.blend_factor >= 1.0 {
                    // Crossfade complete: promote target to primary
                    self.current_clip = self.target_clip.take();
                    self.current_time = self.target_time;
                    self.blend_factor = 0.0;
                    self.blend_duration = 0.0;
                }
            }
        }
    }

    /// Evaluates local joint transform matrices for the given skeleton at current playback time.
    /// Handles clip sampling and crossfade blending between `current_clip` and `target_clip`.
    #[must_use]
    pub fn evaluate_pose(&self, skeleton: &Skeleton) -> Vec<Mat4> {
        let count = skeleton.joints.len();
        if count == 0 {
            return Vec::new();
        }

        // Start with skeleton bind poses
        let mut translations: Vec<Vec3> = vec![Vec3::ZERO; count];
        let mut rotations: Vec<Quat> = vec![Quat::IDENTITY; count];
        let mut scales: Vec<Vec3> = vec![Vec3::ONE; count];

        for (i, joint) in skeleton.joints.iter().enumerate() {
            let (s, r, t) = joint.local_bind_pose.to_scale_rotation_translation();
            translations[i] = t;
            rotations[i] = r;
            scales[i] = s;
        }

        // Sample primary clip
        if let Some(ref clip) = self.current_clip {
            Self::sample_clip_into_pose(
                clip,
                self.current_time,
                &mut translations,
                &mut rotations,
                &mut scales,
            );
        }

        // Sample and blend target clip if crossfading
        if let Some(ref target) = self.target_clip {
            if self.blend_factor > 0.0 {
                let mut target_t = translations.clone();
                let mut target_r = rotations.clone();
                let mut target_s = scales.clone();

                Self::sample_clip_into_pose(
                    target,
                    self.target_time,
                    &mut target_t,
                    &mut target_r,
                    &mut target_s,
                );

                let alpha = self.blend_factor.clamp(0.0, 1.0);
                for i in 0..count {
                    translations[i] = translations[i].lerp(target_t[i], alpha);
                    rotations[i] = rotations[i].slerp(target_r[i], alpha);
                    scales[i] = scales[i].lerp(target_s[i], alpha);
                }
            }
        }

        // Reconstruct TRS matrices
        let mut local_poses = vec![Mat4::IDENTITY; count];
        for i in 0..count {
            local_poses[i] = Mat4::from_scale_rotation_translation(
                scales[i],
                rotations[i].normalize(),
                translations[i],
            );
        }

        local_poses
    }

    /// Internal helper to sample a clip's tracks into TRS arrays.
    fn sample_clip_into_pose(
        clip: &AnimationClip,
        time: f32,
        translations: &mut [Vec3],
        rotations: &mut [Quat],
        scales: &mut [Vec3],
    ) {
        for channel in &clip.channels {
            let idx = channel.joint_index;
            if idx >= translations.len() {
                continue;
            }

            match channel.target_property {
                TargetProperty::Translation => {
                    if let Some(ref track) = channel.vector_track {
                        translations[idx] = track.sample(time);
                    }
                }
                TargetProperty::Rotation => {
                    if let Some(ref track) = channel.rotation_track {
                        rotations[idx] = track.sample(time);
                    }
                }
                TargetProperty::Scale => {
                    if let Some(ref track) = channel.vector_track {
                        scales[idx] = track.sample(time);
                    }
                }
            }
        }
    }
}