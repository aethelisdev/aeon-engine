// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit Tests for Hardware Batching, In-Place Mutation, and 2 Draw Call Guarantees.

use super::engine::bake_ui_geometry;
use super::geometry::BakedUiGeometry;
use irisui::prelude::*;
use irisui::wgpu_backend::DrawCommand;

#[test]
fn test_two_draw_calls_guarantee() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    // Construct 50 simulated asset cards (alternating SDF background + texture icon)
    for i in 0..50 {
        let card = tree.create_node();
        if let Some(node) = tree.get_mut(card) {
            node.computed_rect = Rect::new(i as f32 * 100.0, 0.0, 90.0, 110.0);
            node.style = Style::new()
                .background(Color::rgba(0.1, 0.1, 0.1, 1.0))
                .border(1.0, Color::rgba(0.2, 0.2, 0.2, 1.0));
        }
        let _ = tree.add_child(root, card);

        let icon = tree.create_node();
        if let Some(node) = tree.get_mut(icon) {
            node.computed_rect = Rect::new(i as f32 * 100.0 + 10.0, 10.0, 70.0, 70.0);
            node.set_texture_uv([0.0, 0.0, 1.0, 1.0]);
            node.set_texture_tint(Color::WHITE);
        }
        let _ = tree.add_child(card, icon);
    }

    let mut geometry = BakedUiGeometry::new();
    bake_ui_geometry(&mut geometry, &tree, root, None);

    assert_eq!(geometry.sdf_instances.len(), 50);
    assert_eq!(geometry.texture_instances.len(), 50);

    let mut command_list = DrawCommandList::new();
    geometry.apply_to_command_list(&mut command_list);

    // CRITICAL INVARIANT: Exactly 2 draw calls must be produced (1 SDF batch + 1 Texture batch)
    assert_eq!(command_list.commands.len(), 2);

    match command_list.commands[0] {
        DrawCommand::DrawSdfQuads { start, count } => {
            assert_eq!(start, 0);
            assert_eq!(count, 50);
        }
        _ => panic!("Expected first command to be DrawSdfQuads"),
    }

    match command_list.commands[1] {
        DrawCommand::DrawTexture { start, count } => {
            assert_eq!(start, 0);
            assert_eq!(count, 50);
        }
        _ => panic!("Expected second command to be DrawTexture"),
    }
}

#[test]
fn test_in_place_card_style_mutation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let card = tree.create_node();
    if let Some(node) = tree.get_mut(card) {
        node.computed_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        node.style = Style::new().background(Color::rgba(0.2, 0.2, 0.2, 1.0));
    }
    let _ = tree.add_child(root, card);

    let mut geometry = BakedUiGeometry::new();
    bake_ui_geometry(&mut geometry, &tree, root, None);

    let initial_color = geometry.sdf_instances[0].color;

    // Mutate style in-place (simulating card hover / accent highlight)
    let hovered_style = Style::new()
        .background(Color::rgba(0.0, 0.8, 1.0, 1.0))
        .border(2.0, Color::rgba(0.0, 0.9, 1.0, 1.0));
    geometry.update_card_style(0, &hovered_style);

    let updated_color = geometry.sdf_instances[0].color;
    assert_ne!(initial_color, updated_color);
    assert_eq!(geometry.sdf_instances.len(), 1);
}

#[test]
fn test_empty_tree_baking_safety() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let mut geometry = BakedUiGeometry::new();
    bake_ui_geometry(&mut geometry, &tree, root, None);

    let mut command_list = DrawCommandList::new();
    geometry.apply_to_command_list(&mut command_list);

    assert_eq!(geometry.sdf_instances.len(), 0);
    assert_eq!(geometry.texture_instances.len(), 0);
    assert_eq!(command_list.commands.len(), 0);
}

#[test]
fn test_zero_allocation_when_capacity_retained() {
    let mut geometry = BakedUiGeometry::new();
    let cap_sdf = geometry.sdf_instances.capacity();
    let cap_tex = geometry.texture_instances.capacity();

    geometry.clear_retaining_capacity();

    assert_eq!(geometry.sdf_instances.capacity(), cap_sdf);
    assert_eq!(geometry.texture_instances.capacity(), cap_tex);
}