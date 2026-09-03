// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Floating Toolbar Builder
//!
//! Renders the top-left floating glass toolbar containing camera projection modes,
//! shading mode selector, W/E/R gizmo operation buttons, and coordinate space toggle.

use super::types::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
};
use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use ae_renderer::camera::ProjectionMode;
use irisui::prelude::*;

/// Builds the top-left floating toolbar matching the native  engine aesthetic.
pub fn build_viewport_toolbar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &ViewportHudParams<'_>,
    targets: &mut ViewportHudTargets,
) {
    let bar_x = params.viewport_rect.x + 8.0;
    let bar_y = params.viewport_rect.y + 6.0;
    let bar_h = 26.0;
    let btn_h = 20.0;
    let btn_y = bar_y + 3.0;

    // 1. Camera Projection label & dynamic width
    let is_persp = params.camera.mode == ProjectionMode::Perspective;
    let is_top = !is_persp && params.camera.pitch.0 < -1.5;
    let is_front = !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0 > 1.5;
    let is_right =
        !is_persp && params.camera.pitch.0.abs() < 0.1 && params.camera.yaw.0.abs() < 0.1;

    let (camera_label, cam_w) = if is_persp {
        ("🎥 Perspective", 92.0)
    } else if is_top {
        ("📐 Top", 64.0)
    } else if is_front {
        ("📐 Front", 70.0)
    } else if is_right {
        ("📐 Right", 70.0)
    } else {
        ("📐 Ortho", 70.0)
    };

    // 2. Shading Mode label & dynamic width
    let (shading_label, sh_w) = if params.wireframe_enabled {
        ("🕸 Wireframe", 86.0)
    } else {
        ("💡 Lit", 48.0)
    };

    let gizmo_btn_w = 28.0;
    let space_w = 58.0;
    let sep_w = 1.0;
    let group_gap = 6.0;
    let inner_gap = 2.0;

    let total_w = 6.0
        + cam_w
        + group_gap
        + sep_w
        + group_gap
        + sh_w
        + group_gap
        + sep_w
        + group_gap
        + (gizmo_btn_w * 4.0 + inner_gap * 3.0)
        + group_gap
        + sep_w
        + group_gap
        + space_w
        + 6.0;

    let bar_rect = Rect::new(bar_x, bar_y, total_w, bar_h);

    // Floating Toolbar Base Frame
    let bar_id = tree.create_node();
    if let Some(node) = tree.get_mut(bar_id) {
        node.set_name("ViewportToolbar");
        node.computed_rect = bar_rect;
        node.style = Style::new()
            .background(Color::rgba(0.07, 0.08, 0.10, 0.95))
            .border(1.0, Color::rgba(0.18, 0.19, 0.24, 0.90))
            .border_radius(4.0)
            .box_shadow(0.0, 4.0, 10.0, Color::rgba(0.0, 0.0, 0.0, 0.50));
    }
    let _ = tree.add_child(parent_id, bar_id);

    let mut cur_x = bar_x + 6.0;

    // ── 1. Camera Projection Dropdown ──
    let is_cam_open = params.active_dropdown == Some(ViewportHudDropdownId::CameraMode);
    let cam_rect = Rect::new(cur_x, btn_y, cam_w, btn_h);
    let is_cam_hover = cam_rect.contains_point(params.cursor_pos);

    let cam_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(cam_btn_id) {
        node.set_name("CameraModeBtn");
        node.computed_rect = cam_rect;
        let bg = if is_cam_open || is_cam_hover {
            Color::rgba(0.18, 0.20, 0.26, 1.0)
        } else {
            Color::rgba(0.10, 0.11, 0.14, 0.70)
        };
        node.style = Style::new().background(bg).border_radius(3.0);
    }
    let _ = tree.add_child(bar_id, cam_btn_id);

    let cam_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(cam_txt_id) {
        node.set_name("CameraModeText");
        node.set_text(camera_label);
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_cam_open || is_cam_hover {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.90, 0.92, 0.96, 1.0)
        };
        node.computed_rect = cam_rect;
    }
    let _ = tree.add_child(cam_btn_id, cam_txt_id);
    targets
        .dropdown_triggers
        .push((ViewportHudDropdownId::CameraMode, cam_rect));
    cur_x += cam_w + group_gap;

    // Divider 1
    add_divider(tree, bar_id, cur_x, btn_y + 4.0, 12.0);
    cur_x += sep_w + group_gap;

    // ── 2. Shading Mode Dropdown ──
    let is_sh_open = params.active_dropdown == Some(ViewportHudDropdownId::ShadingMode);
    let sh_rect = Rect::new(cur_x, btn_y, sh_w, btn_h);
    let is_sh_hover = sh_rect.contains_point(params.cursor_pos);

    let sh_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(sh_btn_id) {
        node.set_name("ShadingModeBtn");
        node.computed_rect = sh_rect;
        let bg = if is_sh_open || is_sh_hover {
            Color::rgba(0.18, 0.20, 0.26, 1.0)
        } else {
            Color::rgba(0.10, 0.11, 0.14, 0.70)
        };
        node.style = Style::new().background(bg).border_radius(3.0);
    }
    let _ = tree.add_child(bar_id, sh_btn_id);

    let sh_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(sh_txt_id) {
        node.set_name("ShadingModeText");
        node.set_text(shading_label);
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_sh_open || is_sh_hover {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.90, 0.92, 0.96, 1.0)
        };
        node.computed_rect = sh_rect;
    }
    let _ = tree.add_child(sh_btn_id, sh_txt_id);
    targets
        .dropdown_triggers
        .push((ViewportHudDropdownId::ShadingMode, sh_rect));
    cur_x += sh_w + group_gap;

    // Divider 2
    add_divider(tree, bar_id, cur_x, btn_y + 4.0, 12.0);
    cur_x += sep_w + group_gap;

    // ── 3. Gizmo Mode Controls (Q W E R) ──
    let gizmo_modes = [
        (GizmoMode::Select, "↖ Q"),
        (GizmoMode::Translate, "✛ W"),
        (GizmoMode::Rotate, "⟳ E"),
        (GizmoMode::Scale, "⤡ R"),
    ];

    for (mode, label) in gizmo_modes {
        let is_selected = params.gizmo_mode == mode;
        let g_rect = Rect::new(cur_x, btn_y, gizmo_btn_w, btn_h);
        let is_hover = g_rect.contains_point(params.cursor_pos);

        let g_id = tree.create_node();
        if let Some(node) = tree.get_mut(g_id) {
            node.set_name("GizmoBtn");
            node.computed_rect = g_rect;
            let bg = match (is_selected, is_hover) {
                (true, true) => Color::rgba(0.0, 0.58, 0.80, 1.0),
                (true, false) => Color::rgba(0.0, 0.47, 0.65, 1.0),
                (false, true) => Color::rgba(0.18, 0.20, 0.26, 1.0),
                (false, false) => Color::rgba(0.10, 0.11, 0.14, 0.80),
            };
            node.style = Style::new().background(bg).border_radius(3.0);
        }
        let _ = tree.add_child(bar_id, g_id);

        let g_txt = tree.create_node();
        if let Some(node) = tree.get_mut(g_txt) {
            node.set_name("GizmoBtnText");
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_selected || is_hover {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.63, 0.63, 0.63, 1.0)
            };
            node.computed_rect = g_rect;
        }
        let _ = tree.add_child(g_id, g_txt);
        targets
            .buttons
            .push((ViewportHudAction::SetGizmoMode(mode), g_rect));
        cur_x += gizmo_btn_w + inner_gap;
    }
    cur_x += group_gap - inner_gap;

    // Divider 3
    add_divider(tree, bar_id, cur_x, btn_y + 4.0, 12.0);
    cur_x += sep_w + group_gap;

    // ── 4. World / Local Space Toggle ──
    let is_local = params.gizmo_space == GizmoSpace::Local;
    let space_label = if is_local { "🏠 Local" } else { "🌍 World" };
    let space_rect = Rect::new(cur_x, btn_y, space_w, btn_h);
    let is_space_hover = space_rect.contains_point(params.cursor_pos);

    let space_id = tree.create_node();
    if let Some(node) = tree.get_mut(space_id) {
        node.set_name("GizmoSpaceBtn");
        node.computed_rect = space_rect;
        let bg = match (is_local, is_space_hover) {
            (true, true) => Color::rgba(0.68, 0.37, 0.12, 1.0),
            (true, false) => Color::rgba(0.55, 0.29, 0.10, 1.0),
            (false, true) => Color::rgba(0.18, 0.20, 0.26, 1.0),
            (false, false) => Color::rgba(0.10, 0.11, 0.14, 0.80),
        };
        node.style = Style::new().background(bg).border_radius(3.0);
    }
    let _ = tree.add_child(bar_id, space_id);

    let space_txt = tree.create_node();
    if let Some(node) = tree.get_mut(space_txt) {
        node.set_name("GizmoSpaceText");
        node.set_text(space_label);
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_local || is_space_hover {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.70, 0.70, 0.70, 1.0)
        };
        node.computed_rect = space_rect;
    }
    let _ = tree.add_child(space_id, space_txt);
    targets
        .buttons
        .push((ViewportHudAction::ToggleGizmoSpace, space_rect));
}

/// Helper to render subtle vertical dividers between toolbar groups.
fn add_divider(tree: &mut UiTree, parent_id: WidgetId, x: f32, y: f32, h: f32) {
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("ToolbarDivider");
        node.style = Style::new().background(Color::rgba(1.0, 1.0, 1.0, 0.12));
        node.computed_rect = Rect::new(x, y, 1.0, h);
    }
    let _ = tree.add_child(parent_id, sep_id);
}