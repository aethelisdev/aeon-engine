// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Main Panel Orchestrator for Iris UI.
//!
//! Ties together top breadcrumb and action navigation, category filter chips,
//! left folder tree sidebar, scrollable card/table views, and telemetry footer.
//!

use super::cards::build_asset_grid_cards;
use super::list::build_asset_list_table;
use super::tree::build_folder_tree_sidebar;
use super::types::{AssetsPanelParams, AssetsPanelTargets, BreadcrumbTarget};
use crate::ui::iris_bridge::icons::{ICON_FOLDER, ICON_PLUS};
use crate::ui::panels::assets::types::{AssetBrowserState, AssetCategory, AssetViewMode};
use irisui::prelude::*;
use std::path::PathBuf;

/// Height of the top navigation header toolbar in physical pixels.
pub const ASSETS_TOP_BAR_HEIGHT: f32 = 34.0;

/// Height of the category filter chips toolbar in physical pixels.
pub const ASSETS_CHIPS_BAR_HEIGHT: f32 = 28.0;

/// Height of the bottom telemetry status footer in physical pixels.
pub const ASSETS_FOOTER_HEIGHT: f32 = 24.0;

/// Constructs the complete Content / Asset Browser panel widget hierarchy into the Iris `UiTree`.
pub fn build_assets_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    targets.panel_rect = params.panel_rect;

    // 1. Panel Root Container with Hardware Scissor Clipping
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("AssetsPanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new()
            .background(Color::rgba(0.05, 0.06, 0.08, 1.0))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.70))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, root_id);

    // 2. Top Header Toolbar (Breadcrumbs on Left, Actions, View Mode & Search on Right)
    let tb_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.y,
        params.panel_rect.width,
        ASSETS_TOP_BAR_HEIGHT,
    );
    targets.toolbar_rect = tb_rect;
    let tb_id = tree.create_node();
    if let Some(node) = tree.get_mut(tb_id) {
        node.set_name("AssetsTopToolbar");
        node.computed_rect = tb_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.70));
    }
    let _ = tree.add_child(root_id, tb_id);

    // Left Side: Canonical Vector Folder Logo (`ICON_FOLDER`, Layer 6) + Breadcrumbs
    let mut cur_x = tb_rect.x + 8.0;
    let btn_y = tb_rect.y + 5.0;
    let btn_h = 24.0;

    let folder_logo_size = 18.0;
    let logo_rect = Rect::new(
        cur_x,
        tb_rect.y + (ASSETS_TOP_BAR_HEIGHT - folder_logo_size) * 0.5,
        folder_logo_size,
        folder_logo_size,
    );
    let logo_id = tree.create_node();
    if let Some(node) = tree.get_mut(logo_id) {
        node.set_name("TopBarFolderLogo");
        node.computed_rect = logo_rect;
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(Color::rgba(0.0, 0.90, 1.0, 1.0)); // Aeon Cyan tint
    }
    let _ = tree.add_child(tb_id, logo_id);
    cur_x += folder_logo_size + 6.0;

    // Breadcrumb path segments
    let segments: Vec<String> = params
        .current_folder
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            let sep_rect = Rect::new(cur_x, btn_y, 10.0, btn_h);
            let sep_id = tree.create_node();
            if let Some(node) = tree.get_mut(sep_id) {
                node.set_name("BreadcrumbSeparator");
                node.set_text(">");
                node.font_size = 11.0;
                node.line_height = btn_h;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.40, 0.45, 0.55, 1.0);
                node.computed_rect = sep_rect;
            }
            let _ = tree.add_child(tb_id, sep_id);
            cur_x += 12.0;
        }

        let is_last = i == segments.len() - 1;
        let crumb_w = (segment.len() as f32 * 7.5 + 14.0).max(36.0);
        let crumb_rect = Rect::new(cur_x, btn_y, crumb_w, btn_h);
        let is_crumb_hovered = crumb_rect.contains_point(params.cursor_pos);

        let target_path: PathBuf = segments[..=i].iter().collect();
        targets.breadcrumbs.push(BreadcrumbTarget {
            rect: crumb_rect,
            path: target_path,
        });

        let crumb_id = tree.create_node();
        if let Some(node) = tree.get_mut(crumb_id) {
            node.set_name("BreadcrumbButton");
            node.set_text(segment);
            node.font_size = 11.5;
            node.line_height = btn_h;
            node.text_align = TextAlign::Center;
            node.text_color = if is_last {
                Color::WHITE
            } else if is_crumb_hovered {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else {
                Color::rgba(0.70, 0.74, 0.84, 1.0)
            };
            node.computed_rect = crumb_rect;
            node.style = Style::new()
                .background(if is_crumb_hovered {
                    Color::rgba(0.18, 0.22, 0.30, 0.80)
                } else {
                    Color::TRANSPARENT
                })
                .border_radius(3.0);
        }
        let _ = tree.add_child(tb_id, crumb_id);
        cur_x += crumb_w + 2.0;
    }

    // Right Side: Action Buttons, View Mode Toggle & Search Box
    let mut right_x = tb_rect.right() - 8.0;

    // Search Input Field Box (180px)
    let search_w = 180.0;
    right_x -= search_w;
    let search_rect = Rect::new(right_x, btn_y, search_w, btn_h);
    targets.search_input_rect = search_rect;
    let is_search_hovered = search_rect.contains_point(params.cursor_pos);

    let search_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(search_box_id) {
        node.set_name("AssetsSearchBox");
        node.computed_rect = search_rect;
        let (border_c, border_w) = if params.is_search_focused {
            (Color::rgba(0.0, 0.90, 1.0, 0.95), 1.5)
        } else if is_search_hovered {
            (Color::rgba(0.35, 0.40, 0.52, 0.70), 1.0)
        } else {
            (Color::rgba(0.20, 0.23, 0.30, 0.60), 1.0)
        };
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.09, 0.95))
            .border_radius(4.0)
            .border(border_w, border_c);
    }
    let _ = tree.add_child(tb_id, search_box_id);

    // Search Icon "🔍"
    let s_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(s_icon_id) {
        node.set_name("SearchIcon");
        node.set_text("🔍");
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
        node.computed_rect = Rect::new(right_x + 6.0, btn_y, 14.0, btn_h);
    }
    let _ = tree.add_child(search_box_id, s_icon_id);

    // Search Query Text or Hint
    let display_text = if params.search_query.is_empty() {
        "Search assets..."
    } else {
        params.search_query
    };
    let text_color = if params.search_query.is_empty() {
        Color::rgba(0.40, 0.44, 0.54, 1.0)
    } else {
        Color::rgba(0.95, 0.96, 0.98, 1.0)
    };
    let text_start_x = if params.is_search_focused && params.search_query.is_empty() {
        right_x + 25.5
    } else {
        right_x + 23.0
    };
    let s_text_id = tree.create_node();
    if let Some(node) = tree.get_mut(s_text_id) {
        node.set_name("SearchQueryText");
        node.set_text(display_text);
        node.font_size = 11.0;
        node.line_height = btn_h;
        node.text_color = text_color;
        node.computed_rect = Rect::new(text_start_x, btn_y, search_w - 42.0, btn_h);
    }
    let _ = tree.add_child(search_box_id, s_text_id);

    // Blinking Caret Cursor (500ms cycle)
    if params.is_search_focused && params.blink_caret {
        let caret_x = if params.search_query.is_empty() {
            right_x + 23.0
        } else {
            (right_x + 23.0 + (params.search_query.len() as f32 * 6.6))
                .min(right_x + search_w - 22.0)
        };
        let caret_id = tree.create_node();
        if let Some(node) = tree.get_mut(caret_id) {
            node.set_name("AssetsSearchCaret");
            node.computed_rect = Rect::new(caret_x, btn_y + 4.0, 1.5, btn_h - 8.0);
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.90, 1.0, 1.0))
                .border_radius(0.75);
        }
        let _ = tree.add_child(search_box_id, caret_id);
    }

    // Clear Search "✖" Button
    targets.search_clear_btn_rect = None;
    if !params.search_query.is_empty() {
        let clr_rect = Rect::new(right_x + search_w - 18.0, btn_y + 3.0, 15.0, 18.0);
        targets.search_clear_btn_rect = Some(clr_rect);
        let is_clr_hov = clr_rect.contains_point(params.cursor_pos);
        let clr_id = tree.create_node();
        if let Some(node) = tree.get_mut(clr_id) {
            node.set_name("SearchClearBtn");
            node.set_text("✖");
            node.font_size = 9.0;
            node.line_height = 18.0;
            node.text_align = TextAlign::Center;
            node.text_color = if is_clr_hov {
                Color::WHITE
            } else {
                Color::rgba(0.60, 0.65, 0.75, 1.0)
            };
            node.computed_rect = clr_rect;
        }
        let _ = tree.add_child(search_box_id, clr_id);
    }

    right_x -= 10.0;

    // View Mode Toggles: Grid vs List
    let mode_btn_w = 46.0;
    right_x -= mode_btn_w;
    let grid_rect = Rect::new(right_x, btn_y, mode_btn_w, btn_h);
    targets.grid_toggle_rect = grid_rect;
    build_toggle_btn(
        tree,
        tb_id,
        "⊞ Grid",
        params.view_mode == AssetViewMode::Grid,
        grid_rect,
        params.cursor_pos,
    );

    right_x -= mode_btn_w + 2.0;
    let list_rect = Rect::new(right_x, btn_y, mode_btn_w, btn_h);
    targets.list_toggle_rect = list_rect;
    build_toggle_btn(
        tree,
        tb_id,
        "☰ List",
        params.view_mode == AssetViewMode::List,
        list_rect,
        params.cursor_pos,
    );

    right_x -= 12.0;

    // Action Buttons: Clean, Reveal, + Import
    let clean_w = 56.0;
    right_x -= clean_w;
    let clean_rect = Rect::new(right_x, btn_y, clean_w, btn_h);
    targets.clean_btn_rect = clean_rect;
    build_action_btn(tree, tb_id, "Clean", clean_rect, params.cursor_pos);

    let reveal_w = 60.0;
    right_x -= reveal_w + 4.0;
    let reveal_rect = Rect::new(right_x, btn_y, reveal_w, btn_h);
    targets.reveal_btn_rect = reveal_rect;
    build_action_btn(tree, tb_id, "Reveal", reveal_rect, params.cursor_pos);

    let import_w = 70.0;
    right_x -= import_w + 4.0;
    let import_rect = Rect::new(right_x, btn_y, import_w, btn_h);
    targets.import_btn_rect = import_rect;
    build_import_btn(tree, tb_id, import_rect, params.cursor_pos);

    // 3. Category Filter Chips Row
    let chips_rect = Rect::new(
        params.panel_rect.x,
        tb_rect.bottom(),
        params.panel_rect.width,
        ASSETS_CHIPS_BAR_HEIGHT,
    );
    targets.chips_rect = chips_rect;
    let chips_bar_id = tree.create_node();
    if let Some(node) = tree.get_mut(chips_bar_id) {
        node.set_name("AssetsCategoryChipsBar");
        node.computed_rect = chips_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.09, 0.95))
            .border(1.0, Color::rgba(0.14, 0.16, 0.22, 0.50));
    }
    let _ = tree.add_child(root_id, chips_bar_id);

    build_category_chips(tree, chips_bar_id, chips_rect, params, targets);

    // 4. Split Body (Sidebar + Content Viewport) & Bottom Status Footer
    let middle_y = chips_rect.bottom();
    let middle_h = (params.panel_rect.bottom() - middle_y - ASSETS_FOOTER_HEIGHT).max(40.0);

    let content_x = if !params.sidebar_collapsed {
        let sb_rect = Rect::new(
            params.panel_rect.x,
            middle_y,
            params.sidebar_width,
            middle_h,
        );
        build_folder_tree_sidebar(tree, root_id, sb_rect, params, targets);
        params.panel_rect.x + params.sidebar_width
    } else {
        targets.sidebar_rect = None;
        params.panel_rect.x
    };

    let content_w = (params.panel_rect.right() - content_x).max(60.0);
    let content_rect = Rect::new(content_x, middle_y, content_w, middle_h);
    targets.content_viewport_rect = content_rect;

    let content_vp_id = tree.create_node();
    if let Some(node) = tree.get_mut(content_vp_id) {
        node.set_name("AssetsContentViewport");
        node.computed_rect = content_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.98))
            .clip_children(true);
    }
    let _ = tree.add_child(root_id, content_vp_id);

    // Render Grid vs List View
    match params.view_mode {
        AssetViewMode::Grid => {
            targets.list_rows.clear();
            build_asset_grid_cards(tree, content_vp_id, content_rect, params, targets);
        }
        AssetViewMode::List => {
            targets.grid_cards.clear();
            build_asset_list_table(tree, content_vp_id, content_rect, params, targets);
        }
    }

    // 5. Bottom Status Footer (Telemetry + Collapse Toggle)
    let footer_rect = Rect::new(
        params.panel_rect.x,
        params.panel_rect.bottom() - ASSETS_FOOTER_HEIGHT,
        params.panel_rect.width,
        ASSETS_FOOTER_HEIGHT,
    );
    targets.footer_rect = footer_rect;
    let footer_id = tree.create_node();
    if let Some(node) = tree.get_mut(footer_id) {
        node.set_name("AssetsFooter");
        node.computed_rect = footer_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.09, 0.98))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.70));
    }
    let _ = tree.add_child(root_id, footer_id);

    // Sidebar Toggle Button ("◀" / "▶")
    let toggle_rect = Rect::new(footer_rect.x + 4.0, footer_rect.y + 2.0, 26.0, 20.0);
    targets.sidebar_toggle_btn_rect = toggle_rect;
    let is_tog_hov = toggle_rect.contains_point(params.cursor_pos);
    let tog_id = tree.create_node();
    if let Some(node) = tree.get_mut(tog_id) {
        node.set_name("SidebarToggleButton");
        node.set_text(if params.sidebar_collapsed {
            "▶"
        } else {
            "◀"
        });
        node.font_size = 10.0;
        node.line_height = 20.0;
        node.text_align = TextAlign::Center;
        node.text_color = if is_tog_hov {
            Color::WHITE
        } else {
            Color::rgba(0.65, 0.70, 0.80, 1.0)
        };
        node.computed_rect = toggle_rect;
        node.style = Style::new()
            .background(if is_tog_hov {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.80)
            })
            .border_radius(3.0);
    }
    let _ = tree.add_child(footer_id, tog_id);

    // Folder Logo (`ICON_FOLDER`) + Path in Footer
    let f_logo_size = 14.0;
    let f_logo_rect = Rect::new(
        footer_rect.x + 36.0,
        footer_rect.y + (ASSETS_FOOTER_HEIGHT - f_logo_size) * 0.5,
        f_logo_size,
        f_logo_size,
    );
    let f_logo_id = tree.create_node();
    if let Some(node) = tree.get_mut(f_logo_id) {
        node.set_name("FooterFolderLogo");
        node.computed_rect = f_logo_rect;
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(Color::rgba(0.95, 0.76, 0.28, 0.80));
    }
    let _ = tree.add_child(footer_id, f_logo_id);

    let path_str = params.current_folder.display().to_string();
    let path_rect = Rect::new(
        footer_rect.x + 54.0,
        footer_rect.y,
        300.0,
        ASSETS_FOOTER_HEIGHT,
    );
    targets.footer_folder_rect = path_rect;
    let path_id = tree.create_node();
    if let Some(node) = tree.get_mut(path_id) {
        node.set_name("FooterFolderPath");
        node.set_text(&path_str);
        node.font_size = 10.5;
        node.line_height = ASSETS_FOOTER_HEIGHT;
        node.text_color = Color::rgba(0.55, 0.60, 0.70, 1.0);
        node.computed_rect = path_rect;
    }
    let _ = tree.add_child(footer_id, path_id);

    // Right Telemetry
    let total_size: u64 = params.cached_items.iter().map(|i| i.file_size_bytes).sum();
    let in_memory_count = params
        .cached_items
        .iter()
        .filter(|i| i.is_loaded_in_memory)
        .count();
    let tele_str = format!(
        "{} Items in Scope ({} Total, {} in VRAM)  •  Disk: {}",
        params.filtered_items.len(),
        params.cached_items.len(),
        in_memory_count,
        AssetBrowserState::format_file_size(total_size)
    );
    let tele_w = 340.0;
    let tele_rect = Rect::new(
        footer_rect.right() - tele_w - 10.0,
        footer_rect.y,
        tele_w,
        ASSETS_FOOTER_HEIGHT,
    );
    let tele_id = tree.create_node();
    if let Some(node) = tree.get_mut(tele_id) {
        node.set_name("FooterTelemetry");
        node.set_text(&tele_str);
        node.font_size = 10.5;
        node.line_height = ASSETS_FOOTER_HEIGHT;
        node.text_align = TextAlign::Right;
        node.text_color = Color::rgba(0.50, 0.54, 0.64, 1.0);
        node.computed_rect = tele_rect;
    }
    let _ = tree.add_child(footer_id, tele_id);

    // 6. Right-Click Floating Context Menu (Z-Order Top)
    super::context_menu::build_assets_context_menu(tree, parent_id, params, targets);

    // 7. Interactive Quick Asset Preview Modal (Z-Order Highest)
    super::preview::build_asset_preview_modal(tree, parent_id, params, targets);
}

