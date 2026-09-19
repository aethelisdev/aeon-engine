// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Category Filter Chips Bar for Iris UI Asset Browser.
//!
//! Renders categorized asset filter badges with dynamic count indicators
//! and synchronized selection state.
//!

use super::types::{AssetsPanelParams, AssetsPanelTargets};
use crate::ui::panels::assets::types::{AssetCategory, AssetSource};
use irisui::prelude::*;

/// Builds the category filter chips row with live item counters.
///
/// Iterates through canonical asset categories, calculates filtered asset counts
/// based on the active engine content visibility setting, and produces responsive
/// badge buttons within the Iris UI tree.
pub fn build_category_chips(
    tree: &mut UiTree,
    parent_id: WidgetId,
    chips_rect: Rect,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    let categories = [
        (AssetCategory::All, "All Assets"),
        (AssetCategory::Models3D, "3D Meshes"),
        (AssetCategory::Textures2D, "Textures"),
        (AssetCategory::Shaders, "Shaders"),
        (AssetCategory::Scenes, "Scenes"),
        (AssetCategory::Materials, "Materials"),
        (AssetCategory::Audio, "Audio"),
    ];

    let mut chip_x = chips_rect.x + 8.0;
    let chip_y = chips_rect.y + 3.0;
    let chip_h = 22.0;

    for (cat, label) in categories {
        let count = params
            .cached_items
            .iter()
            .filter(|i| {
                if !params.show_engine_content && i.source == AssetSource::Engine {
                    return false;
                }
                if params.is_2d_mode && i.is_3d {
                    return false;
                }
                if !params.is_2d_mode && !i.is_3d && i.category == AssetCategory::Scenes {
                    return false;
                }
                cat == AssetCategory::All || i.category == cat
            })
            .count();

        let chip_text = format!("{} ({})", label, count);
        let chip_w = (chip_text.len() as f32 * 6.8 + 16.0).max(54.0);
        let chip_rect = Rect::new(chip_x, chip_y, chip_w, chip_h);
        let is_selected = params.active_category == cat;
        let is_hovered = chip_rect.contains_point(params.cursor_pos);

        targets.category_chips.push((cat, chip_rect));

        let chip_id = tree.create_node();
        if let Some(node) = tree.get_mut(chip_id) {
            node.set_name("CategoryChip");
            node.set_text(&chip_text);
            node.font_size = 11.0;
            node.line_height = chip_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_selected {
                Color::WHITE
            } else if is_hovered {
                Color::rgba(0.90, 0.93, 0.98, 1.0)
            } else {
                Color::rgba(0.65, 0.69, 0.78, 1.0)
            };
            node.computed_rect = chip_rect;
            let cat_color = super::cards::resolve_category_color(cat);
            let border_color = if is_selected {
                cat_color
            } else if is_hovered {
                Color::rgba(0.28, 0.32, 0.42, 0.70)
            } else {
                Color::rgba(0.16, 0.18, 0.24, 0.40)
            };
            node.style = Style::new()
                .background(if is_selected {
                    Color::rgba(0.12, 0.16, 0.22, 0.95)
                } else if is_hovered {
                    Color::rgba(0.10, 0.12, 0.16, 0.80)
                } else {
                    Color::rgba(0.08, 0.09, 0.11, 0.60)
                })
                .border_radius(4.0)
                .border(1.0, border_color);
        }
        let _ = tree.add_child(parent_id, chip_id);
        chip_x += chip_w + 6.0;
    }
}