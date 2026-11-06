// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Color Property Rows & Autonomous Color Picker Popups
//!
//! Provides the [`UiScope::prop_color`] and [`UiScope::prop_color_styled`] methods
//! for immediate two-way bound color editing with interactive preview swatches and
//! autonomous floating 2D HSV color picker popups on [`iris_core::UiLayer::Popup`].

use std::ops::Deref;

use iris_core::geometry::Rect;
use iris_core::response::WidgetResponse;
use iris_core::{
    AlignItems, Color, JustifyContent, Style, TextAlign, WidgetCursor, WidgetId, WidgetRole,
};

use crate::color_picker::HsvColorPickerState;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::combine_seeds;
use crate::declarative::widget::DEFAULT_PROPERTY_LABEL_WIDTH;
use crate::declarative::widget::color_picker::ColorPicker;

/// Comprehensive interaction response returned by evaluating a declarative color property row.
///
/// Implements [`Deref<Target = WidgetResponse>`] for transparent access to generic interaction
/// flags (`clicked()`, `hovered()`) while providing color-specific signals (`.changed()`, `.committed()`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyColorResponse {
    /// Generic widget response of the interactive color swatch button pill.
    pub response: WidgetResponse,
    /// Allocated widget identifier of the interactive color swatch pill node.
    pub swatch_id: WidgetId,
    /// Whether the color value was mutated during this interaction pass.
    pub changed: bool,
    /// Whether dragging ended this frame (triggering an Undo/Redo commit snapshot).
    pub committed: bool,
    /// Active concrete [`Color`] value.
    pub color: Color,
    /// Whether the elevated floating color picker popup is currently open.
    pub is_open: bool,
}

