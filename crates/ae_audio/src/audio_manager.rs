// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::components::{AudioListener, AudioSource};
use crate::spatial::SpatialAudioMath;
use ae_core::ecs::Position;
use ae_world::{COOKED_AUDIO_MAGIC, CookedAudio, VfsProvider};
use glam::Vec3;
use rodio::Source;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

/// High-performance audio mixer managing hardware output streams, active sound sinks,
/// and real-time 3D spatial updates.
///
/// Integrates with the engine event bus and Module Isolation architecture (`EngineModule::Audio`).
/// When audio is disabled, all mixing operations are bypassed with zero CPU/VRAM allocations.
pub struct AudioManager {
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    sinks: HashMap<hecs::Entity, (rodio::Sink, String)>,
    vfs: Option<Arc<dyn VfsProvider>>,
    master_volume: f32,
    is_muted: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    /// Creates and initializes the `AudioManager` in silent headless fallback mode without hardware output.
    ///
    /// Useful for automated CI pipelines, server headless execution, or environments without physical audio devices.
    pub fn new_silent() -> Self {
        Self {
            _stream: None,
            stream_handle: None,
            sinks: HashMap::new(),
            vfs: None,
            master_volume: 1.0,
            is_muted: false,
        }
    }

    /// Creates and initializes the `AudioManager` with hardware output device streams.
    ///
    /// If running in headless CI environments (`CI`, `GITHUB_ACTIONS`, or `AEON_HEADLESS_AUDIO`)
    /// or if no physical audio output device is present or audio driver initialization fails,
    /// falls back gracefully to silent audio mode without crashing or interrupting engine execution.
    pub fn new() -> Self {
        if std::env::var("AEON_HEADLESS_AUDIO").is_ok()
            || (cfg!(target_os = "windows")
                && (std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()))
        {
            log::info!(
                "🔊 Headless environment detected. Initializing audio manager in silent mode."
            );
            return Self::new_silent();
        }

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
            vfs: None,
            master_volume: 1.0,
            is_muted: false,
        }
    }

    /// Attaches a [`VfsProvider`] used to stream compressed or cooked audio payloads from
    /// `.aepak` bundles or partitioned project directories without raw disk paths.
    pub fn set_vfs_provider(&mut self, vfs: Option<Arc<dyn VfsProvider>>) {
        self.vfs = vfs;
    }

    /// Returns the attached [`VfsProvider`], if any.
    #[must_use]
    pub fn vfs_provider(&self) -> Option<&Arc<dyn VfsProvider>> {
        self.vfs.as_ref()
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

    /// Triggers sound playback for a specific entity's `AudioSource`, reading via the attached
    /// [`VfsProvider`] when present or falling back to workspace filesystem bytes.
    pub fn play_sound(&mut self, entity: hecs::Entity, source: &AudioSource) -> bool {
        if self.is_muted || source.sound_path.trim().is_empty() {
            return false;
        }
        let bytes = match self.read_audio_bytes(&source.sound_path) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("Failed to read audio '{}': {}", source.sound_path, e);
                return false;
            }
        };
        self.play_sound_from_bytes(entity, source, bytes)
    }

    /// Triggers sound playback for `entity` by reading `source.sound_path` directly from `vfs`.
    pub fn play_sound_from_vfs(
        &mut self,
        entity: hecs::Entity,
        source: &AudioSource,
        vfs: &dyn VfsProvider,
    ) -> bool {
        if self.is_muted || source.sound_path.trim().is_empty() {
            return false;
        }
        let bytes = match read_audio_from_vfs(vfs, &source.sound_path) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("Failed to read VFS audio '{}': {}", source.sound_path, e);
                return false;
            }
        };
        self.play_sound_from_bytes(entity, source, bytes)
    }

    /// Triggers sound playback directly from an in-memory encoded or [`CookedAudio`] byte buffer.
    pub fn play_sound_from_bytes(
        &mut self,
        entity: hecs::Entity,
        source: &AudioSource,
        bytes: Vec<u8>,
    ) -> bool {
        if self.is_muted {
            return false;
        }

        // Lazy reconnect if hardware output stream was unavailable at startup
        if self.stream_handle.is_none()
            && let Ok((stream, handle)) = rodio::OutputStream::try_default()
        {
            log::info!("🔊 Audio Manager reconnected to hardware output device.");
            self._stream = Some(stream);
            self.stream_handle = Some(handle);
        }

        let Some(handle) = &self.stream_handle else {
            return false;
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

        if let Err(e) = append_decoded_audio_to_sink(&sink, bytes, source.looping) {
            log::warn!("Failed to decode sound '{}': {}", source.sound_path, e);
            return false;
        }

        self.sinks.insert(entity, (sink, source.sound_path.clone()));
        log::info!(
            "🔊 Playing sound {:?} on entity {:?}",
            source.sound_path,
            entity
        );
        true
    }

    fn read_audio_bytes(&self, sound_path: &str) -> Result<Vec<u8>, String> {
        if let Some(vfs) = &self.vfs
            && let Ok(bytes) = read_audio_from_vfs(vfs.as_ref(), sound_path)
        {
            return Ok(bytes);
        }
        std::fs::read(sound_path)
            .or_else(|_| std::fs::read(std::path::Path::new("assets").join(sound_path)))
            .map_err(|e| e.to_string())
    }

    /// Stops sound playback for a specific entity's `AudioSource`.
    pub fn stop_sound(&mut self, entity: hecs::Entity) {
        if let Some((sink, _)) = self.sinks.remove(&entity) {
            sink.stop();
        }
    }

    /// Per-frame 3D Spatial Audio and ECS update loop.
    ///
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
            for (sink, _) in self.sinks.values() {
                sink.pause();
            }
            return;
        }

        // Unpause any paused sinks when re-enabled
        for (sink, _) in self.sinks.values() {
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

        // 2. Remove stopped/finished sinks OR sinks whose AudioSource component was removed or stopped
        self.sinks.retain(|entity, (sink, _)| {
            let is_active = if let Ok(source) = world.get::<&AudioSource>(*entity) {
                !sink.empty() && source.is_playing
            } else {
                false
            };
            if !is_active {
                sink.stop();
                if let Ok(mut src) = world.get::<&mut AudioSource>(*entity) {
                    src.is_playing = false;
                }
            }
            is_active
        });

        // 3. Process AudioSource entities (without hard Position dependency)
        for (entity, source) in world.query::<(hecs::Entity, &mut AudioSource)>().iter() {
            // Check if active sink sound_path has changed
            if let Some((_, active_path)) = self.sinks.get(&entity)
                && active_path != &source.sound_path
            {
                self.stop_sound(entity);
            }

            // Auto-play on start or resume if is_playing is true but sink is missing
            let should_start =
                (source.play_on_start || source.is_playing) && !self.sinks.contains_key(&entity);
            if should_start && !source.sound_path.trim().is_empty() {
                let success = self.play_sound(entity, source);
                source.is_playing = success;
                source.play_on_start = false;
            }

            // Update spatial 3D volume & attenuation
            if source.is_playing
                && let Some((sink, _)) = self.sinks.get(&entity)
            {
                if source.is_spatial
                    && let Ok(pos) = world.get::<&Position>(entity)
                {
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

/// Reads an audio file by virtual path from a [`VfsProvider`], checking raw virtual path first
/// and falling back to `"assets/<path>"` or `"Content/<path>"` workspace prefixes.
fn read_audio_from_vfs(vfs: &dyn VfsProvider, sound_path: &str) -> Result<Vec<u8>, String> {
    let trimmed = sound_path.trim();
    if trimmed.is_empty() {
        return Err("Empty audio path".to_string());
    }
    if let Ok(bytes) = vfs.read_bytes(trimmed) {
        return Ok(bytes);
    }
    let normalized = trimmed.replace('\\', "/");
    let stripped = normalized.trim_start_matches("./").trim_start_matches('/');
    if stripped != trimmed
        && let Ok(bytes) = vfs.read_bytes(stripped)
    {
        return Ok(bytes);
    }
    if !stripped.starts_with("assets/")
        && let Ok(bytes) = vfs.read_bytes(&format!("assets/{stripped}"))
    {
        return Ok(bytes);
    }
    if !stripped.starts_with("Content/")
        && let Ok(bytes) = vfs.read_bytes(&format!("Content/{stripped}"))
    {
        return Ok(bytes);
    }
    Err(format!("Audio asset not found in VFS: '{sound_path}'"))
}

/// Decodes either a [`CookedAudio`] payload (`SamplesBuffer`) or compressed WAV/OGG/FLAC bytes
/// in memory and queues them onto `sink`.
fn append_decoded_audio_to_sink(
    sink: &rodio::Sink,
    bytes: Vec<u8>,
    looping: bool,
) -> Result<(), String> {
    if bytes.starts_with(&COOKED_AUDIO_MAGIC) {
        let cooked = CookedAudio::decode(&bytes).map_err(|e| e.to_string())?;
        let samples = cooked.samples_i16().map_err(|e| e.to_string())?.to_vec();
        let buffer =
            rodio::buffer::SamplesBuffer::new(cooked.channels, cooked.sample_rate, samples);
        if looping {
            sink.append(buffer.repeat_infinite());
        } else {
            sink.append(buffer);
        }
        return Ok(());
    }

    let decoder = rodio::Decoder::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    if looping {
        sink.append(decoder.repeat_infinite());
    } else {
        sink.append(decoder);
    }
    Ok(())
}

/// Decodes raw WAV/OGG/MP3/FLAC bytes (or passes through an existing [`CookedAudio`] envelope)
/// into a `16`-byte aligned [`CookedAudio`] PCM buffer for offline asset cooking.
///
/// # Errors
/// Returns a descriptive error string when `bytes` cannot be decoded as audio.
pub fn decode_audio_bytes_to_cooked(bytes: &[u8]) -> Result<CookedAudio, String> {
    if bytes.starts_with(&COOKED_AUDIO_MAGIC) {
        return CookedAudio::decode(bytes).map_err(|e| e.to_string());
    }
    let decoder = rodio::Decoder::new(Cursor::new(bytes.to_vec())).map_err(|e| e.to_string())?;
    let sample_rate = decoder.sample_rate().max(1);
    let channels = decoder.channels().max(1);
    let samples: Vec<i16> = decoder.collect();
    Ok(CookedAudio::from_samples_i16(
        sample_rate,
        channels,
        &samples,
    ))
}

#[cfg(test)]
mod vfs_tests {
    use super::*;
    use ae_world::{AssetKind, PakBuilder, PakCodec, PakMemoryVfsProvider};

    #[test]
    fn test_audio_vfs_reads_cooked_audio_without_filesystem() {
        let cooked = CookedAudio::from_samples_i16(44_100, 1, &[0, 500, -500, 1000, -1000]);
        let cooked_bytes = cooked.encode().expect("encode cooked audio");

        let mut builder = PakBuilder::new();
        builder.add_asset(
            "Content/sfx/chime.cookaud",
            AssetKind::Audio,
            PakCodec::Raw,
            cooked_bytes,
        );
        let pak = builder.build_to_vec().expect("build pak");
        let vfs = PakMemoryVfsProvider::from_bytes(pak).expect("mount pak");

        let bytes = read_audio_from_vfs(&vfs, "sfx/chime.cookaud").expect("read from Content/");
        let decoded = CookedAudio::decode(&bytes).expect("decode cooked audio");
        assert_eq!(decoded.sample_rate, 44_100);
        assert_eq!(decoded.channels, 1);
        assert_eq!(
            decoded.samples_i16().expect("pcm samples"),
            &[0, 500, -500, 1000, -1000]
        );
    }
}