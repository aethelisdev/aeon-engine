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
/// Returns the screen-space bounding box of the constructed color picker popup, if active.
///
/// **Tech Debt:** 234-node HSV color picker cell explosion is temporary; will be replaced by a single GPU SDF gradient quad/shader primitive in irisui.
pub fn build_color_picker_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    actions: &mut Vec<super::types::InspectorAction>,
) -> Option<Rect> {
    if !params.is_color_picker_open {
        return None;
    }

    // Anchor to the Object Color swatch rect if available, or fallback gracefully
    let swatch_tag = appearance_color_swatch_tag();
    let anchor_rect = tree
        .node_rect_tagged(swatch_tag, Point::ZERO)
        .or_else(|| tree.tag_rect(swatch_tag))
        .unwrap_or(Rect::new(
            params.panel_rect.x + 20.0,
            params.panel_rect.y + 120.0,
            40.0,
            20.0,
        ));

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

                        let close_resp = header.container_tagged(
                            "ColorPickerCloseButton",
                            close_style,
                            WidgetRole::Button,
                            TAG_INSPECTOR_COLOR_PICKER_CLOSE,
                            |btn| {
                                btn.label_styled_hover_passive(
                                    "CloseGlyph",
                                    "✖",
                                    9.0,
                                    (Color::rgba(0.70, 0.73, 0.80, 0.90), Color::WHITE),
                                    TextAlign::Center,
                                    Style::new().height(close_size),
                                );
                            },
                        );
                        if close_resp.response.clicked() {
                            actions.push(super::types::InspectorAction::ToggleColorPicker);
                        }
                    });

                    // 2. Middle Row: Declarative 2D HSV ColorPicker Widget
                    let mut hsv_state = HsvColorPickerState {
                        hue: params.inspector_hsv[0],
                        saturation: params.inspector_hsv[1],
                        value: params.inspector_hsv[2],
                        alpha: 1.0,
                    };

                    let picker_resp = card.add(
                        ColorPicker::new(&mut hsv_state)
                            .sv_size(160.0, 130.0)
                            .hue_width(18.0)
                            .gap(8.0)
                            .tags(
                                TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
                                TAG_INSPECTOR_COLOR_PICKER_HUE_BAR,
                            ),
                    );

                    if picker_resp.changed()
                        && let Some(entity) = params.selected_entity
                    {
                        actions.push(super::types::InspectorAction::LiveSetObjectColor(
                            entity,
                            picker_resp.color(),
                        ));
                    }

                    if picker_resp.drag_released()
                        && let Some(entity) = params.selected_entity
                    {
                        actions.push(super::types::InspectorAction::CommitColorEdit(entity));
                    }

                    // 3. Bottom Preview Row: Live Color Swatch and HEX string
                    let preview_style = Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .gap(8.0)
                        .height(24.0);

                    card.container_named("ColorPickerPreviewRow", preview_style, |bottom| {
                        let current_col = picker_resp.color();
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

    Some(Rect::new(popup_x, popup_y, popup_w, popup_h))
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
            let (h, s, v) = HsvColorPickerState::rgb_to_hsv(col.r, col.g, col.b);
            let preserved_hue = if s < 1e-3 || v < 1e-3 {
                inspector_hsv[0]
            } else {
                h
            };
            *inspector_hsv = [preserved_hue, s, v];
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

        let mut actions = Vec::new();
        build_color_picker_popup(&mut tree, root, &params, &mut actions);

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