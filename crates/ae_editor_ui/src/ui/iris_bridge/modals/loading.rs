// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Loading Splash Overlay
//!
//! Renders a hardware-accelerated GPU SDF splash blocker during background asset processing
//! with breathing neon glow, dynamic trailing dots, and a smooth indeterminate sliding progress bar.

use irisui::prelude::*;

/// Width of the asset loading splash card in physical pixels.
pub const LOADING_CARD_WIDTH: f32 = 300.0;
/// Height of the asset loading splash card in physical pixels.
pub const LOADING_CARD_HEIGHT: f32 = 104.0;

/// Interactive boundary targets for the loading overlay.
pub struct LoadingOverlayTargets {
    /// Full screen bounding box of the blocker scrim.
    pub scrim_rect: Rect,
    /// Bounding box of the center splash card.
    pub card_rect: Rect,
}

/// Parameters for constructing the asset loading splash indicator.
pub struct LoadingOverlayParams {
    /// Viewport width in physical pixels.
    pub screen_width: f32,
    /// Viewport height in physical pixels.
    pub screen_height: f32,
    /// Current elapsed time in seconds for smooth continuous animations.
    pub time_secs: f32,
}

/// Constructs the centered asset loading splash indicator in the UI tree.
pub fn build_loading_overlay(
    tree: &mut UiTree,
    params: LoadingOverlayParams,
) -> (WidgetId, LoadingOverlayTargets) {
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;
    let time_secs = params.time_secs;

    let left = ((screen_width - LOADING_CARD_WIDTH) * 0.5).max(0.0).round();
    let top = ((screen_height - LOADING_CARD_HEIGHT) * 0.5)
        .max(28.0)
        .round();
    let card_rect = Rect::new(left, top, LOADING_CARD_WIDTH, LOADING_CARD_HEIGHT);
    let scrim_rect = Rect::new(0.0, 0.0, screen_width, screen_height);

    let pulse = (time_secs * 3.0).sin() * 0.5 + 0.5;

    // 1. Blocker Scrim
    let scrim_id = tree.create_node();
    if let Some(node) = tree.get_mut(scrim_id) {
        node.set_name("LoadingScrim");
        node.computed_rect = scrim_rect;
        node.style = Style::new().background(Color::rgba(0.0, 0.0, 0.0, 0.60));
    }

    // 2. Glassmorphic SDF Splash Card with breathing border & neon glow
    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("LoadingCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .border(
                1.0,
                Color::rgba(
                    0.15 + 0.10 * pulse,
                    0.40 + 0.35 * pulse,
                    0.60 + 0.35 * pulse,
                    0.70 + 0.30 * pulse,
                ),
            )
            .border_radius(8.0)
            .box_shadow(
                0.0,
                8.0,
                24.0 + 8.0 * pulse,
                Color::rgba(0.0, 0.35 * pulse, 0.75 * pulse, 0.45),
            );
    }
    let _ = tree.add_child(scrim_id, card_id);

    // 3. Loading Title Label (Cyan Accent with cycling trailing dots)
    let dot_count = (time_secs * 3.0) as usize % 4;
    let dots = match dot_count {
        1 => ".  ",
        2 => ".. ",
        3 => "...",
        _ => "   ",
    };
    let title_id = tree.create_node();
    if let Some(node) = tree.get_mut(title_id) {
        node.set_name("LoadingTitle");
        node.set_text(format!("🚀  Loading Assets{}", dots));
        node.font_size = 14.5;
        node.line_height = 20.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.0, 0.90, 1.0, 1.0);
        node.computed_rect = Rect::new(left + 16.0, top + 18.0, LOADING_CARD_WIDTH - 32.0, 20.0);
    }
    let _ = tree.add_child(card_id, title_id);

    // 4. Loading Subtext Label
    let subtext_id = tree.create_node();
    if let Some(node) = tree.get_mut(subtext_id) {
        node.set_name("LoadingSubtext");
        node.set_text("Processing geometry, materials & textures");
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::rgba(0.65, 0.68, 0.76, 1.0);
        node.computed_rect = Rect::new(left + 16.0, top + 44.0, LOADING_CARD_WIDTH - 32.0, 18.0);
    }
    let _ = tree.add_child(card_id, subtext_id);

    // 5. Indeterminate Progress Bar Track
    let track_w = LOADING_CARD_WIDTH - 48.0;
    let track_rect = Rect::new(left + 24.0, top + 74.0, track_w, 4.0);
    let track_id = tree.create_node();
    if let Some(node) = tree.get_mut(track_id) {
        node.set_name("LoadingProgressTrack");
        node.computed_rect = track_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 1.0))
            .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.80))
            .border_radius(2.0);
    }
    let _ = tree.add_child(card_id, track_id);

    // 6. Sliding Glowing Cyan Slug / Ribbon
    let slug_w = 64.0;
    let slide_t = (time_secs * 2.2).sin() * 0.5 + 0.5;
    let slug_x = left + 24.0 + (track_w - slug_w) * slide_t;
    let slug_rect = Rect::new(slug_x, top + 74.0, slug_w, 4.0);
    let slug_id = tree.create_node();
    if let Some(node) = tree.get_mut(slug_id) {
        node.set_name("LoadingProgressSlug");
        node.computed_rect = slug_rect;
        node.style = Style::new()
            .background(Color::rgba(0.0, 0.90, 1.0, 0.95))
            .border_radius(2.0)
            .box_shadow(0.0, 0.0, 8.0, Color::rgba(0.0, 0.90, 1.0, 0.70));
    }
    let _ = tree.add_child(track_id, slug_id);

    (
        scrim_id,
        LoadingOverlayTargets {
            scrim_rect,
            card_rect,
        },
    )
}