/// Builds the category filter chips row with live item counters.
fn build_category_chips(
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
        let count = if cat == AssetCategory::All {
            params.cached_items.len()
        } else {
            params
                .cached_items
                .iter()
                .filter(|i| i.category == cat)
                .count()
        };

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

/// Helper to build an elevated action button.
fn build_action_btn(
    tree: &mut UiTree,
    parent_id: WidgetId,
    label: &'static str,
    rect: Rect,
    cursor_pos: Point,
) {
    let is_hov = rect.contains_point(cursor_pos);
    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("ActionBtn");
        node.set_text(label);
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_align = TextAlign::Center;
        node.text_color = if is_hov {
            Color::WHITE
        } else {
            Color::rgba(0.78, 0.82, 0.90, 1.0)
        };
        node.computed_rect = rect;
        node.style = Style::new()
            .background(if is_hov {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.90)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_hov {
                    Color::rgba(0.35, 0.42, 0.55, 0.80)
                } else {
                    Color::rgba(0.20, 0.23, 0.30, 0.50)
                },
            );
    }
    let _ = tree.add_child(parent_id, btn_id);
}

/// Helper to build the "+ Import" button with vector plus icon.
fn build_import_btn(tree: &mut UiTree, parent_id: WidgetId, rect: Rect, cursor_pos: Point) {
    let is_hov = rect.contains_point(cursor_pos);
    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("ImportBtn");
        node.computed_rect = rect;
        node.style = Style::new()
            .background(if is_hov {
                Color::rgba(0.20, 0.26, 0.36, 1.0)
            } else {
                Color::rgba(0.13, 0.16, 0.22, 0.95)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_hov {
                    Color::rgba(0.0, 0.85, 1.0, 0.80)
                } else {
                    Color::rgba(0.22, 0.28, 0.38, 0.60)
                },
            );
    }
    let _ = tree.add_child(parent_id, btn_id);

    // Vector Plus Icon
    let p_size = 12.0;
    let p_rect = Rect::new(
        rect.x + 8.0,
        rect.y + (rect.height - p_size) * 0.5,
        p_size,
        p_size,
    );
    let p_id = tree.create_node();
    if let Some(node) = tree.get_mut(p_id) {
        node.set_name("ImportPlusIcon");
        node.computed_rect = p_rect;
        node.set_texture_uv(ICON_PLUS);
        node.set_texture_tint(if is_hov {
            Color::WHITE
        } else {
            Color::rgba(0.0, 0.90, 1.0, 1.0)
        });
    }
    let _ = tree.add_child(btn_id, p_id);

    // "Import" label
    let lbl_rect = Rect::new(rect.x + 22.0, rect.y, rect.width - 24.0, rect.height);
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("ImportBtnText");
        node.set_text("Import");
        node.font_size = 11.0;
        node.line_height = rect.height;
        node.text_color = if is_hov {
            Color::WHITE
        } else {
            Color::rgba(0.85, 0.89, 0.96, 1.0)
        };
        node.computed_rect = lbl_rect;
    }
    let _ = tree.add_child(btn_id, lbl_id);
}

