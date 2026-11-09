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

use super::registry::ComponentRenderContext;
use ae_core::ecs::{UiAnchor, UiElement};
use irisui::prelude::*;
use irisui::widgets::declarative::scope::ComboRowLayout;

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
/// [`UiScope::action_tag`]) and by the Inspector panel, which maps a click
/// on that tag into `EngineUiAction::RemoveComponent`. Both sides must use this single constant
/// so the encoded tag and the removed component name can never diverge.
pub const UI_ELEMENT_COMPONENT_NAME: &str = "UiElement";

/// Renders the declarative `📐 2D Screen Transform` card by mutating `UiElement` in-place
/// and snapshotting Undo/Redo sessions on edit commits.
pub fn draw_ui_transform_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    u: &mut UiElement,
) {
    scope.card_panel("UiTransformCard", |card| {
        card.card_header_desc(
            CardHeaderDescriptor::new("2D Screen Transform")
                .icon("📐")
                .color(card.theme().accent)
                .action("×", UiScope::action_tag(UI_ELEMENT_COMPONENT_NAME)),
        );

        // Row 1: Anchor Preset Selector Dropdown
        card.prop_combobox_mapped_styled(
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
            ComboRowLayout::new(52.0, 90.0),
            &mut u.anchor,
            index_to_anchor,
        )
        .on_change(|| {
            ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
            ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
        });

        // Row 2: Offset X & Y (px)
        card.prop_vec2("Offset", &mut u.offset, ["X", "Y"], 1.0)
            .on_commit(|_| {
                ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
                ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
            });

        // Row 3: Size W & H (px)
        card.prop_vec2("Size", &mut u.size, ["W", "H"], 1.0)
            .on_commit(|_| {
                ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
                ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
            });

        // Row 4: Pivot X & Y (0.0 .. 1.0)
        card.prop_vec2("Pivot", &mut u.pivot, ["X", "Y"], 0.05)
            .on_commit(|_| {
                ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
                ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
            });

        // Row 5: Z-Index (Layer) and Alpha
        card.prop_i32("Layer (Z):", &mut u.z_index, -1000..=1000)
            .on_commit(|_| {
                ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
                ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
            });

        card.prop_f32("Alpha (α):", &mut u.alpha, 0.0..=1.0, 0.05)
            .on_commit(|_| {
                ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
                ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
            });

        // Row 6: Visibility Checkbox
        card.prop_checkbox("Visible", &mut u.visible).on_click(|| {
            ctx.begin_edit(UI_ELEMENT_COMPONENT_NAME);
            ctx.commit_edit(UI_ELEMENT_COMPONENT_NAME);
        });
    });
}

/// Builds the `📐 2D Screen Transform` card inside the declarative [`UiScope`].
pub fn build_ui_transform_card(scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
    scope.when_ok(
        ctx.world.get::<&mut UiElement>(ctx.entity),
        |card, mut u| {
            draw_ui_transform_card(card, ctx, &mut u);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

    fn create_dummy_params<'a>(
        world: &'a hecs::World,
        entity: hecs::Entity,
        euler: &'a [f32; 3],
        swatches: &'a [[f32; 4]],
    ) -> InspectorPanelParams<'a> {
        InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 320.0, 800.0),
            world,
            selected_entity: Some(entity),
            inspector_euler: euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        }
    }

    #[test]
    fn test_draw_ui_transform_card_standalone() {
        let mut world = hecs::World::new();
        let entity = world.spawn((UiElement {
            anchor: UiAnchor::Center,
            offset: [15.0, 25.0],
            size: [120.0, 45.0],
            pivot: [0.5, 0.5],
            z_index: 3,
            alpha: 0.9,
            visible: true,
        },));

        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = create_dummy_params(&world, entity, &euler, &swatches);

        let mut actions = Vec::new();
        let mut ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            10.0,
            20.0,
            260.0,
        );

        let mut element = UiElement {
            anchor: UiAnchor::Center,
            offset: [15.0, 25.0],
            size: [120.0, 45.0],
            pivot: [0.5, 0.5],
            z_index: 3,
            alpha: 0.9,
            visible: true,
        };

        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_transform_card(&mut scope, &mut ctx, &mut element);

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("UiTransformCard")),
            "UiTransformCard container must be rendered"
        );
        tree.assert_node_exists("NumBox_Offset_X");
        tree.assert_node_exists("NumBox_Offset_Y");
        tree.assert_node_exists("NumBox_Size_W");
        tree.assert_node_exists("NumBox_Size_H");
        tree.assert_node_exists("NumBox_Pivot_X");
        tree.assert_node_exists("NumBox_Pivot_Y");
    }
}