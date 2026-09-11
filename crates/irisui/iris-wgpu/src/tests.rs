// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for `iris-wgpu` backend pipeline, retained command stream, and quad generation.

use super::*;
use iris_core::{Color, Rect, Style};

#[test]
fn test_quad_instance_generation() {
    let rect = Rect::new(10.0, 20.0, 200.0, 100.0);
    let style = Style::new()
        .background(Color::RED)
        .border(2.0, Color::WHITE)
        .border_radius(12.0);

    let quad = QuadInstance::from_style(rect, &style, None);

    assert_eq!(quad.rect, [10.0, 20.0, 200.0, 100.0]);
    assert_eq!(quad.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(quad.border_color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(quad.border_width, [2.0, 2.0, 2.0, 2.0]);
    assert_eq!(quad.corner_radii, [12.0, 12.0, 12.0, 12.0]);
}

#[test]
fn test_texture_quad_command_stream() {
    let mut list = DrawCommandList::new();
    assert!(list.commands.is_empty());
    assert!(list.texture_quads.is_empty());

    let tq = TextureQuadInstance {
        rect: [10.0, 10.0, 24.0, 24.0],
        uv_rect: [0.0, 0.0, 0.25, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        clip_rect: [0.0, 0.0, 0.0, 0.0],
    };
    list.push_texture_quad(tq);
    list.push_texture_quad(tq);

    assert_eq!(list.texture_quads.len(), 2);
    assert_eq!(list.commands.len(), 1);
    assert_eq!(
        list.commands[0],
        DrawCommand::DrawTexture { start: 0, count: 2 }
    );

    list.clear();
    assert!(list.commands.is_empty());
    assert!(list.texture_quads.is_empty());
}

#[test]
fn test_external_texture_quad_command_stream() {
    use iris_core::node::ExternalTextureId;

    let mut list = DrawCommandList::new();
    assert!(list.external_texture_quads.is_empty());

    let ext_id = ExternalTextureId(42);
    let eq =
        ExternalTextureQuadInstance::new(Rect::new(0.0, 0.0, 800.0, 600.0), Color::WHITE, None);
    list.push_external_texture_quad(ext_id, eq);

    assert_eq!(list.external_texture_quads.len(), 1);
    assert_eq!(list.commands.len(), 1);
    assert_eq!(
        list.commands[0],
        DrawCommand::DrawExternalTexture {
            id: ext_id,
            instance_index: 0,
        }
    );

    list.clear();
    assert!(list.commands.is_empty());
    assert!(list.external_texture_quads.is_empty());
}

#[test]
fn test_command_list_preserves_capacity_on_clear() {
    let mut list = DrawCommandList::new();
    for i in 0..16 {
        list.push_quad(QuadInstance::from_style(
            Rect::new(i as f32, 0.0, 10.0, 10.0),
            &Style::default(),
            None,
        ));
    }
    assert_eq!(list.quads.len(), 16);
    assert_eq!(list.commands.len(), 1);

    list.clear();
    assert!(list.quads.is_empty());
    assert!(list.commands.is_empty());
    assert!(list.quads.capacity() >= 16);
}