// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Bottom Action Bar Builder
//!
//! Renders `➕ Add Component` and `💾 Save as Prefab` action buttons using declarative [`UiScope`].
//!

use super::tags::{TAG_INSPECTOR_ADD_COMPONENT, TAG_INSPECTOR_SAVE_PREFAB};
use super::types::InspectorPanelParams;
use irisui::prelude::*;

/// Output node handles created during Inspector footer construction.
pub struct FooterNodes {
    /// Add component button node ID.
    pub add_comp_btn_id: WidgetId,
    /// Save prefab button node ID.
    pub save_prefab_btn_id: WidgetId,
}

/// Builds the Inspector bottom action bar declaratively into the parent [`UiScope`].
pub fn build_inspector_footer(
    scope: &mut UiScope<'_>,
    params: &InspectorPanelParams<'_>,
) -> FooterNodes {
    let padding_x = 8.0;
    let footer_h = 24.0;
    let btn_gap = 8.0;

    let is_add_hovered = params.hovered_tag == Some(TAG_INSPECTOR_ADD_COMPONENT);
    let is_save_hovered = params.hovered_tag == Some(TAG_INSPECTOR_SAVE_PREFAB);

    let (add_bg, add_border, add_text_col) = if params.is_add_menu_open {
        (
            Color::rgba(0.118, 0.125, 0.145, 1.0),
            Color::rgba(0.353, 0.376, 0.439, 0.95),
            Color::WHITE,
        )
    } else if is_add_hovered {
        (
            Color::rgba(0.200, 0.208, 0.235, 1.0),
            Color::rgba(0.271, 0.282, 0.329, 0.95),
            Color::WHITE,
        )
    } else {
        (
            Color::rgba(0.157, 0.165, 0.188, 0.98),
            Color::rgba(0.212, 0.220, 0.259, 0.85),
            Color::rgba(0.886, 0.894, 0.918, 1.0),
        )
    };

    let (save_bg, save_border, save_text_col) = if is_save_hovered {
        (
            Color::rgba(0.200, 0.208, 0.235, 1.0),
            Color::rgba(0.271, 0.282, 0.329, 0.95),
            Color::WHITE,
        )
    } else {
        (
            Color::rgba(0.157, 0.165, 0.188, 0.98),
            Color::rgba(0.212, 0.220, 0.259, 0.85),
            Color::rgba(0.886, 0.894, 0.918, 1.0),
        )
    };

    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(btn_gap)
        .height(34.0)
        .margin_insets(Insets::new(4.0, padding_x, 6.0, padding_x));

    let mut add_comp_btn_id = WidgetId::default();
    let mut save_prefab_btn_id = WidgetId::default();

    scope.container_named("InspectorFooter", row_style, |row| {
        // 1. `➕ Add Component` Button
        let add_btn_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .gap(5.0)
            .flex_grow(1.0)
            .height(footer_h)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
            .background(add_bg)
            .border(1.0, add_border)
            .border_radius(5.0);

        add_comp_btn_id = row.container_tagged(
            "AddComponentBtn",
            add_btn_style,
            WidgetRole::Button,
            TAG_INSPECTOR_ADD_COMPONENT,
            |btn| {
                btn.icon_named(
                    "AddComponentPlusIcon",
                    crate::ui::iris_bridge::icons::ICON_PLUS,
                    add_text_col,
                    11.0,
                );
                btn.label_styled_passive_wrapped(
                    "AddComponentBtnText",
                    "Add Component",
                    WrappedLabelDescriptor::new(11.0, add_text_col, Style::new().height(footer_h))
                        .align(TextAlign::Left)
                        .wrap(TextWrap::None),
                );
            },
        );

        // 2. `💾 Save as Prefab` Button
        let save_btn_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .flex_grow(1.0)
            .height(footer_h)
            .background(save_bg)
            .border(1.0, save_border)
            .border_radius(5.0);

        save_prefab_btn_id = row.container_tagged(
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
                        .wrap(TextWrap::None),
                );
            },
        );
    });

    FooterNodes {
        add_comp_btn_id,
        save_prefab_btn_id,
    }
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
            scroll_y: 0.0,
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
            hovered_tag: Some(TAG_INSPECTOR_ADD_COMPONENT),
        };

        let mut scope = UiScope::new(&mut tree, root);
        let nodes = build_inspector_footer(&mut scope, &params);
        scope.finish_layout(params.panel_rect);

        assert!(nodes.add_comp_btn_id != WidgetId::default());
        assert!(nodes.save_prefab_btn_id != WidgetId::default());

        let add_btn = tree
            .get(nodes.add_comp_btn_id)
            .expect("Add button must exist");
        let save_btn = tree
            .get(nodes.save_prefab_btn_id)
            .expect("Save button must exist");

        assert_eq!(add_btn.tag, TAG_INSPECTOR_ADD_COMPONENT);
        assert_eq!(save_btn.tag, TAG_INSPECTOR_SAVE_PREFAB);

        assert!(add_btn.computed_rect.width > 100.0);
        assert_eq!(add_btn.computed_rect.height, 24.0);
        assert!(save_btn.computed_rect.width > 100.0);
        assert_eq!(save_btn.computed_rect.height, 24.0);

        // Verify button colors reflected hover state
        assert_eq!(
            add_btn.style.background_color,
            Color::rgba(0.200, 0.208, 0.235, 1.0)
        );
    }
}