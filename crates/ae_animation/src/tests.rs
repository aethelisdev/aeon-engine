// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for skeletal hierarchy, keyframe interpolation, and skinning palette calculations.

#[cfg(test)]
mod tests {
    use crate::clip::{
        AnimationClip, Channel, Interpolation, Keyframe, RotationTrack, TargetProperty, VectorTrack,
    };
    use crate::player::AnimationPlayer;
    use crate::skeleton::{Joint, Skeleton};
    use crate::skinning::{BoneCapacityPreset, compute_skinning_matrices};
    use glam::{Mat4, Quat, Vec3};

    #[test]
    fn test_skeleton_creation_and_flat_tree_evaluation() {
        let root = Joint::new(
            "Root",
            None,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            Mat4::IDENTITY,
        );
        let spine = Joint::new(
            "Spine",
            Some(0),
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            Mat4::IDENTITY,
        );

        let skeleton = Skeleton::from_joints(vec![root, spine]);
        assert_eq!(skeleton.len(), 2);

        let local_poses = vec![
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
        ];

        let globals = skeleton.evaluate_global_transforms(&local_poses);
        assert_eq!(globals.len(), 2);
        assert_eq!(
            Vec3::new(
                globals[0].w_axis.x,
                globals[0].w_axis.y,
                globals[0].w_axis.z
            ),
            Vec3::new(0.0, 1.0, 0.0)
        );
        assert_eq!(
            Vec3::new(
                globals[1].w_axis.x,
                globals[1].w_axis.y,
                globals[1].w_axis.z
            ),
            Vec3::new(0.0, 3.0, 0.0)
        );
    }

    #[test]
    fn test_vector_track_linear_interpolation() {
        let track = VectorTrack {
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    value: Vec3::ZERO,
                },
                Keyframe {
                    time: 2.0,
                    value: Vec3::new(10.0, 20.0, 30.0),
                },
            ],
            interpolation: Interpolation::Linear,
        };

        assert_eq!(track.sample(0.0), Vec3::ZERO);
        assert_eq!(track.sample(1.0), Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(track.sample(2.0), Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_rotation_track_slerp_interpolation() {
        let q0 = Quat::IDENTITY;
        let q1 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2); // 90 deg Y

        let track = RotationTrack {
            keyframes: vec![
                Keyframe {
                    time: 0.0,
                    value: q0,
                },
                Keyframe {
                    time: 2.0,
                    value: q1,
                },
            ],
            interpolation: Interpolation::Linear,
        };

        let sampled_mid = track.sample(1.0);
        let expected_mid = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4); // 45 deg Y
        assert!((sampled_mid.dot(expected_mid) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_animation_player_crossfade() {
        let mut player = AnimationPlayer::new();

        let mut clip_a = AnimationClip::new("ClipA", 2.0);
        clip_a.channels.push(Channel {
            joint_index: 0,
            target_property: TargetProperty::Translation,
            vector_track: Some(VectorTrack {
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        value: Vec3::ZERO,
                    },
                    Keyframe {
                        time: 2.0,
                        value: Vec3::new(10.0, 0.0, 0.0),
                    },
                ],
                interpolation: Interpolation::Linear,
            }),
            rotation_track: None,
        });

        let mut clip_b = AnimationClip::new("ClipB", 2.0);
        clip_b.channels.push(Channel {
            joint_index: 0,
            target_property: TargetProperty::Translation,
            vector_track: Some(VectorTrack {
                keyframes: vec![
                    Keyframe {
                        time: 0.0,
                        value: Vec3::new(100.0, 0.0, 0.0),
                    },
                    Keyframe {
                        time: 2.0,
                        value: Vec3::new(200.0, 0.0, 0.0),
                    },
                ],
                interpolation: Interpolation::Linear,
            }),
            rotation_track: None,
        });

        player.play(clip_a);
        player.crossfade(clip_b, 1.0);
        assert_eq!(player.blend_factor, 0.0);
        assert!(player.target_clip.is_some());

        player.update(0.5); // 50% blend
        assert!((player.blend_factor - 0.5).abs() < 1e-4);

        player.update(0.6); // Fully blended into target clip
        assert_eq!(player.blend_factor, 0.0);
        assert!(player.target_clip.is_none());
        assert_eq!(player.current_clip.as_ref().unwrap().name, "ClipB");
    }

    #[test]
    fn test_ssbo_skinning_palette_alignment_and_bytes() {
        let skeleton = Skeleton::from_joints(vec![Joint::new(
            "Joint0",
            None,
            Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )]);

        let globals = vec![Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0))];
        let palette = compute_skinning_matrices(&skeleton, &globals);

        assert_eq!(palette.len(), 1);
        assert_eq!(palette.as_bytes().len(), 64); // 4x4 float matrix = 64 bytes
        assert_eq!(BoneCapacityPreset::MobileMeshSection as usize, 75);
        assert_eq!(BoneCapacityPreset::StandardDesktop as usize, 256);
        assert_eq!(BoneCapacityPreset::UnlimitedSsbo as usize, 65536);
    }
}