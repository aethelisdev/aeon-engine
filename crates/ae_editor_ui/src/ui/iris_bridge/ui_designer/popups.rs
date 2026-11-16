// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Dropdown Popups
//!
//! Renders the Aspect Ratio selector dropdown and the `➕ Add Element` palette popup
//! using 100% declarative [`UiScope`] dropdown menu primitives and 64-bit hardware semantic tags.

use super::types::{
    CanvasAspectRatio, UiDesignerAction, UiDesignerPanelParams, UiDesignerPanelState,
    UiElementType, make_add_item_tag, make_aspect_item_tag,
};
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PopupPlacement;

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

/// Builds the Aspect Ratio dropdown menu popup when [`super::types::UI_DESIGNER_TAG_ASPECT_BTN`] is active.
pub fn build_aspect_ratio_popup(
    scope: &mut UiScope<'_>,
    params: &UiDesignerPanelParams<'_>,
    designer: &mut UiDesignerPanelState,
) {
    scope.scoped_popup_anchored_to_tag(
        "AspectRatioPopup",
        super::types::UI_DESIGNER_TAG_ASPECT_BTN,
        Rect::new(
            params.panel_rect.x + 8.0,
            params.panel_rect.y + 5.0,
            142.0,
            24.0,
        ),
        160.0,
        PopupPlacement::Below,
        |popup_scope| {
            ASPECT_RATIO_PRESETS
                .iter()
                .enumerate()
                .for_each(|(idx, &preset)| {
                    popup_scope
                        .dropdown_item_autoclose(
                            make_aspect_item_tag(idx),
                            "",
                            preset.label(),
                            (preset == params.state.aspect_ratio).then_some("✓"),
                            true,
                        )
                        .on_click(|| designer.dispatch(UiDesignerAction::SetAspectRatio(preset)));
                });
        },
    );
}

/// Builds the `➕ Add Element` palette popup when [`super::types::UI_DESIGNER_TAG_ADD_ELEMENT_BTN`] is active.
pub fn build_add_element_popup(
    scope: &mut UiScope<'_>,
    params: &UiDesignerPanelParams<'_>,
    designer: &mut UiDesignerPanelState,
) {
    scope.scoped_popup_anchored_to_tag(
        "AddElementPopup",
        super::types::UI_DESIGNER_TAG_ADD_ELEMENT_BTN,
        Rect::new(
            params.panel_rect.x + 8.0,
            params.panel_rect.y + 5.0,
            118.0,
            24.0,
        ),
        200.0,
        PopupPlacement::Below,
        |popup_scope| {
            UI_ELEMENT_TYPES
                .iter()
                .enumerate()
                .for_each(|(idx, &elem_type)| {
                    popup_scope
                        .dropdown_item_autoclose(
                            make_add_item_tag(idx),
                            elem_type.icon(),
                            elem_type.label(),
                            None,
                            true,
                        )
                        .on_click(|| designer.dispatch(UiDesignerAction::SpawnElement(elem_type)));
                });
        },
    );
}