// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Floating Toolbar Builder
//!
//! Renders the top-left floating glass toolbar containing camera projection modes,
//! shading mode selector, Q/W/E/R gizmo operation buttons, and snapping controls using
//! declarative `UiScope::anchored_select_pill` and direct [`ActionBus<EngineUiAction>`] emission.

use super::popup::{
    CAMERA_MODE_OPTIONS, GRID_SNAP_OPTIONS, SHADING_MODE_OPTIONS, camera_mode_selected_index,
    grid_snap_selected_index, resolve_camera_mode_action, resolve_grid_snap_action,
    resolve_shading_mode_action, shading_mode_selected_index,
};
use super::types::{
    CameraViewOrientation, ShadingViewMode, TAG_GIZMO_ROTATE, TAG_GIZMO_SCALE, TAG_GIZMO_SELECT,
    TAG_GIZMO_SPACE_TOGGLE, TAG_GIZMO_TRANSLATE, TAG_SNAP_GRID_DROPDOWN, TAG_SNAP_GRID_TOGGLE,
    TAG_VIEWPORT_SHADING_MODE, toolbar_total_width, view_modes_group_width,
};
use crate::ui::iris_bridge::icons::{
    ICON_LOCAL, ICON_ROTATE, ICON_SCALE, ICON_SELECT, ICON_SNAP_GRID, ICON_TRANSLATE, ICON_WORLD,
};
use crate::ui::types::EngineUiAction;
use ae_editor::editor_state::EditorState;
use ae_editor::snapping::{SnapMode, SnapSettings};
use ae_gizmos::{GizmoMode, GizmoSpace};
use ae_renderer::camera::Camera;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

#[inline]
fn emit_opt_action(action_bus: &mut ActionBus<EngineUiAction>, action: Option<EngineUiAction>) {
    action.into_iter().for_each(|act| action_bus.emit(act));
}

#[inline]
fn toggled_snap_settings(snapping: &SnapSettings) -> SnapSettings {
    SnapSettings {
        mode: match snapping.mode {
            SnapMode::Off => SnapMode::Toggle,
            _ => SnapMode::Off,
        },
        current_enabled: snapping.mode == SnapMode::Off,
        ..*snapping
    }
}

#[inline]
fn build_view_modes_group(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    camera: &Camera,
    wireframe_enabled: bool,
    is_2d: bool,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    tb.group_styled(
        ToolbarGroup::new()
            .width(view_modes_group_width(camera, wireframe_enabled, is_2d))
            .gap(0.0)
            .background(Color::rgba(0.08, 0.09, 0.12, 0.92))
            .border_color(Color::rgba(0.24, 0.26, 0.32, 0.85))
            .corner_radius(4.0),
        |group| {
            if is_2d {
                group.mode_button_tagged(
                    CameraViewOrientation::from_camera(camera, true).icon_uv(),
                    CameraViewOrientation::from_camera(camera, true).label(),
                    CameraViewOrientation::from_camera(camera, true).tag(),
                    false,
                    CameraViewOrientation::from_camera(camera, true).width(),
                    CameraViewOrientation::from_camera(camera, true).corner_radii(),
                );
                return;
            }

            group.anchored_select_pill(
                AnchoredSelectPillSpec {
                    popup_name: "CameraModeDropdown",
                    tag: CameraViewOrientation::from_camera(camera, false).tag(),
                    label: CameraViewOrientation::from_camera(camera, false).label(),
                    trigger_style: SelectPillTriggerStyle::Mode {
                        icon_uv: CameraViewOrientation::from_camera(camera, false).icon_uv(),
                        width: CameraViewOrientation::from_camera(camera, false).width(),
                        corner_radii: CameraViewOrientation::from_camera(camera, false)
                            .corner_radii(),
                    },
                    popup_width: 150.0,
                    placement: PopupPlacement::Below,
                    options: CAMERA_MODE_OPTIONS,
                    selected_index: camera_mode_selected_index(camera, false),
                },
                |idx| emit_opt_action(action_bus, resolve_camera_mode_action(idx, camera)),
            );

            group.divider();

            group.anchored_select_pill(
                AnchoredSelectPillSpec {
                    popup_name: "ShadingModeDropdown",
                    tag: TAG_VIEWPORT_SHADING_MODE,
                    label: ShadingViewMode::from_wireframe(wireframe_enabled).label(),
                    trigger_style: SelectPillTriggerStyle::Mode {
                        icon_uv: Some(ShadingViewMode::from_wireframe(wireframe_enabled).icon_uv()),
                        width: ShadingViewMode::from_wireframe(wireframe_enabled).width(),
                        corner_radii: ShadingViewMode::from_wireframe(wireframe_enabled)
                            .corner_radii(),
                    },
                    popup_width: 124.0,
                    placement: PopupPlacement::Below,
                    options: SHADING_MODE_OPTIONS,
                    selected_index: shading_mode_selected_index(wireframe_enabled),
                },
                |idx| emit_opt_action(action_bus, resolve_shading_mode_action(idx)),
            );
        },
    );
}

