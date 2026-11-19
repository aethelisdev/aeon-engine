// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Root Orchestrator
//!
//! Dispatches Viewport HUD toolbar, compass, camera HUD, and Play Mode overlays
//! purely through [`UiScope`] with zero manual `finish_layout` calls and zero imperative `let`s.

use super::camera_hud;
use super::compass;
use super::play_hud;
use super::toolbar;
use super::types::ViewportHudParams;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Builds viewport HUD overlays purely through a declarative [`UiScope`].
///
/// Layout solving is owned strictly by `Workbench`, eliminating redundant per-panel `finish_layout` passes.
pub fn build_viewport_hud_scope(
    scope: &mut UiScope<'_>,
    params: &ViewportHudParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    if params.viewport_rect.width < 20.0 || params.viewport_rect.height < 20.0 {
        return;
    }

    if params.is_editing {
        toolbar::build_viewport_toolbar(scope, params, action_bus);

        scope.when(!params.is_2d, |s| {
            compass::build_scene_navigation_compass(s, params);
            camera_hud::build_camera_hud(s, params);
        });
    } else {
        play_hud::build_play_hud(scope, params, action_bus);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::viewport_hud::{
        TAG_CAMERA_HUD, TAG_COMPASS_CANVAS, TAG_GIZMO_TRANSLATE, TAG_PLAY_EXIT, TAG_PLAY_RESUME,
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
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        let mut bus = ActionBus::new();
        build_viewport_hud_scope(&mut UiScope::new(&mut tree, root), &params, &mut bus);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(tags.contains(&TAG_COMPASS_CANVAS));
        assert!(tags.contains(&TAG_CAMERA_HUD));
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
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: true,
        };

        let mut bus = ActionBus::new();
        build_viewport_hud_scope(&mut UiScope::new(&mut tree, root), &params, &mut bus);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(!tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(!tags.contains(&TAG_VIEWPORT_SHADING_MODE));
        assert!(tags.contains(&TAG_GIZMO_TRANSLATE));
        assert!(!tags.contains(&TAG_COMPASS_CANVAS));
        assert!(!tags.contains(&TAG_CAMERA_HUD));
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
            selected_entity: None,
            world: &world,
            is_editing: false,
            is_2d: false,
        };

        let mut bus = ActionBus::new();
        build_viewport_hud_scope(&mut UiScope::new(&mut tree, root), &params, &mut bus);

        let tags: Vec<u64> = tree.iter().map(|(_, n)| n.tag).collect();
        assert!(!tags.contains(&TAG_VIEWPORT_CAMERA_MODE));
        assert!(!tags.contains(&TAG_COMPASS_CANVAS));
        assert!(tags.contains(&TAG_PLAY_EXIT));
        assert!(tags.contains(&TAG_PLAY_RESUME));
    }

    #[test]
    fn test_build_viewport_hud_registers_active_dropdown_tag_and_outside_click() {
        let camera = make_test_camera();
        let snapping = SnapSettings::default();
        let world = World::new();
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root creation failed");

        let params = ViewportHudParams {
            viewport_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            camera: &camera,
            wireframe_enabled: false,
            gizmo_mode: GizmoMode::Select,
            gizmo_space: GizmoSpace::World,
            snapping: &snapping,
            cursor_pos: Point::new(100.0, 100.0),
            selected_entity: None,
            world: &world,
            is_editing: true,
            is_2d: false,
        };

        tree.open_popup(TAG_VIEWPORT_CAMERA_MODE);
        let mut bus = ActionBus::new();
        {
            let mut scope = UiScope::new(&mut tree, root);
            build_viewport_hud_scope(&mut scope, &params, &mut bus);
            scope.finish_layout(params.viewport_rect);
        }

        assert!(tree.is_popup_tag_active(TAG_VIEWPORT_CAMERA_MODE));
        assert!(tree.has_open_popup());
        assert!(
            tree.iter()
                .any(|(_, n)| n.layer == UiLayer::Popup && n.role == WidgetRole::DropdownPopup)
        );

        tree.dispatch_event(&UiEvent::MouseDown {
            point: Point::new(700.0, 500.0),
            button: MouseButton::Left,
        });
        assert!(!tree.has_open_popup());
        assert!(!tree.is_popup_tag_active(TAG_VIEWPORT_CAMERA_MODE));

        tree.clear();
        let root2 = tree.create_root().expect("Root2");
        {
            let mut scope = UiScope::new(&mut tree, root2);
            build_viewport_hud_scope(&mut scope, &params, &mut bus);
        }
        assert!(
            !tree
                .iter()
                .any(|(_, n)| n.layer == UiLayer::Popup && n.role == WidgetRole::DropdownPopup),
            "Closed popup should not render after outside click"
        );
    }
}