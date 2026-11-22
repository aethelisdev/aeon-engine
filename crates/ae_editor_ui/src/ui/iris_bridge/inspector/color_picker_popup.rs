// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating 2D HSV Color Picker Popup Builder
//!
//! Renders hardware-accelerated GPU SDF floating popup with a 2D Saturation-Value box,
//! vertical Rainbow Hue spectrum bar, live color preview, and close button via
//! [`UiScope::color_picker_popup_rgba`] and [`UiScope::color_picker_popup_detached`].

use super::appearance::appearance_color_swatch_tag;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;

/// Semantic tag for the floating 2D HSV Color Picker popup card container.
pub const TAG_INSPECTOR_COLOR_PICKER_CARD: u64 =
    UiTag::from_str("ae.inspector.color_picker.card").raw();

/// Semantic tag for the floating 2D HSV Color Picker close `✖` button.
pub const TAG_INSPECTOR_COLOR_PICKER_CLOSE: u64 =
    UiTag::from_str("ae.inspector.color_picker.close").raw();

/// Semantic tag for the floating 2D HSV Color Picker 2D Saturation-Value box.
pub const TAG_INSPECTOR_COLOR_PICKER_SV_BOX: u64 =
    UiTag::from_str("ae.inspector.color_picker.sv_box").raw();

/// Semantic tag for the floating 2D HSV Color Picker vertical rainbow Hue spectrum bar.
pub const TAG_INSPECTOR_COLOR_PICKER_HUE_BAR: u64 =
    UiTag::from_str("ae.inspector.color_picker.hue_bar").raw();

const fn to_ecs_color(color: Color) -> ae_core::ecs::Color {
    ae_core::ecs::Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}

fn map_some_rect(
    (rect, action): (Rect, Option<EngineUiAction>),
) -> (Option<Rect>, Option<EngineUiAction>) {
    (Some(rect), action)
}

/// Constructs the [`ColorPickerPopupConfig`] for the Inspector floating 2D HSV Color Picker popup.
#[must_use]
pub fn inspector_color_picker_config() -> ColorPickerPopupConfig<'static> {
    ColorPickerPopupConfig::new_auto(
        "🎨 Color Picker",
        ColorPickerPopupTags::new(
            appearance_color_swatch_tag(),
            TAG_INSPECTOR_COLOR_PICKER_CARD,
            TAG_INSPECTOR_COLOR_PICKER_CLOSE,
            TAG_INSPECTOR_COLOR_PICKER_SV_BOX,
            TAG_INSPECTOR_COLOR_PICKER_HUE_BAR,
        ),
    )
}

/// Renders the floating 2D HSV Color Picker popup bound directly to `entity`'s [`ae_core::ecs::Color`]
/// component in-place when active in [`irisui::prelude::UiTree`], returning `(Option<Rect>, Option<EngineUiAction>)`.
pub fn draw_entity_color_picker_popup(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    color: &mut ae_core::ecs::Color,
) -> (Option<Rect>, Option<EngineUiAction>) {
    scope
        .scoped_color_picker_popup_rgba(
            (&mut color.r, &mut color.g, &mut color.b, &mut color.a),
            inspector_color_picker_config(),
        )
        .map(|resp| {
            map_some_rect(resp.into_rect_and_commit(|old_c, new_c| {
                EngineUiAction::ModifyColor(entity, to_ecs_color(old_c), to_ecs_color(new_c))
            }))
        })
        .unwrap_or((None, None))
}

/// Renders the floating 2D HSV Color Picker popup when no [`ae_core::ecs::Color`] component
/// is currently attached and the swatch popup tag is active, returning `(Option<Rect>, Option<EngineUiAction>)`.
pub fn draw_detached_color_picker_popup(
    scope: &mut UiScope<'_>,
    selected_entity: Option<hecs::Entity>,
) -> (Option<Rect>, Option<EngineUiAction>) {
    scope
        .scoped_color_picker_popup_detached(
            Color::rgba(0.60, 0.75, 0.95, 1.0),
            inspector_color_picker_config(),
        )
        .map(|resp| {
            map_some_rect(resp.into_rect_and_opt_commit(|old_c, new_c| {
                selected_entity.map(|entity| {
                    EngineUiAction::ModifyColor(entity, to_ecs_color(old_c), to_ecs_color(new_c))
                })
            }))
        })
        .unwrap_or((None, None))
}

/// Builds the floating 2D HSV Color Picker popup for the currently active Inspector entity when active,
/// mutating [`ae_core::ecs::Color`] live in-place and returning `(Option<Rect>, Option<EngineUiAction>)`.
pub fn build_color_picker_popup_with_action(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    selected_entity: Option<hecs::Entity>,
) -> (Option<Rect>, Option<EngineUiAction>) {
    match selected_entity.and_then(|ent| {
        world
            .get::<&mut ae_core::ecs::Color>(ent)
            .ok()
            .map(|c| (ent, c))
    }) {
        Some((entity, mut color)) => draw_entity_color_picker_popup(scope, entity, &mut color),
        None => draw_detached_color_picker_popup(scope, selected_entity),
    }
}

/// Builds the floating 2D HSV Color Picker popup for the currently active Inspector entity using declarative [`UiScope`].
///
/// Returns the screen-space bounding box of the constructed color picker popup, if active.
pub fn build_color_picker_popup(
    scope: &mut UiScope<'_>,
    world: &hecs::World,
    selected_entity: Option<hecs::Entity>,
) -> Option<Rect> {
    build_color_picker_popup_with_action(scope, world, selected_entity).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_color_picker_popup_structure() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_core::ecs::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },));

        // Create a dummy node with the swatch tag so anchor_rect can be found
        let mut scope = UiScope::new(&mut tree, root);
        scope.container_tagged(
            "DummySwatchAnchor",
            Style::new()
                .position_absolute()
                .left(50.0)
                .top(100.0)
                .width(38.0)
                .height(18.0),
            WidgetRole::Button,
            appearance_color_swatch_tag(),
            |s| {
                s.label_styled_passive(
                    "SwatchDummyLabel",
                    "Swatch",
                    10.0,
                    Color::WHITE,
                    TextAlign::Center,
                    Style::new().width(38.0).height(18.0),
                );
            },
        );
        scope.finish_layout(Rect::new(0.0, 0.0, 320.0, 600.0));

        tree.open_popup(appearance_color_swatch_tag());
        let mut scope = UiScope::new(&mut tree, root);
        build_color_picker_popup(&mut scope, &world, Some(entity));

        let popup_card = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ColorPickerPopupCard"));
        assert!(popup_card.is_some(), "ColorPickerPopupCard must be emitted");
        assert_eq!(popup_card.unwrap().1.layer, UiLayer::Popup);

        let close_btn = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_COLOR_PICKER_CLOSE);
        assert!(close_btn.is_some(), "Close button must be tagged");

        let sv_box = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_COLOR_PICKER_SV_BOX);
        assert!(sv_box.is_some(), "SV box must be tagged");

        let hue_bar = tree
            .iter()
            .find(|(_, n)| n.tag == TAG_INSPECTOR_COLOR_PICKER_HUE_BAR);
        assert!(hue_bar.is_some(), "Hue bar must be tagged");
    }
}