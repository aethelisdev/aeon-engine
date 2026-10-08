// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating 2D HSV Color Picker Popup Builder
//!
//! Renders hardware-accelerated GPU SDF floating popup with a 2D Saturation-Value box,
//! vertical Rainbow Hue spectrum bar, live color preview, and close button.
//!
//! > **Tech Debt:** 234-node HSV color picker cell explosion is temporary;
//! > will be replaced by a single GPU SDF gradient quad/shader primitive in irisui.

use super::appearance::appearance_color_swatch_tag;
use super::tags::{
    TAG_INSPECTOR_COLOR_PICKER_CARD, TAG_INSPECTOR_COLOR_PICKER_CLOSE,
    TAG_INSPECTOR_COLOR_PICKER_HUE_BAR, TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
};
use super::types::InspectorPanelParams;
use irisui::prelude::*;

/// Builds the floating 2D HSV Color Picker popup for the currently active Inspector entity using declarative [`UiScope`].
///
/// **Tech Debt:** 234-node HSV color picker cell explosion is temporary; will be replaced by a single GPU SDF gradient quad/shader primitive in irisui.
pub fn build_color_picker_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
) {
    if !params.is_color_picker_open {
        return;
    }

    // Anchor to the Object Color swatch rect if available
    let swatch_tag = appearance_color_swatch_tag();
    let anchor_rect = tree
        .iter()
        .find(|(_, node)| node.tag == swatch_tag)
        .map(|(_, node)| node.computed_rect);

    let Some(anchor_rect) = anchor_rect else {
        return;
    };

    let popup_w = 206.0;
    let popup_h = 224.0;

    let popup_x = anchor_rect
        .x
        .min(params.panel_rect.right() - popup_w - 6.0)
        .max(params.panel_rect.x + 6.0);
    let popup_y = if anchor_rect.bottom() + popup_h > params.panel_rect.bottom() - 30.0 {
        (anchor_rect.y - popup_h - 4.0).max(30.0)
    } else {
        anchor_rect.bottom() + 4.0
    };

    let mut scope = UiScope::new(tree, parent_id);
    scope.dropdown_menu_card_named(
        "ColorPickerPopupCard",
        popup_x,
        popup_y,
        popup_w,
        |dropdown_card| {
            dropdown_card.container_tagged(
                "ColorPickerContent",
                Style::new().flex_col().gap(6.0),
                WidgetRole::DropdownPopup,
                TAG_INSPECTOR_COLOR_PICKER_CARD,
                |card| {
                    // 1. Header Row: Title and Close button
                    let header_style = Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .height(20.0);

                    card.container_named("ColorPickerHeaderRow", header_style, |header| {
                        header.label_styled_passive(
                            "ColorPickerHeaderLabel",
                            "🎨 Color Picker",
                            11.5,
                            Color::rgba(0.886, 0.894, 0.918, 1.0),
                            TextAlign::Left,
                            Style::new().height(18.0),
                        );

                        let close_size = 16.0;
                        let close_style = Style::new()
                            .width(close_size)
                            .height(close_size)
                            .background(Color::rgba(0.157, 0.165, 0.188, 0.98))
                            .hover_background(Color::rgba(0.35, 0.12, 0.12, 0.95))
                            .border_radius(3.0)
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center);

                        header.container_tagged(
                            "ColorPickerCloseButton",
                            close_style,
                            WidgetRole::Button,
                            TAG_INSPECTOR_COLOR_PICKER_CLOSE,
                            |btn| {
                                let txt_id = btn.label_styled_passive(
                                    "CloseGlyph",
                                    "✖",
                                    9.0,
                                    Color::rgba(0.70, 0.73, 0.80, 0.90),
                                    TextAlign::Center,
                                    Style::new().height(close_size),
                                );
                                btn.set_hover_text_color(txt_id, Color::WHITE);
                            },
                        );
                    });

                    // 2. Middle Row: 2D SV Box + Vertical Hue Bar
                    let middle_style = Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .gap(8.0)
                        .height(130.0);

                    let hue = params.inspector_hsv[0];
                    let saturation = params.inspector_hsv[1];
                    let value = params.inspector_hsv[2];

                    card.container_named("ColorPickerMiddleRow", middle_style, |middle| {
                        // 2a. 2D SV Box
                        let sv_w = 160.0;
                        let sv_h = 130.0;
                        let sv_style = Style::new()
                            .width(sv_w)
                            .height(sv_h)
                            .border(1.0, Color::rgba(0.25, 0.28, 0.35, 0.90))
                            .border_radius(4.0)
                            .clip_children(true);

                        middle.container_tagged(
                            "ColorPickerSvContainer",
                            sv_style,
                            WidgetRole::Button,
                            TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
                            |sv_box| {
                                // Tech Debt: 234-node HSV color picker cell explosion is temporary;
                                // will be replaced by a single GPU SDF gradient quad/shader primitive in irisui.
                                let grid_nx = 16;
                                let grid_ny = 13;
                                let cell_w = sv_w / (grid_nx as f32);
                                let cell_h = sv_h / (grid_ny as f32);

                                for j in 0..grid_ny {
                                    for i in 0..grid_nx {
                                        let s = (i as f32 + 0.5) / (grid_nx as f32);
                                        let v = 1.0 - (j as f32 + 0.5) / (grid_ny as f32);
                                        let col = hsv_to_rgb(hue, s, v);

                                        let cell_style = Style::new()
                                            .position_absolute()
                                            .left((i as f32) * cell_w)
                                            .top((j as f32) * cell_h)
                                            .width(cell_w + 0.2)
                                            .height(cell_h + 0.2)
                                            .background(col);

                                        sv_box.empty_box(cell_style);
                                    }
                                }

                                // Indicator Ring
                                let ring_size = 10.0;
                                let ring_x = (saturation * sv_w - ring_size * 0.5)
                                    .clamp(0.0, sv_w - ring_size);
                                let ring_y = ((1.0 - value) * sv_h - ring_size * 0.5)
                                    .clamp(0.0, sv_h - ring_size);
                                let ring_style = Style::new()
                                    .position_absolute()
                                    .left(ring_x)
                                    .top(ring_y)
                                    .width(ring_size)
                                    .height(ring_size)
                                    .border(2.0, Color::WHITE)
                                    .border_radius(5.0)
                                    .box_shadow(0.0, 1.0, 3.0, Color::rgba(0.0, 0.0, 0.0, 0.90));

                                sv_box.empty_box(ring_style);
                            },
                        );

                        // 2b. Vertical Hue Bar
                        let hue_w = 18.0;
                        let hue_style = Style::new()
                            .width(hue_w)
                            .height(sv_h)
                            .border(1.0, Color::rgba(0.25, 0.28, 0.35, 0.90))
                            .border_radius(4.0)
                            .clip_children(true);

                        middle.container_tagged(
                            "ColorPickerHueContainer",
                            hue_style,
                            WidgetRole::Button,
                            TAG_INSPECTOR_COLOR_PICKER_HUE_BAR,
                            |hue_bar| {
                                // Tech Debt: 234-node HSV color picker cell explosion is temporary;
                                // will be replaced by a single GPU SDF gradient quad/shader primitive in irisui.
                                let hue_steps = 26;
                                let step_h = sv_h / (hue_steps as f32);

                                for step in 0..hue_steps {
                                    let h_val = (step as f32 + 0.5) / (hue_steps as f32) * 360.0;
                                    let col = hsv_to_rgb(h_val, 1.0, 1.0);

                                    let strip_style = Style::new()
                                        .position_absolute()
                                        .left(0.0)
                                        .top((step as f32) * step_h)
                                        .width(hue_w)
                                        .height(step_h + 0.2)
                                        .background(col);

                                    hue_bar.empty_box(strip_style);
                                }

                                // Hue Indicator Line
                                let ind_h = 4.0;
                                let ind_y =
                                    (hue / 360.0 * sv_h - ind_h * 0.5).clamp(0.0, sv_h - ind_h);
                                let ind_style = Style::new()
                                    .position_absolute()
                                    .left(0.0)
                                    .top(ind_y)
                                    .width(hue_w)
                                    .height(ind_h)
                                    .background(Color::WHITE)
                                    .border(1.0, Color::BLACK)
                                    .border_radius(1.0);

                                hue_bar.empty_box(ind_style);
                            },
                        );
                    });

                    // 3. Bottom Preview Row: Live Color Swatch and HEX string
                    let preview_style = Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .gap(8.0)
                        .height(24.0);

                    card.container_named("ColorPickerPreviewRow", preview_style, |bottom| {
                        let current_col = hsv_to_rgb(hue, saturation, value);
                        let swatch_style = Style::new()
                            .width(42.0)
                            .height(20.0)
                            .background(current_col)
                            .border(1.0, Color::rgba(0.40, 0.44, 0.55, 0.90))
                            .border_radius(4.0);

                        bottom.empty_box(swatch_style);

                        bottom.label_styled_passive(
                            "LiveHexText",
                            params.inspector_color_hex,
                            11.0,
                            Color::rgba(0.85, 0.88, 0.95, 1.0),
                            TextAlign::Left,
                            Style::new().flex_grow(1.0).height(20.0),
                        );
                    });
                },
            );
        },
    );
}

