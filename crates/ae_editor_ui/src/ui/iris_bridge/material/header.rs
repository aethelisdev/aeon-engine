// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Material & Surface Studio Header Bar
//!
//! Renders the top title bar and active geometry badge using [`UiScope`].
//!

use crate::ui::iris_bridge::icons::{ICON_CUBE, ICON_WORLD};
use irisui::prelude::*;

/// Standard height of the Material Studio title bar.
pub const MATERIAL_HEADER_HEIGHT: f32 = 32.0;

/// Constructs the header bar with title and active geometry mode indicator directly on [`UiScope`].
pub fn build_material_header(
    scope: &mut UiScope<'_>,
    entity: Option<hecs::Entity>,
    world: &hecs::World,
) -> f32 {
    let header_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
        .background(Color::rgba(0.082, 0.086, 0.102, 0.98))
        .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.90))
        .height(MATERIAL_HEADER_HEIGHT);

    scope.container(header_style, |hdr| {
        // Left: icon + title
        hdr.row(|left| {
            left.icon(ICON_WORLD, Color::rgba(0.0, 0.85, 1.0, 0.95), 16.0);
            left.label(
                "Material & Surface Studio",
                11.5,
                Color::rgba(0.92, 0.93, 0.95, 1.0),
                TextAlign::Left,
            );
        });

        // Right-aligned Entity Geometry Badge
        if let Some(ent) = entity {
            let has_model = world.get::<&ae_core::ecs::ModelId>(ent).is_ok();
            let has_sprite = world.get::<&ae_core::ecs::SpriteId>(ent).is_ok();

            if has_model || has_sprite {
                let (badge_text, badge_icon, badge_color) = if has_model {
                    ("3D Model", ICON_CUBE, Color::rgba(0.0, 0.80, 0.95, 1.0))
                } else {
                    ("2D Sprite", ICON_WORLD, Color::rgba(0.30, 0.85, 0.45, 1.0))
                };

                let badge_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .gap(4.0)
                    .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                    .background(Color::rgba(0.12, 0.13, 0.16, 0.95))
                    .border(1.0, Color::rgba(0.20, 0.22, 0.27, 0.90))
                    .border_radius(4.0)
                    .height(20.0);

                hdr.container(badge_style, |badge| {
                    badge.icon(badge_icon, badge_color, 12.0);
                    badge.label(badge_text, 10.0, badge_color, TextAlign::Left);
                });
            }
        }
    });

    MATERIAL_HEADER_HEIGHT
}