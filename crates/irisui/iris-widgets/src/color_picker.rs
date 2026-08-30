// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hardware-Accelerated 2D HSV Color Picker (`iris-widgets::color_picker`)
//!
//! Provides a full-featured 2D Saturation-Value gradient box, vertical rainbow Hue spectrum bar,
//! interactive indicator rings, live color preview, and bidirectional RGB/HSV/HEX conversions.

use iris_core::{Color, Point, Rect, Style, TextAlign, UiTree, WidgetId};

/// Converts standard RGB components (0.0 ..= 1.0) to HSV representation.
/// Returns `(hue, saturation, value)` where:
/// - `hue`: `0.0 .. 360.0` degrees
/// - `saturation`: `0.0 ..= 1.0`
/// - `value`: `0.0 ..= 1.0`
#[inline]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max > 1e-5 { delta / max } else { 0.0 };

    let h = if delta < 1e-5 {
        0.0
    } else if (max - r).abs() < 1e-5 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-5 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Converts HSV color representation to standard `Color` (RGBA float).
/// - `h`: Hue in degrees `0.0 .. 360.0`
/// - `s`: Saturation `0.0 ..= 1.0`
/// - `v`: Value / Brightness `0.0 ..= 1.0`
#[inline]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h_norm = (h % 360.0 + 360.0) % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h_norm / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match (h_norm / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::rgba(r1 + m, g1 + m, b1 + m, 1.0)
}

/// Dynamic state tracker for the 2D HSV Color Picker widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsvColorPickerState {
    /// Active Hue in degrees `0.0 .. 360.0`.
    pub hue: f32,
    /// Active Saturation `0.0 ..= 1.0`.
    pub saturation: f32,
    /// Active Value / Brightness `0.0 ..= 1.0`.
    pub value: f32,
    /// Active Alpha transparency `0.0 ..= 1.0`.
    pub alpha: f32,
}

impl Default for HsvColorPickerState {
    fn default() -> Self {
        Self {
            hue: 180.0,
            saturation: 0.8,
            value: 0.9,
            alpha: 1.0,
        }
    }
}

impl HsvColorPickerState {
    /// Constructs a new `HsvColorPickerState` from an existing `Color`.
    #[must_use]
    pub fn from_color(color: Color) -> Self {
        let (h, s, v) = rgb_to_hsv(color.r, color.g, color.b);
        Self {
            hue: h,
            saturation: s,
            value: v,
            alpha: color.a,
        }
    }

    /// Converts the current HSV state to a concrete `Color` instance.
    #[must_use]
    pub fn to_color(&self) -> Color {
        let mut c = hsv_to_rgb(self.hue, self.saturation, self.value);
        c.a = self.alpha;
        c
    }

    /// Returns the uppercase HEX string representation (e.g. `"#38BDF8"`).
    #[must_use]
    pub fn to_hex(&self) -> String {
        let c = self.to_color();
        let r = (c.r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (c.g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (c.b.clamp(0.0, 1.0) * 255.0) as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    /// Updates Saturation and Value based on an absolute click/drag point over the 2D SV box.
    pub fn update_from_sv_point(&mut self, point: Point, sv_rect: Rect) {
        if sv_rect.width <= 0.0 || sv_rect.height <= 0.0 {
            return;
        }
        let rel_x = (point.x - sv_rect.x).clamp(0.0, sv_rect.width);
        let rel_y = (point.y - sv_rect.y).clamp(0.0, sv_rect.height);

        self.saturation = (rel_x / sv_rect.width).clamp(0.0, 1.0);
        self.value = (1.0 - (rel_y / sv_rect.height)).clamp(0.0, 1.0);
    }

    /// Updates Hue based on an absolute click/drag point over the vertical Hue bar.
    pub fn update_from_hue_point(&mut self, point: Point, hue_rect: Rect) {
        if hue_rect.height <= 0.0 {
            return;
        }
        let rel_y = (point.y - hue_rect.y).clamp(0.0, hue_rect.height);
        self.hue = (rel_y / hue_rect.height * 360.0).clamp(0.0, 360.0);
    }
}

/// Hit-testing bounding targets for interactive 2D HSV color picker elements.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HsvColorPickerTargets {
    /// Bounding rectangle of the entire floating popup card.
    pub card_rect: Rect,
    /// Bounding rectangle of the 2D Saturation-Value gradient box.
    pub sv_box_rect: Rect,
    /// Bounding rectangle of the vertical Rainbow Hue spectrum bar.
    pub hue_bar_rect: Rect,
    /// Bounding rectangle of the live color preview bar.
    pub preview_rect: Rect,
    /// Bounding rectangle of the close `✖` button.
    pub close_btn_rect: Option<Rect>,
}

/// Builder for constructing retained 2D HSV Color Pickers in the `UiTree`.
pub struct HsvColorPickerBuilder<'a> {
    tree: &'a mut UiTree,
    parent_id: WidgetId,
    anchor_rect: Rect,
    state: HsvColorPickerState,
    cursor_pos: Point,
    screen_rect: Rect,
}

impl<'a> HsvColorPickerBuilder<'a> {
    /// Creates a new `HsvColorPickerBuilder`.
    pub fn new(
        tree: &'a mut UiTree,
        parent_id: WidgetId,
        anchor_rect: Rect,
        state: HsvColorPickerState,
        cursor_pos: Point,
        screen_rect: Rect,
    ) -> Self {
        Self {
            tree,
            parent_id,
            anchor_rect,
            state,
            cursor_pos,
            screen_rect,
        }
    }

