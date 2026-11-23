// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Material and Surface Studio unit tests.

use super::*;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::widgets::declarative::{combine_seeds, hash_label, hash_label_with_seed};
use irisui::workbench::ActionBus;

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

#[inline]
fn material_root_tag(label: &str) -> u64 {
    hash_label_with_seed(combine_seeds(0, hash_label("MaterialPanelRoot")), label)
}

fn render_material_host(
    tree: &mut UiTree,
    panel_rect: Rect,
    entity: Option<ae_ecs::Entity>,
    world: &ae_ecs::World,
    (textures, models): (
        &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    ),
    bus: &mut ActionBus<EngineUiAction>,
    events: &[(u64, InteractionEvent)],
) -> WidgetId {
    let root = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .expect("root");
    let mut scope = UiScope::with_tagged_interactions(tree, root, events, None);
    scope.container(
        Style::new()
            .position_absolute()
            .left(panel_rect.x)
            .top(panel_rect.y)
            .width(panel_rect.width)
            .height(panel_rect.height)
            .flex_col(),
        |s| {
            crate::ui::panel_layout::PanelId::MaterialEditor
                .surface()
                .expect("Material surface")
                .show(s, |panel_scope| {
                    build_material_panel(panel_scope, entity, world, textures, models, bus);
                });
        },
    );
    scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
    root
}

#[test]
fn test_material_panel_build_empty_state() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let world = ae_ecs::World::new();
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(100.0, 100.0, 300.0, 400.0);

    let mut bus = ActionBus::new();
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("MaterialHost").bounds(panel_rect).show(|s| {
            crate::ui::panel_layout::PanelId::MaterialEditor
                .surface()
                .expect("Material surface")
                .show(s, |panel_scope| {
                    build_material_panel(panel_scope, None, &world, &textures, &models, &mut bus);
                });
        });
    }

    assert!(tree.find_node_by_name("MaterialPanelRoot").is_some());
    assert_eq!(tree.max_scroll(MATERIAL_TAG_VIEWPORT), 0.0);
}

#[test]
fn test_material_panel_build_no_geometry() {
    let mut tree = UiTree::new();
    let mut world = ae_ecs::World::new();
    let ent = world.spawn(("TestEntity",));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(0.0, 0.0, 300.0, 400.0);

    let mut bus = ActionBus::new();
    let root = render_material_host(
        &mut tree,
        panel_rect,
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus,
        &[],
    );

    assert!(find_tag_recursive(
        &tree,
        root,
        material_root_tag("Add Texture / Sprite")
    ));
}

#[test]
fn test_material_panel_build_sprite_view() {
    let mut tree = UiTree::new();
    let mut world = ae_ecs::World::new();
    let sprite_h = ae_renderer::asset::AssetHandle::default();
    let ent = world.spawn((
        ae_plugin_api::SpriteId(sprite_h),
        ae_plugin_api::Color::new(1.0, 0.5, 0.2, 1.0),
    ));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(0.0, 0.0, 320.0, 450.0);

    let mut bus = ActionBus::new();
    let root = render_material_host(
        &mut tree,
        panel_rect,
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus,
        &[],
    );

    assert!(find_tag_recursive(
        &tree,
        root,
        material_root_tag("Change Texture")
    ));
    assert!(find_tag_recursive(
        &tree,
        root,
        material_root_tag("🗑 Remove")
    ));
}

#[test]
fn test_material_panel_does_not_mutate_parent_rect() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let menubar = {
        let mut scope = UiScope::new(&mut tree, root);
        let _menubar_id = scope.container(
            Style::new()
                .position_absolute()
                .left(0.0)
                .top(0.0)
                .width(1920.0)
                .height(crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT),
            |sub| {
                sub.label("Top Menu");
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));
        tree.get(root).unwrap().children[0]
    };

    let world = ae_ecs::World::new();
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();

    let mut bus = ActionBus::new();
    build_material_panel(
        &mut UiScope::new(&mut tree, root),
        None,
        &world,
        &textures,
        &models,
        &mut bus,
    );

    let root_rect = tree.get(root).unwrap().computed_rect;
    let menubar_rect = tree.get(menubar).unwrap().computed_rect;
    assert_eq!(root_rect, Rect::new(0.0, 0.0, 1920.0, 1080.0));
    assert_eq!(
        menubar_rect,
        Rect::new(
            0.0,
            0.0,
            1920.0,
            crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT
        )
    );
}

