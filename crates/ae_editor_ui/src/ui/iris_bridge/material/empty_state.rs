// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Empty States
//!
//! Declarative centered empty state and fallback cards matching the Animation Timeline
//! studio aesthetics using [`UiScope`].
//!

use super::types::MATERIAL_TAG_ADD_TEXTURE;
use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_PLUS, ICON_WORLD};
use irisui::prelude::*;

/// Builds an empty-state placeholder card when no entity is currently selected directly on [`UiScope`].
pub fn build_no_entity_selected(scope: &mut UiScope<'_>, panel_width: f32) {
    let card_w = (panel_width - 24.0).max(100.0);
    let card_style = Style::new()
        .flex_col()
        .align_items(AlignItems::Center)
        .width(card_w)
        .background(Color::rgba(0.09, 0.10, 0.12, 0.95))
        .border(1.0, Color::rgba(0.18, 0.20, 0.25, 0.80))
        .border_radius(8.0)
        .padding_insets(Insets::new(14.0, 14.0, 14.0, 14.0))
        .gap(4.0);

    scope.container(card_style, |card| {
        let icon_row = Style::new()
            .flex_row()
            .justify_content(JustifyContent::Center)
            .height(30.0);
        card.container(icon_row, |r| {
            r.icon(ICON_WORLD, Color::rgba(0.0, 0.85, 1.0, 0.65), 28.0);
        });
        card.label(
            "No Entity Selected",
            12.0,
            Color::rgba(0.90, 0.92, 0.95, 1.0),
            TextAlign::Center,
        );
        card.label(
            "Select a 3D model or 2D sprite in the",
            10.5,
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            TextAlign::Center,
        );
        card.label(
            "viewport or hierarchy to edit materials.",
            10.5,
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            TextAlign::Center,
        );
    });
}

/// Builds an empty-state placeholder card when the selected entity has no ModelId or SpriteId.
pub fn build_no_renderable_geometry(scope: &mut UiScope<'_>, panel_width: f32) {
    let card_w = (panel_width - 24.0).max(100.0);
    let card_style = Style::new()
        .flex_col()
        .align_items(AlignItems::Center)
        .width(card_w)
        .background(Color::rgba(0.09, 0.10, 0.12, 0.95))
        .border(1.0, Color::rgba(0.18, 0.20, 0.25, 0.80))
        .border_radius(8.0)
        .padding_insets(Insets::new(14.0, 14.0, 14.0, 14.0))
        .gap(4.0);

    scope.container(card_style, |card| {
        let icon_row = Style::new()
            .flex_row()
            .justify_content(JustifyContent::Center)
            .height(30.0);
        card.container(icon_row, |r| {
            r.icon(ICON_CUBE, Color::rgba(0.95, 0.65, 0.25, 0.75), 28.0);
        });
        card.label(
            "No Renderable Geometry",
            12.0,
            Color::rgba(0.90, 0.92, 0.95, 1.0),
            TextAlign::Center,
        );
        card.label(
            "Selected entity does not have a",
            10.5,
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            TextAlign::Center,
        );
        card.label(
            "3D Model or 2D Sprite component attached.",
            10.5,
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            TextAlign::Center,
        );
        let btn_row = Style::new()
            .flex_row()
            .justify_content(JustifyContent::Center)
            .padding_insets(Insets::new(6.0, 0.0, 0.0, 0.0));
        card.container(btn_row, |r| {
            r.button_with_icon_styled_tagged(
                ICON_PLUS,
                Color::rgba(0.0, 0.85, 1.0, 0.95),
                "Add Texture / Sprite",
                Some(160.0),
                MATERIAL_TAG_ADD_TEXTURE,
            );
        });
    });
}