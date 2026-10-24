// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::render::engine::RenderFrameScratch;

#[test]
fn test_quantize_dimension_arithmetic_invariants() {
    // Zero dimensions or zero bucket size must return 0 without division-by-zero panics
    assert_eq!(quantize_dimension(0, 64), 0);
    assert_eq!(quantize_dimension(50, 0), 0);
    assert_eq!(quantize_dimension(0, 0), 0);

    // Sub-bucket rounding
    assert_eq!(quantize_dimension(1, 64), 64);
    assert_eq!(quantize_dimension(63, 64), 64);
    assert_eq!(quantize_dimension(64, 64), 64);

    // Next bucket boundary transitions
    assert_eq!(quantize_dimension(65, 64), 128);
    assert_eq!(quantize_dimension(128, 64), 128);
    assert_eq!(quantize_dimension(129, 64), 192);

    // Common display resolutions with 64px bucketing
    assert_eq!(quantize_dimension(1920, 64), 1920);
    assert_eq!(quantize_dimension(1921, 64), 1984);
    assert_eq!(quantize_dimension(1080, 64), 1088);

    // Arbitrary non-power-of-two bucket sizes
    assert_eq!(quantize_dimension(10, 7), 14);
    assert_eq!(quantize_dimension(21, 7), 21);
}

#[test]
fn test_render_frame_scratch_sort_transparent_model_draws_invariants() {
    let mut scratch = RenderFrameScratch::default();

    // 1. Empty buffer must be a no-op without panic
    scratch.sort_transparent_model_draws();
    assert!(scratch.transparent_model_draws.is_empty());

    let handle_a = crate::asset::AssetHandle::default();
    let handle_b = crate::asset::AssetHandle::default();

    // 2. Single element buffer must remain valid without mutation
    scratch
        .transparent_model_draws
        .push(forward::TransparentSubmeshDraw {
            distance_sq: 42.0,
            handle: handle_a,
            submesh_index: 0,
            buffer_offset: 0,
            instance_index: 0,
            override_tex: None,
        });
    scratch.sort_transparent_model_draws();
    assert_eq!(scratch.transparent_model_draws.len(), 1);
    assert_eq!(scratch.transparent_model_draws[0].distance_sq, 42.0);

    // 3. Multi-instance and multi-submesh draw ordering (Back-to-Front / Descending distance)
    scratch
        .transparent_model_draws
        .push(forward::TransparentSubmeshDraw {
            distance_sq: 15.0,
            handle: handle_a,
            submesh_index: 1,
            buffer_offset: 0,
            instance_index: 0,
            override_tex: None,
        });
    scratch
        .transparent_model_draws
        .push(forward::TransparentSubmeshDraw {
            distance_sq: 150.0,
            handle: handle_b,
            submesh_index: 0,
            buffer_offset: 10,
            instance_index: 1,
            override_tex: None,
        });
    scratch
        .transparent_model_draws
        .push(forward::TransparentSubmeshDraw {
            distance_sq: 0.5,
            handle: handle_b,
            submesh_index: 1,
            buffer_offset: 10,
            instance_index: 1,
            override_tex: None,
        });
    scratch
        .transparent_model_draws
        .push(forward::TransparentSubmeshDraw {
            distance_sq: 42.0, // Equal distance submesh draw
            handle: handle_b,
            submesh_index: 2,
            buffer_offset: 10,
            instance_index: 2,
            override_tex: None,
        });

    scratch.sort_transparent_model_draws();

    assert_eq!(scratch.transparent_model_draws.len(), 5);
    assert_eq!(scratch.transparent_model_draws[0].distance_sq, 150.0);
    assert_eq!(scratch.transparent_model_draws[1].distance_sq, 42.0);
    assert_eq!(scratch.transparent_model_draws[2].distance_sq, 42.0);
    assert_eq!(scratch.transparent_model_draws[3].distance_sq, 15.0);
    assert_eq!(scratch.transparent_model_draws[4].distance_sq, 0.5);

    // Invariant: strictly monotonically non-increasing (farthest first)
    for i in 1..scratch.transparent_model_draws.len() {
        assert!(
            scratch.transparent_model_draws[i - 1].distance_sq
                >= scratch.transparent_model_draws[i].distance_sq
        );
    }
}

#[test]
fn test_render_frame_scratch_sort_extreme_floats_and_infinities() {
    let mut scratch = RenderFrameScratch::default();
    let handle = crate::asset::AssetHandle::default();

    let extreme_values = [
        f32::NEG_INFINITY,
        10.0,
        f32::INFINITY,
        0.0,
        f32::NAN,
        100.0,
        -5.0,
    ];

    for &val in &extreme_values {
        scratch
            .transparent_model_draws
            .push(forward::TransparentSubmeshDraw {
                distance_sq: val,
                handle,
                submesh_index: 0,
                buffer_offset: 0,
                instance_index: 0,
                override_tex: None,
            });
    }

    // Must sort without panic using IEEE-754 total_cmp
    scratch.sort_transparent_model_draws();
    assert_eq!(scratch.transparent_model_draws.len(), extreme_values.len());

    // In total_cmp ordering, positive infinity is greater than real numbers, and real numbers are greater than -inf
    assert!(
        scratch.transparent_model_draws[0].distance_sq.is_nan()
            || scratch.transparent_model_draws[0].distance_sq.is_infinite()
    );
}