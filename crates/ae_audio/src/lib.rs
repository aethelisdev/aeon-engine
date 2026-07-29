// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon Engine - Spatial 3D Audio Subsystem (`ae_audio`)
//!
//! Provides hardware-accelerated 3D spatial audio playback, distance attenuation,
//! stereo channel panning, `AudioSource` & `AudioListener` ECS components, and
//! module isolation support (`EngineModule::Audio`).
//!

pub mod audio_manager;
pub mod components;
pub mod spatial;

pub use audio_manager::AudioManager;
pub use components::{AudioListener, AudioSource};
pub use glam::Vec3;
pub use spatial::SpatialAudioMath;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_audio_manager_creation() {
        let manager = AudioManager::new();
        assert_eq!(manager.master_volume(), 1.0);
        assert!(!manager.is_muted());
    }

    #[test]
    fn test_spatial_audio_math_attenuation() {
        let listener = Vec3::ZERO;
        let emitter_near = Vec3::new(0.5, 0.0, 0.0);
        let emitter_mid = Vec3::new(25.0, 0.0, 0.0);
        let emitter_far = Vec3::new(100.0, 0.0, 0.0);

        let min_dist = 1.0;
        let max_dist = 50.0;

        let atten_near = SpatialAudioMath::compute_distance_attenuation(
            emitter_near,
            listener,
            min_dist,
            max_dist,
        );
        let atten_mid = SpatialAudioMath::compute_distance_attenuation(
            emitter_mid,
            listener,
            min_dist,
            max_dist,
        );
        let atten_far = SpatialAudioMath::compute_distance_attenuation(
            emitter_far,
            listener,
            min_dist,
            max_dist,
        );

        assert_eq!(atten_near, 1.0);
        assert!(atten_mid > 0.0 && atten_mid < 1.0);
        assert_eq!(atten_far, 0.0);
    }

    #[test]
    fn test_audio_module_isolation_toggle_resume() {
        let mut world = hecs::World::new();
        let mut audio_manager = AudioManager::new();

        let ent = world.spawn((
            ae_core::ecs::Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            AudioSource::new("non_existent_file.wav"),
        ));

        // 1. Update with audio ENABLED: should handle missing file gracefully
        audio_manager.update(&world, Vec3::ZERO, Vec3::X, true);

        // 2. Update with audio DISABLED: should pause without crashing or leaking sinks
        audio_manager.update(&world, Vec3::ZERO, Vec3::X, false);

        // 3. Update with audio RE-ENABLED: should resume audio system state gracefully
        audio_manager.update(&world, Vec3::ZERO, Vec3::X, true);

        assert!(world.contains(ent));
    }
}