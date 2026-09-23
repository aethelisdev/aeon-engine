// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Material and Surface Studio unit tests.
//!

use super::*;
use ae_renderer::render::types::SubmeshAlphaMode;
use irisui::prelude::*;

fn find_tag_recursive(tree: &UiTree, node_id: WidgetId, target_tag: u64) -> bool {
    find_node_by_tag(tree, node_id, target_tag).is_some()
}

fn find_node_by_tag(tree: &UiTree, node_id: WidgetId, target_tag: u64) -> Option<Rect> {
    if let Some(node) = tree.get(node_id) {
        if node.tag == target_tag {
            return Some(node.computed_rect);
        }
        for &child in &node.children {
            if let Some(r) = find_node_by_tag(tree, child, target_tag) {
                return Some(r);
            }
        }
    }
    None
}

#[test]
fn test_material_panel_build_empty_state() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let world = hecs::World::new();
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(100.0, 100.0, 300.0, 400.0);

    let params = MaterialPanelParams {
        panel_rect,
        entity: None,
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        hovered_tag: None,
        events: &[],
    };

    let mut targets = MaterialPanelTargets::default();
    build_material_panel(&mut tree, root, &params, &mut targets);

    assert_eq!(targets.panel_rect, panel_rect);
    assert!(targets.active_model.is_none());
    assert!(targets.content_height > 0.0);
}

#[test]
fn test_material_panel_build_no_geometry() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut world = hecs::World::new();
    let ent = world.spawn(("TestEntity",));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(0.0, 0.0, 300.0, 400.0);

    let params = MaterialPanelParams {
        panel_rect,
        entity: Some(ent),
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        hovered_tag: None,
        events: &[],
    };

    let mut targets = MaterialPanelTargets::default();
    build_material_panel(&mut tree, root, &params, &mut targets);

    assert!(find_tag_recursive(&tree, root, MATERIAL_TAG_ADD_TEXTURE));
}

#[test]
fn test_material_panel_build_sprite_view() {
    let mut tree = UiTree::new();
    let root = PanelBuilder::new(&mut tree).build();
    let mut world = hecs::World::new();
    let sprite_h = ae_renderer::asset::AssetHandle::default();
    let ent = world.spawn((
        ae_core::ecs::SpriteId(sprite_h),
        ae_core::ecs::Color::new(1.0, 0.5, 0.2, 1.0),
    ));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(0.0, 0.0, 320.0, 450.0);

    let params = MaterialPanelParams {
        panel_rect,
        entity: Some(ent),
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        hovered_tag: None,
        events: &[],
    };

    let mut targets = MaterialPanelTargets::default();
    build_material_panel(&mut tree, root, &params, &mut targets);

    assert!(find_tag_recursive(&tree, root, MATERIAL_TAG_SPRITE_CHANGE));
    assert!(find_tag_recursive(&tree, root, MATERIAL_TAG_SPRITE_REMOVE));
}

#[test]
fn test_material_panel_click_hit_testing() {
    let mut world = hecs::World::new();
    let ent = world.spawn(("Entity1",));
    let handle = ae_renderer::asset::AssetHandle::default();

    // Click Change Texture
    let act1 = handle_material_click(MATERIAL_TAG_SPRITE_CHANGE, Some(ent), None);
    assert_eq!(act1, Some(MaterialAction::PickAndAssignEntityTexture(ent)));

    // Click Remove Texture
    let act2 = handle_material_click(MATERIAL_TAG_SPRITE_REMOVE, Some(ent), None);
    assert_eq!(act2, Some(MaterialAction::RemoveTextureFromEntity(ent)));

    // Click Add Texture
    let act3 = handle_material_click(MATERIAL_TAG_ADD_TEXTURE, Some(ent), None);
    assert_eq!(act3, Some(MaterialAction::PickAndAssignEntityTexture(ent)));

    // Click Add Color
    let act4 = handle_material_click(MATERIAL_TAG_ADD_COLOR, Some(ent), None);
    assert_eq!(act4, Some(MaterialAction::AddColorComponent(ent)));

    // Click Submesh Alpha Button
    let alpha_tag = make_submesh_alpha_tag(1, SubmeshAlphaMode::Mask);
    let act5 = handle_material_click(alpha_tag, Some(ent), Some(handle));
    assert_eq!(
        act5,
        Some(MaterialAction::SetModelSubmeshAlphaMode(
            handle,
            1,
            SubmeshAlphaMode::Mask
        ))
    );

    // Click Submesh Texture Button
    let tex_tag = make_submesh_texture_tag(1);
    let act6 = handle_material_click(tex_tag, Some(ent), Some(handle));
    assert_eq!(
        act6,
        Some(MaterialAction::PickAndSetSubmeshTexture(handle, 1))
    );

    // Click with unknown / 0 tag
    let act7 = handle_material_click(0, Some(ent), Some(handle));
    assert_eq!(act7, None);

    // Scroll test
    let targets = MaterialPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 400.0, 500.0),
        active_model: Some(handle),
        content_height: 800.0,
    };
    let new_scroll = handle_material_scroll(1.0, 50.0, &targets);
    assert_eq!(new_scroll, 26.0);
}

