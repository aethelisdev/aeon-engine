// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Virtual Canvas & Element Projection
//!
//! Renders the letterboxed virtual canvas surface, background grid lines,
//! and projected in-game UI element hierarchies using declarative [`UiScope`]
//! absolute layout and 64-bit hardware semantic tags in proper local coordinates.
//!

use super::anchors::{AnchorGuideParams, build_anchor_pin_and_guide};
use super::toolbar::UI_DESIGNER_TOOLBAR_HEIGHT;
use super::types::{
    UI_DESIGNER_TAG_CANVAS_BOARD, UI_DESIGNER_TAG_LETTERBOX, UiDesignerCanvasMetrics,
    UiDesignerPanelParams, UiElementDragContext, encode_element_tag,
};
use ae_core::ecs::UiElement;
use ae_core::ui::{UiDrawCommand, UiLayoutResolver, UiTextAlignment};
use irisui::prelude::*;

/// Builds the virtual canvas viewport, background grid, and rendered UI elements.
pub fn build_designer_canvas(
    scope: &mut UiScope<'_>,
    params: &UiDesignerPanelParams<'_>,
    out_contexts: &mut Vec<UiElementDragContext>,
) -> UiDesignerCanvasMetrics {
    let avail_w = params.panel_rect.width;
    let avail_h = (params.panel_rect.height - UI_DESIGNER_TOOLBAR_HEIGHT).max(10.0);

    let [screen_w, screen_h] = params.state.aspect_ratio.resolution();

    // ── 1. Compute Fitted Virtual Canvas Dimensions & Local Offsets ───────────
    let margin = 32.0;
    let max_w = (avail_w - margin * 2.0).max(100.0);
    let max_h = (avail_h - margin * 2.0).max(100.0);

    let scale_w = max_w / screen_w;
    let scale_h = max_h / screen_h;
    let base_scale = scale_w.min(scale_h) * params.state.zoom;

    let canvas_w = screen_w * base_scale;
    let canvas_h = screen_h * base_scale;

    let canvas_local_x = (avail_w - canvas_w) * 0.5 + params.state.pan_offset[0];
    let canvas_local_y = (avail_h - canvas_h) * 0.5 + params.state.pan_offset[1];

    let canvas_screen_x = params.panel_rect.x + canvas_local_x;
    let canvas_screen_y = params.panel_rect.y + UI_DESIGNER_TOOLBAR_HEIGHT + canvas_local_y;

    let canvas_screen_rect = Rect::new(canvas_screen_x, canvas_screen_y, canvas_w, canvas_h);

    let metrics = UiDesignerCanvasMetrics {
        panel_rect: params.panel_rect,
        canvas_rect: canvas_screen_rect,
        resolution: [screen_w, screen_h],
        base_scale,
        current_zoom: params.state.zoom,
        snap_grid: params.state.snap_grid,
    };

    let to_local_pos = |cx: f32, cy: f32| -> Point {
        Point::new((cx / screen_w) * canvas_w, (cy / screen_h) * canvas_h)
    };

    let to_canvas_pos = |pt: Point| -> [f32; 2] {
        if canvas_w <= 0.0 || canvas_h <= 0.0 {
            return [0.0, 0.0];
        }
        let rel_x = (pt.x - canvas_screen_rect.x) / canvas_w;
        let rel_y = (pt.y - canvas_screen_rect.y) / canvas_h;
        [rel_x * screen_w, rel_y * screen_h]
    };

    // ── 2. Letterbox Outer Background (Local to PanelRoot) ────────────────────
    let letterbox_style = Style::new()
        .position_absolute()
        .left(0.0)
        .top(UI_DESIGNER_TOOLBAR_HEIGHT)
        .width(avail_w)
        .height(avail_h)
        .background(Color::rgba(0.055, 0.063, 0.086, 1.0))
        .clip_children(true);

    scope.container_tagged(
        "UiDesignerLetterbox",
        letterbox_style,
        WidgetRole::Default,
        UI_DESIGNER_TAG_LETTERBOX,
        |lb_scope| {
            // ── 3. Virtual Canvas Viewport Surface (Local to Letterbox) ─────────
            let canvas_style = Style::new()
                .position_absolute()
                .left(canvas_local_x)
                .top(canvas_local_y)
                .width(canvas_w)
                .height(canvas_h)
                .background(Color::rgba(0.086, 0.102, 0.137, 1.0))
                .border(1.5, Color::rgba(0.0, 0.706, 0.941, 0.85))
                .clip_children(true);

            lb_scope.container_tagged(
                "UiDesignerCanvasSurface",
                canvas_style,
                WidgetRole::Default,
                UI_DESIGNER_TAG_CANVAS_BOARD,
                |canvas_scope| {
                    // ── 4. Background Grid Lines (Local to CanvasSurface) ──────
                    if params.state.show_grid {
                        build_canvas_grid(canvas_scope, canvas_w, canvas_h, base_scale);
                    }

                    // ── 5. Resolve and Render In-Game UI Elements from ECS ─────
                    let mouse_canvas_pos = if canvas_screen_rect.contains_point(params.cursor_pos) {
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

                    for cmd in draw_commands.iter() {
                        match cmd {
                            UiDrawCommand::Rect {
                                rect,
                                fill_color,
                                border_color,
                                border_width,
                                border_radius,
                                ..
                            } => {
                                let min_p = to_local_pos(rect.min_x, rect.min_y);
                                let max_p = to_local_pos(rect.max_x, rect.max_y);
                                let w = (max_p.x - min_p.x).max(0.0);
                                let h = (max_p.y - min_p.y).max(0.0);

                                let mut style = Style::new()
                                    .position_absolute()
                                    .left(min_p.x)
                                    .top(min_p.y)
                                    .width(w)
                                    .height(h)
                                    .background(Color::rgba(
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
                                canvas_scope.empty_box_passive(style);
                            }
                            UiDrawCommand::Text {
                                pos,
                                text,
                                font_size,
                                color,
                                alignment,
                                ..
                            } => {
                                let p = to_local_pos(pos[0], pos[1]);
                                let scaled_font_size = (*font_size * base_scale).max(8.0);
                                let approx_w = (text.len() as f32) * (scaled_font_size * 0.65);
                                let approx_h = scaled_font_size * 1.3;

                                let (text_x, iris_align) = match alignment {
                                    UiTextAlignment::Left => (p.x, TextAlign::Left),
                                    UiTextAlignment::Center => {
                                        (p.x - approx_w * 0.5, TextAlign::Center)
                                    }
                                    UiTextAlignment::Right => (p.x - approx_w, TextAlign::Right),
                                };
                                let text_y = p.y - approx_h * 0.5;

                                let text_style = Style::new()
                                    .position_absolute()
                                    .left(text_x)
                                    .top(text_y)
                                    .width(approx_w)
                                    .height(approx_h);

                                canvas_scope.container_passive(text_style, |ts| {
                                    ts.label(
                                        text.as_str(),
                                        scaled_font_size,
                                        Color::rgba(color[0], color[1], color[2], color[3]),
                                        iris_align,
                                    );
                                });
                            }
                            UiDrawCommand::Image { rect, tint, .. } => {
                                let min_p = to_local_pos(rect.min_x, rect.min_y);
                                let max_p = to_local_pos(rect.max_x, rect.max_y);
                                let w = (max_p.x - min_p.x).max(0.0);
                                let h = (max_p.y - min_p.y).max(0.0);

                                let img_style = Style::new()
                                    .position_absolute()
                                    .left(min_p.x)
                                    .top(min_p.y)
                                    .width(w)
                                    .height(h)
                                    .background(Color::rgba(tint[0], tint[1], tint[2], tint[3]));

                                canvas_scope.empty_box_passive(img_style);
                            }
                        }
                    }

                    // ── 6. Element Tagged Nodes, Drag Contexts, & Selection Outlines ──
                    for (ent, elem) in params.world.query::<(hecs::Entity, &UiElement)>().iter() {
                        if !elem.visible {
                            continue;
                        }

                        let elem_rect = elem.compute_rect(screen_w, screen_h);
                        let local_min = to_local_pos(elem_rect.min_x, elem_rect.min_y);
                        let local_max = to_local_pos(elem_rect.max_x, elem_rect.max_y);
                        let local_w = (local_max.x - local_min.x).max(1.0);
                        let local_h = (local_max.y - local_min.y).max(1.0);
                        let local_elem_rect = Rect::new(local_min.x, local_min.y, local_w, local_h);

                        let elem_idx = out_contexts.len();
                        out_contexts.push(UiElementDragContext {
                            entity: ent,
                            anchor_origin: elem.anchor.compute_origin(screen_w, screen_h),
                            initial_offset: elem.offset,
                        });

                        let elem_tag = encode_element_tag(elem_idx);
                        let is_selected = params.selected_entity == Some(ent);

                        // Element screen-space hitbox for hover detection
                        let screen_elem_x = canvas_screen_rect.x + local_min.x;
                        let screen_elem_y = canvas_screen_rect.y + local_min.y;
                        let screen_elem_rect =
                            Rect::new(screen_elem_x, screen_elem_y, local_w, local_h);
                        let is_hovered = screen_elem_rect.contains_point(params.cursor_pos);

                        let elem_style = Style::new()
                            .position_absolute()
                            .left(local_elem_rect.x)
                            .top(local_elem_rect.y)
                            .width(local_elem_rect.width)
                            .height(local_elem_rect.height)
                            .background(Color::TRANSPARENT);

                        canvas_scope.empty_box_tagged(
                            "UiCanvasElement",
                            elem_style,
                            WidgetRole::Button,
                            elem_tag,
                        );

                        if is_selected {
                            // Glowing cyan selection outline (Passive: non-blocking hit-test)
                            let outline_style = Style::new()
                                .position_absolute()
                                .left(local_elem_rect.x - 2.0)
                                .top(local_elem_rect.y - 2.0)
                                .width(local_elem_rect.width + 4.0)
                                .height(local_elem_rect.height + 4.0)
                                .border(1.8, Color::rgba(0.0, 0.85, 1.0, 0.95))
                                .border_radius(2.0);
                            canvas_scope.empty_box_passive(outline_style);

                            // 4 corner sizing handles (Passive: non-blocking hit-test)
                            let handle_size = 6.0;
                            let corners = [
                                Point::new(local_elem_rect.x, local_elem_rect.y),
                                Point::new(
                                    local_elem_rect.x + local_elem_rect.width,
                                    local_elem_rect.y,
                                ),
                                Point::new(
                                    local_elem_rect.x,
                                    local_elem_rect.y + local_elem_rect.height,
                                ),
                                Point::new(
                                    local_elem_rect.x + local_elem_rect.width,
                                    local_elem_rect.y + local_elem_rect.height,
                                ),
                            ];
                            for corner in corners.iter() {
                                let handle_style = Style::new()
                                    .position_absolute()
                                    .left(corner.x - handle_size * 0.5)
                                    .top(corner.y - handle_size * 0.5)
                                    .width(handle_size)
                                    .height(handle_size)
                                    .background(Color::rgba(1.0, 1.0, 1.0, 0.98))
                                    .border(1.0, Color::rgba(0.0, 0.5, 0.8, 0.9))
                                    .border_radius(1.0);
                                canvas_scope.empty_box_passive(handle_style);
                            }

                            // Anchor guide lines and pins
                            if params.state.show_anchor_guides {
                                build_anchor_pin_and_guide(
                                    canvas_scope,
                                    &AnchorGuideParams {
                                        elem,
                                        local_elem_rect,
                                        screen_w,
                                        screen_h,
                                        base_scale,
                                        to_local_pos: &to_local_pos,
                                    },
                                );
                            }
                        } else if is_hovered {
                            // Passive hover outline so it doesn't occlude element clicks
                            let hover_style = Style::new()
                                .position_absolute()
                                .left(local_elem_rect.x - 1.0)
                                .top(local_elem_rect.y - 1.0)
                                .width(local_elem_rect.width + 2.0)
                                .height(local_elem_rect.height + 2.0)
                                .border(1.0, Color::rgba(0.0, 0.706, 0.941, 0.50))
                                .border_radius(2.0);
                            canvas_scope.empty_box_passive(hover_style);
                        }
                    }

                    // ── 7. Canvas Dimensions Indicator (Local to CanvasSurface) ──
                    let dim_text = format!("{:.0} × {:.0}", screen_w, screen_h);
                    let dim_style = Style::new()
                        .position_absolute()
                        .left(8.0)
                        .top(canvas_h - 22.0)
                        .width(100.0)
                        .height(16.0);
                    canvas_scope.container_passive(dim_style, |ds| {
                        ds.label(
                            dim_text.as_str(),
                            10.0,
                            Color::rgba(0.0, 0.706, 0.941, 0.75),
                            TextAlign::Left,
                        );
                    });
                },
            );
        },
    );

    metrics
}

/// Builds vertical and horizontal background grid lines on the virtual canvas.
fn build_canvas_grid(scope: &mut UiScope<'_>, canvas_w: f32, canvas_h: f32, base_scale: f32) {
    let grid_step = 64.0 * base_scale;
    if grid_step < 8.0 {
        return;
    }

    let minor_color = Color::rgba(1.0, 1.0, 1.0, 0.035);
    let major_color = Color::rgba(1.0, 1.0, 1.0, 0.080);

    // Vertical grid lines
    let mut x = 0.0;
    let mut col_idx = 0;
    while x <= canvas_w {
        let is_major = col_idx % 4 == 0;
        let line_style = Style::new()
            .position_absolute()
            .left(x)
            .top(0.0)
            .width(1.0)
            .height(canvas_h)
            .background(if is_major { major_color } else { minor_color });
        scope.empty_box_passive(line_style);
        x += grid_step;
        col_idx += 1;
    }

    // Horizontal grid lines
    let mut y = 0.0;
    let mut row_idx = 0;
    while y <= canvas_h {
        let is_major = row_idx % 4 == 0;
        let line_style = Style::new()
            .position_absolute()
            .left(0.0)
            .top(y)
            .width(canvas_w)
            .height(1.0)
            .background(if is_major { major_color } else { minor_color });
        scope.empty_box_passive(line_style);
        y += grid_step;
        row_idx += 1;
    }
}