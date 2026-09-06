// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Virtual Canvas
//!
//! Computes virtual canvas bounds, projection matrices, background grid lines,
//! and translates ECS `UiLayoutResolver` drawing commands into native Iris UI nodes.
//!

use super::anchors::{AnchorGuideParams, build_anchor_pin_and_guide};
use super::toolbar::UI_DESIGNER_TOOLBAR_HEIGHT;
use super::types::{UiDesignerPanelParams, UiDesignerPanelTargets, UiElementHitTarget};
use ae_core::ecs::UiElement;
use ae_core::ui::{UiDrawCommand, UiLayoutResolver, UiTextAlignment};
use irisui::prelude::*;

/// Builds the virtual canvas viewport, background grid, and rendered UI elements.
pub fn build_designer_canvas(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    targets: &mut UiDesignerPanelTargets,
) {
    let avail_x = params.panel_rect.x;
    let avail_y = params.panel_rect.y + UI_DESIGNER_TOOLBAR_HEIGHT;
    let avail_w = params.panel_rect.width;
    let avail_h = (params.panel_rect.height - UI_DESIGNER_TOOLBAR_HEIGHT).max(10.0);
    let avail_rect = Rect::new(avail_x, avail_y, avail_w, avail_h);

    let [screen_w, screen_h] = params.state.aspect_ratio.resolution();

    // ── 1. Compute Fitted Virtual Canvas Coordinates Centered in View ──────────
    let margin = 32.0;
    let max_w = (avail_w - margin * 2.0).max(100.0);
    let max_h = (avail_h - margin * 2.0).max(100.0);

    let scale_w = max_w / screen_w;
    let scale_h = max_h / screen_h;
    let base_scale = scale_w.min(scale_h) * params.state.zoom;

    let canvas_w = screen_w * base_scale;
    let canvas_h = screen_h * base_scale;

    let center_x = avail_x + avail_w * 0.5 + params.state.pan_offset[0];
    let center_y = avail_y + avail_h * 0.5 + params.state.pan_offset[1];

    let canvas_rect = Rect::new(
        center_x - canvas_w * 0.5,
        center_y - canvas_h * 0.5,
        canvas_w,
        canvas_h,
    );

    targets.canvas_rect = canvas_rect;
    targets.base_scale = base_scale;
    targets.resolution = [screen_w, screen_h];
    targets.current_zoom = params.state.zoom;
    targets.snap_grid = params.state.snap_grid;

    // Coordinate conversion closures
    let to_screen_pos = |cx: f32, cy: f32| -> Point {
        Point::new(
            canvas_rect.x + (cx / screen_w) * canvas_rect.width,
            canvas_rect.y + (cy / screen_h) * canvas_rect.height,
        )
    };

    let to_canvas_pos = |sp: Point| -> [f32; 2] {
        let rel_x = (sp.x - canvas_rect.x) / canvas_rect.width;
        let rel_y = (sp.y - canvas_rect.y) / canvas_rect.height;
        [rel_x * screen_w, rel_y * screen_h]
    };

    // ── 2. Letterbox Outer Background ─────────────────────────────────────────
    let letterbox_id = tree.create_node();
    if let Some(node) = tree.get_mut(letterbox_id) {
        node.set_name("UiDesignerLetterbox");
        node.computed_rect = avail_rect;
        node.style = Style::new().background(Color::rgba(0.055, 0.063, 0.086, 1.0));
    }
    let _ = tree.add_child(parent_id, letterbox_id);

    // ── 3. Virtual Canvas Viewport Surface ─────────────────────────────────────
    let canvas_surface_id = tree.create_node();
    if let Some(node) = tree.get_mut(canvas_surface_id) {
        node.set_name("UiDesignerCanvasSurface");
        node.computed_rect = canvas_rect;
        node.style = Style::new()
            .background(Color::rgba(0.086, 0.102, 0.137, 1.0))
            .border(1.5, Color::rgba(0.0, 0.706, 0.941, 0.85));
    }
    let _ = tree.add_child(parent_id, canvas_surface_id);

    // ── 4. Background Grid Lines ──────────────────────────────────────────────
    if params.state.show_grid {
        build_canvas_grid(tree, parent_id, canvas_rect, base_scale);
    }

    // ── 5. Resolve and Render In-Game UI Elements from ECS ─────────────────────
    let mouse_canvas_pos = if canvas_rect.contains_point(params.cursor_pos) {
        Some(to_canvas_pos(params.cursor_pos))
    } else {
        None
    };

    let draw_commands = UiLayoutResolver::resolve_draw_commands(
        params.world,
        screen_w,
        screen_h,
        mouse_canvas_pos,
        false,
    );

    for (idx, cmd) in draw_commands.iter().enumerate() {
        match cmd {
            UiDrawCommand::Rect {
                rect,
                fill_color,
                border_color,
                border_width,
                border_radius,
                ..
            } => {
                let min_p = to_screen_pos(rect.min_x, rect.min_y);
                let max_p = to_screen_pos(rect.max_x, rect.max_y);
                let w = (max_p.x - min_p.x).max(0.0);
                let h = (max_p.y - min_p.y).max(0.0);
                let draw_rect = Rect::new(min_p.x, min_p.y, w, h);

                let rect_id = tree.create_node();
                if let Some(node) = tree.get_mut(rect_id) {
                    node.set_name(format!("UiDrawRect_{}", idx));
                    node.computed_rect = draw_rect;

                    let mut style = Style::new().background(Color::rgba(
                        fill_color[0],
                        fill_color[1],
                        fill_color[2],
                        fill_color[3],
                    ));

                    if border_color[3] > 0.01 && *border_width > 0.0 {
                        style = style.border(
                            (*border_width * base_scale).max(1.0),
                            Color::rgba(
                                border_color[0],
                                border_color[1],
                                border_color[2],
                                border_color[3],
                            ),
                        );
                    }
                    if *border_radius > 0.0 {
                        style = style.border_radius(*border_radius * base_scale);
                    }
                    node.style = style;
                }
                let _ = tree.add_child(parent_id, rect_id);
            }
            UiDrawCommand::Text {
                pos,
                text,
                font_size,
                color,
                alignment,
                ..
            } => {
                let p = to_screen_pos(pos[0], pos[1]);
                let scaled_font_size = (*font_size * base_scale).max(8.0);
                let approx_w = (text.len() as f32) * (scaled_font_size * 0.65);
                let approx_h = scaled_font_size * 1.3;

                let (text_x, iris_align) = match alignment {
                    UiTextAlignment::Left => (p.x, TextAlign::Left),
                    UiTextAlignment::Center => (p.x - approx_w * 0.5, TextAlign::Center),
                    UiTextAlignment::Right => (p.x - approx_w, TextAlign::Right),
                };
                let text_y = p.y - approx_h * 0.5;

                let text_id = tree.create_node();
                if let Some(node) = tree.get_mut(text_id) {
                    node.set_name(format!("UiDrawText_{}", idx));
                    node.computed_rect = Rect::new(text_x, text_y, approx_w, approx_h);
                    node.set_text(text.clone());
                    node.font_size = scaled_font_size;
                    node.line_height = approx_h;
                    node.text_align = iris_align;
                    node.text_color = Color::rgba(color[0], color[1], color[2], color[3]);
                }
                let _ = tree.add_child(parent_id, text_id);
            }
            UiDrawCommand::Image { rect, tint, .. } => {
                let min_p = to_screen_pos(rect.min_x, rect.min_y);
                let max_p = to_screen_pos(rect.max_x, rect.max_y);
                let w = (max_p.x - min_p.x).max(0.0);
                let h = (max_p.y - min_p.y).max(0.0);

                let img_id = tree.create_node();
                if let Some(node) = tree.get_mut(img_id) {
                    node.set_name(format!("UiDrawImage_{}", idx));
                    node.computed_rect = Rect::new(min_p.x, min_p.y, w, h);
                    node.style =
                        Style::new().background(Color::rgba(tint[0], tint[1], tint[2], tint[3]));
                }
                let _ = tree.add_child(parent_id, img_id);
            }
        }
    }

    // ── 6. Element Hit-Testing, Selection Outline, and Handles ────────────────
    for (ent, elem) in params.world.query::<(hecs::Entity, &UiElement)>().iter() {
        if !elem.visible {
            continue;
        }

        let elem_rect = elem.compute_rect(screen_w, screen_h);
        let screen_min = to_screen_pos(elem_rect.min_x, elem_rect.min_y);
        let screen_max = to_screen_pos(elem_rect.max_x, elem_rect.max_y);
        let screen_w_elem = (screen_max.x - screen_min.x).max(1.0);
        let screen_h_elem = (screen_max.y - screen_min.y).max(1.0);
        let screen_elem_rect = Rect::new(screen_min.x, screen_min.y, screen_w_elem, screen_h_elem);

        targets.element_rects.push((ent, screen_elem_rect));
        targets.element_targets.push(UiElementHitTarget {
            entity: ent,
            rect: screen_elem_rect,
            anchor_origin: elem.anchor.compute_origin(screen_w, screen_h),
            initial_offset: elem.offset,
        });

        let is_selected = params.selected_entity == Some(ent);
        let is_hovered = screen_elem_rect.contains_point(params.cursor_pos);

        if is_selected {
            // Glowing cyan selection outline
            let outline_id = tree.create_node();
            if let Some(node) = tree.get_mut(outline_id) {
                node.set_name("UiElementSelectionOutline");
                node.computed_rect = Rect::new(
                    screen_elem_rect.x - 2.0,
                    screen_elem_rect.y - 2.0,
                    screen_elem_rect.width + 4.0,
                    screen_elem_rect.height + 4.0,
                );
                node.style = Style::new()
                    .border(1.8, Color::rgba(0.0, 0.85, 1.0, 0.95))
                    .border_radius(2.0);
            }
            let _ = tree.add_child(parent_id, outline_id);

            // 4 corner sizing handles
            let handle_size = 6.0;
            let corners = [
                Point::new(screen_elem_rect.x, screen_elem_rect.y),
                Point::new(screen_elem_rect.right(), screen_elem_rect.y),
                Point::new(screen_elem_rect.x, screen_elem_rect.bottom()),
                Point::new(screen_elem_rect.right(), screen_elem_rect.bottom()),
            ];
            for (c_idx, corner) in corners.iter().enumerate() {
                let handle_id = tree.create_node();
                if let Some(node) = tree.get_mut(handle_id) {
                    node.set_name(format!("UiElementHandle_{}", c_idx));
                    node.computed_rect = Rect::new(
                        corner.x - handle_size * 0.5,
                        corner.y - handle_size * 0.5,
                        handle_size,
                        handle_size,
                    );
                    node.style = Style::new()
                        .background(Color::rgba(1.0, 1.0, 1.0, 0.98))
                        .border(1.0, Color::rgba(0.0, 0.5, 0.8, 0.9))
                        .border_radius(1.0);
                }
                let _ = tree.add_child(parent_id, handle_id);
            }

            // Anchor guide lines and pins
            if params.state.show_anchor_guides {
                build_anchor_pin_and_guide(
                    tree,
                    parent_id,
                    &AnchorGuideParams {
                        elem,
                        screen_elem_rect,
                        screen_w,
                        screen_h,
                        base_scale,
                        to_screen_pos: &to_screen_pos,
                    },
                );
            }
        } else if is_hovered {
            let hover_id = tree.create_node();
            if let Some(node) = tree.get_mut(hover_id) {
                node.set_name("UiElementHoverOutline");
                node.computed_rect = Rect::new(
                    screen_elem_rect.x - 1.0,
                    screen_elem_rect.y - 1.0,
                    screen_elem_rect.width + 2.0,
                    screen_elem_rect.height + 2.0,
                );
                node.style = Style::new()
                    .border(1.2, Color::rgba(0.0, 0.85, 1.0, 0.50))
                    .border_radius(2.0);
            }
            let _ = tree.add_child(parent_id, hover_id);
        }
    }

    // ── 7. Canvas Info Footer Indicator ───────────────────────────────────────
    let info_text = format!(
        "Canvas: {:.0}x{:.0} | Zoom: {:.0}%",
        screen_w,
        screen_h,
        params.state.zoom * 100.0
    );
    let footer_id = tree.create_node();
    if let Some(node) = tree.get_mut(footer_id) {
        node.set_name("UiDesignerFooterInfo");
        node.computed_rect = Rect::new(
            canvas_rect.right() - 220.0,
            canvas_rect.bottom() - 20.0,
            212.0,
            16.0,
        );
        node.set_text(info_text);
        node.font_size = 10.0;
        node.line_height = 14.0;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.55, 0.60, 0.72, 0.80);
    }
    let _ = tree.add_child(parent_id, footer_id);
}