/// Helper to build a selectable view mode toggle button.
fn build_toggle_btn(
    tree: &mut UiTree,
    parent_id: WidgetId,
    label: &'static str,
    is_active: bool,
    rect: Rect,
    cursor_pos: Point,
) {
    let is_hov = rect.contains_point(cursor_pos);
    let btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(btn_id) {
        node.set_name("ViewModeToggleBtn");
        node.set_text(label);
        node.font_size = 10.5;
        node.line_height = rect.height;
        node.text_align = TextAlign::Center;
        node.text_color = if is_active {
            Color::rgba(0.0, 0.95, 1.0, 1.0)
        } else if is_hov {
            Color::WHITE
        } else {
            Color::rgba(0.65, 0.70, 0.80, 1.0)
        };
        node.computed_rect = rect;
        node.style = Style::new()
            .background(if is_active {
                Color::rgba(0.12, 0.18, 0.26, 0.95)
            } else if is_hov {
                Color::rgba(0.14, 0.16, 0.22, 0.80)
            } else {
                Color::rgba(0.08, 0.09, 0.12, 0.60)
            })
            .border_radius(4.0)
            .border(
                1.0,
                if is_active {
                    Color::rgba(0.0, 0.85, 1.0, 0.85)
                } else if is_hov {
                    Color::rgba(0.30, 0.35, 0.45, 0.60)
                } else {
                    Color::rgba(0.16, 0.18, 0.24, 0.40)
                },
            );
    }
    let _ = tree.add_child(parent_id, btn_id);
}