// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified scene component registry combining core `ae_plugin_api` components with
//! higher-level animation (`AnimationPlayer`, `Skeleton`) and audio components.

use ae_ecs::ComponentRegistry;
use std::sync::OnceLock;

/// Returns the shared process-wide [`ComponentRegistry`] covering all serializable
/// scene, animation, and audio components.
pub fn scene_component_registry() -> &'static ComponentRegistry {
    static REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = ComponentRegistry::new();
        ae_plugin_api::register_engine_components(&mut registry);
        registry.register_with_default::<ae_animation::AnimationPlayer>(
            "AnimationPlayer",
            ae_animation::AnimationPlayer::default,
        );
        registry.register_with_default::<ae_animation::Skeleton>(
            "Skeleton",
            ae_animation::Skeleton::default,
        );
        registry.register_with_default::<ae_audio::AudioSource>(
            "AudioSource",
            ae_audio::AudioSource::default,
        );
        registry.register_with_default::<ae_audio::AudioListener>(
            "AudioListener",
            ae_audio::AudioListener::default,
        );
        registry
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_ecs::{World, component_type_hash};

    #[test]
    fn test_scene_registry_roundtrips_animation_and_audio_components() {
        let registry = scene_component_registry();
        for required in [
            "AnimationPlayer",
            "Skeleton",
            "AudioSource",
            "AudioListener",
        ] {
            assert!(
                registry.get_by_name(required).is_some(),
                "{required} must be registered in scene_component_registry"
            );
        }

        let mut world = World::new();
        let player = ae_animation::AnimationPlayer {
            speed: 2.25,
            ..Default::default()
        };
        let skel = ae_animation::Skeleton::from_joints(vec![ae_animation::Joint::new(
            "Root",
            None,
            Default::default(),
            Default::default(),
        )]);
        let src_ent = world.spawn((player, skel));

        let player_handler = registry
            .get_by_name("AnimationPlayer")
            .expect("AnimationPlayer handler");
        let skel_handler = registry.get_by_name("Skeleton").expect("Skeleton handler");

        let player_bytes = player_handler
            .capture(&world, src_ent)
            .expect("capture AnimationPlayer");
        let skel_bytes = skel_handler
            .capture(&world, src_ent)
            .expect("capture Skeleton");

        let dst_ent = world.spawn(());
        player_handler
            .apply_versioned(
                &mut world,
                dst_ent,
                player_handler.schema_version(),
                &player_bytes,
            )
            .expect("restore AnimationPlayer");
        skel_handler
            .apply_versioned(
                &mut world,
                dst_ent,
                skel_handler.schema_version(),
                &skel_bytes,
            )
            .expect("restore Skeleton");

        let restored_speed = world
            .inspect::<ae_animation::AnimationPlayer, _>(dst_ent, |p| p.speed)
            .expect("restored AnimationPlayer");
        assert!((restored_speed - 2.25).abs() < 1e-6);
        let joint_count = world
            .inspect::<ae_animation::Skeleton, _>(dst_ent, |s| s.len())
            .expect("restored Skeleton");
        assert_eq!(joint_count, 1);

        // Unknown hash must fail lookup safely without panicking
        assert!(
            registry
                .get_by_type_hash(component_type_hash("NonExistentSceneComponent"))
                .is_none()
        );
    }
}