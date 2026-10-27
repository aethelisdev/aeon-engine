// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Bottom Action Bar Builder
//!
//! Renders `➕ Add Component` and `💾 Save as Prefab` action buttons using declarative [`UiScope`].
//!

use super::tags::{TAG_INSPECTOR_ADD_COMPONENT, TAG_INSPECTOR_SAVE_PREFAB};
use super::types::{InspectorAction, InspectorPanelParams};
use irisui::prelude::*;
use irisui::widgets::declarative::scope::WrappedLabelDescriptor;

/// Output node handles created during Inspector footer construction.
/// Builds the Inspector bottom action bar declaratively into the parent [`UiScope`].
pub fn build_inspector_footer(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
    actions: &mut Vec<InspectorAction>,
) {
    let padding_x = 8.0;
    let footer_h = 24.0;
    let btn_gap = 8.0;
    let available_w = (params.panel_rect.width - padding_x * 2.0 - btn_gap).max(0.0);
    let btn_w = (available_w * 0.5).floor();

    let (add_bg, add_border, add_text_col) = if params.is_add_menu_open {
        (
            Color::rgba(0.118, 0.125, 0.145, 1.0),
            Color::rgba(0.353, 0.376, 0.439, 0.95),
            Color::WHITE,
        )
    } else {
        (
            Color::rgba(0.157, 0.165, 0.188, 0.98),
            Color::rgba(0.212, 0.220, 0.259, 0.85),
            Color::rgba(0.886, 0.894, 0.918, 1.0),
        )
    };

    let (save_bg, save_border, save_text_col) = (
        Color::rgba(0.157, 0.165, 0.188, 0.98),
        Color::rgba(0.212, 0.220, 0.259, 0.85),
        Color::rgba(0.886, 0.894, 0.918, 1.0),
    );

    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(btn_gap)
        .height(34.0)
        .flex_shrink(0.0)
        .margin_insets(Insets::new(4.0, padding_x, 6.0, padding_x));

    scope.container_named("InspectorFooter", row_style, |row| {
        // 1. `➕ Add Component` Button
        let mut add_btn_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .gap(5.0)
            .width(btn_w)
            .flex_grow(1.0)
            .height(footer_h)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
            .background(add_bg)
            .border(1.0, add_border)
            .border_radius(5.0);

        let is_add_pressed =
            row.tree().event_state.pressed_tag == Some(TAG_INSPECTOR_ADD_COMPONENT);

        if is_add_pressed {
            add_btn_style = add_btn_style
                .background(Color::rgba(0.090, 0.098, 0.118, 1.0))
                .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85));
        } else if !params.is_add_menu_open {
            add_btn_style = add_btn_style
                .hover_background(Color::rgba(0.200, 0.208, 0.235, 1.0))
                .hover_border(1.0, Color::rgba(0.271, 0.282, 0.329, 0.95));
        }

        let add_resp = row.container_tagged_resp(
            "AddComponentBtn",
            add_btn_style,
            WidgetRole::Button,
            TAG_INSPECTOR_ADD_COMPONENT,
            |btn| {
                if params.is_add_menu_open {
                    btn.icon_named(
                        "AddComponentPlusIcon",
                        crate::ui::iris_bridge::icons::ICON_PLUS,
                        add_text_col,
                        11.0,
                    );
                    btn.label_styled_passive_wrapped(
                        "AddComponentBtnText",
                        "Add Component",
                        WrappedLabelDescriptor::new(
                            11.0,
                            add_text_col,
                            Style::new().height(footer_h),
                        )
                        .align(TextAlign::Left)
                        .wrap(TextWrap::None),
                    );
                } else {
                    btn.icon_hover_named(
                        "AddComponentPlusIcon",
                        crate::ui::iris_bridge::icons::ICON_PLUS,
                        add_text_col,
                        Color::WHITE,
                        11.0,
                    );
                    btn.label_styled_passive_wrapped(
                        "AddComponentBtnText",
                        "Add Component",
                        WrappedLabelDescriptor::new(
                            11.0,
                            add_text_col,
                            Style::new().height(footer_h),
                        )
                        .align(TextAlign::Left)
                        .wrap(TextWrap::None)
                        .hover_color(Color::WHITE),
                    );
                }
            },
        );
        if add_resp.clicked() {
            if params.is_add_menu_open {
                actions.push(InspectorAction::CloseAddComponentMenu);
            } else {
                actions.push(InspectorAction::OpenAddComponentMenu(Point::ZERO));
            }
        }

        // 2. `💾 Save as Prefab` Button
        let is_save_pressed = row.tree().event_state.pressed_tag == Some(TAG_INSPECTOR_SAVE_PREFAB);

        let mut save_btn_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .gap(5.0)
            .width(btn_w)
            .flex_grow(1.0)
            .height(footer_h)
            .background(save_bg)
            .border(1.0, save_border)
            .hover_background(Color::rgba(0.200, 0.208, 0.235, 1.0))
            .hover_border(1.0, Color::rgba(0.271, 0.282, 0.329, 0.95))
            .border_radius(5.0);

        if is_save_pressed {
            save_btn_style = save_btn_style
                .background(Color::rgba(0.090, 0.098, 0.118, 1.0))
                .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85));
        }

        let save_resp = row.container_tagged_resp(
            "SavePrefabBtn",
            save_btn_style,
            WidgetRole::Button,
            TAG_INSPECTOR_SAVE_PREFAB,
            |btn| {
                btn.label_styled_passive_wrapped(
                    "SavePrefabBtnText",
                    "💾 Save as Prefab",
                    WrappedLabelDescriptor::new(11.0, save_text_col, Style::new().height(footer_h))
                        .align(TextAlign::Center)
                        .wrap(TextWrap::None)
                        .hover_color(Color::WHITE),
                );
            },
        );
        if save_resp.clicked()
            && let Some(entity) = params.selected_entity
        {
            actions.push(InspectorAction::SaveAsPrefab(entity));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_inspector_footer_layout_and_tags() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root must be created");
        let world = hecs::World::new();

        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world: &world,
            selected_entity: None,
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(12.0, 775.0), // Hover over Add Component button
            active_dropdown: None,
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_number_input: None,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut scope = UiScope::new(&mut tree, root);
        let mut actions = Vec::new();
        build_inspector_footer(&mut scope, &params, &mut actions);
        scope.finish_layout(params.panel_rect);

        let add_btn = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_ADD_COMPONENT)
            .map(|(_, n)| n)
            .expect("Add button must exist");
        let save_btn = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_SAVE_PREFAB)
            .map(|(_, n)| n)
            .expect("Save button must exist");

        assert_eq!(add_btn.tag, TAG_INSPECTOR_ADD_COMPONENT);
        assert_eq!(save_btn.tag, TAG_INSPECTOR_SAVE_PREFAB);

        assert!(add_btn.computed_rect.width > 100.0);
        assert_eq!(add_btn.computed_rect.height, 24.0);
        assert!(save_btn.computed_rect.width > 100.0);
        assert_eq!(save_btn.computed_rect.height, 24.0);

        // Verify button declarative hover state
        assert_eq!(
            add_btn.style.hover_background,
            Some(Color::rgba(0.200, 0.208, 0.235, 1.0))
        );
        assert_eq!(
            save_btn.style.hover_background,
            Some(Color::rgba(0.200, 0.208, 0.235, 1.0))
        );
    }
}