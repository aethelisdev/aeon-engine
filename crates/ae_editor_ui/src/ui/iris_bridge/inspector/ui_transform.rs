// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Screen Transform (RectTransform) Component Inspector Card
//!
//! Renders the 2D layout properties of `UiElement` using declarative `UiScope`:
//! - Anchor Presets (interactive mapped ComboBox dropdown)
//! - Screen Offset X and Y (declarative 2D vector drag with "px" suffix)
//! - Size Width and Height (declarative 2D vector drag with "px" suffix)
//! - Pivot X and Y (declarative 2D vector drag)
//! - Z-Index (layer ordering integer scrubber)
//! - Alpha (opacity scalar scrubber)
//! - Visibility toggle checkbox

use super::components::physics::helpers::{ComponentHeaderProps, build_declarative_card_header};
use super::context::{PropComboMapConfig, PropComboRowLayout, PropVec2Config};
use super::registry::ComponentRenderContext;
use ae_core::ecs::{UiAnchor, UiElement};
use irisui::prelude::*;

/// Converts a [`UiAnchor`] enum variant into its zero-based combobox dropdown index.
#[inline]
#[must_use]
pub fn anchor_to_index(anchor: UiAnchor) -> usize {
    match anchor {
        UiAnchor::TopLeft => 0,
        UiAnchor::TopCenter => 1,
        UiAnchor::TopRight => 2,
        UiAnchor::CenterLeft => 3,
        UiAnchor::Center => 4,
        UiAnchor::CenterRight => 5,
        UiAnchor::BottomLeft => 6,
        UiAnchor::BottomCenter => 7,
        UiAnchor::BottomRight => 8,
    }
}

/// Converts a zero-based combobox dropdown index into its corresponding [`UiAnchor`] variant.
#[inline]
#[must_use]
pub fn index_to_anchor(idx: usize) -> UiAnchor {
    match idx {
        0 => UiAnchor::TopLeft,
        1 => UiAnchor::TopCenter,
        2 => UiAnchor::TopRight,
        3 => UiAnchor::CenterLeft,
        4 => UiAnchor::Center,
        5 => UiAnchor::CenterRight,
        6 => UiAnchor::BottomLeft,
        7 => UiAnchor::BottomCenter,
        8 => UiAnchor::BottomRight,
        _ => UiAnchor::Center,
    }
}

/// Canonical component identifier of the 2D screen transform card (`ae_core::ecs::UiElement`).
///
/// Shared by the card header (delete button tag derivation through
/// [`super::tags::encode_component_delete_tag`]) and by the Inspector panel, which maps a click
/// on that tag into `EngineUiAction::RemoveComponent`. Both sides must use this single constant
/// so the encoded tag and the removed component name can never diverge.
pub const UI_ELEMENT_COMPONENT_NAME: &str = "UiElement";

/// Builds the `📐 2D Screen Transform` card inside the declarative [`UiScope`].
pub fn build_ui_transform_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    let Ok(mut u) = ctx.world.get::<&mut UiElement>(ctx.entity) else {
        return;
    };

    let label_w = if ctx.card_w < 220.0 { 42.0 } else { 52.0 };

    scope.card_panel("UiTransformCard", |card| {
        build_declarative_card_header(
            card,
            ComponentHeaderProps {
                atlas_icon: None,
                icon: "📐",
                display_title: "2D Screen Transform",
                header_color: card.theme().accent,
                component_name: UI_ELEMENT_COMPONENT_NAME,
            },
            false,
        );

        // Row 1: Anchor Preset Selector Dropdown
        ctx.prop_combobox_mapped(
            card,
            "UiElement",
            PropComboMapConfig::new(
                "Anchor:",
                anchor_to_index(u.anchor),
                &[
                    "Top-Left",
                    "Top-Center",
                    "Top-Right",
                    "Center-Left",
                    "Center",
                    "Center-Right",
                    "Bottom-Left",
                    "Bottom-Center",
                    "Bottom-Right",
                ],
                PropComboRowLayout::new(label_w, 90.0),
            ),
            &mut u.anchor,
            index_to_anchor,
        );

        // Row 2: Offset X & Y (px)
        ctx.prop_vec2(
            card,
            "UiElement",
            "Offset",
            &mut u.offset,
            PropVec2Config::new(["X", "Y"], 1.0, 0, "px"),
        );

        // Row 3: Size W & H (px)
        ctx.prop_vec2(
            card,
            "UiElement",
            "Size",
            &mut u.size,
            PropVec2Config::new(["W", "H"], 1.0, 0, "px"),
        );

        // Row 4: Pivot X & Y (0.0 .. 1.0)
        ctx.prop_vec2(
            card,
            "UiElement",
            "Pivot",
            &mut u.pivot,
            PropVec2Config::new(["X", "Y"], 0.05, 2, ""),
        );

        // Row 5: Z-Index (Layer) and Alpha
        ctx.prop_i32(
            card,
            "UiElement",
            "Layer (Z):",
            &mut u.z_index,
            -1000..=1000,
        );
        ctx.prop_f32(
            card,
            "UiElement",
            "Alpha (α):",
            &mut u.alpha,
            0.0..=1.0,
            0.05,
        );

        // Row 6: Visibility Checkbox
        ctx.prop_checkbox(card, "UiElement", "Visible", &mut u.visible);
    });
}