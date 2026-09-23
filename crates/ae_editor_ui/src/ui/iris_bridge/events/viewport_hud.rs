// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interaction and event handling subsystem for the 3D Viewport HUD overlay.

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use crate::ui::iris_bridge::viewport_hud::{
    TAG_COMPASS_NEG_X, TAG_COMPASS_NEG_Y, TAG_COMPASS_NEG_Z, TAG_COMPASS_POS_X, TAG_COMPASS_POS_Y,
    TAG_COMPASS_POS_Z, TAG_GIZMO_ROTATE, TAG_GIZMO_SCALE, TAG_GIZMO_SELECT, TAG_GIZMO_SPACE_TOGGLE,
    TAG_GIZMO_TRANSLATE, TAG_PLAY_EXIT, TAG_PLAY_RESUME, TAG_VIEWPORT_CAMERA_MODE,
    TAG_VIEWPORT_SHADING_MODE, ViewportHudAction, ViewportHudDropdownId,
};
use ae_editor::gizmo::GizmoMode;
use ae_editor::scene_gizmo::SceneViewSnap;
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Handles mouse interactions and dropdown selections for the 3D Viewport HUD.
    ///
    /// Evaluates clicks natively via Iris UI $O(1)$ semantic tag hit-testing with zero rect iteration.
    pub(crate) fn handle_viewport_hud_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        if !self.viewport_hud.is_active {
            return None;
        }
        let mut result = IrisOverlayEventResult::default();

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();

            // 1. If an active dropdown popup is open, check if an item was clicked
            if let Some(dd_id) = self.viewport_hud.dropdown {
                let hit_target = self.tree.hit_test_target(click_point);
                if let Some(ref hit) = hit_target
                    && hit.layer == UiLayer::Popup
                    && hit.role == WidgetRole::DropdownItem
                {
                    self.viewport_hud
                        .actions
                        .push(ViewportHudAction::SelectDropdownItem(
                            dd_id,
                            hit.tag as usize,
                        ));
                    self.viewport_hud.dropdown = None;
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                } else if let Some(ref hit) = hit_target
                    && (hit.tag == TAG_VIEWPORT_CAMERA_MODE || hit.tag == TAG_VIEWPORT_SHADING_MODE)
                {
                    let clicked_dd = if hit.tag == TAG_VIEWPORT_CAMERA_MODE {
                        ViewportHudDropdownId::CameraMode
                    } else {
                        ViewportHudDropdownId::ShadingMode
                    };
                    if self.viewport_hud.dropdown == Some(clicked_dd) {
                        self.viewport_hud.dropdown = None;
                    } else {
                        self.viewport_hud.dropdown = Some(clicked_dd);
                    }
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                    result.consumed = true;
                    return Some(result);
                } else {
                    // Clicked outside dropdown items; dismiss active dropdown
                    self.viewport_hud.dropdown = None;
                    self.chrome.needs_layout_rebuild = true;
                    self.notifier.tag_all();
                }
            }

            // 2. Native O(1) Hit-Testing via semantic tags
            if let Some(hit) = self.tree.hit_test_target(click_point) {
                match hit.tag {
                    TAG_VIEWPORT_CAMERA_MODE => {
                        self.viewport_hud.dropdown = if self.viewport_hud.dropdown
                            == Some(ViewportHudDropdownId::CameraMode)
                        {
                            None
                        } else {
                            Some(ViewportHudDropdownId::CameraMode)
                        };
                        self.chrome.needs_layout_rebuild = true;
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_VIEWPORT_SHADING_MODE => {
                        self.viewport_hud.dropdown = if self.viewport_hud.dropdown
                            == Some(ViewportHudDropdownId::ShadingMode)
                        {
                            None
                        } else {
                            Some(ViewportHudDropdownId::ShadingMode)
                        };
                        self.chrome.needs_layout_rebuild = true;
                        self.notifier.tag_all();
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_GIZMO_SELECT => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::SetGizmoMode(GizmoMode::Select));
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_GIZMO_TRANSLATE => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::SetGizmoMode(GizmoMode::Translate));
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_GIZMO_ROTATE => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::SetGizmoMode(GizmoMode::Rotate));
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_GIZMO_SCALE => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::SetGizmoMode(GizmoMode::Scale));
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_GIZMO_SPACE_TOGGLE => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::ToggleGizmoSpace);
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_PLAY_RESUME => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::ResumeGame);
                        result.consumed = true;
                        return Some(result);
                    }
                    TAG_PLAY_EXIT => {
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::ExitToEditor);
                        result.consumed = true;
                        return Some(result);
                    }
                    t if (TAG_COMPASS_POS_X..=TAG_COMPASS_NEG_Z).contains(&t) => {
                        let snap = match t {
                            TAG_COMPASS_POS_X => SceneViewSnap::Right,
                            TAG_COMPASS_POS_Y => SceneViewSnap::Top,
                            TAG_COMPASS_POS_Z => SceneViewSnap::Front,
                            TAG_COMPASS_NEG_X => SceneViewSnap::Left,
                            TAG_COMPASS_NEG_Y => SceneViewSnap::Bottom,
                            TAG_COMPASS_NEG_Z => SceneViewSnap::Back,
                            _ => SceneViewSnap::Perspective,
                        };
                        self.viewport_hud
                            .actions
                            .push(ViewportHudAction::SnapCamera(snap));
                        result.consumed = true;
                        return Some(result);
                    }
                    _ => {}
                }
            }
        }

        None
    }
}