// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Aspect Ratio Presets & Add-Element Palette
//!
//! Provides static dropdown preset definitions for aspect ratios and declarative
//! spawn palette items routed directly to [`ActionBus<EngineUiAction>`].

use super::types::{
    CanvasAspectRatio, UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UiElementType, make_add_item_tag,
    make_aspect_item_tag,
};
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use irisui::dock::DeferredNotifier;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Canonical aspect ratio preset ordering exposed in the toolbar selector.
pub const ASPECT_RATIO_PRESETS: [CanvasAspectRatio; 4] = [
    CanvasAspectRatio::Ratio16x9,
    CanvasAspectRatio::Ratio16x10,
    CanvasAspectRatio::Ratio4x3,
    CanvasAspectRatio::Ratio21x9,
];

/// Declarative options consumed by `anchored_select_pill` for canvas aspect ratios.
pub const ASPECT_RATIO_OPTIONS: &[AnchoredSelectOption<'_>] = &[
    AnchoredSelectOption::new(make_aspect_item_tag(0), "📐", "16:9 (1080p Full HD)"),
    AnchoredSelectOption::new(make_aspect_item_tag(1), "📐", "16:10 (WUXGA)"),
    AnchoredSelectOption::new(make_aspect_item_tag(2), "📐", "4:3 (Classic)"),
    AnchoredSelectOption::new(make_aspect_item_tag(3), "📐", "21:9 (Ultrawide)"),
];

/// Returns the zero-based option index matching `ratio` inside [`ASPECT_RATIO_PRESETS`].
#[inline]
#[must_use]
pub const fn aspect_ratio_index(ratio: CanvasAspectRatio) -> usize {
    match ratio {
        CanvasAspectRatio::Ratio16x9 => 0,
        CanvasAspectRatio::Ratio16x10 => 1,
        CanvasAspectRatio::Ratio4x3 => 2,
        CanvasAspectRatio::Ratio21x9 => 3,
    }
}

/// Ordered catalog of UI element types spawnable from the UI Designer palette.
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

/// Builds the anchored `Add Element` palette popup when active in [`UiTree`](irisui_core::UiTree).
///
/// Clicking an item emits [`EngineUiAction::SpawnUiElement`] directly onto `action_bus` and marks
/// [`PanelId::UiDesigner`] dirty for redraw.
pub fn build_add_element_popup(
    scope: &mut UiScope<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
    notifier: &mut DeferredNotifier<PanelId>,
) {
    scope.scoped_popup_anchored_auto(
        "AddElementPopup",
        UI_DESIGNER_TAG_ADD_ELEMENT_BTN,
        200.0,
        PopupPlacement::Below,
        |menu| {
            for (idx, &elem_type) in UI_ELEMENT_TYPES.iter().enumerate() {
                menu.dropdown_item_autoclose(
                    make_add_item_tag(idx),
                    elem_type.icon(),
                    elem_type.label(),
                    None,
                    true,
                )
                .on_click(|| {
                    action_bus.emit(EngineUiAction::SpawnUiElement(elem_type));
                    notifier.tag_redraw(PanelId::UiDesigner);
                });
            }
        },
    );
}