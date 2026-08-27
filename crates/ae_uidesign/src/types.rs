// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Core types, enums, and interaction records for the Aeon UI Designer (AUD).
//!

use serde::{Deserialize, Serialize};

/// Canonical UI element type variants for spawning and categorizing in the UI Designer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UiElementType {
    /// Generic rectangle/panel container.
    Panel,
    /// Standard progress and gauge fill meter.
    ProgressBar,
    /// Single or multi-line typography text label.
    Text,
    /// Interactive clickable button.
    Button,
    /// Image, sprite, or texture quad.
    Image,
    /// Numeric slider bar with draggable thumb.
    Slider,
    /// Binary toggle checkbox with text label.
    Checkbox,
    /// Single-line interactive text input field.
    TextInput,
    /// HUD Preset: Player health gauge bound to gameplay events.
    HealthBar,
    /// HUD Preset: Gameplay score readout bound to score events.
    ScoreDisplay,
}

impl UiElementType {
    /// Returns human-readable label for editor menus.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Panel => "Panel / Canvas Box",
            Self::ProgressBar => "Progress Bar",
            Self::Text => "Text Label",
            Self::Button => "Interactive Button",
            Self::Image => "Image / Icon",
            Self::Slider => "Numeric Slider",
            Self::Checkbox => "Toggle Checkbox",
            Self::TextInput => "Text Input Field",
            Self::HealthBar => "Health Bar (Player Tag)",
            Self::ScoreDisplay => "Score Display (Score Tag)",
        }
    }

    /// Returns the standard Unicode icon associated with this UI element.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Panel => "🟩",
            Self::ProgressBar => "📊",
            Self::Text => "🔤",
            Self::Button => "🔘",
            Self::Image => "🖼️",
            Self::Slider => "🎚️",
            Self::Checkbox => "☑️",
            Self::TextInput => "📝",
            Self::HealthBar => "❤️",
            Self::ScoreDisplay => "⭐",
        }
    }
}

/// Predefined standard screen aspect ratio presets for the 2D visual canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CanvasAspectRatio {
    /// Standard 16:9 widescreen format (1920x1080 Full HD).
    #[default]
    Ratio16x9,
    /// 16:10 productivity widescreen format (1920x1200 WUXGA).
    Ratio16x10,
    /// Classic 4:3 fullscreen monitor format (1440x1080).
    Ratio4x3,
    /// 21:9 cinema ultrawide format (2560x1080).
    Ratio21x9,
}

impl CanvasAspectRatio {
    /// Returns the virtual reference resolution `[width, height]` in pixels.
    pub fn resolution(&self) -> [f32; 2] {
        match self {
            Self::Ratio16x9 => [1920.0, 1080.0],
            Self::Ratio16x10 => [1920.0, 1200.0],
            Self::Ratio4x3 => [1440.0, 1080.0],
            Self::Ratio21x9 => [2560.0, 1080.0],
        }
    }

    /// Returns display label for the aspect ratio dropdown selector.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ratio16x9 => "16:9 (1080p Full HD)",
            Self::Ratio16x10 => "16:10 (WUXGA)",
            Self::Ratio4x3 => "4:3 (Classic)",
            Self::Ratio21x9 => "21:9 (Ultrawide)",
        }
    }
}

/// Active drag interaction state for moving UI elements in the 2D canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiDragState {
    /// The entity being dragged.
    pub entity: hecs::Entity,
    /// Cached canvas origin of the element's anchor point at drag start.
    pub anchor_origin: [f32; 2],
    /// Mouse position in virtual canvas coordinates at drag start.
    pub drag_start_mouse_canvas: [f32; 2],
    /// Offset of the UI element at drag start.
    pub initial_offset: [f32; 2],
}

/// Event actions emitted by the UI Designer panel to be consumed by the editor/engine.
#[derive(Debug, Clone, PartialEq)]
pub enum UiDesignerAction {
    /// Requests spawning a new UI element into the active ECS world.
    SpawnElement(UiElementType),
    /// Requests selecting or deselecting a UI element entity.
    SelectEntity(Option<hecs::Entity>),
    /// Updates the offset of a UI element during dragging.
    UpdateElementOffset {
        entity: hecs::Entity,
        offset: [f32; 2],
    },
}