    /// Builds the complete 2D HSV Color Picker layout into the `UiTree` and returns targets.
    pub fn build(self) -> (WidgetId, HsvColorPickerTargets) {
        let tree = self.tree;
        let mut targets = HsvColorPickerTargets::default();

        let popup_w = 206.0;
        let popup_h = 224.0;
        let padding = 8.0;

        let popup_x = self
            .anchor_rect
            .x
            .min(self.screen_rect.right() - popup_w - 6.0)
            .max(self.screen_rect.x + 6.0);
        let popup_y = if self.anchor_rect.bottom() + popup_h > self.screen_rect.bottom() - 30.0 {
            (self.anchor_rect.y - popup_h - 4.0).max(30.0)
        } else {
            self.anchor_rect.bottom() + 4.0
        };

        let card_rect = Rect::new(popup_x, popup_y, popup_w, popup_h);
        targets.card_rect = card_rect;

        // 1. Root Floating Popup Container
        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("ColorPickerPopupCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.082, 0.086, 0.102, 0.98))
                .border(1.0, Color::rgba(0.200, 0.208, 0.235, 0.95))
                .border_radius(6.0)
                .box_shadow(0.0, 10.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.80));
        }
        let _ = tree.add_child(self.parent_id, card_id);

        let mut cur_y = popup_y + padding;

        // 2. Header Row: 🎨 Color Picker   [✖]
        let hdr_w = popup_w - padding * 2.0;
        let hdr_lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(hdr_lbl_id) {
            node.set_name("ColorPickerHeaderLabel");
            node.set_text("🎨 Color Picker");
            node.font_size = 11.5;
            node.line_height = 18.0;
            node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
            node.computed_rect = Rect::new(popup_x + padding, cur_y, hdr_w - 20.0, 18.0);
        }
        let _ = tree.add_child(card_id, hdr_lbl_id);

        // Close button
        let close_size = 16.0;
        let close_rect = Rect::new(
            popup_x + popup_w - padding - close_size,
            cur_y + 1.0,
            close_size,
            close_size,
        );
        targets.close_btn_rect = Some(close_rect);
        let is_close_hovered = close_rect.contains_point(self.cursor_pos);

        let close_id = tree.create_node();
        if let Some(node) = tree.get_mut(close_id) {
            node.set_name("ColorPickerCloseButton");
            node.computed_rect = close_rect;
            let (bg, text_col) = if is_close_hovered {
                (Color::rgba(0.35, 0.12, 0.12, 0.95), Color::WHITE)
            } else {
                (
                    Color::rgba(0.157, 0.165, 0.188, 0.98),
                    Color::rgba(0.70, 0.73, 0.80, 0.90),
                )
            };
            node.style = Style::new().background(bg).border_radius(3.0);
            node.set_text("✖");
            node.font_size = 9.0;
            node.line_height = close_size;
            node.text_align = TextAlign::Center;
            node.text_color = text_col;
        }
        let _ = tree.add_child(card_id, close_id);

        cur_y += 22.0;

        // 3. Middle Section: 2D SV Box (Left) + Vertical Hue Bar (Right)
        let sv_w = 160.0;
        let sv_h = 130.0;
        let hue_w = 18.0;
        let gap = 8.0;

        let sv_rect = Rect::new(popup_x + padding, cur_y, sv_w, sv_h);
        targets.sv_box_rect = sv_rect;

