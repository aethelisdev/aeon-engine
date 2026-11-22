// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Top Toolbar
//!
//! Renders the 34px flex-flow top control strip featuring an `anchored_select_pill` for
//! canvas aspect ratio selection, stack-formatted zoom controls, snap cycling, anchor/grid
//! toggles, and the `Add Element` palette trigger without imperative bindings or heap allocations.

use super::popups::{ASPECT_RATIO_OPTIONS, ASPECT_RATIO_PRESETS, aspect_ratio_index};
use super::types::{
    UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN, UI_DESIGNER_TAG_ASPECT_BTN,
    UI_DESIGNER_TAG_GRID_BTN, UI_DESIGNER_TAG_SNAP_BTN, UI_DESIGNER_TAG_TOOLBAR,
    UI_DESIGNER_TAG_ZOOM_IN, UI_DESIGNER_TAG_ZOOM_OUT, UI_DESIGNER_TAG_ZOOM_RESET, UiDesignerState,
};
use crate::ui::iris_bridge::icons::ICON_SNAP_GRID;
use crate::ui::panel_layout::PanelId;
use irisui::dock::DeferredNotifier;
use irisui::prelude::*;

/// Height of the UI Designer top control bar in screen pixels.
pub const UI_DESIGNER_TOOLBAR_HEIGHT: f32 = 34.0;

/// Cycles the grid snap spacing through `[Free, 8px, 16px, 32px]`.
#[inline]
#[must_use]
const fn cycle_snap_grid(current: Option<f32>) -> Option<f32> {
    match current {
        None => Some(8.0),
        Some(s) if s < 10.0 => Some(16.0),
        Some(s) if s < 20.0 => Some(32.0),
        Some(_) => None,
    }
}

/// Emits the 34px top control bar inside the flex-column UI Designer panel container.
///
/// Mutates `state` directly on user interaction and marks [`PanelId::UiDesigner`] dirty
/// on `notifier` with zero intermediate `let` statements or heap string formatting.
pub fn build_designer_toolbar(
    scope: &mut UiScope<'_>,
    state: &mut UiDesignerState,
    notifier: &mut DeferredNotifier<PanelId>,
) {
    scope.container_tagged(
        "UiDesignerToolbar",
        Style::new()
            .height(UI_DESIGNER_TOOLBAR_HEIGHT)
            .flex_shrink(0.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(5.0, 8.0, 5.0, 8.0))
            .gap(6.0)
            .background(Color::rgba(0.080, 0.088, 0.110, 0.98))
            .border(1.0, Color::rgba(0.18, 0.21, 0.28, 0.90))
            .box_shadow(0.0, 2.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 0.60)),
        WidgetRole::Default,
        UI_DESIGNER_TAG_TOOLBAR,
        |tb| {
            tb.anchored_select_pill(
                AnchoredSelectPillSpec {
                    popup_name: "AspectRatioPopup",
                    tag: UI_DESIGNER_TAG_ASPECT_BTN,
                    label: StackString::<48>::from_args(format_args!(
                        "📐 {} ▾",
                        state.aspect_ratio.label()
                    ))
                    .as_str(),
                    trigger_style: SelectPillTriggerStyle::DropdownSized {
                        width: 142.0,
                        height: 24.0,
                    },
                    popup_width: 175.0,
                    placement: PopupPlacement::Below,
                    options: ASPECT_RATIO_OPTIONS,
                    selected_index: Some(aspect_ratio_index(state.aspect_ratio)),
                },
                |idx| {
                    state.aspect_ratio =
                        ASPECT_RATIO_PRESETS[idx.min(ASPECT_RATIO_PRESETS.len() - 1)];
                    notifier.tag_redraw(PanelId::UiDesigner);
                },
            );

            tb.vertical_divider(20.0, Color::rgba(0.22, 0.25, 0.32, 0.80));

            tb.add(
                ToolbarButton::new("−")
                    .width(24.0)
                    .tag(UI_DESIGNER_TAG_ZOOM_OUT)
                    .tooltip("Zoom out canvas"),
            )
            .on_click(|| {
                state.zoom = (state.zoom - 0.1).clamp(0.2, 5.0);
                notifier.tag_redraw(PanelId::UiDesigner);
            });

            tb.add(
                ToolbarButton::from_args(format_args!("{:.0}%", state.zoom * 100.0))
                    .width(50.0)
                    .tag(UI_DESIGNER_TAG_ZOOM_RESET)
                    .tooltip("Reset canvas zoom to 100% and center view"),
            )
            .on_click(|| {
                state.zoom = 1.0;
                state.pan_offset = [0.0, 0.0];
                notifier.tag_redraw(PanelId::UiDesigner);
            });

            tb.add(
                ToolbarButton::new("+")
                    .width(24.0)
                    .tag(UI_DESIGNER_TAG_ZOOM_IN)
                    .tooltip("Zoom in canvas"),
            )
            .on_click(|| {
                state.zoom = (state.zoom + 0.1).clamp(0.2, 5.0);
                notifier.tag_redraw(PanelId::UiDesigner);
            });

            tb.vertical_divider(20.0, Color::rgba(0.22, 0.25, 0.32, 0.80));

            tb.add(
                match state.snap_grid {
                    Some(step) => ToolbarButton::from_args(format_args!("Snap: {step:.0}px")),
                    None => ToolbarButton::new("Snap: Free"),
                }
                .width(82.0)
                .tag(UI_DESIGNER_TAG_SNAP_BTN)
                .tooltip("Toggle grid snap interval (Free, 8px, 16px, 32px)"),
            )
            .on_click(|| {
                state.snap_grid = cycle_snap_grid(state.snap_grid);
                notifier.tag_redraw(PanelId::UiDesigner);
            });

            tb.add(
                ToolbarButton::new("⚓ Anchors")
                    .active(state.show_anchor_guides)
                    .tag(UI_DESIGNER_TAG_ANCHORS_BTN)
                    .tooltip("Toggle anchor guides and snap visualization"),
            )
            .on_click(|| {
                state.show_anchor_guides = !state.show_anchor_guides;
                notifier.tag_redraw(PanelId::UiDesigner);
            });

            tb.add(
                ToolbarButton::new("Grid")
                    .icon(ICON_SNAP_GRID)
                    .active(state.show_grid)
                    .tag(UI_DESIGNER_TAG_GRID_BTN)
                    .tooltip("Toggle canvas background alignment grid"),
            )
            .on_click(|| {
                state.show_grid = !state.show_grid;
                notifier.tag_redraw(PanelId::UiDesigner);
            });

            tb.vertical_divider(20.0, Color::rgba(0.22, 0.25, 0.32, 0.80));

            tb.add_popup_trigger(
                Button::new("Add Element")
                    .width(118.0)
                    .fill(Color::rgba(0.20, 0.90, 0.60, 0.25))
                    .text_color(Color::rgba(0.20, 0.90, 0.60, 1.0))
                    .tag(UI_DESIGNER_TAG_ADD_ELEMENT_BTN)
                    .tooltip("Open UI element creation palette"),
                UI_DESIGNER_TAG_ADD_ELEMENT_BTN,
            );
        },
    );
}