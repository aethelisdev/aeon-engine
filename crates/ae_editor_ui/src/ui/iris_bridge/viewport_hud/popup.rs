// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Dropdown Popup Renderer
//!
//! Renders top-layer floating popup menus for Camera Projection Modes and Shading Modes.

use super::types::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
};
use ae_renderer::camera::ProjectionMode;
use irisui::prelude::*;

/// Renders active floating popup menus in the Viewport HUD.
pub fn render_viewport_hud_dropdown_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    active_dd: ViewportHudDropdownId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let Some(&(_, btn_rect)) = targets
        .dropdown_triggers
        .iter()
        .find(|(id, _)| *id == active_dd)
    else {
        return;
    };

    let (items_count, options): (usize, Vec<(String, ViewportHudAction, bool)>) = match active_dd {
        ViewportHudDropdownId::CameraMode => {
            let is_persp = params.camera.mode == ProjectionMode::Perspective;
            let is_top = !is_persp && params.camera.pitch.0 < -1.5;
            let is_front =
                !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0 > 1.5;
            let is_right =
                !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0.abs() < 0.1;

            let target = params.camera.target;
            let d = 10.0;

            let opts = vec![
                (
                    "🎥 Perspective".to_string(),
                    ViewportHudAction::SetCameraMode(ProjectionMode::Perspective),
                    is_persp,
                ),
                (
                    "📐 Top".to_string(),
                    ViewportHudAction::SetCameraTransform {
                        pitch: cgmath::Rad(-std::f32::consts::FRAC_PI_2 + 0.001),
                        yaw: cgmath::Rad(0.0),
                        position: cgmath::Point3::new(target.x, target.y + d, target.z),
                    },
                    is_top,
                ),
                (
                    "📐 Front".to_string(),
                    ViewportHudAction::SetCameraTransform {
                        pitch: cgmath::Rad(0.0),
                        yaw: cgmath::Rad(std::f32::consts::FRAC_PI_2),
                        position: cgmath::Point3::new(target.x, target.y, target.z - d),
                    },
                    is_front,
                ),
                (
                    "📐 Right".to_string(),
                    ViewportHudAction::SetCameraTransform {
                        pitch: cgmath::Rad(0.0),
                        yaw: cgmath::Rad(0.0),
                        position: cgmath::Point3::new(target.x + d, target.y, target.z),
                    },
                    is_right,
                ),
            ];
            (opts.len(), opts)
        }
        ViewportHudDropdownId::ShadingMode => {
            let opts = vec![
                (
                    "💡 Lit".to_string(),
                    ViewportHudAction::ToggleWireframe,
                    !params.wireframe_enabled,
                ),
                (
                    "🕸 Wireframe".to_string(),
                    ViewportHudAction::ToggleWireframe,
                    params.wireframe_enabled,
                ),
            ];
            (opts.len(), opts)
        }
    };

    let popup_h = (items_count as f32) * 22.0 + 4.0;
    let popup_rect = Rect::new(
        btn_rect.x,
        btn_rect.y + btn_rect.height + 2.0,
        btn_rect.width.max(120.0),
        popup_h,
    );
    targets.active_dropdown_popup_rect = Some(popup_rect);

    let popup_id = tree.create_node();
    if let Some(node) = tree.get_mut(popup_id) {
        node.set_name("ViewportHudPopup");
        node.computed_rect = popup_rect;
        node.style = Style::new()
            .background(Color::rgba(0.07, 0.08, 0.11, 0.98))
            .border(1.0, Color::rgba(0.24, 0.28, 0.38, 0.70))
            .border_radius(4.0)
            .box_shadow(0.0, 6.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.75));
    }
    let _ = tree.add_child(parent_id, popup_id);

    for (idx, (label, action, is_selected)) in options.into_iter().enumerate() {
        let item_y = popup_rect.y + 2.0 + (idx as f32) * 22.0;
        let item_rect = Rect::new(popup_rect.x + 2.0, item_y, popup_rect.width - 4.0, 20.0);
        let is_hovered = item_rect.contains_point(params.cursor_pos);

        let item_id = tree.create_node();
        if let Some(node) = tree.get_mut(item_id) {
            node.set_name("ViewportHudPopupItem");
            node.computed_rect = item_rect;
            let bg = if is_selected {
                Color::rgba(0.0, 0.35, 0.45, 0.85)
            } else if is_hovered {
                Color::rgba(0.20, 0.23, 0.32, 0.95)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.0)
            };
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(popup_id, item_id);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("ViewportHudItemText");
            node.set_text(&label);
            node.font_size = 11.0;
            node.line_height = 20.0;
            node.text_color = if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else if is_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.85, 0.88, 0.95, 1.0)
            };
            node.computed_rect =
                Rect::new(item_rect.x + 6.0, item_rect.y, item_rect.width - 12.0, 20.0);
        }
        let _ = tree.add_child(item_id, lbl_id);

        targets
            .active_dropdown_items
            .push((action, item_rect, label));
    }
}