#[inline]
fn build_gizmo_group(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    gizmo_mode: GizmoMode,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    tb.group_styled(
        ToolbarGroup::new().width(140.0).gap(4.0).transparent(),
        |gg| {
            [
                (GizmoMode::Select, ICON_SELECT, TAG_GIZMO_SELECT),
                (GizmoMode::Translate, ICON_TRANSLATE, TAG_GIZMO_TRANSLATE),
                (GizmoMode::Rotate, ICON_ROTATE, TAG_GIZMO_ROTATE),
                (GizmoMode::Scale, ICON_SCALE, TAG_GIZMO_SCALE),
            ]
            .into_iter()
            .for_each(|(mode, uv, tag)| {
                gg.selectable_tool_tagged(gizmo_mode == mode, uv, tag)
                    .on_click(|| action_bus.emit(EngineUiAction::SetGizmoMode(mode)));
            });
        },
    );
}

#[inline]
fn build_snapping_group(
    tb: &mut irisui::widgets::declarative::ToolbarScope<'_, '_>,
    snapping: &SnapSettings,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    tb.group_styled(
        ToolbarGroup::new().width(84.0).gap(2.0).transparent(),
        |gg| {
            gg.toggle_tool_tagged(
                snapping.mode != SnapMode::Off,
                ICON_SNAP_GRID,
                TAG_SNAP_GRID_TOGGLE,
            )
            .on_click(|| {
                action_bus.emit(EngineUiAction::UpdateSnapSettings(toggled_snap_settings(
                    snapping,
                )));
            });

            gg.anchored_select_pill(
                AnchoredSelectPillSpec {
                    popup_name: "GridSnapDropdown",
                    tag: TAG_SNAP_GRID_DROPDOWN,
                    label: snap_size_label(snapping.grid_size),
                    trigger_style: SelectPillTriggerStyle::Dropdown { width: 50.0 },
                    popup_width: 110.0,
                    placement: PopupPlacement::Below,
                    options: GRID_SNAP_OPTIONS,
                    selected_index: grid_snap_selected_index(snapping),
                },
                |idx| emit_opt_action(action_bus, resolve_grid_snap_action(idx, snapping)),
            );
        },
    );
}

/// Builds the top-left floating viewport toolbar with projection, shading, gizmo mode, coordinate space, and snapping controls.
///
/// Zero imperative `let` statements; emits [`EngineUiAction`] directly into [`ActionBus`].
pub fn build_viewport_toolbar(
    scope: &mut UiScope<'_>,
    camera: &Camera,
    editor_state: &EditorState,
    snapping: &SnapSettings,
    is_2d: bool,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    Toolbar::floating("ViewportToolbar", 8.0, 6.0)
        .width(toolbar_total_width(
            camera,
            editor_state.wireframe_enabled,
            is_2d,
        ))
        .transparent()
        .gap(8.0)
        .show(scope, |tb| {
            build_view_modes_group(
                tb,
                camera,
                editor_state.wireframe_enabled,
                is_2d,
                action_bus,
            );
            build_gizmo_group(tb, editor_state.gizmo_mode, action_bus);
            tb.toggle_tool_tagged(
                editor_state.gizmo_space == GizmoSpace::Local,
                if editor_state.gizmo_space == GizmoSpace::Local {
                    ICON_LOCAL
                } else {
                    ICON_WORLD
                },
                TAG_GIZMO_SPACE_TOGGLE,
            )
            .on_click(|| action_bus.emit(EngineUiAction::ToggleGizmoSpace));
            build_snapping_group(tb, snapping, action_bus);
        });
}

