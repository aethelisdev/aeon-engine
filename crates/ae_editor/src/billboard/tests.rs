// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Viewport 3D Billboard Subsystem
//!
//! Validates vertex memory alignment, uniform byte layout, layer mapping,
//! and coordinate invariants.

use super::types::{BillboardIconType, BillboardUniform, BillboardVertex};

#[test]
fn test_billboard_vertex_layout() {
    assert_eq!(
        std::mem::size_of::<BillboardVertex>(),
        (3 + 2 + 2 + 1 + 4 + 4 + 4) * 4
    );
    let layout = BillboardVertex::layout();
    assert_eq!(
        layout.array_stride,
        std::mem::size_of::<BillboardVertex>() as u64
    );
}

#[test]
fn test_billboard_uniform_size() {
    // 16 floats (mat4x4) + 2 floats (viewport) + 1 float (icon_size) + 1 float (pad) = 20 floats = 80 bytes
    assert_eq!(std::mem::size_of::<BillboardUniform>(), 80);
    assert_eq!(std::mem::size_of::<BillboardUniform>() % 16, 0);
}

#[test]
fn test_billboard_icon_type_layer_mappings() {
    assert_eq!(BillboardIconType::Light.layer_index(), 8.0);
    assert_eq!(BillboardIconType::Camera.layer_index(), 9.0);
    assert_eq!(BillboardIconType::AudioSource.layer_index(), 15.0);
}

#[test]
fn test_billboard_default_tints() {
    let light_tint = BillboardIconType::Light.default_tint();
    assert!(light_tint[0] > 0.5); // Yellow/warm
    assert_eq!(light_tint[3], 1.0);

    let audio_tint = BillboardIconType::AudioSource.default_tint();
    assert!(audio_tint[2] > 0.5); // Blue/cyan
    assert_eq!(audio_tint[3], 1.0);
}