        let hue_rect = Rect::new(sv_rect.right() + gap, cur_y, hue_w, sv_h);
        targets.hue_bar_rect = hue_rect;

        // 3a. Render 2D Saturation-Value Matrix
        let sv_container_id = tree.create_node();
        if let Some(node) = tree.get_mut(sv_container_id) {
            node.set_name("ColorPickerSvContainer");
            node.computed_rect = sv_rect;
            node.style = Style::new()
                .border(1.0, Color::rgba(0.25, 0.28, 0.35, 0.90))
                .border_radius(4.0)
                .clip_children(true);
        }
        let _ = tree.add_child(card_id, sv_container_id);

        let grid_nx = 16;
        let grid_ny = 13;
        let cell_w = sv_w / (grid_nx as f32);
        let cell_h = sv_h / (grid_ny as f32);

        for j in 0..grid_ny {
            for i in 0..grid_nx {
                let s = (i as f32 + 0.5) / (grid_nx as f32);
                let v = 1.0 - (j as f32 + 0.5) / (grid_ny as f32);
                let col = hsv_to_rgb(self.state.hue, s, v);

                let cell_rect = Rect::new(
                    sv_rect.x + (i as f32) * cell_w,
                    sv_rect.y + (j as f32) * cell_h,
                    cell_w + 0.2, // Small subpixel overlap for  gradient
                    cell_h + 0.2,
                );

                let cell_id = tree.create_node();
                if let Some(node) = tree.get_mut(cell_id) {
                    node.set_name(format!("SvCell_{}_{}", i, j));
                    node.computed_rect = cell_rect;
                    node.style = Style::new().background(col);
                }
                let _ = tree.add_child(sv_container_id, cell_id);
            }
        }

        // Indicator Ring on SV Box
        let ring_size = 10.0;
        let ring_x = (sv_rect.x + self.state.saturation * sv_w - ring_size * 0.5)
            .clamp(sv_rect.x, sv_rect.right() - ring_size);
        let ring_y = (sv_rect.y + (1.0 - self.state.value) * sv_h - ring_size * 0.5)
            .clamp(sv_rect.y, sv_rect.bottom() - ring_size);
        let ring_rect = Rect::new(ring_x, ring_y, ring_size, ring_size);

        let ring_id = tree.create_node();
        if let Some(node) = tree.get_mut(ring_id) {
            node.set_name("ColorPickerSvIndicatorRing");
            node.computed_rect = ring_rect;
            node.style = Style::new()
                .border(2.0, Color::WHITE)
                .border_radius(5.0)
                .box_shadow(0.0, 1.0, 3.0, Color::rgba(0.0, 0.0, 0.0, 0.90));
        }
        let _ = tree.add_child(card_id, ring_id);

        // 3b. Render Vertical Rainbow Hue Bar
        let hue_container_id = tree.create_node();
        if let Some(node) = tree.get_mut(hue_container_id) {
            node.set_name("ColorPickerHueContainer");
            node.computed_rect = hue_rect;
            node.style = Style::new()
                .border(1.0, Color::rgba(0.25, 0.28, 0.35, 0.90))
                .border_radius(4.0)
                .clip_children(true);
        }
        let _ = tree.add_child(card_id, hue_container_id);

        let hue_steps = 26;
        let step_h = sv_h / (hue_steps as f32);

        for step in 0..hue_steps {
            let h_val = (step as f32 + 0.5) / (hue_steps as f32) * 360.0;
            let col = hsv_to_rgb(h_val, 1.0, 1.0);

            let strip_rect = Rect::new(
                hue_rect.x,
                hue_rect.y + (step as f32) * step_h,
                hue_w,
                step_h + 0.2, // Small overlap for silky smooth gradient
            );

            let strip_id = tree.create_node();
            if let Some(node) = tree.get_mut(strip_id) {
                node.set_name(format!("HueStrip_{}", step));
                node.computed_rect = strip_rect;
                node.style = Style::new().background(col);
            }
            let _ = tree.add_child(hue_container_id, strip_id);
        }

        // Indicator Bar on Hue Slider
        let hue_indicator_y = (hue_rect.y + (self.state.hue / 360.0) * sv_h - 2.0)
            .clamp(hue_rect.y, hue_rect.bottom() - 4.0);
        let hue_ind_rect = Rect::new(hue_rect.x - 2.0, hue_indicator_y, hue_w + 4.0, 4.0);

