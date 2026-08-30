// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Appearance and Color Palette Inspector Card Builder
//!
//! Renders object color swatch, HEX text input, and quick-select palette swatches.

use super::registry::ComponentRenderContext;
use irisui::prelude::*;

/// Builds the `🎨 Appearance` card in the `UiTree` and returns the computed height.
pub fn build_appearance_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
) -> f32 {
    let padding = 8.0;
    let swatch_size = 16.0;
    let swatch_gap = 6.0;

    // Calculate how many palette rows will be required
    let max_row_w = (ctx.card_w - padding * 2.0).max(100.0);
    let swatches_per_row = ((max_row_w + swatch_gap) / (swatch_size + swatch_gap)).floor() as usize;
    let swatches_per_row = swatches_per_row.max(7);

    let total_swatches = 7 + ctx.params.saved_swatches.len();
    let num_palette_rows = total_swatches.div_ceil(swatches_per_row);
    let num_palette_rows = num_palette_rows.max(1);

    let palette_h = (num_palette_rows as f32) * (swatch_size + swatch_gap) - swatch_gap;
    let card_h = 24.0 + 22.0 + 4.0 + 22.0 + 6.0 + palette_h + padding * 2.0 + 2.0;
    let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

    // 1. Outer Card Container
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("AppearanceCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0);
    }
    let _ = tree.add_child(parent_id, card_id);

    // 2. Card Header
    let hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(hdr_id) {
        node.set_name("AppearanceHeader");
        node.set_text("🎨 Appearance");
        node.font_size = 11.5;
        node.line_height = 20.0;
        node.text_color = Color::rgba(0.886, 0.894, 0.918, 1.0);
        node.computed_rect = Rect::new(
            ctx.base_x + padding,
            ctx.base_y + padding,
            ctx.card_w - padding * 2.0,
            20.0,
        );
    }
    let _ = tree.add_child(card_id, hdr_id);

    let mut cur_y = ctx.base_y + padding + 22.0;

    // Fetch current ECS Color or fallback to light blue
    let obj_color = ctx
        .world
        .get::<&ae_core::ecs::Color>(ctx.entity)
        .map(|c| Color::rgba(c.r, c.g, c.b, c.a))
        .unwrap_or(Color::rgba(0.60, 0.75, 0.95, 1.0));

    // 3. Row 1: Object Color: [Swatch]  Hex: [#6699cc]
    let row1_h = 22.0;
    let lbl1_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl1_id) {
        node.set_name("ObjectColorLabel");
        node.set_text("Object Color:");
        node.font_size = 11.0;
        node.line_height = row1_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding, cur_y, 75.0, row1_h);
    }
    let _ = tree.add_child(card_id, lbl1_id);

    // Color Swatch Box
    let swatch_rect = Rect::new(ctx.base_x + padding + 78.0, cur_y + 2.0, 38.0, row1_h - 4.0);
    ctx.targets.color_swatch_rect = Some(swatch_rect);
    let is_swatch_hovered = swatch_rect.contains_point(ctx.params.cursor_pos);
    let swatch_border = if is_swatch_hovered || ctx.params.is_color_picker_open {
        Color::rgba(1.0, 1.0, 1.0, 0.95)
    } else {
        Color::rgba(0.85, 0.88, 0.95, 0.70)
    };

    let swatch_id = tree.create_node();
    if let Some(node) = tree.get_mut(swatch_id) {
        node.set_name("ColorSwatchBox");
        node.computed_rect = swatch_rect;
        node.style = Style::new()
            .background(obj_color)
            .border(1.0, swatch_border)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, swatch_id);

    // Hex label
    let hex_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(hex_lbl_id) {
        node.set_name("HexPrefixLabel");
        node.set_text("Hex:");
        node.font_size = 11.0;
        node.line_height = row1_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(swatch_rect.right() + 8.0, cur_y, 30.0, row1_h);
    }
    let _ = tree.add_child(card_id, hex_lbl_id);

    // Hex Input Box (Compact ~60px wide)
    let hex_box_w = 64.0;
    let hex_rect = Rect::new(swatch_rect.right() + 38.0, cur_y, hex_box_w, row1_h);
    ctx.targets.hex_input_rect = Some(hex_rect);
    let is_hex_focused = ctx.params.active_hex_buffer.is_some();
    let is_hex_hovered = hex_rect.contains_point(ctx.params.cursor_pos);

    let (hex_bg, hex_border) = if is_hex_focused {
        (
            Color::rgba(0.180, 0.190, 0.220, 1.0),
            Color::rgba(0.85, 0.88, 0.98, 0.95),
        )
    } else if is_hex_hovered {
        (
            Color::rgba(0.180, 0.190, 0.220, 1.0),
            Color::rgba(0.35, 0.38, 0.45, 0.95),
        )
    } else {
        (
            Color::rgba(0.157, 0.165, 0.188, 0.98),
            Color::rgba(0.212, 0.220, 0.259, 0.85),
        )
    };

    let hex_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(hex_box_id) {
        node.set_name("HexInputBox");
        node.computed_rect = hex_rect;
        node.style = Style::new()
            .background(hex_bg)
            .border(1.0, hex_border)
            .border_radius(5.0);
    }
    let _ = tree.add_child(card_id, hex_box_id);

    let hex_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(hex_txt_id) {
        node.set_name("HexInputText");
        let hex_val = if let Some(buf) = ctx.params.active_hex_buffer {
            if ctx.params.blink_caret {
                format!("{}|", buf)
            } else {
                buf.to_string()
            }
        } else if ctx.params.inspector_color_hex.is_empty() {
            "#6699cc".to_string()
        } else {
            ctx.params.inspector_color_hex.to_string()
        };
        node.set_text(hex_val);
        node.font_size = 10.5;
        node.line_height = row1_h;
        node.text_align = TextAlign::Center;
        node.text_color = if is_hex_focused {
            Color::WHITE
        } else {
            Color::rgba(0.886, 0.894, 0.918, 1.0)
        };
        node.computed_rect = hex_rect;
    }
    let _ = tree.add_child(hex_box_id, hex_txt_id);

    cur_y += row1_h + 4.0;

    // 4. Row 2: Add to Palette: [+] [🗑]
    let lbl2_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl2_id) {
        node.set_name("AddPaletteLabel");
        node.set_text("Add to Palette:");
        node.font_size = 11.0;
        node.line_height = row1_h;
        node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
        node.computed_rect = Rect::new(ctx.base_x + padding, cur_y, 88.0, row1_h);
    }
    let _ = tree.add_child(card_id, lbl2_id);

    // [+] Button
    let btn_size = 18.0;
    let add_pal_rect = Rect::new(ctx.base_x + padding + 90.0, cur_y + 2.0, btn_size, btn_size);
    ctx.targets.add_palette_btn_rect = Some(add_pal_rect);
    let is_add_hovered = add_pal_rect.contains_point(ctx.params.cursor_pos);

    let add_pal_id = tree.create_node();
    if let Some(node) = tree.get_mut(add_pal_id) {
        node.set_name("AddPaletteBtn");
        node.computed_rect = add_pal_rect;
        let (bg, border, text_col) = if is_add_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.82, 0.84, 0.88, 1.0),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
        node.set_text("+");
        node.font_size = 13.0;
        node.line_height = btn_size;
        node.text_align = TextAlign::Center;
        node.text_color = text_col;
    }
    let _ = tree.add_child(card_id, add_pal_id);

    // [🗑] Button
    let clr_pal_rect = Rect::new(add_pal_rect.right() + 4.0, cur_y + 2.0, btn_size, btn_size);
    ctx.targets.clear_palette_btn_rect = Some(clr_pal_rect);
    let is_clr_hovered = clr_pal_rect.contains_point(ctx.params.cursor_pos);

    let clr_pal_id = tree.create_node();
    if let Some(node) = tree.get_mut(clr_pal_id) {
        node.set_name("ClearPaletteBtn");
        node.computed_rect = clr_pal_rect;
        let (bg, border, text_col) = if is_clr_hovered {
            (
                Color::rgba(0.35, 0.10, 0.10, 0.95),
                Color::rgba(0.70, 0.18, 0.18, 0.85),
                Color::rgba(1.0, 0.40, 0.40, 1.0),
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.70, 0.73, 0.80, 0.90),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
        node.set_text("🗑");
        node.font_size = 10.5;
        node.line_height = btn_size;
        node.text_align = TextAlign::Center;
        node.text_color = text_col;
    }
    let _ = tree.add_child(card_id, clr_pal_id);

    cur_y += row1_h + 5.0;

    // 5. Row 3+: Palette Swatches (Default 7 + User Saved Swatches)
    let default_palette = [
        Color::rgba(1.0, 1.0, 1.0, 1.0),
        Color::rgba(0.55, 0.58, 0.64, 1.0),
        Color::rgba(0.10, 0.10, 0.12, 1.0),
        Color::rgba(0.95, 0.22, 0.22, 1.0),
        Color::rgba(0.15, 0.88, 0.35, 1.0),
        Color::rgba(0.20, 0.45, 0.98, 1.0),
        Color::rgba(0.98, 0.90, 0.15, 1.0),
    ];

    let mut sw_x = ctx.base_x + padding;
    let start_x = sw_x;

    // 5a. Render default 7 palette swatches
    for (i, &col) in default_palette.iter().enumerate() {
        if sw_x + swatch_size > ctx.base_x + ctx.card_w - padding {
            sw_x = start_x;
            cur_y += swatch_size + swatch_gap;
        }

        let sw_rect = Rect::new(sw_x, cur_y, swatch_size, swatch_size);
        let is_hovered = sw_rect.contains_point(ctx.params.cursor_pos);

        let sw_id = tree.create_node();
        if let Some(node) = tree.get_mut(sw_id) {
            node.set_name(format!("PaletteDef_{}", i));
            node.computed_rect = sw_rect;
            let border_col = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.212, 0.220, 0.259, 0.85)
            };
            node.style = Style::new()
                .background(col)
                .border(1.0, border_col)
                .border_radius(3.0);
        }
        let _ = tree.add_child(card_id, sw_id);

        ctx.targets.palette_swatches.push((i, sw_rect, col));
        sw_x += swatch_size + swatch_gap;
    }

    // 5b. Render user saved swatches
    for (idx, &s) in ctx.params.saved_swatches.iter().enumerate() {
        if sw_x + swatch_size > ctx.base_x + ctx.card_w - padding {
            sw_x = start_x;
            cur_y += swatch_size + swatch_gap;
        }

        let col = Color::rgba(s[0], s[1], s[2], s[3]);
        let sw_rect = Rect::new(sw_x, cur_y, swatch_size, swatch_size);
        let is_hovered = sw_rect.contains_point(ctx.params.cursor_pos);

        let sw_id = tree.create_node();
        if let Some(node) = tree.get_mut(sw_id) {
            node.set_name(format!("PaletteSaved_{}", idx));
            node.computed_rect = sw_rect;
            let border_col = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.35, 0.38, 0.45, 0.85)
            };
            node.style = Style::new()
                .background(col)
                .border(1.0, border_col)
                .border_radius(3.0);
        }
        let _ = tree.add_child(card_id, sw_id);

        ctx.targets.palette_swatches.push((7 + idx, sw_rect, col));
        sw_x += swatch_size + swatch_gap;
    }

    card_h
}