// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use serde::{Deserialize, Serialize};

/// ECS component attached to entities that emit 2D stereo or 3D spatial audio cues.
/// Stores sound asset file references (`.wav`, `.ogg`, `.mp3`), playback parameters
/// (volume gain, pitch modulation, looping toggle), and 3D spatial attenuation bounds
/// (`min_distance`, `max_distance`). Integrates with `hecs::World` and
/// `ae_audio::audio_manager::AudioManager`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioSource {
    /// Absolute or workspace-relative path to sound file.
    pub sound_path: String,
    /// Playback volume gain scaling factor (0.0 = silent, 1.0 = normal, >1.0 = amplified).
    pub volume: f32,
    /// Playback pitch multiplier (1.0 = original pitch, 0.5 = octave down, 2.0 = octave up).
    pub pitch: f32,
    /// Whether 3D spatial positioning, Doppler attenuation, and stereo panning are enabled.
    pub is_spatial: bool,
    /// Whether the sound automatically loops when reaching EOF.
    pub looping: bool,
    /// Whether the sound starts playing immediately when spawned into the scene.
    pub play_on_start: bool,
    /// Real-time playback status toggle.
    pub is_playing: bool,
    /// Minimum distance for full 100% volume attenuation before distance falloff begins.
    pub min_distance: f32,
    /// Maximum distance boundary beyond which spatial audio becomes completely silent.
    pub max_distance: f32,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            sound_path: String::new(),
            volume: 1.0,
            pitch: 1.0,
            is_spatial: true,
            looping: false,
            play_on_start: true,
            is_playing: false,
            min_distance: 1.0,
            max_distance: 50.0,
        }
    }
}

impl AudioSource {
    /// Creates a new `AudioSource` with default spatial audio settings for a given sound file path.
    pub fn new(sound_path: impl Into<String>) -> Self {
        Self {
            sound_path: sound_path.into(),
            ..Default::default()
        }
    }
}

/// ECS marker component designating an entity as the active 3D Spatial Audio Listener (microphone/ear).
/// Typically attached to the active Camera or Player entity. The `AudioManager` uses
/// `AudioListener` position and forward vector orientation to calculate real-time stereo
/// panning, distance falloff attenuation, and Doppler shifts for all active `AudioSource` emitters.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioListener;