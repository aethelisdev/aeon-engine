// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::{SpriteInstance, SpriteVertex};

#[test]
fn test_sprite_vertex_layout_alignment() {
    let layout = SpriteVertex::desc();
    assert_eq!(
        layout.array_stride,
        std::mem::size_of::<SpriteVertex>() as u64
    );
    assert_eq!(layout.attributes.len(), 2);
}

#[test]
fn test_sprite_instance_layout_alignment() {
    let layout = SpriteInstance::desc();
    assert_eq!(
        layout.array_stride,
        std::mem::size_of::<SpriteInstance>() as u64
    );
    assert_eq!(layout.step_mode, wgpu::VertexStepMode::Instance);
}

#[test]
fn test_sprite_instance_bytemuck_pod() {
    let instance = SpriteInstance {
        model_matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    };

    let bytes = bytemuck::bytes_of(&instance);
    assert_eq!(bytes.len(), std::mem::size_of::<SpriteInstance>());
}