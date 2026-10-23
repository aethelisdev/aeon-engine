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
    let card_bg = scope.theme().card_bg;
    let card_border = scope.theme().card_border;
    let card_style = Style::new()
        .flex_col()
        .align_items(AlignItems::Center)
        .width(card_w)
        .background(card_bg)
        .border(1.0, card_border)
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
        card.subheading("No Entity Selected");
        card.colored_label(
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            "Select a 3D model or 2D sprite in the",
        );
        card.colored_label(
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            "viewport or hierarchy to edit materials.",
        );
    });
}

/// Builds an empty-state placeholder card when the selected entity has no ModelId or SpriteId.
pub fn build_no_renderable_geometry(
    scope: &mut UiScope<'_>,
    panel_width: f32,
    entity: Option<hecs::Entity>,
    actions: &mut Vec<super::types::MaterialAction>,
) {
    let card_w = (panel_width - 24.0).max(100.0);
    let card_bg = scope.theme().card_bg;
    let card_border = scope.theme().card_border;
    let card_style = Style::new()
        .flex_col()
        .align_items(AlignItems::Center)
        .width(card_w)
        .background(card_bg)
        .border(1.0, card_border)
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
        card.subheading("No Renderable Geometry");
        card.colored_label(
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            "Selected entity does not have a",
        );
        card.colored_label(
            Color::rgba(0.55, 0.58, 0.64, 1.0),
            "3D Model or 2D Sprite component attached.",
        );
        let btn_row = Style::new()
            .flex_row()
            .justify_content(JustifyContent::Center)
            .padding_insets(Insets::new(6.0, 0.0, 0.0, 0.0));
        card.container(btn_row, |r| {
            if r.button_with_icon_styled_tagged(
                ICON_PLUS,
                Color::rgba(0.0, 0.85, 1.0, 0.95),
                "Add Texture / Sprite",
                Some(160.0),
                MATERIAL_TAG_ADD_TEXTURE,
            )
            .clicked()
                && let Some(ent) = entity
            {
                actions.push(super::types::MaterialAction::PickAndAssignEntityTexture(
                    ent,
                ));
            }
        });
    });
}