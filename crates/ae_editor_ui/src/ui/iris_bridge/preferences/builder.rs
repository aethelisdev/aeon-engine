// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Dialog Builder
//!
//! Assembles the complete hardware-accelerated GPU SDF Preferences modal dialog,
//! including glassmorphic card framing, titlebar, sidebar tab navigation, and content routing.

use super::tabs::*;
use super::types::{PreferencesParams, PreferencesTargets};
use irisui::prelude::*;

/// Width of the preferences modal card in physical pixels.
pub const PREF_CARD_WIDTH: f32 = 760.0;
/// Height of the preferences modal card in physical pixels.
pub const PREF_CARD_HEIGHT: f32 = 540.0;
/// Width of the left sidebar navigation tab strip in physical pixels.
pub const SIDEBAR_WIDTH: f32 = 160.0;
/// Height of the titlebar in physical pixels.
pub const TITLEBAR_HEIGHT: f32 = 36.0;

/// Sidebar tabs descriptor list: `(label, tab_index)`.
pub const SIDEBAR_TABS: [(&str, u8); 10] = [
    ("General", 0),
    ("Graphics", 1),
    ("Editor", 2),
    ("Navigation", 3),
    ("Input", 7),
    ("Keymap", 4),
    ("System", 5),
    ("Add-ons", 6),
    ("Modules", 9),
    ("Experimental", 8),
];

