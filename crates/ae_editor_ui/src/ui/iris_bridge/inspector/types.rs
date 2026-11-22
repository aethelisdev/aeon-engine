// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Type Definitions and Interaction State
//!
//! Provides the core data structures, actions, category definitions,
//! and interaction state models for the Iris UI GPU SDF Inspector panel.

use irisui::prelude::DEFAULT_PROPERTY_LABEL_WIDTH;

/// Canonical default column width in logical pixels allocated for Inspector property labels.
pub const DEFAULT_INSPECTOR_LABEL_WIDTH: f32 = DEFAULT_PROPERTY_LABEL_WIDTH;

/// 8-Category classification for the `➕ Add Component` cascading menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentCategory {
    /// Skeletal and keyframe animation components.
    Animation,
    /// Positional and ambient audio source components.
    Audio,
    /// Gameplay logic, character controllers, actions, and tags.
    Gameplay,
    /// ECS Scene hierarchy, parenting, and transform components.
    Hierarchy,
    /// Rigidbodies, colliders, and physics materials.
    Physics,
    /// Lights, cameras, meshes, and material renderers.
    Rendering,
    /// In-game 2D UI and HUD designer components.
    UiHud,
    /// Dynamic scripting and custom reflected components.
    CustomDynamic,
}

impl ComponentCategory {
    /// Returns the human-readable display title for this category.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Animation => "Animation",
            Self::Audio => "Audio",
            Self::Gameplay => "Gameplay",
            Self::Hierarchy => "Hierarchy",
            Self::Physics => "Physics",
            Self::Rendering => "Rendering",
            Self::UiHud => "UI & HUD",
            Self::CustomDynamic => "Custom / Dynamic",
        }
    }

    /// Returns the category icon.
    #[must_use]
    pub fn icon(self) -> &'static str {
        match self {
            Self::Animation => "🎬",
            Self::Audio => "🔊",
            Self::Gameplay => "🎮",
            Self::Hierarchy => "🌳",
            Self::Physics => "🛡️",
            Self::Rendering => "💡",
            Self::UiHud => "🎨",
            Self::CustomDynamic => "⚡",
        }
    }

    /// Returns the optional GPU texture atlas icon coordinates for this category.
    #[must_use]
    pub const fn atlas_icon(self) -> Option<[f32; 4]> {
        match self {
            Self::Audio => Some(crate::ui::iris_bridge::icons::ICON_AUDIO),
            Self::Physics => Some(crate::ui::iris_bridge::icons::ICON_GEAR),
            Self::Rendering => Some(crate::ui::iris_bridge::icons::ICON_LIGHT),
            Self::Hierarchy => Some(crate::ui::iris_bridge::icons::ICON_FOLDER),
            _ => None,
        }
    }

    /// Converts the component category into a unique numeric tag for widget routing.
    #[inline]
    pub const fn to_tag(self) -> u64 {
        match self {
            Self::Animation => 1,
            Self::Audio => 2,
            Self::Gameplay => 3,
            Self::Hierarchy => 4,
            Self::Physics => 5,
            Self::Rendering => 6,
            Self::UiHud => 7,
            Self::CustomDynamic => 8,
        }
    }

    /// Reconstructs the component category from a widget tag if it represents a branch.
    #[inline]
    pub const fn from_tag(tag: u64) -> Option<Self> {
        match tag {
            1 => Some(Self::Animation),
            2 => Some(Self::Audio),
            3 => Some(Self::Gameplay),
            4 => Some(Self::Hierarchy),
            5 => Some(Self::Physics),
            6 => Some(Self::Rendering),
            7 => Some(Self::UiHud),
            8 => Some(Self::CustomDynamic),
            _ => None,
        }
    }
}

pub use super::state::{InspectorPanelState, InspectorState};