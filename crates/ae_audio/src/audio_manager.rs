// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::components::{AudioListener, AudioSource};
use crate::spatial::SpatialAudioMath;
use ae_core::ecs::Position;
use glam::Vec3;
use rodio::Source;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

/// High-performance audio mixer managing hardware output streams, active sound sinks,
/// and real-time 3D spatial updates.
/// Integrates with the engine event bus and Module Isolation architecture (`EngineModule::Audio`).
/// When audio is disabled, all mixing operations are bypassed with zero CPU/VRAM allocations.
pub struct AudioManager {
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    sinks: HashMap<hecs::Entity, rodio::Sink>,
    master_volume: f32,
    is_muted: bool,
}

impl AudioManager {
    /// Creates and initializes the `AudioManager` with hardware output device streams.
    /// If no physical audio output device is present or audio driver initialization fails,
    /// falls back gracefully without crashing or interrupting engine execution.
    pub fn new() -> Self {
        let (stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok((s, h)) => {
                log::info!(
                    "🔊 Audio Manager initialized successfully with hardware output device."
                );
                (Some(s), Some(h))
            }
            Err(e) => {
                log::warn!(
                    "🔊 Audio output device initialization warning: {}. Running in silent audio fallback mode.",
                    e
                );
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle,
            sinks: HashMap::new(),
            master_volume: 1.0,
            is_muted: false,
        }
    }

    /// Sets global master volume gain multiplier `[0.0, 1.0+]`.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.max(0.0);
    }

    /// Returns current master volume gain.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Toggles master mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.is_muted = muted;
    }

    /// Returns master mute state.
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    /// Triggers sound playback for a specific entity's `AudioSource`.
    pub fn play_sound(&mut self, entity: hecs::Entity, source: &AudioSource) -> bool {
        if self.is_muted || self.stream_handle.is_none() {
            return false;
        }

        let handle = match &self.stream_handle {
            Some(h) => h,
            None => return false,
        };

        if source.sound_path.trim().is_empty() {
            return false;
        }

        let file = match File::open(&source.sound_path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("Failed to open sound file '{:?}': {}", source.sound_path, e);
                return false;
            }
        };

        let reader = BufReader::new(file);
        let decoder = match rodio::Decoder::new(reader) {
            Ok(d) => d,
            Err(e) => {
                log::warn!(
                    "Failed to decode sound file '{:?}': {}",
                    source.sound_path,
                    e
                );
                return false;
            }
        };

        let sink = match rodio::Sink::try_new(handle) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to create audio sink: {}", e);
                return false;
            }
        };

        sink.set_volume(source.volume * self.master_volume);
        sink.set_speed(source.pitch);

        if source.looping {
            sink.append(decoder.repeat_infinite());
        } else {
            sink.append(decoder);
        }

        self.sinks.insert(entity, sink);
        log::info!(
            "🔊 Playing sound {:?} on entity {:?}",
            source.sound_path,
            entity
        );
        true
    }

    /// Stops sound playback for a specific entity's `AudioSource`.
    pub fn stop_sound(&mut self, entity: hecs::Entity) {
        if let Some(sink) = self.sinks.remove(&entity) {
            sink.stop();
        }
    }

    /// Per-frame 3D Spatial Audio and ECS update loop.
    /// Iterates active `AudioSource` entities, queries `AudioListener` ear position,
    /// computes distance attenuation falloff and panning gain, and updates hardware audio sinks.
    /// Bypasses all processing if `is_audio_enabled` is `false` (Module Isolation).
    pub fn update(
        &mut self,
        world: &hecs::World,
        fallback_listener_pos: Vec3,
        fallback_listener_right: Vec3,
        is_audio_enabled: bool,
    ) {
        // --- MODULE ISOLATION: Pause all hardware audio sinks if disabled ---
        if !is_audio_enabled || self.is_muted {
            for (_entity, sink) in self.sinks.iter() {
                sink.pause();
            }
            return;
        }

        // Unpause any paused sinks when re-enabled
        for (_entity, sink) in self.sinks.iter() {
            if sink.is_paused() {
                sink.play();
            }
        }

        // 1. Locate active AudioListener entity position & listener orientation
        let mut listener_pos = fallback_listener_pos;
        let mut listener_right = fallback_listener_right;

        if let Some((_ent, pos, _listener)) = world
            .query::<(hecs::Entity, &Position, &AudioListener)>()
            .iter()
            .next()
        {
            listener_pos = Vec3::new(pos.x, pos.y, pos.z);
            listener_right = Vec3::X; // Default listener right axis
        }

        // 2. Remove stopped/finished sinks OR sinks whose AudioSource component was removed (Trash Icon)
        self.sinks.retain(|entity, sink| {
            let has_source = world.get::<&AudioSource>(*entity).is_ok();
            let active = !sink.empty() && has_source;
            if !active {
                sink.stop();
                if let Ok(mut src) = world.get::<&mut AudioSource>(*entity) {
                    src.is_playing = false;
                }
            }
            active
        });

        // 3. Process AudioSource entities
        for (entity, pos, source) in world
            .query::<(hecs::Entity, &Position, &mut AudioSource)>()
            .iter()
        {
            // Auto-play on start or resume if is_playing is true but sink is missing
            if (source.play_on_start || source.is_playing) && !self.sinks.contains_key(&entity) {
                let success = self.play_sound(entity, source);
                source.is_playing = success;
                source.play_on_start = false;
            }

            // Update spatial 3D volume & attenuation
            if source.is_playing {
                if let Some(sink) = self.sinks.get(&entity) {
                    if source.is_spatial {
                        let emitter_pos = Vec3::new(pos.x, pos.y, pos.z);
                        let attenuation = SpatialAudioMath::compute_distance_attenuation(
                            emitter_pos,
                            listener_pos,
                            source.min_distance,
                            source.max_distance,
                        );
                        let (left_gain, right_gain) = SpatialAudioMath::compute_stereo_panning(
                            emitter_pos,
                            listener_pos,
                            listener_right,
                        );
                        let spatial_gain = attenuation * ((left_gain + right_gain) * 0.5);

                        sink.set_volume(source.volume * spatial_gain * self.master_volume);
                    } else {
                        sink.set_volume(source.volume * self.master_volume);
                    }
                    sink.set_speed(source.pitch);
                }
            }
        }
    }
}