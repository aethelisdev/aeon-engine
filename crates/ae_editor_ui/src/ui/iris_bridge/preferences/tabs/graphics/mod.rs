// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Tab Subsystem
//!
//! Orchestrates the rendering of Shadows, Performance, Anti-Aliasing, Post-Processing,
//! Environment & Sky, and Procedural Clouds preference cards.

pub mod environment;
pub mod helpers;
pub mod performance;
pub mod popup;
pub mod shadows;
pub mod types;

pub use popup::render_graphics_dropdown_popup;
pub use types::*;

use super::super::types::{PreferencesParams, PreferencesTargets};
use environment::build_environment_card;
use irisui::prelude::*;
use performance::{build_aa_card, build_perf_card, build_post_processing_card};
use shadows::build_shadows_card;

/// Builds the complete Graphics preferences tab content.
pub fn build_graphics_tab(
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

    // 1. Heading
    let heading_id = tree.create_node();
    if let Some(node) = tree.get_mut(heading_id) {
        node.set_name("GraphicsHeading");
        node.set_text("Graphics Settings");
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
        node.set_name("GraphicsSep");
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

    // 3. Shadows Card
    let sh_h = build_shadows_card(
        tree,
        parent_id,
        CardLayoutContext {
            base_x,
            y_offset: virtual_y - scroll_y,
            content_w,
            content_rect_y: content_rect.y,
        },
        params,
        targets,
    );
    virtual_y += sh_h + 14.0;

    // 4. Performance Card
    let perf_h = build_perf_card(
        tree,
        parent_id,
        CardLayoutContext {
            base_x,
            y_offset: virtual_y - scroll_y,
            content_w,
            content_rect_y: content_rect.y,
        },
        params,
        targets,
    );
    virtual_y += perf_h + 14.0;

    // 5. Anti-Aliasing Card
    let aa_h = build_aa_card(
        tree,
        parent_id,
        CardLayoutContext {
            base_x,
            y_offset: virtual_y - scroll_y,
            content_w,
            content_rect_y: content_rect.y,
        },
        params,
        targets,
    );
    virtual_y += aa_h + 14.0;

    // 6. Post-Processing Card
    let pp_h = build_post_processing_card(
        tree,
        parent_id,
        CardLayoutContext {
            base_x,
            y_offset: virtual_y - scroll_y,
            content_w,
            content_rect_y: content_rect.y,
        },
        params,
        targets,
    );
    virtual_y += pp_h + 14.0;

    // 7. Environment & Sky Card
    let env_h = build_environment_card(
        tree,
        parent_id,
        CardLayoutContext {
            base_x,
            y_offset: virtual_y - scroll_y,
            content_w,
            content_rect_y: content_rect.y,
        },
        params,
        targets,
    );
    virtual_y += env_h + 20.0;

    // 8. Render active dropdown popup if open
    if let Some(active_dd) = params.active_dropdown {
        render_graphics_dropdown_popup(
            tree,
            parent_id,
            active_dd,
            params.graphics_settings,
            targets,
            params.cursor_pos,
        );
    }

    virtual_y
}