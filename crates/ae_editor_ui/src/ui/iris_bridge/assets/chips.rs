// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Category Filter Chips Bar for the Iris UI Content / Asset Browser.
//!
//! Renders an interactive horizontal strip of category filter pills:
//! `All`, `3D Models`, `Textures`, `Shaders`, `Scenes`, `Materials`, `Audio`.

use super::panel::ASSETS_CHIPS_BAR_HEIGHT;
use super::types::{AssetsPanelAction, AssetsPanelParams, encode_chip_tag};
use crate::assets::types::AssetCategory;
use irisui::prelude::*;

/// Category definitions with display label and pill width in logical pixels.
pub const CATEGORY_CHIPS: &[(AssetCategory, &str, f32)] = &[
    (AssetCategory::All, "All", 46.0),
    (AssetCategory::Models3D, "3D Models", 82.0),
    (AssetCategory::Textures2D, "Textures", 74.0),
    (AssetCategory::Shaders, "Shaders", 70.0),
    (AssetCategory::Scenes, "Scenes", 66.0),
    (AssetCategory::Materials, "Materials", 76.0),
    (AssetCategory::Audio, "Audio", 60.0),
];

/// Declarative construction of the Category Filter Chips horizontal bar using [`UiScope`].
pub fn build_category_chips_scope(
    scope: &mut UiScope<'_>,
    params: &AssetsPanelParams<'_>,
    actions: &mut Vec<AssetsPanelAction>,
) {
    scope.container_named(
        "CategoryChipsRow",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .width(params.panel_rect.width)
            .height(ASSETS_CHIPS_BAR_HEIGHT)
            .padding_insets(Insets::symmetric(0.0, 8.0))
            .gap(6.0)
            .background(scope.theme().panel_bg),
        |row| {
            for (idx, &(cat, label, width)) in CATEGORY_CHIPS.iter().enumerate() {
                row.add(
                    Chip::new(label)
                        .selected(active_category(row, params.active_category) == cat)
                        .accent_color(Color::rgba(0.0, 0.85, 1.0, 0.90))
                        .width(width)
                        .tag(encode_chip_tag(idx as u8)),
                )
                .on_click(|| {
                    actions.push(AssetsPanelAction::SelectCategory(cat));
                });
            }
        },
    );
}

#[inline]
fn active_category(scope: &UiScope<'_>, current: AssetCategory) -> AssetCategory {
    CATEGORY_CHIPS
        .iter()
        .enumerate()
        .find_map(|(idx, &(cat, _, _))| {
            (scope.is_tag_clicked(encode_chip_tag(idx as u8))
                || scope.is_tag_pressed(encode_chip_tag(idx as u8)))
            .then_some(cat)
        })
        .unwrap_or(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn test_category_chips_order_matches_canonical_all() {
        for (i, &(chip_cat, _, _)) in CATEGORY_CHIPS.iter().enumerate() {
            assert_eq!(chip_cat, AssetCategory::ALL[i]);
        }
    }

    #[test]
    fn test_category_chips_renders_in_tree() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let mut actions = Vec::new();
        let current_folder = Path::new("assets");
        let thumbnail_layers = HashMap::new();

        let params = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder,
            search_query: "",
            active_category: AssetCategory::All,
            view_mode: crate::assets::types::AssetViewMode::Grid,
            selected_asset: None,
            cached_items: &[],
            filtered_items: &[],
            subfolders: &[],
            is_2d_mode: false,
            show_engine_content: false,
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            cursor_pos: Point::new(0.0, 0.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            thumbnail_layers: &thumbnail_layers,
            cursor_byte_idx: 0,
        };

        build_category_chips_scope(&mut UiScope::new(&mut tree, root_id), &params, &mut actions);

        let row_id = tree
            .find_node_by_name("CategoryChipsRow")
            .expect("CategoryChipsRow must exist");
        let row_node = tree.get(row_id).expect("Row node must exist");
        assert_eq!(row_node.children.len(), CATEGORY_CHIPS.len());
    }

    #[test]
    fn test_click_audio_category_chip() {
        use crate::ui::iris_bridge::assets::panel::build_assets_panel;

        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("Root node creation failed");
        let mut actions = Vec::new();
        let current_folder = Path::new("assets");
        let thumbnail_layers = HashMap::new();

        let params = AssetsPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 1000.0, 600.0),
            screen_size: (1280.0, 720.0),
            current_folder,
            search_query: "",
            active_category: AssetCategory::All,
            view_mode: crate::assets::types::AssetViewMode::Grid,
            selected_asset: None,
            cached_items: &[],
            filtered_items: &[],
            subfolders: &[],
            is_2d_mode: false,
            show_engine_content: false,
            sidebar_width: 180.0,
            sidebar_collapsed: false,
            cursor_pos: Point::new(10.0, 10.0),
            blink_caret: false,
            active_context_menu: None,
            active_preview_modal: None,
            thumbnail_layers: &thumbnail_layers,
            cursor_byte_idx: 0,
        };

        build_assets_panel(
            &mut UiScope::new(&mut tree, root_id),
            &params,
            &mut TreeState::default(),
            &mut actions,
        );

        let audio_tag = encode_chip_tag(6);
        let mut audio_node_info = None;
        tree.traverse_depth_first(root_id, &mut |id, node| {
            if node.tag == audio_tag {
                audio_node_info = Some((id, node.computed_rect));
            }
        });

        let (_audio_id, audio_rect) = audio_node_info.expect("Audio chip must exist in UI tree");
        let click_pos = Point::new(
            audio_rect.x + audio_rect.width * 0.5,
            audio_rect.y + audio_rect.height * 0.5,
        );
        let down_res = tree.dispatch_event(&UiEvent::MouseDown {
            point: click_pos,
            button: MouseButton::Left,
        });
        assert!(down_res.consumed);

        let up_res = tree.dispatch_event(&UiEvent::MouseUp {
            point: click_pos,
            button: MouseButton::Left,
        });
        assert!(up_res.consumed);
        assert_eq!(tree.event_state.frame_clicked_tag, Some(audio_tag));

        let mut click_actions = Vec::new();
        let mut params_clicked = params;
        params_clicked.cursor_pos = click_pos;

        build_assets_panel(
            &mut UiScope::new(&mut tree, root_id),
            &params_clicked,
            &mut TreeState::default(),
            &mut click_actions,
        );

        assert!(
            click_actions.contains(&AssetsPanelAction::SelectCategory(AssetCategory::Audio)),
            "Audio chip click must emit AssetsPanelAction::SelectCategory(AssetCategory::Audio), got: {:?}",
            click_actions
        );
    }
}