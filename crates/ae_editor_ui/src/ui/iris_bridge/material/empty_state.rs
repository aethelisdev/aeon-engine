// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Empty States
//!
//! Declarative centered empty state and fallback cards stretching cleanly inside
//! flexbox containers via [`UiScope`] without hardcoded width subtraction.

use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_PLUS, ICON_WORLD};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Builds an empty-state placeholder card when no entity is currently selected directly on [`UiScope`].
pub fn build_no_entity_selected(scope: &mut UiScope<'_>) {
    scope.container(
        Style::new()
            .flex_col()
            .align_items(AlignItems::Center)
            .min_width(100.0)
            .background(scope.theme().card_bg)
            .border(1.0, scope.theme().card_border)
            .border_radius(8.0)
            .padding_insets(Insets::new(14.0, 14.0, 14.0, 14.0))
            .gap(4.0),
        |card| {
            card.container(
                Style::new()
                    .flex_row()
                    .justify_content(JustifyContent::Center)
                    .height(30.0),
                |r| {
                    r.icon(ICON_WORLD, Color::rgba(0.0, 0.85, 1.0, 0.65), 28.0);
                },
            );
            card.subheading("No Entity Selected");
            card.colored_label(
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                "Select a 3D model or 2D sprite in the",
            );
            card.colored_label(
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                "viewport or hierarchy to edit materials.",
            );
        },
    );
}

/// Builds an empty-state placeholder card when the selected entity has no `ModelId` or `SpriteId`.
pub fn build_no_renderable_geometry(
    scope: &mut UiScope<'_>,
    entity: Option<hecs::Entity>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.container(
        Style::new()
            .flex_col()
            .align_items(AlignItems::Center)
            .min_width(100.0)
            .background(scope.theme().card_bg)
            .border(1.0, scope.theme().card_border)
            .border_radius(8.0)
            .padding_insets(Insets::new(14.0, 14.0, 14.0, 14.0))
            .gap(4.0),
        |card| {
            card.container(
                Style::new()
                    .flex_row()
                    .justify_content(JustifyContent::Center)
                    .height(30.0),
                |r| {
                    r.icon(ICON_CUBE, Color::rgba(0.95, 0.65, 0.25, 0.75), 28.0);
                },
            );
            card.subheading("No Renderable Geometry");
            card.colored_label(
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                "Selected entity does not have a",
            );
            card.colored_label(
                Color::rgba(0.55, 0.58, 0.64, 1.0),
                "3D Model or 2D Sprite component attached.",
            );
            card.container(
                Style::new()
                    .flex_row()
                    .justify_content(JustifyContent::Center)
                    .padding_insets(Insets::new(6.0, 0.0, 0.0, 0.0)),
                |r| {
                    r.button_with_icon_styled_tagged(
                        ICON_PLUS,
                        Color::rgba(0.0, 0.85, 1.0, 0.95),
                        "Add Texture / Sprite",
                        Some(160.0),
                        r.tag_for("Add Texture / Sprite"),
                    )
                    .on_click(|| {
                        entity.into_iter().for_each(|ent| {
                            action_bus.emit(EngineUiAction::PickAndAssignEntityTexture(ent));
                        });
                    });
                },
            );
        },
    );
}