/// Dispatches interactive 2D HSV color picker actions: start, live preview, and atomic commit.
pub fn handle_color_edit_action(
    color_edit_start: &mut Option<(hecs::Entity, ae_core::ecs::Color)>,
    inspector_hsv: &mut [f32; 3],
    inspector_color_hex: &mut String,
    world: &hecs::World,
    ui_actions: &mut Vec<crate::ui::types::EngineUiAction>,
    action: super::types::InspectorAction,
) {
    let fallback = ae_core::ecs::Color {
        r: 0.60,
        g: 0.75,
        b: 0.95,
        a: 1.0,
    };
    match action {
        super::types::InspectorAction::StartColorEdit(entity) => {
            if color_edit_start.is_none() {
                let cur = world
                    .get::<&ae_core::ecs::Color>(entity)
                    .map(|c| *c)
                    .unwrap_or(fallback);
                *color_edit_start = Some((entity, cur));
            }
        }
        super::types::InspectorAction::LiveSetObjectColor(entity, col) => {
            if color_edit_start.is_none() {
                let cur = world
                    .get::<&ae_core::ecs::Color>(entity)
                    .map(|c| *c)
                    .unwrap_or(fallback);
                *color_edit_start = Some((entity, cur));
            }
            let new_col = ae_core::ecs::Color {
                r: col.r,
                g: col.g,
                b: col.b,
                a: col.a,
            };
            if let Ok(mut existing) = world.get::<&mut ae_core::ecs::Color>(entity) {
                *existing = new_col;
            }
            let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
            *inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
            let (h, s, v) = irisui::prelude::rgb_to_hsv(col.r, col.g, col.b);
            *inspector_hsv = [h, s, v];
        }
        super::types::InspectorAction::CommitColorEdit(entity) => {
            if let Some((snap_entity, start_col)) = color_edit_start.take()
                && snap_entity == entity
            {
                let final_col = world
                    .get::<&ae_core::ecs::Color>(entity)
                    .map(|c| *c)
                    .unwrap_or(start_col);
                if start_col != final_col {
                    ui_actions.push(crate::ui::types::EngineUiAction::ModifyColor(
                        entity, start_col, final_col,
                    ));
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_color_picker_popup_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },));

        // Create a dummy node with the swatch tag so anchor_rect can be found
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "DummySwatchAnchor",
            Style::new()
                .position_absolute()
                .left(50.0)
                .top(100.0)
                .width(38.0)
                .height(18.0),
            WidgetRole::Button,
            appearance_color_swatch_tag(),
            |s| {
                s.label_styled_passive(
                    "SwatchDummyLabel",
                    "Swatch",
                    10.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().width(38.0).height(18.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 320.0, 600.0));

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];

        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ff0000",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            scroll_y: 0.0,
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: true,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 1.0, 1.0],
            blink_caret: false,
        };

        build_color_picker_popup(&mut tree, root, &params);

        let popup_card = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ColorPickerPopupCard"));
        assert!(popup_card.is_some(), "ColorPickerPopupCard must be emitted");
        assert_eq!(popup_card.unwrap().1.layer, UiLayer::Popup);

        let close_btn = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_COLOR_PICKER_CLOSE);
        assert!(close_btn.is_some(), "Close button must be tagged");

        let sv_box = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_COLOR_PICKER_SV_BOX);
        assert!(sv_box.is_some(), "SV box must be tagged");

        let hue_bar = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_COLOR_PICKER_HUE_BAR);
        assert!(hue_bar.is_some(), "Hue bar must be tagged");
    }
}