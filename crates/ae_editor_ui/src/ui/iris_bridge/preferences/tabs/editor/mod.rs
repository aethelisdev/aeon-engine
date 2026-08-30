// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Preferences Module
//!
//! Provides modular card renderers for editor settings including snapping, history limits,
//! physics simulation rate, and live runtime hot reload.

pub mod history;
pub mod physics;
pub mod runtime;
pub mod snapping;
pub mod types;

use self::history::build_history_card;
use self::physics::build_physics_card;
use self::runtime::build_runtime_card;
use self::snapping::{build_snapping_card, render_snap_mode_dropdown_popup};
use self::types::EditorCardContext;
pub use self::types::SNAP_MODE_OPTIONS;

use super::super::types::{PreferencesDropdownId, PreferencesParams, PreferencesTargets};
use irisui::prelude::*;

/// Builds the Editor preferences tab content.
pub fn build_editor_tab(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let mut virtual_y = 16.0;
    let scroll_y = params.scroll_offset_y;
    let content_w = content_rect.width - 32.0;
    let base_x = content_rect.x + 16.0;
    let snapping = params.snapping_settings;
    let cfg = params.editor_config;

    // 1. Heading
    let heading_id = tree.create_node();
    if let Some(node) = tree.get_mut(heading_id) {
        node.set_name("EditorHeading");
        node.set_text("Editor Settings");
        node.font_size = 17.0;
        node.line_height = 22.0;
        node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            22.0,
        );
    }
    let _ = tree.add_child(parent_id, heading_id);
    virtual_y += 24.0;

    // 2. Separator
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("EditorSep");
        node.style = Style::new().background(Color::rgba(0.20, 0.22, 0.30, 0.70));
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            1.0,
        );
    }
    let _ = tree.add_child(parent_id, sep_id);
    virtual_y += 16.0;

    let card_ctx = EditorCardContext {
        base_x,
        content_y: content_rect.y,
        content_w,
        scroll_y,
        val_box_w: 64.0,
        val_box_h: 22.0,
        cursor_pos: params.cursor_pos,
        collapsed_sections: params.collapsed_sections,
        active_number_input: params.active_number_input,
        blink_caret: params.blink_caret,
        active_dropdown: params.active_dropdown,
    };

    // ── 3. Snapping Card ──
    let (snap_h, snap_combo_rect) =
        build_snapping_card(tree, parent_id, virtual_y, card_ctx, snapping, targets);
    virtual_y += snap_h + 14.0;

    // ── 4. History Settings Card ──
    let hist_h = build_history_card(tree, parent_id, virtual_y, card_ctx, cfg, targets);
    virtual_y += hist_h + 14.0;

    // ── 5. Physics Settings Card ──
    let phys_h = build_physics_card(tree, parent_id, virtual_y, card_ctx, cfg, targets);
    virtual_y += phys_h + 14.0;

    // ── 6. Runtime Settings Card ──
    let rt_h = build_runtime_card(
        tree,
        parent_id,
        virtual_y,
        card_ctx,
        params.enable_live_updates,
        targets,
    );
    virtual_y += rt_h + 20.0;

    // Dropdown popup for SnapMode
    if let Some(combo_rect) = snap_combo_rect
        && params.active_dropdown == Some(PreferencesDropdownId::SnapMode)
    {
        render_snap_mode_dropdown_popup(
            tree,
            parent_id,
            combo_rect,
            snapping,
            params.cursor_pos,
            targets,
        );
    }

    virtual_y
}