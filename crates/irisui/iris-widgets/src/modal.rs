// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generic modal dialogue tags, actions, semantic tag evaluation, and visual styling.
//!
//! Declarative modal layouts and backdrop scrims are constructed directly via
//! [`crate::UiScope::modal_scrim`] and [`crate::UiScope::modal_dialog`].

use iris_core::color::Color;

/// Persistent semantic tag for full-screen backdrop modal scrims.
pub const MODAL_TAG_SCRIM: u64 = 0xF001;
/// Persistent semantic tag for modal header '✖' close buttons.
pub const MODAL_TAG_CLOSE: u64 = 0xF002;
/// Persistent semantic tag for modal primary confirm action buttons.
pub const MODAL_TAG_CONFIRM: u64 = 0xF003;
/// Persistent semantic tag for modal secondary cancel action buttons.
pub const MODAL_TAG_CANCEL: u64 = 0xF004;
/// Persistent semantic tag for modal destructive danger confirm action buttons.
pub const MODAL_TAG_DANGER: u64 = 0xF005;

/// High-level semantic actions triggered by modal dialog clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalDialogAction {
    /// Header '✖' close button was activated.
    Close,
    /// Primary confirm action button was activated.
    Confirm,
    /// Secondary cancel action button was activated.
    Cancel,
    /// Backdrop scrim was clicked (dismissal).
    ScrimDismiss,
}

/// Evaluates a semantic tag dispatched from a hit test to resolve modal action.
#[inline]
pub fn evaluate_modal_tag(tag: u64) -> Option<ModalDialogAction> {
    match tag {
        MODAL_TAG_SCRIM => Some(ModalDialogAction::ScrimDismiss),
        MODAL_TAG_CLOSE => Some(ModalDialogAction::Close),
        MODAL_TAG_CONFIRM | MODAL_TAG_DANGER => Some(ModalDialogAction::Confirm),
        MODAL_TAG_CANCEL => Some(ModalDialogAction::Cancel),
        _ => None,
    }
}

/// Styling and color metrics configuration for modal dialogue boxes.
#[derive(Debug, Clone)]
pub struct ModalDialogStyle {
    /// Background fill color of the main dialog card.
    pub background: Color,
    /// Border stroke thickness in logical pixels.
    pub border_width: f32,
    /// Border stroke color.
    pub border_color: Color,
    /// Outer corner radius of the dialog card.
    pub border_radius: f32,
    /// Drop shadow vertical offset.
    pub box_shadow_offset_y: f32,
    /// Drop shadow blur radius.
    pub box_shadow_blur: f32,
    /// Drop shadow color.
    pub box_shadow_color: Color,
    /// Color of the semi-transparent screen backdrop scrim blocker.
    pub scrim_color: Color,
    /// Height of the top header bar in logical pixels.
    pub header_height: f32,
    /// Background color of the header bar.
    pub header_background: Color,
    /// Border color separating the header bar.
    pub header_border_color: Color,
    /// Title text label color.
    pub title_color: Color,
    /// Idle color of the header '✖' close button glyph.
    pub close_color_idle: Color,
    /// Hovered color of the header '✖' close button glyph.
    pub close_color_hover: Color,
    /// Idle background fill color of the primary Confirm button.
    pub confirm_btn_bg_idle: Color,
    /// Hovered background fill color of the primary Confirm button.
    pub confirm_btn_bg_hover: Color,
    /// Text color of the primary Confirm button.
    pub confirm_btn_text: Color,
    /// Idle background fill color of the secondary Cancel button.
    pub cancel_btn_bg_idle: Color,
    /// Hovered background fill color of the secondary Cancel button.
    pub cancel_btn_bg_hover: Color,
    /// Text color of the secondary Cancel button.
    pub cancel_btn_text: Color,
}

impl Default for ModalDialogStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(0.08, 0.08, 0.10, 0.98),
            border_width: 1.0,
            border_color: Color::rgba(0.20, 0.22, 0.28, 1.0),
            border_radius: 8.0,
            box_shadow_offset_y: 8.0,
            box_shadow_blur: 24.0,
            box_shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.70),
            scrim_color: Color::rgba(0.0, 0.0, 0.0, 0.55),
            header_height: 32.0,
            header_background: Color::rgba(0.06, 0.06, 0.08, 1.0),
            header_border_color: Color::rgba(0.18, 0.20, 0.26, 1.0),
            title_color: Color::rgba(0.88, 0.88, 0.92, 1.0),
            close_color_idle: Color::rgba(0.60, 0.60, 0.65, 1.0),
            close_color_hover: Color::rgba(1.0, 0.35, 0.35, 1.0),
            confirm_btn_bg_idle: Color::rgba(0.14, 0.40, 0.58, 1.0),
            confirm_btn_bg_hover: Color::rgba(0.18, 0.50, 0.72, 1.0),
            confirm_btn_text: Color::WHITE,
            cancel_btn_bg_idle: Color::rgba(0.14, 0.15, 0.18, 1.0),
            cancel_btn_bg_hover: Color::rgba(0.20, 0.22, 0.28, 1.0),
            cancel_btn_text: Color::rgba(0.75, 0.78, 0.84, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_modal_tags() {
        assert_eq!(
            evaluate_modal_tag(MODAL_TAG_SCRIM),
            Some(ModalDialogAction::ScrimDismiss)
        );
        assert_eq!(
            evaluate_modal_tag(MODAL_TAG_CLOSE),
            Some(ModalDialogAction::Close)
        );
        assert_eq!(
            evaluate_modal_tag(MODAL_TAG_CONFIRM),
            Some(ModalDialogAction::Confirm)
        );
        assert_eq!(
            evaluate_modal_tag(MODAL_TAG_DANGER),
            Some(ModalDialogAction::Confirm)
        );
        assert_eq!(
            evaluate_modal_tag(MODAL_TAG_CANCEL),
            Some(ModalDialogAction::Cancel)
        );
        assert_eq!(evaluate_modal_tag(0x1234), None);
    }
}