#[test]
fn test_material_panel_does_not_mutate_parent_rect() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }
    let menubar = tree.create_node();
    if let Some(node) = tree.get_mut(menubar) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 28.0);
    }
    tree.add_child(root, menubar).unwrap();

    let world = hecs::World::new();
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(1400.0, 28.0, 520.0, 600.0);

    let params = MaterialPanelParams {
        panel_rect,
        entity: None,
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        hovered_tag: None,
        events: &[],
    };

    let mut targets = MaterialPanelTargets::default();
    build_material_panel(&mut tree, root, &params, &mut targets);

    let root_rect = tree.get(root).unwrap().computed_rect;
    let menubar_rect = tree.get(menubar).unwrap().computed_rect;
    assert_eq!(root_rect, Rect::new(0.0, 0.0, 1920.0, 1080.0));
    assert_eq!(menubar_rect, Rect::new(0.0, 0.0, 1920.0, 28.0));
}

#[test]
fn test_material_panel_scroll_offset_and_empty_centering() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let mut world = hecs::World::new();
    let sprite_h = ae_renderer::asset::AssetHandle::default();
    let ent = world.spawn((
        ae_core::ecs::SpriteId(sprite_h),
        ae_core::ecs::Color::new(1.0, 0.5, 0.2, 1.0),
    ));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(100.0, 50.0, 300.0, 600.0);

    // 1. Build with scroll_y = 0.0
    let params_0 = MaterialPanelParams {
        panel_rect,
        entity: Some(ent),
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        hovered_tag: None,
        events: &[],
    };
    let mut targets_0 = MaterialPanelTargets::default();
    build_material_panel(&mut tree, root, &params_0, &mut targets_0);

    // Find the sprite change button's Y position at scroll 0
    let change_btn_0 =
        find_node_by_tag(&tree, root, MATERIAL_TAG_SPRITE_CHANGE).expect("Button must exist");
    let y_0 = change_btn_0.y;

    // 2. Build with scroll_y = 50.0
    let mut tree_scroll = UiTree::new();
    let root_scroll = tree_scroll.create_root().unwrap();
    let params_50 = MaterialPanelParams {
        panel_rect,
        entity: Some(ent),
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 50.0,
        hovered_tag: None,
        events: &[],
    };
    let mut targets_50 = MaterialPanelTargets::default();
    build_material_panel(&mut tree_scroll, root_scroll, &params_50, &mut targets_50);

    let change_btn_50 = find_node_by_tag(&tree_scroll, root_scroll, MATERIAL_TAG_SPRITE_CHANGE)
        .expect("Button must exist");
    let y_50 = change_btn_50.y;

    // The button must have moved up by exactly 50.0 pixels!
    assert_eq!(y_0 - y_50, 50.0);

    // 3. Verify empty state vertical centering (must not be at top 36px)
    let mut tree_empty = UiTree::new();
    let root_empty = tree_empty.create_root().unwrap();
    let empty_ent = world.spawn(("EmptyEntity",));
    let params_empty = MaterialPanelParams {
        panel_rect,
        entity: Some(empty_ent),
        world: &world,
        textures: &textures,
        models: &models,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        hovered_tag: None,
        events: &[],
    };
    let mut targets_empty = MaterialPanelTargets::default();
    build_material_panel(
        &mut tree_empty,
        root_empty,
        &params_empty,
        &mut targets_empty,
    );

    let add_btn = find_node_by_tag(&tree_empty, root_empty, MATERIAL_TAG_ADD_TEXTURE)
        .expect("Add texture button must exist in empty state");
    // Button must be vertically centered in the 600px panel, well past y = 150.0
    assert!(
        add_btn.y > panel_rect.y + 150.0,
        "Empty state button must be centered in panel body, got y={}",
        add_btn.y
    );
}