// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Subsystem
//!
//! Exposes 100% native Iris UI GPU SDF panel rendering for PBR texture inspection,
//! submesh alpha blending modes, 2D sprite surface settings, and hardware texture array icons.
//!

pub mod empty_state;
pub mod events;
pub mod header;
pub mod panel;
pub mod sprite_view;
pub mod submesh_view;
pub mod types;

pub use events::{handle_material_click, handle_material_scroll};
pub use header::{MATERIAL_HEADER_HEIGHT, build_material_header};
pub use panel::build_material_panel;
pub use types::{MaterialAction, MaterialPanelParams, MaterialPanelTargets};

#[cfg(test)]
mod tests {
    use super::*;
    use ae_renderer::render::types::SubmeshAlphaMode;
    use irisui::prelude::*;

    #[test]
    fn test_material_panel_build_empty_state() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
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
        };

        let mut targets = MaterialPanelTargets::default();
        build_material_panel(&mut tree, root, &params, &mut targets);

        assert_eq!(targets.panel_rect, panel_rect);
        assert!(targets.btn_change_texture.is_none());
        assert!(targets.btn_remove_texture.is_none());
        assert!(targets.content_height > 0.0);
    }

    #[test]
    fn test_material_panel_build_no_geometry() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
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
        };

        let mut targets = MaterialPanelTargets::default();
        build_material_panel(&mut tree, root, &params, &mut targets);

        assert!(targets.btn_add_texture.is_some());
    }

    #[test]
    fn test_material_panel_build_sprite_view() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
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
        };

        let mut targets = MaterialPanelTargets::default();
        build_material_panel(&mut tree, root, &params, &mut targets);

        assert!(targets.btn_change_texture.is_some());
        assert!(targets.btn_remove_texture.is_some());
    }

    #[test]
    fn test_material_panel_click_hit_testing() {
        let mut world = hecs::World::new();
        let ent = world.spawn(("Entity1",));
        let handle = ae_renderer::asset::AssetHandle::default();

        let mut targets = MaterialPanelTargets {
            panel_rect: Rect::new(0.0, 0.0, 400.0, 500.0),
            btn_change_texture: Some(Rect::new(10.0, 10.0, 60.0, 24.0)),
            btn_remove_texture: Some(Rect::new(80.0, 10.0, 40.0, 24.0)),
            btn_add_texture: Some(Rect::new(10.0, 50.0, 80.0, 24.0)),
            btn_add_color: Some(Rect::new(10.0, 80.0, 80.0, 24.0)),
            submesh_alpha_buttons: vec![(
                handle,
                1,
                SubmeshAlphaMode::Mask,
                Rect::new(10.0, 120.0, 50.0, 20.0),
            )],
            submesh_texture_buttons: vec![(handle, 1, Rect::new(70.0, 120.0, 50.0, 20.0))],
            content_height: 300.0,
        };

        // Click Change Texture
        let act1 = handle_material_click(Point::new(20.0, 20.0), Some(ent), &targets);
        assert_eq!(act1, Some(MaterialAction::PickAndAssignEntityTexture(ent)));

        // Click Remove Texture
        let act2 = handle_material_click(Point::new(90.0, 20.0), Some(ent), &targets);
        assert_eq!(act2, Some(MaterialAction::RemoveTextureFromEntity(ent)));

        // Click Submesh Alpha Button
        let act3 = handle_material_click(Point::new(20.0, 130.0), Some(ent), &targets);
        assert_eq!(
            act3,
            Some(MaterialAction::SetModelSubmeshAlphaMode(
                handle,
                1,
                SubmeshAlphaMode::Mask
            ))
        );

        // Click Submesh Texture Button
        let act4 = handle_material_click(Point::new(80.0, 130.0), Some(ent), &targets);
        assert_eq!(
            act4,
            Some(MaterialAction::PickAndSetSubmeshTexture(handle, 1))
        );

        // Click Outside
        let act5 = handle_material_click(Point::new(300.0, 300.0), Some(ent), &targets);
        assert_eq!(act5, None);

        // Scroll test
        targets.content_height = 800.0;
        let new_scroll = handle_material_scroll(1.0, 50.0, &targets);
        assert_eq!(new_scroll, 26.0);
    }
}