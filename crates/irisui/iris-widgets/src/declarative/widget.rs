// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unified Declarative Widget Trait & Builders
//!
//! Provides the [`Widget`] trait and idiomatic builder structs ([`Button`], [`ToolbarButton`],
//! [`Checkbox`], [`EmptyBox`], [`ScrollArea`], [`Toolbar`]) to eliminate combinatorial method explosion
//! (`*_tagged`, `*_named_*`, `*_styled_*`) in favor of a unified `scope.add(widget)` pattern.
//!

pub mod button;
pub mod checkbox;
pub mod chip;
pub mod color_picker;
pub mod color_swatch;
pub mod combobox;
pub mod drag_vec3;
pub mod empty_box;
pub mod numeric_drag;
pub mod scroll_area;
pub mod toolbar;
pub mod toolbar_button;

pub use button::Button;
pub use checkbox::Checkbox;
pub use chip::Chip;
pub use color_picker::{ColorPicker, ColorPickerResponse};
pub use color_swatch::ColorSwatch;
pub use combobox::{ComboBox, ComboBoxResponse};
pub use drag_vec3::{DragVec3, Vec3Response};
pub use empty_box::EmptyBox;
pub use numeric_drag::{NumericDrag, ScalarResponse};
pub use scroll_area::ScrollAreaView;
pub use toolbar::Toolbar;
pub use toolbar_button::ToolbarButton;

use super::scope::core::UiScope;

/// Trait implemented by configurable declarative widgets.
///
/// Enables consumers to use the unified builder pattern:
/// ```ignore
/// scope.add(Button::new("▶ Play").fill(Color::GREEN).tag(PLAY_TAG));
/// ```
pub trait Widget {
    /// Return type produced by evaluating this widget in the UI scope.
    type Response;

    /// Evaluates and attaches this widget to the given UI scope.
    fn ui(self, scope: &mut UiScope<'_>) -> Self::Response;
}