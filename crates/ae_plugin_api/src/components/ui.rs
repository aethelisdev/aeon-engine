// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular In-Game UI, Canvas Layout, and HUD Component Primitives.
//!
//! Provides hardware-agnostic 2D UI elements, anchoring rules, and widget definitions
//! for rich interactive game user interfaces, health meters, text typography, and input controls.
//!

use serde::{Deserialize, Serialize};

/// Screen-space anchor positions defining alignment on the 2D canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UiAnchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl UiAnchor {
    /// Computes the absolute pixel origin `[x, y]` on a screen of dimensions `(screen_w, screen_h)`.
    #[inline]
    pub fn compute_origin(self, screen_w: f32, screen_h: f32) -> [f32; 2] {
        match self {
            Self::TopLeft => [0.0, 0.0],
            Self::TopCenter => [screen_w * 0.5, 0.0],
            Self::TopRight => [screen_w, 0.0],
            Self::CenterLeft => [0.0, screen_h * 0.5],
            Self::Center => [screen_w * 0.5, screen_h * 0.5],
            Self::CenterRight => [screen_w, screen_h * 0.5],
            Self::BottomLeft => [0.0, screen_h],
            Self::BottomCenter => [screen_w * 0.5, screen_h],
            Self::BottomRight => [screen_w, screen_h],
        }
    }
}

/// Screen-space 2D bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct UiRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl UiRect {
    /// Constructs a new rectangle from min and max coordinates.
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Computes width of the rectangle.
    pub fn width(self) -> f32 {
        (self.max_x - self.min_x).max(0.0)
    }

    /// Computes height of the rectangle.
    pub fn height(self) -> f32 {
        (self.max_y - self.min_y).max(0.0)
    }

    /// Checks if a 2D point lies within the rectangle bounds.
    pub fn contains(self, point: [f32; 2]) -> bool {
        point[0] >= self.min_x
            && point[0] <= self.max_x
            && point[1] >= self.min_y
            && point[1] <= self.max_y
    }
}

/// Texture image slicing and scaling modes for 2D UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UiSliceMode {
    #[default]
    Stretch,
    Fit,
    NineSlice,
    Tile,
}

/// Text alignment options for typography labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UiTextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Layout flow types for auto-arranging child UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UiLayoutType {
    #[default]
    Horizontal,
    Vertical,
    Grid,
}

/// Primary 2D UI positioning, sizing, and anchoring component.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiElement {
    /// Screen anchor position.
    pub anchor: UiAnchor,
    /// Offset distance `[x, y]` relative to the anchor point.
    pub offset: [f32; 2],
    /// Width and height `[w, h]` in screen-space pixels.
    pub size: [f32; 2],
    /// Pivot alignment normalized coordinates `[0.0 to 1.0]`. Default is `[0.5, 0.5]` (center).
    pub pivot: [f32; 2],
    /// Render order layer index (higher values render on top).
    pub z_index: i32,
    /// Element opacity multiplier `[0.0 to 1.0]`.
    pub alpha: f32,
    /// Element visibility toggle.
    pub visible: bool,
}

impl Default for UiElement {
    fn default() -> Self {
        Self {
            anchor: UiAnchor::TopLeft,
            offset: [0.0, 0.0],
            size: [100.0, 30.0],
            pivot: [0.5, 0.5],
            z_index: 0,
            alpha: 1.0,
            visible: true,
        }
    }
}

impl UiElement {
    /// Creates a new positioned UI element with standard dimensions.
    pub fn new(anchor: UiAnchor, offset: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            anchor,
            offset,
            size,
            pivot: [0.5, 0.5],
            z_index: 0,
            alpha: 1.0,
            visible: true,
        }
    }

    /// Computes the screen-space bounding rectangle given canvas dimensions.
    pub fn compute_rect(&self, screen_w: f32, screen_h: f32) -> UiRect {
        let origin = self.anchor.compute_origin(screen_w, screen_h);
        let center_x = origin[0] + self.offset[0];
        let center_y = origin[1] + self.offset[1];
        let half_w = self.size[0] * self.pivot[0];
        let half_h = self.size[1] * self.pivot[1];

        UiRect::new(
            center_x - half_w,
            center_y - half_h,
            center_x + (self.size[0] - half_w),
            center_y + (self.size[1] - half_h),
        )
    }
}

/// Panel container component providing styled background box and border framing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiPanel {
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub corner_radius: f32,
}

impl Default for UiPanel {
    fn default() -> Self {
        Self {
            background_color: [0.08, 0.10, 0.14, 0.85],
            border_color: [0.25, 0.35, 0.45, 0.80],
            border_width: 1.0,
            corner_radius: 4.0,
        }
    }
}

/// Text label component rendered on a UI element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiText {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub alignment: UiTextAlignment,
    pub shadow_color: Option<[f32; 4]>,
}

