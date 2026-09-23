// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Root builder orchestrating toolbar, compass, and camera HUD overlays.
//!

use super::camera_hud;
use super::compass;
use super::play_hud;
use super::popup;
use super::toolbar;
use super::types::ViewportHudParams;
use irisui::prelude::*;

/// Builds the complete Viewport HUD overlay hierarchy into the UI Tree.
///
/// Dispatches declarative HUD components: top-left floating toolbar, top-right
/// 3D navigation compass, bottom-right camera info HUD, and optional active dropdown popups.
/// Subtree layout and hover states are resolved declaratively across the viewport bounds.
pub fn build_viewport_hud(tree: &mut UiTree, parent_id: WidgetId, params: &ViewportHudParams<'_>) {
    if params.viewport_rect.width < 20.0 || params.viewport_rect.height < 20.0 {
        return;
    }

    if params.is_editing {
        // 1. Top-left floating toolbar
        toolbar::build_viewport_toolbar(tree, parent_id, params);

        // In 2D dimension mode, 3D compass and 3D camera Euler angle telemetry are hidden
        if !params.is_2d {
            // 2. Top-right 3D Scene Navigation Compass (visible across all 3D projection modes)
            compass::build_scene_navigation_compass(tree, parent_id, params);

            // 3. Bottom-right Camera Info HUD (positioned via Style::position_absolute, right(8.0), bottom(8.0))
            camera_hud::build_camera_hud(tree, parent_id, params);
        }

        // 4. Resolve complete HUD layout and hover reconciliation across viewport bounds
        let mut scope = UiScope::new(tree, parent_id);
        scope.finish_layout_with_hover(params.viewport_rect, params.cursor_pos);

        // 5. Active dropdown popup if open
        if let Some(active_dd) = params.active_dropdown {
            popup::render_viewport_hud_dropdown_popup(tree, parent_id, active_dd, params);
        }
    } else {
        // Play Mode HUD & In-Game Pause Menu Overlay
        play_hud::build_play_hud(tree, parent_id, params);

        let mut scope = UiScope::new(tree, parent_id);
        scope.finish_layout_with_hover(params.viewport_rect, params.cursor_pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::viewport_hud::{
        TAG_COMPASS_CANVAS, TAG_GIZMO_TRANSLATE, TAG_PLAY_EXIT, TAG_PLAY_RESUME,
        TAG_VIEWPORT_CAMERA_MODE, TAG_VIEWPORT_SHADING_MODE,
    };
    use ae_editor::gizmo::{GizmoMode, GizmoSpace};
    use ae_editor::snapping::SnapSettings;
    use ae_renderer::camera::{Camera, ProjectionMode};
    use hecs::World;

    fn make_test_camera() -> Camera {
        Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_build_viewport_hud_editing_3d() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        build_viewport_hud(&mut tree, root, &params);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        // Verifies toolbar is present
        assert!(tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        // Verifies 3D compass is present
        assert!(tags.contains(&TAG_COMPASS_CANVAS));
    }

    #[test]
    fn test_build_viewport_hud_editing_2d() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: true,
        };

        build_viewport_hud(&mut tree, root, &params);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        // In 2D mode, 3D camera and shading dropdowns are suppressed, gizmo tools remain
        assert!(!tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(!tags.contains(&TAG_VIEWPORT_SHADING_MODE));
        assert!(tags.contains(&TAG_GIZMO_TRANSLATE));
        // 3D compass must be hidden in 2D mode
        assert!(!tags.contains(&TAG_COMPASS_CANVAS));
    }

    #[test]
    fn test_build_viewport_hud_play_mode() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let mut world = World::new();
        let _player = world.spawn((
            ae_core::ecs::PlayerTag,
            ae_core::ecs::CharacterAction::default(),
        ));
        let _pause = world.spawn((ae_core::ui::PauseMenuUiTag,));
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            active_dropdown: None,
            selected_entity: None,
            world: &world,
            is_editing: false,
            is_2d: false,
        };

        build_viewport_hud(&mut tree, root, &params);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        // In play mode, edit toolbar and compass are hidden, pause menu items are shown
        assert!(!tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(!tags.contains(&TAG_COMPASS_CANVAS));
        assert!(tags.contains(&TAG_PLAY_EXIT));
        assert!(tags.contains(&TAG_PLAY_RESUME));
    }
}