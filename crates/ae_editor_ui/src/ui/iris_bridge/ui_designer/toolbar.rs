// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Toolbar
//!
//! Renders the elevated 34px top control bar with aspect ratio preset selection,
//! zoom controls, grid snapping toggles, anchor guides, and canonical hardware atlas `ICON_PLUS` button
//! using 100% declarative [`UiScope`] flexbox architecture and 64-bit semantic tags.
//!

use super::types::{
    UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN, UI_DESIGNER_TAG_ASPECT_BTN,
    UI_DESIGNER_TAG_GRID_BTN, UI_DESIGNER_TAG_SNAP_BTN, UI_DESIGNER_TAG_TOOLBAR,
    UI_DESIGNER_TAG_ZOOM_IN, UI_DESIGNER_TAG_ZOOM_OUT, UI_DESIGNER_TAG_ZOOM_RESET,
    UiDesignerPanelParams,
};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;

/// Height of the UI Designer top control bar in screen pixels.
pub const UI_DESIGNER_TOOLBAR_HEIGHT: f32 = 34.0;

/// Builds the 34px elevated top control bar in the declarative [`UiScope`] tree.
pub fn build_designer_toolbar(scope: &mut UiScope<'_>, params: &UiDesignerPanelParams<'_>) {
    let tb_style = Style::new()
        .position_absolute()
        .left(0.0)
        .top(0.0)
        .width(params.panel_rect.width)
        .height(UI_DESIGNER_TOOLBAR_HEIGHT)
        .flex_row()
        .align_items(AlignItems::Center)
        .padding_insets(Insets::new(5.0, 8.0, 5.0, 8.0))
        .gap(6.0)
        .background(Color::rgba(0.080, 0.088, 0.110, 0.98))
        .border(1.0, Color::rgba(0.18, 0.21, 0.28, 0.90))
        .box_shadow(0.0, 2.0, 8.0, Color::rgba(0.0, 0.0, 0.0, 0.60));

    scope.container_tagged(
        "UiDesignerToolbar",
        tb_style,
        WidgetRole::Default,
        UI_DESIGNER_TAG_TOOLBAR,
        |tb| {
            // ── 1. Aspect Ratio Selector Button ───────────────────────────────────
            let aspect_label = format!("📐 {} ▾", params.state.aspect_ratio.label());
            tb.toolbar_button_tagged(aspect_label, 142.0, UI_DESIGNER_TAG_ASPECT_BTN);

            tb.vertical_divider(20.0, Color::rgba(0.22, 0.25, 0.32, 0.80));

            // ── 2. Zoom Controls: Zoom Out (−), Reset (100%), Zoom In (+) ─────────
            tb.toolbar_button_tagged("−", 24.0, UI_DESIGNER_TAG_ZOOM_OUT);

            let zoom_text = format!("{:.0}%", params.state.zoom * 100.0);
            tb.toolbar_button_tagged(zoom_text, 50.0, UI_DESIGNER_TAG_ZOOM_RESET);

            tb.toolbar_button_tagged("+", 24.0, UI_DESIGNER_TAG_ZOOM_IN);

            tb.vertical_divider(20.0, Color::rgba(0.22, 0.25, 0.32, 0.80));

            // ── 3. Grid Snap Toggle ───────────────────────────────────────────────
            let snap_text = match params.state.snap_grid {
                Some(s) => format!("Snap: {:.0}px", s),
                None => "Snap: Free".to_string(),
            };
            tb.toolbar_button_tagged(snap_text, 82.0, UI_DESIGNER_TAG_SNAP_BTN);

            // ── 4. Anchors & Grid Toggles ─────────────────────────────────────────
            tb.toggle_pill_tagged(
                "⚓ Anchors",
                params.state.show_anchor_guides,
                UI_DESIGNER_TAG_ANCHORS_BTN,
            );
            tb.toggle_pill_tagged("⊞ Grid", params.state.show_grid, UI_DESIGNER_TAG_GRID_BTN);

            tb.vertical_divider(20.0, Color::rgba(0.22, 0.25, 0.32, 0.80));

            // ── 5. ➕ Add Element Palette Button ──────────────────────────────────
            tb.button_with_icon_styled_tagged(
                ICON_PLUS,
                Color::rgba(0.20, 0.90, 0.60, 1.0),
                "Add Element",
                Some(118.0),
                UI_DESIGNER_TAG_ADD_ELEMENT_BTN,
            );
        },
    );
}