impl Deref for PropertyColorResponse {
    type Target = WidgetResponse;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl PropertyColorResponse {
    /// Constructs a new [`PropertyColorResponse`].
    #[inline]
    #[must_use]
    pub const fn new(
        response: WidgetResponse,
        swatch_id: WidgetId,
        changed: bool,
        committed: bool,
        color: Color,
        is_open: bool,
    ) -> Self {
        Self {
            response,
            swatch_id,
            changed,
            committed,
            color,
            is_open,
        }
    }

    /// Returns `true` if the color channels changed during this interaction frame.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }

    /// Returns `true` if pointer drag release or numeric edit commit occurred this frame.
    #[inline]
    #[must_use]
    pub const fn committed(&self) -> bool {
        self.committed
    }

    /// Returns the active concrete [`Color`].
    #[inline]
    #[must_use]
    pub const fn color(&self) -> Color {
        self.color
    }

    /// Returns whether the elevated color picker popup menu is currently open.
    #[inline]
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.is_open
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative two-way bound color property row modifying `color` in-place.
    ///
    /// Renders a left-aligned property label defaulting to [`DEFAULT_PROPERTY_LABEL_WIDTH`], an interactive
    /// color swatch button pill displaying the current hex string, and autonomously manages an elevated floating
    /// 2D HSV [`ColorPicker`] popup menu on [`iris_core::UiLayer::Popup`]. Mutates `*color` immediately upon interaction
    /// and returns a [`PropertyColorResponse`].
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `color` - Mutable reference to the bound [`Color`] state modified in-place upon selection.
    pub fn prop_color(&mut self, label: &str, color: &mut Color) -> PropertyColorResponse {
        self.prop_color_styled(label, color, None, None)
    }

    /// Emits a declarative two-way bound color property row with custom column width and swatch width.
    ///
    /// # Arguments
    /// * `label` - Human-readable property description displayed on the left (supports `##` separator).
    /// * `color` - Mutable reference to the bound [`Color`] state modified in-place upon selection.
    /// * `label_width` - Optional column width in physical pixels allocated for the label. Defaults to [`DEFAULT_PROPERTY_LABEL_WIDTH`].
    /// * `swatch_width` - Optional width in physical pixels allocated for the color swatch button pill. Defaults to `72.0`.
    pub fn prop_color_styled(
        &mut self,
        label: &str,
        color: &mut Color,
        label_width: Option<f32>,
        swatch_width: Option<f32>,
    ) -> PropertyColorResponse {
        let base_tag = self.tag_for(label);
        let was_open = self.tree.active_combobox_tag == Some(base_tag);
        let mut is_open = was_open;
        let mut changed = false;
        let mut committed = false;
        let mut response = WidgetResponse::default();
        let mut slot_swatch_id = WidgetId::default();
        let mut slot_executed = false;

        let lbl_w = label_width.unwrap_or(DEFAULT_PROPERTY_LABEL_WIDTH);
        let swatch_w = swatch_width.unwrap_or(72.0);

        let row_id = self.property_row_styled(label, Some(lbl_w), |slot| {
            let swatch_id = slot.tree.create_node();
            slot_swatch_id = swatch_id;

            let border_col = if is_open {
                Color::rgba(0.0, 0.75, 1.0, 0.95)
            } else {
                Color::rgba(0.24, 0.26, 0.32, 0.85)
            };

            if let Some(node) = slot.tree.get_mut(swatch_id) {
                node.set_name("ColorSwatchPill");
                node.interactive = true;
                node.role = WidgetRole::Button;
                node.tag = base_tag;
                node.cursor = Some(WidgetCursor::Pointer);
                node.set_style(
                    Style::new()
                        .width(swatch_w)
                        .height(20.0)
                        .background(*color)
                        .border(1.0, border_col)
                        .border_radius(3.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center),
                );
            }
            let _ = slot.tree.add_child(slot.parent, swatch_id);

            // Compute luminance-adjusted text color for readable hex label
            let lum = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
            let text_col = if lum > 0.55 {
                Color::rgba(0.08, 0.09, 0.12, 0.95)
            } else {
                Color::WHITE
            };

            let hex_str = format!(
                "#{:02X}{:02X}{:02X}",
                (color.r.clamp(0.0, 1.0) * 255.0) as u8,
                (color.g.clamp(0.0, 1.0) * 255.0) as u8,
                (color.b.clamp(0.0, 1.0) * 255.0) as u8
            );

            let mut swatch_scope = slot.child_scope(swatch_id);
            swatch_scope.label_styled_passive(
                "ColorHexText",
                &hex_str,
                9.5,
                text_col,
                TextAlign::Center,
                Style::new().width(swatch_w).height(16.0),
            );

            let (clicked, hovered, _) = slot.check_interaction(swatch_id);

            // Toggle open state upon swatch click
            if clicked {
                if was_open {
                    slot.tree.active_combobox_tag = None;
                    is_open = false;
                } else {
                    slot.tree.active_combobox_tag = Some(base_tag);
                    is_open = true;
                }
            }

            // Render elevated floating 2D HSV color picker on UiLayer::Popup when open
            if is_open {
                let anchor_rect = slot
                    .tree
                    .tag_rect(base_tag)
                    .or_else(|| {
                        slot.tree
                            .get(swatch_id)
                            .map(|n| n.computed_rect)
                            .filter(|r| r.width > 0.0)
                    })
                    .unwrap_or(Rect::new(0.0, 0.0, swatch_w, 20.0));

                let popup_w = 196.0;
                let root_id = slot.tree.root().unwrap_or(slot.parent);
                let mut root_scope = slot.child_scope(root_id);

                let popup_resp = root_scope.popup_anchored_named(
                    "ColorPickerPopup",
                    anchor_rect,
                    popup_w,
                    |popup| {
                        let mut hsv = HsvColorPickerState::from_color(*color);
                        let picker_resp = popup.add(
                            ColorPicker::new(&mut hsv)
                                .sv_size(150.0, 110.0)
                                .tag(combine_seeds(base_tag, 0xC010_C0DE)),
                        );
                        (
                            picker_resp.changed(),
                            picker_resp.drag_released(),
                            picker_resp.color(),
                        )
                    },
                );

                let (picker_changed, drag_released, new_col) = popup_resp.inner;
                if picker_changed {
                    *color = new_col;
                    changed = true;
                }
                if drag_released {
                    committed = true;
                }

                let popup_rect = popup_resp.response.rect;

                // Outside click dismissal
                if !clicked
                    && let Some(click_pos) = slot
                        .tree
                        .event_state
                        .frame_clicked_pos
                        .or(slot.tree.event_state.press_origin)
                    && !popup_rect.contains_point(click_pos)
                    && !anchor_rect.contains_point(click_pos)
                {
                    slot.tree.active_combobox_tag = None;
                    is_open = false;
                }
            }

            response = WidgetResponse::new(swatch_id, clicked, hovered, changed);
            slot_executed = true;
        });

        if !slot_executed {
            response = self.response(row_id);
        }

        PropertyColorResponse::new(
            response,
            slot_swatch_id,
            changed,
            committed,
            *color,
            is_open,
        )
    }
}