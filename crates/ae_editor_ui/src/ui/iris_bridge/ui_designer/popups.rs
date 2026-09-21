// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Dropdown Popups
//!
//! Renders the Aspect Ratio selector dropdown and the `➕ Add Element` palette popup
//! using standardized [`ComboboxPopupBuilder`].
//!

use super::types::{
    CanvasAspectRatio, UiDesignerPanelParams, UiDesignerPanelTargets, UiElementType,
};
use irisui::prelude::*;
use irisui::widgets::{ComboboxPopupBuilder, ComboboxPopupStyle};

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
pub fn build_aspect_ratio_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    targets: &UiDesignerPanelTargets,
) {
    if !params.is_aspect_dropdown_open {
        return;
    }

    let anchor = match targets.btn_aspect {
        Some(r) => r,
        None => return,
    };

    let selected_index = ASPECT_RATIO_PRESETS
        .iter()
        .position(|&r| r == params.state.aspect_ratio);

    let labels: Vec<&str> = ASPECT_RATIO_PRESETS.iter().map(|r| r.label()).collect();

    let style = ComboboxPopupStyle {
        background: Color::rgba(0.090, 0.095, 0.110, 0.98),
        border_width: 1.0,
        border_color: Color::rgba(0.0, 0.70, 0.90, 0.95),
        border_radius: 6.0,
        shadow_y: 6.0,
        shadow_blur: 16.0,
        shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.75),
        item_idle_bg: Color::TRANSPARENT,
        item_hover_bg: Color::rgba(0.16, 0.18, 0.22, 0.95),
        item_selected_bg: Color::rgba(0.0, 0.35, 0.48, 0.95),
        text_idle_color: Color::rgba(0.75, 0.78, 0.85, 1.0),
        text_hover_color: Color::rgba(0.90, 0.92, 0.96, 1.0),
        text_selected_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        font_size: 10.5,
        row_height: 24.0,
        item_padding_x: 8.0,
    };

    ComboboxPopupBuilder::new(anchor)
        .name("AspectRatioPopup")
        .width(160.0)
        .items(&labels)
        .selected_index(selected_index)
        .cursor_pos(params.cursor_pos)
        .style(style)
        .build(tree, parent_id);
}

/// Builds the `➕ Add Element` palette popup if open.
pub fn build_add_element_popup(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    targets: &UiDesignerPanelTargets,
) {
    if !params.is_add_menu_open {
        return;
    }

    let anchor = match targets.btn_add_element {
        Some(r) => r,
        None => return,
    };

    let items_with_icons: Vec<(&str, Option<&str>)> = UI_ELEMENT_TYPES
        .iter()
        .map(|e| (e.label(), Some(e.icon())))
        .collect();

    let style = ComboboxPopupStyle {
        background: Color::rgba(0.090, 0.095, 0.110, 0.98),
        border_width: 1.0,
        border_color: Color::rgba(0.0, 0.70, 0.90, 0.95),
        border_radius: 6.0,
        shadow_y: 6.0,
        shadow_blur: 16.0,
        shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.75),
        item_idle_bg: Color::TRANSPARENT,
        item_hover_bg: Color::rgba(0.0, 0.32, 0.44, 0.95),
        item_selected_bg: Color::rgba(0.0, 0.35, 0.48, 0.95),
        text_idle_color: Color::rgba(0.80, 0.83, 0.89, 1.0),
        text_hover_color: Color::WHITE,
        text_selected_color: Color::WHITE,
        font_size: 10.5,
        row_height: 24.0,
        item_padding_x: 6.0,
    };

    ComboboxPopupBuilder::new(anchor)
        .name("AddElementPopup")
        .width(200.0)
        .align_right(true)
        .items_with_icons(&items_with_icons)
        .cursor_pos(params.cursor_pos)
        .style(style)
        .build(tree, parent_id);
}