/// Formats a snapping grid size in world units into a compact UI label string.
#[inline]
#[must_use]
pub fn snap_size_label(size: f32) -> &'static str {
    if (size - 0.05).abs() < 0.001 {
        "0.05m"
    } else if (size - 0.1).abs() < 0.001 {
        "0.1m"
    } else if (size - 0.25).abs() < 0.001 {
        "0.25m"
    } else if (size - 0.5).abs() < 0.001 {
        "0.5m"
    } else if (size - 1.0).abs() < 0.001 {
        "1.0m"
    } else if (size - 2.0).abs() < 0.001 {
        "2.0m"
    } else if (size - 5.0).abs() < 0.001 {
        "5.0m"
    } else if (size - 10.0).abs() < 0.001 {
        "10.0m"
    } else {
        "1.0m"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::viewport_hud::TAG_VIEWPORT_CAMERA_MODE;
    use ae_editor::snapping::SnapSettings;
    use ae_gizmos::GizmoMode;
    use ae_renderer::camera::{Camera, ProjectionMode};

    fn make_camera(mode: ProjectionMode) -> Camera {
        Camera {
            position: cgmath::Point3::new(0.0, 5.0, 10.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_toolbar_camera_mode_dropdown_trigger_registered_in_orthographic_mode() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_root().expect("Root node creation failed");
        let camera = make_camera(ProjectionMode::Orthographic);
        let snapping = SnapSettings::default();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };
        let viewport_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);

        let mut bus = ActionBus::new();
        build_viewport_toolbar(
            &mut UiScope::new(&mut tree, parent_id),
            &camera,
            &editor_state,
            &snapping,
            false,
            &mut bus,
        );
        UiScope::new(&mut tree, parent_id).finish_layout(viewport_rect);

        let cam_node = tree
            .iter()
            .find(|(_, node)| node.tag == TAG_VIEWPORT_CAMERA_MODE)
            .expect("CameraMode dropdown node must be present in Orthographic mode");
        assert!(cam_node.1.computed_rect.width > 0.0 && cam_node.1.computed_rect.height > 0.0);
    }

    #[test]
    fn test_toolbar_camera_mode_dropdown_trigger_registered_in_perspective_mode() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_root().expect("Root node creation failed");
        let camera = make_camera(ProjectionMode::Perspective);
        let snapping = SnapSettings::default();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };
        let viewport_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);

        let mut bus = ActionBus::new();
        build_viewport_toolbar(
            &mut UiScope::new(&mut tree, parent_id),
            &camera,
            &editor_state,
            &snapping,
            false,
            &mut bus,
        );
        UiScope::new(&mut tree, parent_id).finish_layout(viewport_rect);

        let cam_node = tree
            .iter()
            .find(|(_, node)| node.tag == TAG_VIEWPORT_CAMERA_MODE)
            .expect("CameraMode dropdown node must be present in Perspective mode");
        assert!(cam_node.1.computed_rect.width > 0.0 && cam_node.1.computed_rect.height > 0.0);
    }

    #[test]
    fn test_toolbar_mode_2d_suppresses_camera_mode_dropdown() {
        let mut tree = UiTree::new();
        let parent_id = tree.create_root().expect("Root node creation failed");
        let camera = make_camera(ProjectionMode::Orthographic);
        let snapping = SnapSettings::default();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };

        let mut bus = ActionBus::new();
        build_viewport_toolbar(
            &mut UiScope::new(&mut tree, parent_id),
            &camera,
            &editor_state,
            &snapping,
            true,
            &mut bus,
        );

        assert!(
            !tree.iter().any(|(_, n)| n.tag == TAG_VIEWPORT_CAMERA_MODE),
            "CameraMode dropdown trigger must be suppressed in 2D mode"
        );
        assert!(
            !tree.iter().any(|(_, n)| n.tag == TAG_VIEWPORT_SHADING_MODE),
            "ShadingMode dropdown trigger must be suppressed in 2D mode"
        );
    }

    #[test]
    fn test_toolbar_button_click_dispatches_actions_to_action_bus() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");
        let camera = make_camera(ProjectionMode::Perspective);
        let snapping = SnapSettings::default();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };
        let viewport_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);

        let mut bus = ActionBus::new();
        build_viewport_toolbar(
            &mut UiScope::new(&mut tree, root),
            &camera,
            &editor_state,
            &snapping,
            false,
            &mut bus,
        );
        UiScope::new(&mut tree, root).finish_layout(viewport_rect);

        let (translate_id, translate_rect) = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_GIZMO_TRANSLATE)
            .map(|(id, n)| (id, n.computed_rect))
            .expect("Translate gizmo button node must exist");

        let click_pt = Point::new(translate_rect.x + 4.0, translate_rect.y + 4.0);
        tree.dispatch_event(&UiEvent::MouseDown {
            point: click_pt,
            button: MouseButton::Left,
        });
        tree.dispatch_event(&UiEvent::MouseUp {
            point: click_pt,
            button: MouseButton::Left,
        });

        assert_eq!(tree.event_state.frame_clicked_id, Some(translate_id));

        let mut rebuild_tree = UiTree::new();
        let rebuild_root = rebuild_tree.create_root().expect("Root creation failed");
        rebuild_tree.event_state = tree.event_state;

        build_viewport_toolbar(
            &mut UiScope::new(&mut rebuild_tree, rebuild_root),
            &camera,
            &editor_state,
            &snapping,
            false,
            &mut bus,
        );

        assert!(
            bus.drain()
                .any(|a| matches!(a, EngineUiAction::SetGizmoMode(GizmoMode::Translate))),
            "Clicking Translate button must emit EngineUiAction::SetGizmoMode(Translate) into ActionBus"
        );
    }

    #[test]
    fn test_toolbar_snapping_controls_registered_in_ui_tree() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation failed");
        let camera = make_camera(ProjectionMode::Perspective);
        let snapping = SnapSettings::default();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Select,
            ..Default::default()
        };
        let viewport_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);

        let mut bus = ActionBus::new();
        build_viewport_toolbar(
            &mut UiScope::new(&mut tree, root),
            &camera,
            &editor_state,
            &snapping,
            false,
            &mut bus,
        );
        UiScope::new(&mut tree, root).finish_layout(viewport_rect);

        assert!(tree.iter().any(|(_, n)| n.tag == TAG_SNAP_GRID_TOGGLE));
        assert!(tree.iter().any(|(_, n)| n.tag == TAG_SNAP_GRID_DROPDOWN));
    }

    #[test]
    fn test_active_gizmo_button_preserves_theme_active_background_after_layout() {
        let mut tree = UiTree::new();
        let custom_active = Color::rgba(0.0, 0.55, 0.85, 0.95);
        tree.theme = Theme {
            button_active_bg: custom_active,
            ..Theme::dark_default()
        };
        let root = tree.create_root().expect("Root node creation failed");
        tree.set_cursor_position(Point::new(0.0, 0.0));

        let camera = make_camera(ProjectionMode::Perspective);
        let snapping = SnapSettings::default();
        let editor_state = EditorState {
            gizmo_mode: GizmoMode::Translate,
            ..Default::default()
        };
        let viewport_rect = Rect::new(260.0, 56.0, 800.0, 600.0);

        let mut bus = ActionBus::new();
        build_viewport_toolbar(
            &mut UiScope::new(&mut tree, root),
            &camera,
            &editor_state,
            &snapping,
            false,
            &mut bus,
        );
        UiScope::new(&mut tree, root).finish_layout(viewport_rect);

        let translate_btn = tree
            .node_tagged(TAG_GIZMO_TRANSLATE)
            .expect("Translate button must exist");
        assert_eq!(
            translate_btn.style.background_color, custom_active,
            "Active gizmo button must keep theme.button_active_bg after layout"
        );
    }
}