impl Default for UiText {
    fn default() -> Self {
        Self {
            text: "Text Label".to_string(),
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
            alignment: UiTextAlignment::Left,
            shadow_color: Some([0.0, 0.0, 0.0, 0.6]),
        }
    }
}

impl UiText {
    /// Creates a new text label with specified string and font size.
    pub fn new(text: impl Into<String>, font_size: f32) -> Self {
        Self {
            text: text.into(),
            font_size,
            color: [1.0, 1.0, 1.0, 1.0],
            alignment: UiTextAlignment::Left,
            shadow_color: Some([0.0, 0.0, 0.0, 0.6]),
        }
    }

    /// Builder method to specify text RGBA color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Builder method to specify text alignment.
    pub fn with_alignment(mut self, alignment: UiTextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Progress bar component for health meters, mana bars, and experience gauges.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiProgressBar {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub fill_color: [f32; 4],
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub corner_radius: f32,
}

impl Default for UiProgressBar {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            value: 100.0,
            fill_color: [0.2, 0.85, 0.35, 1.0], // Neon Green
            background_color: [0.08, 0.10, 0.14, 0.85],
            border_color: [0.3, 0.4, 0.5, 0.8],
            corner_radius: 3.0,
        }
    }
}

impl UiProgressBar {
    /// Computes the normalized fill percentage in range `[0.0, 1.0]`.
    #[inline]
    pub fn fraction(self) -> f32 {
        let range = self.max - self.min;
        if range.abs() < 1e-4 {
            0.0
        } else {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        }
    }
}

/// Interactive button component with multi-state visual feedback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiButton {
    pub text: String,
    pub normal_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub pressed_color: [f32; 4],
    pub disabled_color: [f32; 4],
    pub is_enabled: bool,
}

impl Default for UiButton {
    fn default() -> Self {
        Self {
            text: "Button".to_string(),
            normal_color: [0.18, 0.22, 0.28, 1.0],
            hover_color: [0.26, 0.34, 0.44, 1.0],
            pressed_color: [0.12, 0.15, 0.20, 1.0],
            disabled_color: [0.10, 0.10, 0.12, 0.50],
            is_enabled: true,
        }
    }
}

impl UiButton {
    /// Creates a new button with specified label and default styling.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }
}

/// Image / icon display component supporting custom sprites and 9-slice frames.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiImage {
    pub sprite_id: Option<u64>,
    pub tint: [f32; 4],
    pub uv_rect: [f32; 4],
    pub slice_mode: UiSliceMode,
}

impl Default for UiImage {
    fn default() -> Self {
        Self {
            sprite_id: None,
            tint: [1.0, 1.0, 1.0, 1.0],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            slice_mode: UiSliceMode::Stretch,
        }
    }
}

/// Numerical slider component for audio volume, sensitivity, and numeric parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiSlider {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub track_color: [f32; 4],
    pub thumb_color: [f32; 4],
    pub step: Option<f32>,
}

impl Default for UiSlider {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: 0.5,
            track_color: [0.15, 0.18, 0.24, 1.0],
            thumb_color: [0.35, 0.65, 0.95, 1.0],
            step: None,
        }
    }
}

/// Boolean toggle checkbox component for settings and options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiCheckbox {
    pub is_checked: bool,
    pub label: String,
    pub box_color: [f32; 4],
    pub check_color: [f32; 4],
}

impl Default for UiCheckbox {
    fn default() -> Self {
        Self {
            is_checked: false,
            label: "Option".to_string(),
            box_color: [0.15, 0.18, 0.24, 1.0],
            check_color: [0.25, 0.85, 0.45, 1.0],
        }
    }
}

/// Interactive single-line text input field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiTextInput {
    pub text: String,
    pub placeholder: String,
    pub max_length: Option<usize>,
    pub is_focused: bool,
}

impl Default for UiTextInput {
    fn default() -> Self {
        Self {
            text: String::new(),
            placeholder: "Enter text...".to_string(),
            max_length: Some(64),
            is_focused: false,
        }
    }
}

/// Auto-layout group component for aligning child UI widgets horizontally or vertically.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutGroup {
    pub layout_type: UiLayoutType,
    pub spacing: f32,
    pub padding: [f32; 4], // [left, top, right, bottom]
}

impl Default for UiLayoutGroup {
    fn default() -> Self {
        Self {
            layout_type: UiLayoutType::Vertical,
            spacing: 8.0,
            padding: [8.0, 8.0, 8.0, 8.0],
        }
    }
}

/// Marker component for the in-game Player Health Bar HUD element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerHealthBarTag;

/// Marker component for the in-game Score Display HUD element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScoreDisplayTag;

/// Marker component for center screen aiming reticle / crosshair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReticleTag;