// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Header Bar
//!
//! Renders the top title bar and active geometry badge using [`UiScope`].

use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_WORLD};
use irisui::prelude::*;

/// Standard height of the Material Studio title bar.
pub const MATERIAL_HEADER_HEIGHT: f32 = 32.0;

#[inline]
fn entity_geometry_badge(
    world: &ae_ecs::World,
    entity: Option<ae_ecs::Entity>,
) -> Option<(&'static str, [f32; 4], Color)> {
    entity.and_then(|ent| {
        if world.has::<ae_plugin_api::ModelId>(ent) {
            Some(("3D Model", ICON_CUBE, Color::rgba(0.0, 0.80, 0.95, 1.0)))
        } else if world.has::<ae_plugin_api::SpriteId>(ent) {
            Some(("2D Sprite", ICON_WORLD, Color::rgba(0.30, 0.85, 0.45, 1.0)))
        } else {
            None
        }
    })
}

/// Constructs the header bar with title and active geometry mode indicator directly on [`UiScope`].
pub fn build_material_header(
    scope: &mut UiScope<'_>,
    entity: Option<ae_ecs::Entity>,
    world: &ae_ecs::World,
) -> f32 {
    scope.container(
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .background(scope.theme().toolbar_bg)
            .border(1.0, scope.theme().toolbar_border)
            .height(MATERIAL_HEADER_HEIGHT),
        |hdr| {
            hdr.row(|left| {
                left.icon(ICON_WORLD, Color::rgba(0.0, 0.85, 1.0, 0.95), 16.0);
                left.label("Material & Surface Studio");
            });

            hdr.when_some(
                entity_geometry_badge(world, entity),
                |right, (badge_text, badge_icon, badge_color)| {
                    right.container(
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .gap(4.0)
                            .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                            .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                            .border(1.0, Color::rgba(0.20, 0.22, 0.27, 0.90))
                            .border_radius(4.0)
                            .height(20.0),
                        |badge| {
                            badge.icon(badge_icon, badge_color, 12.0);
                            badge.colored_label(badge_color, badge_text);
                        },
                    );
                },
            );
        },
    );

    MATERIAL_HEADER_HEIGHT
}