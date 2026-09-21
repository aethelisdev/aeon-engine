// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Anchor Guidelines & Origin Pins
//!
//! Renders visual anchor origin pins (⚓) and orthogonal guidelines connecting
//! elements to their canvas anchors for intuitive responsive HUD design.
//!

use ae_core::ecs::{UiAnchor, UiElement};
use irisui::prelude::*;

/// Descriptor parameters for rendering visual anchor guidelines and origin pins.
pub struct AnchorGuideParams<'a, F: Fn(f32, f32) -> Point> {
    /// Target UI element with anchor configuration.
    pub elem: &'a UiElement,
    /// Computed screen bounding rectangle of the target element.
    pub screen_elem_rect: Rect,
    /// Reference canvas width in virtual pixels.
    pub screen_w: f32,
    /// Reference canvas height in virtual pixels.
    pub screen_h: f32,
    /// Base scaling factor converting virtual pixels to screen space.
    pub base_scale: f32,
    /// Closure mapping virtual canvas coordinates `(x, y)` to screen `Point`.
    pub to_screen_pos: &'a F,
}

/// Builds visual anchor guidelines and origin pin widgets for a selected UI element.
pub fn build_anchor_pin_and_guide<F: Fn(f32, f32) -> Point>(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AnchorGuideParams<'_, F>,
) {
    let [anchor_x, anchor_y] = params
        .elem
        .anchor
        .compute_origin(params.screen_w, params.screen_h);
    let anchor_p = (params.to_screen_pos)(anchor_x, anchor_y);
    let elem_center = Point::new(
        params.screen_elem_rect.x + params.screen_elem_rect.width * 0.5,
        params.screen_elem_rect.y + params.screen_elem_rect.height * 0.5,
    );

    let guide_color = Color::rgba(1.0, 0.80, 0.20, 0.70);

    // ── 1. Horizontal Guideline Segment ───────────────────────────────────────
    let min_x = anchor_p.x.min(elem_center.x);
    let line_w = (anchor_p.x - elem_center.x).abs();
    if line_w > 1.0 {
        let h_line_id = tree.create_node();
        if let Some(node) = tree.get_mut(h_line_id) {
            node.set_name("UiDesignerAnchorGuideH");
            node.computed_rect = Rect::new(min_x, anchor_p.y - 0.5, line_w, 1.0);
            node.style = Style::new().background(guide_color);
        }
        let _ = tree.add_child(parent_id, h_line_id);
    }

    // ── 2. Vertical Guideline Segment ─────────────────────────────────────────
    let min_y = anchor_p.y.min(elem_center.y);
    let line_h = (anchor_p.y - elem_center.y).abs();
    if line_h > 1.0 {
        let v_line_id = tree.create_node();
        if let Some(node) = tree.get_mut(v_line_id) {
            node.set_name("UiDesignerAnchorGuideV");
            node.computed_rect = Rect::new(elem_center.x - 0.5, min_y, 1.0, line_h);
            node.style = Style::new().background(guide_color);
        }
        let _ = tree.add_child(parent_id, v_line_id);
    }

    // ── 3. Anchor Origin Pin ──────────────────────────────────────────────────
    let pin_radius = (4.5 * params.base_scale.max(0.8)).clamp(3.0, 8.0);
    let pin_id = tree.create_node();
    if let Some(node) = tree.get_mut(pin_id) {
        node.set_name("UiDesignerAnchorPin");
        node.computed_rect = Rect::new(
            anchor_p.x - pin_radius,
            anchor_p.y - pin_radius,
            pin_radius * 2.0,
            pin_radius * 2.0,
        );
        node.style = Style::new()
            .background(Color::rgba(1.0, 0.75, 0.15, 0.95))
            .border(1.0, Color::rgba(0.20, 0.15, 0.05, 0.90))
            .border_radius(pin_radius);
    }
    let _ = tree.add_child(parent_id, pin_id);

    // ── 4. Anchor Micro Label ─────────────────────────────────────────────────
    let anchor_name = match params.elem.anchor {
        UiAnchor::TopLeft => "TopLeft",
        UiAnchor::TopCenter => "TopCenter",
        UiAnchor::TopRight => "TopRight",
        UiAnchor::CenterLeft => "CenterLeft",
        UiAnchor::Center => "Center",
        UiAnchor::CenterRight => "CenterRight",
        UiAnchor::BottomLeft => "BottomLeft",
        UiAnchor::BottomCenter => "BottomCenter",
        UiAnchor::BottomRight => "BottomRight",
    };

    let label_id = tree.create_node();
    if let Some(node) = tree.get_mut(label_id) {
        node.set_name("UiDesignerAnchorLabel");
        node.set_text(format!("⚓ {}", anchor_name));
        node.font_size = (9.0 * params.base_scale.max(0.75)).clamp(8.0, 12.0);
        node.line_height = 12.0;
        node.text_align = TextAlign::Left;
        node.text_color = Color::rgba(1.0, 0.85, 0.35, 0.95);
        node.computed_rect = Rect::new(anchor_p.x + 6.0, anchor_p.y - 14.0, 100.0, 14.0);
    }
    let _ = tree.add_child(parent_id, label_id);
}