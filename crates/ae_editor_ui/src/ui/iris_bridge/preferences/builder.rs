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

    let tabs: Vec<TabbedDialogTab<'_>> = SIDEBAR_TABS
        .iter()
        .map(|&(label, idx)| TabbedDialogTab::new(label, idx))
        .collect();

    let frame = TabbedDialogBuilder::new(card_rect, "⚙  Preferences", &tabs)
        .sidebar_width(SIDEBAR_WIDTH)
        .active_tab(params.active_tab)
        .cursor_pos(Some(params.cursor_pos))
        .build(tree);

    let mut targets = PreferencesTargets {
        title_bar_rect: frame.titlebar_rect,
        card_rect,
        close_button: frame.close_btn_rect,
        tabs: frame.tab_rects,
        content_rect: frame.content_rect,
        total_content_height: 0.0,
        toggles: Vec::new(),
        sliders: Vec::new(),
        dropdowns: Vec::new(),
        section_toggles: Vec::new(),
        number_inputs: Vec::new(),
        scrollbar: None,
    };

    let total_h = match params.active_tab {
        0 => build_general_tab(
            tree,
            frame.content_id,
            frame.content_rect,
            &params,
            &mut targets,
        ),
        1 => build_graphics_tab(
            tree,
            frame.content_id,
            frame.content_rect,
            &params,
            &mut targets,
        ),
        2 => build_editor_tab(
            tree,
            frame.content_id,
            frame.content_rect,
            &params,
            &mut targets,
        ),
        9 => build_modules_tab(
            tree,
            frame.content_id,
            frame.content_rect,
            &params,
            &mut targets,
        ),
        other => build_info_tab(
            tree,
            frame.content_id,
            frame.content_rect,
            other,
            &params,
            &mut targets,
        ),
    };
    targets.total_content_height = total_h;

    // 8. Custom Scrollbar Indicator via iris-widgets ScrollAreaBuilder
    let scroll_frame = ScrollAreaBuilder::new(frame.content_rect, total_h)
        .name("PrefScroll")
        .scroll_y(params.scroll_offset_y)
        .cursor_pos(Some(params.cursor_pos))
        .is_dragging(params.is_scrollbar_dragging)
        .style(ScrollAreaStyle {
            thickness: 6.0,
            inset: 3.0,
            ..ScrollAreaStyle::dark_default()
        })
        .build(tree, frame.card_id);

    targets.scrollbar = scroll_frame.scrollbar;

    (frame.card_id, targets)
}