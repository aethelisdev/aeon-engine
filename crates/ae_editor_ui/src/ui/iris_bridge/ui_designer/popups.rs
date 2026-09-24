// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Dropdown Popups
//!
//! Renders the Aspect Ratio selector dropdown and the `➕ Add Element` palette popup
//! using 100% declarative [`UiScope`] dropdown menu primitives and 64-bit hardware semantic tags.
//!

use super::types::{
    CanvasAspectRatio, UiDesignerPanelParams, UiElementType, make_add_item_tag,
    make_aspect_item_tag,
};
use irisui::prelude::*;

/// Standard canvas aspect ratio presets available in the UI Designer.
pub const ASPECT_RATIO_PRESETS: [CanvasAspectRatio; 4] = [
    CanvasAspectRatio::Ratio16x9,
    CanvasAspectRatio::Ratio16x10,
    CanvasAspectRatio::Ratio4x3,
    CanvasAspectRatio::Ratio21x9,
];

/// UI widget element types available in the Add Element palette.
pub const UI_ELEMENT_TYPES: [UiElementType; 10] = [
    UiElementType::Panel,
    UiElementType::ProgressBar,
    UiElementType::Text,
    UiElementType::Button,
    UiElementType::Image,
    UiElementType::Slider,
    UiElementType::Checkbox,
    UiElementType::TextInput,
    UiElementType::HealthBar,
    UiElementType::ScoreDisplay,
];

/// Builds the Aspect Ratio dropdown menu popup if open.
pub fn build_aspect_ratio_popup(scope: &mut UiScope<'_>, params: &UiDesignerPanelParams<'_>) {
    if !params.is_aspect_dropdown_open {
        return;
    }

    let anchor = scope
        .tree()
        .iter()
        .find(|(_, n)| n.tag == super::types::UI_DESIGNER_TAG_ASPECT_BTN)
        .map(|(_, n)| n.computed_rect)
        .unwrap_or_else(|| {
            Rect::new(
                params.panel_rect.x + 8.0,
                params.panel_rect.y + 5.0,
                142.0,
                24.0,
            )
        });

    let popup_x = anchor.x;
    let popup_y = anchor.y + anchor.height + 2.0;
    let popup_w = 160.0;

    scope.dropdown_menu_card_named(
        "AspectRatioPopup",
        popup_x,
        popup_y,
        popup_w,
        |popup_scope| {
            for (idx, preset) in ASPECT_RATIO_PRESETS.iter().enumerate() {
                let is_selected = *preset == params.state.aspect_ratio;
                let shortcut = if is_selected { Some("✓") } else { None };
                popup_scope.dropdown_item(
                    make_aspect_item_tag(idx),
                    "",
                    preset.label(),
                    shortcut,
                    true,
                );
            }
        },
    );
}

/// Builds the `➕ Add Element` palette popup if open.
pub fn build_add_element_popup(scope: &mut UiScope<'_>, params: &UiDesignerPanelParams<'_>) {
    if !params.is_add_menu_open {
        return;
    }

    let anchor = scope
        .tree()
        .iter()
        .find(|(_, n)| n.tag == super::types::UI_DESIGNER_TAG_ADD_ELEMENT_BTN)
        .map(|(_, n)| n.computed_rect)
        .unwrap_or_else(|| {
            Rect::new(
                params.panel_rect.x + 8.0,
                params.panel_rect.y + 5.0,
                118.0,
                24.0,
            )
        });

    let popup_x = anchor.x;
    let popup_y = anchor.y + anchor.height + 2.0;
    let popup_w = 200.0;

    scope.dropdown_menu_card_named(
        "AddElementPopup",
        popup_x,
        popup_y,
        popup_w,
        |popup_scope| {
            for (idx, elem_type) in UI_ELEMENT_TYPES.iter().enumerate() {
                popup_scope.dropdown_item(
                    make_add_item_tag(idx),
                    elem_type.icon(),
                    elem_type.label(),
                    None,
                    true,
                );
            }
        },
    );
}