#[test]
fn test_material_panel_scroll_offset_and_empty_centering() {
    let mut tree = UiTree::new();
    let mut world = ae_ecs::World::new();
    let sprite_h = ae_renderer::asset::AssetHandle::default();
    let ent = world.spawn((
        ae_plugin_api::SpriteId(sprite_h),
        ae_plugin_api::Color::new(1.0, 0.5, 0.2, 1.0),
    ));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(100.0, 50.0, 300.0, 180.0);
    let change_tag = material_root_tag("Change Texture");

    let mut bus_0 = ActionBus::new();
    let root = render_material_host(
        &mut tree,
        panel_rect,
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus_0,
        &[],
    );

    let change_btn_0 = find_node_by_tag(&tree, root, change_tag).expect("Button must exist");
    let y_0 = change_btn_0.y;

    let mut tree_scroll = UiTree::new();
    tree_scroll.set_scroll_offset(MATERIAL_TAG_VIEWPORT, 50.0);
    let mut bus_50 = ActionBus::new();
    let root_scroll = render_material_host(
        &mut tree_scroll,
        panel_rect,
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus_50,
        &[],
    );

    let change_btn_50 =
        find_node_by_tag(&tree_scroll, root_scroll, change_tag).expect("Button must exist");
    let y_50 = change_btn_50.y;
    assert_eq!(y_0 - y_50, 50.0);

    let mut tree_empty = UiTree::new();
    let empty_ent = world.spawn(("EmptyEntity",));
    let mut bus_empty = ActionBus::new();
    let root_empty = render_material_host(
        &mut tree_empty,
        panel_rect,
        Some(empty_ent),
        &world,
        (&textures, &models),
        &mut bus_empty,
        &[],
    );

    let add_btn = find_node_by_tag(
        &tree_empty,
        root_empty,
        material_root_tag("Add Texture / Sprite"),
    )
    .expect("Add texture button must exist in empty state");
    assert!(
        add_btn.y > panel_rect.y + 150.0,
        "Empty state button must be centered in panel body, got y={}",
        add_btn.y
    );
}

#[test]
fn test_material_panel_declarative_actions_dispatch() {
    let mut tree = UiTree::new();
    let mut world = ae_ecs::World::new();
    let sprite_h = ae_renderer::asset::AssetHandle::default();
    let ent = world.spawn((
        ae_plugin_api::SpriteId(sprite_h),
        ae_plugin_api::Color::new(1.0, 0.5, 0.2, 1.0),
    ));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();
    let panel_rect = Rect::new(0.0, 0.0, 320.0, 450.0);

    let events = [(
        material_root_tag("Change Texture"),
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];

    let mut bus = ActionBus::new();
    render_material_host(
        &mut tree,
        panel_rect,
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus,
        &events,
    );

    assert!(
        bus.drain()
            .any(|a| matches!(a, EngineUiAction::PickAndAssignEntityTexture(e) if e == ent)),
        "Clicking Change Texture button must emit EngineUiAction::PickAndAssignEntityTexture"
    );

    let events_remove = [(
        material_root_tag("🗑 Remove"),
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let mut bus_remove = ActionBus::new();
    render_material_host(
        &mut UiTree::new(),
        panel_rect,
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus_remove,
        &events_remove,
    );
    assert!(
        bus_remove
            .drain()
            .any(|a| matches!(a, EngineUiAction::RemoveTextureFromEntity(e) if e == ent)),
        "Clicking Remove button must emit EngineUiAction::RemoveTextureFromEntity"
    );
}

#[test]
fn test_material_add_color_tint_and_scope_seed_isolation() {
    let mut tree = UiTree::new();
    let mut world = ae_ecs::World::new();
    let sprite_h = ae_renderer::asset::AssetHandle::default();
    let ent = world.spawn((ae_plugin_api::SpriteId(sprite_h),));
    let textures = ae_renderer::asset::AssetStorage::new();
    let models = ae_renderer::asset::AssetStorage::new();

    let events = [(
        material_root_tag("Add Color Tint"),
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];
    let mut bus = ActionBus::new();
    render_material_host(
        &mut tree,
        Rect::new(0.0, 0.0, 320.0, 450.0),
        Some(ent),
        &world,
        (&textures, &models),
        &mut bus,
        &events,
    );
    assert!(
        bus.drain()
            .any(|a| matches!(a, EngineUiAction::AddComponent(e, "Color") if e == ent))
    );

    let slot0_tag = hash_label_with_seed(combine_seeds(0, 0usize.into_seed()), "💧 Blend");
    let slot1_tag = hash_label_with_seed(combine_seeds(0, 1usize.into_seed()), "💧 Blend");
    assert_ne!(slot0_tag, slot1_tag);
}