        let hue_ind_id = tree.create_node();
        if let Some(node) = tree.get_mut(hue_ind_id) {
            node.set_name("ColorPickerHueIndicator");
            node.computed_rect = hue_ind_rect;
            node.style = Style::new()
                .background(Color::WHITE)
                .border(1.0, Color::BLACK)
                .border_radius(2.0)
                .box_shadow(0.0, 1.0, 3.0, Color::rgba(0.0, 0.0, 0.0, 0.90));
        }
        let _ = tree.add_child(card_id, hue_ind_id);

        cur_y += sv_h + 8.0;

        // 4. Bottom Row: Live Color Preview Bar + Dynamic HEX Text + RGB Values
        let current_color = self.state.to_color();
        let preview_h = 24.0;
        let preview_rect = Rect::new(popup_x + padding, cur_y, hdr_w, preview_h);
        targets.preview_rect = preview_rect;

        let prev_id = tree.create_node();
        if let Some(node) = tree.get_mut(prev_id) {
            node.set_name("ColorPickerPreviewBar");
            node.computed_rect = preview_rect;
            let hex_label = self.state.to_hex();
            let r = (current_color.r * 255.0) as u8;
            let g = (current_color.g * 255.0) as u8;
            let b = (current_color.b * 255.0) as u8;

            let text_col = if self.state.value > 0.5 && self.state.saturation < 0.6 {
                Color::BLACK
            } else {
                Color::WHITE
            };

            node.set_text(format!("{}   R:{} G:{} B:{}", hex_label, r, g, b));
            node.font_size = 11.0;
            node.line_height = preview_h;
            node.text_align = TextAlign::Center;
            node.text_color = text_col;
            node.style = Style::new()
                .background(current_color)
                .border(1.0, Color::rgba(0.35, 0.38, 0.45, 0.90))
                .border_radius(4.0);
        }
        let _ = tree.add_child(card_id, prev_id);

        (card_id, targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv_and_back() {
        // Red
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
        let col = hsv_to_rgb(h, s, v);
        assert!((col.r - 1.0).abs() < 0.01);
        assert!((col.g - 0.0).abs() < 0.01);
        assert!((col.b - 0.0).abs() < 0.01);

        // Cyan
        let (h, s, v) = rgb_to_hsv(0.0, 1.0, 1.0);
        assert!((h - 180.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
        let col = hsv_to_rgb(h, s, v);
        assert!((col.r - 0.0).abs() < 0.01);
        assert!((col.g - 1.0).abs() < 0.01);
        assert!((col.b - 1.0).abs() < 0.01);

        // White
        let (h, s, v) = rgb_to_hsv(1.0, 1.0, 1.0);
        assert!((s - 0.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
        let col = hsv_to_rgb(h, s, v);
        assert!((col.r - 1.0).abs() < 0.01);
        assert!((col.g - 1.0).abs() < 0.01);
        assert!((col.b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hsv_color_picker_state_mutations() {
        let mut state = HsvColorPickerState::from_color(Color::rgba(0.2, 0.5, 0.8, 1.0));
        assert!(state.hue > 190.0 && state.hue < 230.0);
        assert!(state.saturation > 0.6);
        assert!(state.value > 0.7);

        let sv_rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        state.update_from_sv_point(Point::new(60.0, 30.0), sv_rect);
        assert!((state.saturation - 0.5).abs() < 0.01);
        assert!((state.value - 0.8).abs() < 0.01);

        let hue_rect = Rect::new(120.0, 10.0, 20.0, 100.0);
        state.update_from_hue_point(Point::new(125.0, 60.0), hue_rect);
        assert!((state.hue - 180.0).abs() < 0.01);

        let hex = state.to_hex();
        assert!(hex.starts_with('#') && hex.len() == 7);
    }

    #[test]
    fn test_hsv_color_picker_builder() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let _ = tree.set_root(root);

        let state = HsvColorPickerState::default();
        let (card_id, targets) = HsvColorPickerBuilder::new(
            &mut tree,
            root,
            Rect::new(50.0, 50.0, 40.0, 20.0),
            state,
            Point::new(100.0, 100.0),
            Rect::new(0.0, 0.0, 800.0, 600.0),
        )
        .build();

        assert!(tree.get(card_id).is_some());
        assert!(targets.card_rect.width > 150.0);
        assert!(targets.sv_box_rect.width > 100.0);
        assert!(targets.hue_bar_rect.width > 10.0);
        assert!(targets.close_btn_rect.is_some());
    }
}