/// Constructs the complete Preferences dialog tree and hit targets.
pub fn build_preferences_dialog(
    tree: &mut UiTree,
    params: PreferencesParams<'_>,
) -> (WidgetId, PreferencesTargets) {
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;

    let (left, top) = if let Some(pos) = params.window_pos {
        let max_x = (screen_width - PREF_CARD_WIDTH).max(0.0);
        let max_y = (screen_height - PREF_CARD_HEIGHT).max(28.0);
        (
            pos.x.clamp(0.0, max_x).round(),
            pos.y.clamp(28.0, max_y).round(),
        )
    } else {
        (
            ((screen_width - PREF_CARD_WIDTH) * 0.5).max(0.0).round(),
            ((screen_height - PREF_CARD_HEIGHT) * 0.5).max(28.0).round(),
        )
    };

    let card_rect = Rect::new(left, top, PREF_CARD_WIDTH, PREF_CARD_HEIGHT);
    let title_bar_rect = Rect::new(left, top, PREF_CARD_WIDTH - 36.0, TITLEBAR_HEIGHT);
    let close_btn_rect = Rect::new(left + PREF_CARD_WIDTH - 30.0, top + 7.0, 22.0, 22.0);
    let content_rect = Rect::new(
        left + SIDEBAR_WIDTH,
        top + TITLEBAR_HEIGHT,
        PREF_CARD_WIDTH - SIDEBAR_WIDTH,
        PREF_CARD_HEIGHT - TITLEBAR_HEIGHT,
    );

    let mut targets = PreferencesTargets {
        title_bar_rect,
        card_rect,
        close_button: close_btn_rect,
        tabs: Vec::with_capacity(10),
        content_rect,
        total_content_height: 0.0,
        toggles: Vec::new(),
        sliders: Vec::new(),
        dropdowns: Vec::new(),
        active_dropdown_items: Vec::new(),
        active_dropdown_popup_rect: None,
        section_toggles: Vec::new(),
        number_inputs: Vec::new(),
    };

    // 1. Glassmorphic SDF Main Card (No background darkening scrim)
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("PreferencesCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .border(1.0, Color::rgba(0.22, 0.25, 0.35, 1.0))
            .border_radius(8.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.70));
    }

    // 2. Custom Titlebar Header (Draggable)
    let titlebar_id = tree.create_node();
    if let Some(node) = tree.get_mut(titlebar_id) {
        node.set_name("PreferencesTitlebar");
        node.computed_rect = Rect::new(left, top, PREF_CARD_WIDTH, TITLEBAR_HEIGHT);
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.09, 1.0))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
            .border_radius(8.0);
    }
    let _ = tree.add_child(card_id, titlebar_id);

    // Titlebar Icon & Text
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("PreferencesTitle");
        node.set_text("⚙  Preferences");
        node.font_size = 13.0;
        node.line_height = TITLEBAR_HEIGHT;
        node.text_color = Color::rgba(0.90, 0.92, 0.96, 1.0);
        node.computed_rect = Rect::new(left + 14.0, top, 200.0, TITLEBAR_HEIGHT);
    }
    let _ = tree.add_child(titlebar_id, title_id);

    // Titlebar Close Button '✖'
    let is_close_hovered = close_btn_rect.contains_point(params.cursor_pos);
    let close_id = tree.create_node();
    if let Some(node) = tree.get_mut(close_id) {
        node.set_name("PreferencesCloseButton");
        node.computed_rect = close_btn_rect;
        let bg = if is_close_hovered {
            Color::rgba(0.85, 0.20, 0.20, 0.85)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.0)
        };
        node.style = Style::new().background(bg).border_radius(4.0);
    }
    let _ = tree.add_child(titlebar_id, close_id);

    let close_txt_id = tree.create_node();
    if let Some(node) = tree.get_mut(close_txt_id) {
        node.set_name("CloseText");
        node.set_text("✖");
        node.font_size = 11.0;
        node.line_height = close_btn_rect.height;
        node.text_align = TextAlign::Center;
        node.text_color = if is_close_hovered {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.65, 0.68, 0.76, 1.0)
        };
        node.computed_rect = close_btn_rect;
    }
    let _ = tree.add_child(close_id, close_txt_id);

    // 4. Left Sidebar Navigation Container
    let sidebar_rect = Rect::new(
        left,
        top + TITLEBAR_HEIGHT,
        SIDEBAR_WIDTH,
        PREF_CARD_HEIGHT - TITLEBAR_HEIGHT,
    );
    let sidebar_id = tree.create_node();
    if let Some(node) = tree.get_mut(sidebar_id) {
        node.set_name("PreferencesSidebar");
        node.computed_rect = sidebar_rect;
        node.style = Style::new().background(Color::rgba(0.07, 0.08, 0.10, 0.95));
    }
    let _ = tree.add_child(card_id, sidebar_id);

    // Sidebar Vertical Divider
    let divider_id = tree.create_node();
    if let Some(node) = tree.get_mut(divider_id) {
        node.set_name("SidebarDivider");
        node.computed_rect = Rect::new(
            left + SIDEBAR_WIDTH - 1.0,
            top + TITLEBAR_HEIGHT,
            1.0,
            sidebar_rect.height,
        );
        node.style = Style::new().background(Color::rgba(0.18, 0.20, 0.28, 0.90));
    }
    let _ = tree.add_child(card_id, divider_id);

    // 5. Sidebar Tabs
    let tab_h = 34.0;
    let mut tab_y = top + TITLEBAR_HEIGHT + 8.0;

    for &(label, idx) in &SIDEBAR_TABS {
        let is_selected = params.active_tab == idx;
        let tab_rect = Rect::new(left + 6.0, tab_y, SIDEBAR_WIDTH - 12.0, tab_h);
        let is_hovered = tab_rect.contains_point(params.cursor_pos);

        let tab_node_id = tree.create_node();
        if let Some(node) = tree.get_mut(tab_node_id) {
            node.set_name("SidebarTab");
            node.computed_rect = tab_rect;
            let bg = if is_selected {
                Color::rgba(0.0, 0.24, 0.31, 0.70)
            } else if is_hovered {
                Color::rgba(0.12, 0.14, 0.19, 1.0)
            } else {
                Color::rgba(0.0, 0.0, 0.0, 0.0)
            };
            node.style = Style::new().background(bg).border_radius(4.0);
        }
        let _ = tree.add_child(sidebar_id, tab_node_id);

        // Left Cyan Indicator Line for selected tab
        if is_selected {
            let ind_id = tree.create_node();
            if let Some(node) = tree.get_mut(ind_id) {
                node.set_name("TabIndicator");
                node.computed_rect =
                    Rect::new(tab_rect.x, tab_rect.y + 4.0, 3.5, tab_rect.height - 8.0);
                node.style = Style::new()
                    .background(Color::rgba(0.0, 0.90, 1.0, 1.0))
                    .border_radius(1.5);
            }
            let _ = tree.add_child(tab_node_id, ind_id);
        }

        // Tab Label
        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("TabLabel");
            node.set_text(label);
            node.font_size = 12.5;
            node.line_height = tab_h;
            node.text_color = if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else if is_hovered {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::rgba(0.65, 0.68, 0.76, 1.0)
            };
            node.computed_rect =
                Rect::new(tab_rect.x + 14.0, tab_rect.y, tab_rect.width - 20.0, tab_h);
        }
        let _ = tree.add_child(tab_node_id, lbl_id);

        targets.tabs.push((idx, tab_rect));
        tab_y += tab_h + 2.0;
    }

    // 6. Right Content Area Container
    let content_id = tree.create_node();
    if let Some(node) = tree.get_mut(content_id) {
        node.set_name("PreferencesContentArea");
        node.computed_rect = content_rect;
        node.style = Style::new().clip_children(true);
    }
    let _ = tree.add_child(card_id, content_id);

    // 7. Route Content based on active tab
    let total_h = match params.active_tab {
        0 => build_general_tab(tree, content_id, content_rect, &params, &mut targets),
        1 => build_graphics_tab(tree, content_id, content_rect, &params, &mut targets),
        2 => build_editor_tab(tree, content_id, content_rect, &params, &mut targets),
        9 => build_modules_tab(tree, content_id, content_rect, &params, &mut targets),
        other => build_info_tab(tree, content_id, content_rect, other, &params, &mut targets),
    };
    targets.total_content_height = total_h;

    // 8. Custom Scrollbar Indicator if content overflows
    if total_h > content_rect.height {
        let track_w = 4.0;
        let track_x = content_rect.x + content_rect.width - 8.0;
        let track_y = content_rect.y + 4.0;
        let track_h = content_rect.height - 8.0;

        let track_id = tree.create_node();
        if let Some(node) = tree.get_mut(track_id) {
            node.set_name("PrefScrollTrack");
            node.computed_rect = Rect::new(track_x, track_y, track_w, track_h);
            node.style = Style::new()
                .background(Color::rgba(0.12, 0.14, 0.20, 0.40))
                .border_radius(2.0);
        }
        let _ = tree.add_child(card_id, track_id);

        let max_scroll = (total_h - content_rect.height + 32.0).max(1.0);
        let thumb_h = ((content_rect.height / total_h) * track_h).clamp(24.0, track_h);
        let scroll_ratio = (params.scroll_offset_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = track_y + scroll_ratio * (track_h - thumb_h);

        let thumb_id = tree.create_node();
        if let Some(node) = tree.get_mut(thumb_id) {
            node.set_name("PrefScrollThumb");
            node.computed_rect = Rect::new(track_x, thumb_y, track_w, thumb_h);
            node.style = Style::new()
                .background(Color::rgba(0.28, 0.34, 0.48, 0.85))
                .border_radius(2.0);
        }
        let _ = tree.add_child(card_id, thumb_id);
    }

    (card_id, targets)
}