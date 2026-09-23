// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Loading Splash Overlay
//!
//! Renders a hardware-accelerated GPU SDF splash blocker during background asset processing
//! using pure declarative [`UiScope`] architecture with dynamic trailing dots and an
//! animated indeterminate sliding progress bar.
//!

use irisui::prelude::*;

/// Width of the asset loading splash card in physical pixels.
pub const LOADING_CARD_WIDTH: f32 = 300.0;
/// Height of the asset loading splash card in physical pixels.
pub const LOADING_CARD_HEIGHT: f32 = 104.0;

/// Parameters for constructing the asset loading splash indicator.
pub struct LoadingOverlayParams {
    /// Viewport width in physical pixels.
    pub screen_width: f32,
    /// Viewport height in physical pixels.
    pub screen_height: f32,
    /// Current elapsed time in seconds for smooth continuous animations.
    pub time_secs: f32,
}

/// Constructs the centered asset loading splash indicator in the UI tree using declarative [`UiScope`].
///
/// Builds a headerless modal card with a full-screen semi-transparent backdrop scrim blocker,
/// title with cycling trailing ellipsis, subtext descriptor, and an animated indeterminate
/// horizontal progress slug.
///
/// # Arguments
/// * `tree` - Target Retained-Mode [`UiTree`] receiving the splash hierarchy.
/// * `params` - Sizing and animation parameters encapsulated in [`LoadingOverlayParams`].
///
/// Returns the root allocated [`WidgetId`] (the backdrop scrim blocker).
pub fn build_loading_overlay(tree: &mut UiTree, params: LoadingOverlayParams) -> WidgetId {
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;
    let time_secs = params.time_secs;

    let dot_count = (time_secs * 3.0) as usize % 4;
    let dots = match dot_count {
        1 => ".  ",
        2 => ".. ",
        3 => "...",
        _ => "   ",
    };
    let slide_t = (time_secs * 2.2).sin() * 0.5 + 0.5;

    let parent = tree.root().unwrap_or_default();
    let mut scope = UiScope::new(tree, parent);

    scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.60), |scrim| {
        scrim.modal_card(LOADING_CARD_WIDTH, LOADING_CARD_HEIGHT, |card| {
            card.container(
                Style::new()
                    .flex_col()
                    .align_items(AlignItems::Stretch)
                    .justify_content(JustifyContent::Center)
                    .gap(8.0),
                |col| {
                    col.label(
                        format!("🚀  Loading Assets{}", dots),
                        14.5,
                        Color::rgba(0.0, 0.90, 1.0, 1.0),
                        TextAlign::Center,
                    );
                    col.label(
                        "Processing geometry, materials & textures",
                        11.5,
                        Color::rgba(0.65, 0.68, 0.76, 1.0),
                        TextAlign::Center,
                    );
                    col.indeterminate_progress(slide_t);
                },
            );
        });

        scrim.finish_layout(Rect::new(0.0, 0.0, screen_width, screen_height));
    })
}