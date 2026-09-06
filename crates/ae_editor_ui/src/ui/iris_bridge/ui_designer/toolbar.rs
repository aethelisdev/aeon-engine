// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Toolbar
//!
//! Renders the elevated 34px top control bar with aspect ratio preset selection,
//! zoom controls, grid snapping toggles, anchor guides, and canonical hardware atlas `ICON_PLUS` button.
//!

use super::types::{UiDesignerPanelParams, UiDesignerPanelTargets};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;

/// Height of the UI Designer top control bar in screen pixels.
pub const UI_DESIGNER_TOOLBAR_HEIGHT: f32 = 34.0;

/// Builds the 34px elevated top control bar in the UI tree.
pub fn build_designer_toolbar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    targets: &mut UiDesignerPanelTargets,
) {
    let bar_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.y,
        params.panel_rect.width,
        UI_DESIGNER_TOOLBAR_HEIGHT,
    );

    // 1. Toolbar Background Container
    let bar_id = tree.create_node();
    if let Some(node) = tree.get_mut(bar_id) {
        node.set_name("UiDesignerToolbar");
        node.computed_rect = bar_rect;
        node.style = Style::new()
            .background(Color::rgba(0.080, 0.088, 0.110, 0.98))
            .border(1.0, Color::rgba(0.18, 0.21, 0.28, 0.90))
            .box_shadow(0.0, 2.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 0.60));
    }
    let _ = tree.add_child(parent_id, bar_id);

    let btn_y = bar_rect.y + 5.0;
    let btn_h = 24.0;
    let mut cur_x = bar_rect.x + 8.0;

    // ── 2. Aspect Ratio Selector Button ───────────────────────────────────────
    let aspect_w = 142.0;
    let aspect_rect = Rect::new(cur_x, btn_y, aspect_w, btn_h);
    targets.btn_aspect = Some(aspect_rect);

    let is_aspect_hovered = aspect_rect.contains_point(params.cursor_pos);
    let (asp_bg, asp_border) = if params.is_aspect_dropdown_open {
        (
            Color::rgba(0.0, 0.35, 0.48, 0.95),
            Color::rgba(0.0, 0.85, 1.0, 0.95),
        )
    } else if is_aspect_hovered {
        (
            Color::rgba(0.15, 0.17, 0.22, 0.95),
            Color::rgba(0.28, 0.32, 0.40, 0.90),
        )
    } else {
        (
            Color::rgba(0.11, 0.12, 0.15, 0.95),
            Color::rgba(0.18, 0.20, 0.24, 0.85),
        )
    };

    let asp_id = tree.create_node();
    if let Some(node) = tree.get_mut(asp_id) {
        node.set_name("ToolbarAspectBtn");
        node.computed_rect = aspect_rect;
        node.style = Style::new()
            .background(asp_bg)
            .border(1.0, asp_border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(bar_id, asp_id);

    let asp_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(asp_txt_id) {
        node.set_name("AspectBtnText");
        node.set_text(format!("📐 {} ▾", params.state.aspect_ratio.label()));
        node.font_size = 10.5;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.88, 0.90, 0.95, 1.0);
        node.computed_rect = aspect_rect;
    }
    let _ = tree.add_child(asp_id, asp_txt_id);

    cur_x += aspect_w + 8.0;

    // Divider 1
    build_divider(tree, bar_id, cur_x, btn_y + 2.0);
    cur_x += 7.0;

    // ── 3. Zoom Controls (- , 100% , +) ───────────────────────────────────────
    // Zoom Out Button
    let zoom_out_w = 24.0;
    let z_out_rect = Rect::new(cur_x, btn_y, zoom_out_w, btn_h);
    targets.btn_zoom_out = Some(z_out_rect);
    build_tool_btn(tree, bar_id, z_out_rect, "−", params.cursor_pos);
    cur_x += zoom_out_w + 3.0;

    // Zoom Reset / Readout Button
    let zoom_reset_w = 50.0;
    let z_rst_rect = Rect::new(cur_x, btn_y, zoom_reset_w, btn_h);
    targets.btn_zoom_reset = Some(z_rst_rect);
    let zoom_text = format!("{:.0}%", params.state.zoom * 100.0);
    let is_rst_hovered = z_rst_rect.contains_point(params.cursor_pos);
    let (rst_bg, rst_border) = if is_rst_hovered {
        (
            Color::rgba(0.0, 0.35, 0.48, 0.95),
            Color::rgba(0.0, 0.85, 1.0, 0.95),
        )
    } else {
        (
            Color::rgba(0.11, 0.12, 0.15, 0.95),
            Color::rgba(0.18, 0.20, 0.24, 0.85),
        )
    };

    let rst_id = tree.create_node();
    if let Some(node) = tree.get_mut(rst_id) {
        node.set_name("ZoomResetBtn");
        node.computed_rect = z_rst_rect;
        node.style = Style::new()
            .background(rst_bg)
            .border(1.0, rst_border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(bar_id, rst_id);

    let rst_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(rst_txt_id) {
        node.set_name("ZoomResetText");
        node.set_text(zoom_text);
        node.font_size = 10.5;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.0, 0.85, 1.0, 1.0);
        node.computed_rect = z_rst_rect;
    }
    let _ = tree.add_child(rst_id, rst_txt_id);
    cur_x += zoom_reset_w + 3.0;

    // Zoom In Button
    let zoom_in_w = 24.0;
    let z_in_rect = Rect::new(cur_x, btn_y, zoom_in_w, btn_h);
    targets.btn_zoom_in = Some(z_in_rect);
    build_tool_btn(tree, bar_id, z_in_rect, "+", params.cursor_pos);
    cur_x += zoom_in_w + 8.0;

    // Divider 2
    build_divider(tree, bar_id, cur_x, btn_y + 2.0);
    cur_x += 7.0;

    // ── 4. Grid Snap Button ───────────────────────────────────────────────────
    let snap_w = 82.0;
    let snap_rect = Rect::new(cur_x, btn_y, snap_w, btn_h);
    targets.btn_snap = Some(snap_rect);

    let snap_text = match params.state.snap_grid {
        Some(s) => format!("Snap: {:.0}px", s),
        None => "Snap: Free".to_string(),
    };

    let is_snap_hovered = snap_rect.contains_point(params.cursor_pos);
    let (snp_bg, snp_border) = if is_snap_hovered {
        (
            Color::rgba(0.15, 0.17, 0.22, 0.95),
            Color::rgba(0.28, 0.32, 0.40, 0.90),
        )
    } else {
        (
            Color::rgba(0.11, 0.12, 0.15, 0.95),
            Color::rgba(0.18, 0.20, 0.24, 0.85),
        )
    };

    let snp_id = tree.create_node();
    if let Some(node) = tree.get_mut(snp_id) {
        node.set_name("SnapBtn");
        node.computed_rect = snap_rect;
        node.style = Style::new()
            .background(snp_bg)
            .border(1.0, snp_border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(bar_id, snp_id);

    let snp_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(snp_txt_id) {
        node.set_name("SnapBtnText");
        node.set_text(snap_text);
        node.font_size = 10.0;
        node.line_height = btn_h;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.85, 0.88, 0.92, 1.0);
        node.computed_rect = snap_rect;
    }
    let _ = tree.add_child(snp_id, snp_txt_id);
    cur_x += snap_w + 6.0;

    // ── 5. Anchor & Grid Toggles ──────────────────────────────────────────────
    // Anchor Guides Toggle
    let anch_w = 80.0;
    let anch_rect = Rect::new(cur_x, btn_y, anch_w, btn_h);
    targets.btn_anchors = Some(anch_rect);
    build_toggle_btn(
        tree,
        bar_id,
        anch_rect,
        "⚓ Anchors",
        params.state.show_anchor_guides,
        params.cursor_pos,
    );
    cur_x += anch_w + 4.0;

    // Grid Toggle
    let grid_w = 64.0;
    let grid_rect = Rect::new(cur_x, btn_y, grid_w, btn_h);
    targets.btn_grid = Some(grid_rect);
    build_toggle_btn(
        tree,
        bar_id,
        grid_rect,
        "⊞ Grid",
        params.state.show_grid,
        params.cursor_pos,
    );
    cur_x += grid_w + 8.0;

    // Divider 3
    build_divider(tree, bar_id, cur_x, btn_y + 2.0);
    cur_x += 7.0;

    // ── 6. ➕ Add Element Palette Button ──────────────────────────────────────
    let add_btn_w = 118.0;
    let add_rect = Rect::new(cur_x, btn_y, add_btn_w, btn_h);
    targets.btn_add_element = Some(add_rect);

    let is_add_hovered = add_rect.contains_point(params.cursor_pos);
    let (add_bg, add_border) = if params.is_add_menu_open {
        (
            Color::rgba(0.0, 0.40, 0.55, 0.98),
            Color::rgba(0.0, 0.90, 1.0, 1.0),
        )
    } else if is_add_hovered {
        (
            Color::rgba(0.0, 0.32, 0.44, 0.95),
            Color::rgba(0.0, 0.80, 0.95, 0.95),
        )
    } else {
        (
            Color::rgba(0.0, 0.22, 0.32, 0.90),
            Color::rgba(0.0, 0.55, 0.70, 0.85),
        )
    };

    let add_id = tree.create_node();
    if let Some(node) = tree.get_mut(add_id) {
        node.set_name("AddElementBtn");
        node.computed_rect = add_rect;
        node.style = Style::new()
            .background(add_bg)
            .border(1.0, add_border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(bar_id, add_id);

    // ICON_PLUS quad
    let p_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(p_icon_id) {
        node.set_name("AddElementPlusIcon");
        node.computed_rect = Rect::new(add_rect.x + 8.0, add_rect.y + 6.0, 12.0, 12.0);
        node.set_texture_uv(ICON_PLUS);
        node.set_texture_tint(Color::rgba(0.0, 0.95, 1.0, 1.0));
    }
    let _ = tree.add_child(add_id, p_icon_id);

    let add_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(add_txt_id) {
        node.set_name("AddElementText");
        node.set_text("Add Element");
        node.font_size = 10.5;
        node.line_height = btn_h;
        node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        node.computed_rect = Rect::new(add_rect.x + 24.0, add_rect.y, add_btn_w - 28.0, btn_h);
    }
    let _ = tree.add_child(add_id, add_txt_id);
}

fn build_divider(tree: &mut UiTree, parent_id: WidgetId, x: f32, y: f32) {
    let div_id = tree.create_node();
    if let Some(node) = tree.get_mut(div_id) {
        node.set_name("ToolbarDivider");
        node.computed_rect = Rect::new(x, y, 1.0, 20.0);
        node.style = Style::new().background(Color::rgba(0.18, 0.20, 0.24, 0.85));
    }
    let _ = tree.add_child(parent_id, div_id);
}

fn build_tool_btn(
    tree: &mut UiTree,
    parent_id: WidgetId,
    rect: Rect,
    label: &str,
    cursor_pos: Point,
) {
    let is_hovered = rect.contains_point(cursor_pos);
    let (bg, border) = if is_hovered {
        (
            Color::rgba(0.15, 0.17, 0.22, 0.95),
            Color::rgba(0.28, 0.32, 0.40, 0.90),
        )
    } else {
        (
            Color::rgba(0.11, 0.12, 0.15, 0.95),
            Color::rgba(0.18, 0.20, 0.24, 0.85),
        )
    };

    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("ToolBtn");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, btn_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name("ToolBtnText");
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.85, 0.88, 0.92, 1.0);
        node.computed_rect = rect;
    }
    let _ = tree.add_child(btn_id, txt_id);
}

fn build_toggle_btn(
    tree: &mut UiTree,
    parent_id: WidgetId,
    rect: Rect,
    label: &str,
    is_active: bool,
    cursor_pos: Point,
) {
    let is_hovered = rect.contains_point(cursor_pos);
    let (bg, border, txt_col) = if is_active {
        (
            Color::rgba(0.0, 0.30, 0.40, 0.95),
            Color::rgba(0.0, 0.80, 0.95, 0.95),
            Color::rgba(1.0, 1.0, 1.0, 1.0),
        )
    } else if is_hovered {
        (
            Color::rgba(0.15, 0.17, 0.22, 0.95),
            Color::rgba(0.28, 0.32, 0.40, 0.90),
            Color::rgba(0.80, 0.83, 0.88, 1.0),
        )
    } else {
        (
            Color::rgba(0.11, 0.12, 0.15, 0.95),
            Color::rgba(0.18, 0.20, 0.24, 0.85),
            Color::rgba(0.60, 0.63, 0.70, 1.0),
        )
    };

    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("ToggleBtn");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, btn_id);

    let txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(txt_id) {
        node.set_name("ToggleBtnText");
        node.set_text(label);
        node.font_size = 10.0;
        node.line_height = rect.height;
        node.text_align = TextAlign::Center;
        node.text_color = txt_col;
        node.computed_rect = rect;
    }
    let _ = tree.add_child(btn_id, txt_id);
}