/// Builds vertical and horizontal background grid lines on the virtual canvas.
fn build_canvas_grid(tree: &mut UiTree, parent_id: WidgetId, canvas_rect: Rect, base_scale: f32) {
    let grid_step = 64.0 * base_scale;
    if grid_step < 8.0 {
        return;
    }

    let minor_color = Color::rgba(1.0, 1.0, 1.0, 0.035);
    let major_color = Color::rgba(1.0, 1.0, 1.0, 0.080);

    // Vertical lines
    let mut x = canvas_rect.x;
    let mut col = 0;
    while x <= canvas_rect.right() {
        let is_major = col % 4 == 0;
        let line_id = tree.create_node();
        if let Some(node) = tree.get_mut(line_id) {
            node.set_name("CanvasGridLineV");
            node.computed_rect = Rect::new(x, canvas_rect.y, 1.0, canvas_rect.height);
            node.style = Style::new().background(if is_major { major_color } else { minor_color });
        }
        let _ = tree.add_child(parent_id, line_id);
        x += grid_step;
        col += 1;
    }

    // Horizontal lines
    let mut y = canvas_rect.y;
    let mut row = 0;
    while y <= canvas_rect.bottom() {
        let is_major = row % 4 == 0;
        let line_id = tree.create_node();
        if let Some(node) = tree.get_mut(line_id) {
            node.set_name("CanvasGridLineH");
            node.computed_rect = Rect::new(canvas_rect.x, y, canvas_rect.width, 1.0);
            node.style = Style::new().background(if is_major { major_color } else { minor_color });
        }
        let _ = tree.add_child(parent_id, line_id);
        y += grid_step;
        row += 1;
    }
}