// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Type Definitions and Interaction State
//!
//! Provides the core data structures, actions, category definitions,
//! and interaction state models for the Iris UI GPU SDF Inspector panel.

use irisui::prelude::*;

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

/// User actions emitted by interactive widgets in the Inspector panel.
#[derive(Debug, Clone, PartialEq)]
pub enum InspectorAction {
    /// Renames the specified entity in the ECS world.
    RenameEntity(hecs::Entity, String),
    /// Removes an entire component from the specified entity.
    RemoveComponent(hecs::Entity, &'static str),
    /// Adds a new component to the specified entity from the Add Menu.
    AddComponent(hecs::Entity, &'static str),
    /// Opens the 8-category Add Component cascading menu.
    OpenAddComponentMenu(Point),
    /// Closes the Add Component menu.
    CloseAddComponentMenu,
    /// Opens an Add Component subcategory flyout menu.
    OpenAddSubmenu(ComponentCategory),
    /// Closes the active Add Component subcategory flyout menu.
    CloseAddSubmenu,
    /// Signals that editing has begun on a named component of an entity, optionally supplying pre-edit snapshot bytes.
    StartComponentEdit(hecs::Entity, &'static str, Option<Vec<u8>>),
    /// Signals that editing has completed on a named component of an entity, committing undo history if state changed.
    CommitComponentEdit(hecs::Entity, &'static str),
    /// Commits an atomic modify undo action on a component with explicit pre-edit and post-edit byte snapshots.
    CommitComponentModify(hecs::Entity, &'static str, Vec<u8>, Vec<u8>),
    /// Synchronizes authoritatively inspected Euler rotation angles [rx, ry, rz] in degrees.
    SyncInspectorEuler(f32, f32, f32),
}

/// Input parameters supplied to the Inspector panel layout builder.
pub struct InspectorPanelParams<'a> {
    /// Bounding rectangle of the docked Inspector panel tab.
    pub panel_rect: Rect,
    /// The ECS world containing entities and components.
    pub world: &'a hecs::World,
    /// Currently selected entity in the editor.
    pub selected_entity: Option<hecs::Entity>,
    /// Euler angle cache for rotation editing: `[yaw, pitch, roll]` in degrees.
    pub inspector_euler: &'a [f32; 3],
    /// Hex color string cache for object appearance editing (e.g. `"#6699cc"`).
    pub inspector_color_hex: &'a str,
    /// Saved swatches palette: list of RGBA float arrays `[r, g, b, a]`.
    pub saved_swatches: &'a [[f32; 4]],
    /// Current mouse cursor position in global screen coordinates.
    pub cursor_pos: Point,
    /// Active open Add Component category flyout (if any).
    pub active_submenu: Option<ComponentCategory>,
    /// Whether the top-level Add Component menu is open.
    pub is_add_menu_open: bool,
    /// Whether the floating Color Picker popup is open.
    pub is_color_picker_open: bool,
    /// Active entity rename text buffer if currently being edited.
    pub active_rename_buffer: Option<&'a str>,
    /// Whether all text in the active entity rename buffer is currently selected in blue.
    pub is_rename_all_selected: bool,
    /// Active HEX color text input editing buffer (e.g. `"#ffffff"`).
    pub active_hex_buffer: Option<&'a str>,
    /// Live HSV color cache: `[hue (0..360), saturation (0..1), value (0..1)]`.
    pub inspector_hsv: [f32; 3],
    /// Caret blink phase indicator for text inputs.
    pub blink_caret: bool,
}

impl<'a> InspectorPanelParams<'a> {
    /// Resolves the human-readable display name for the inspected entity,
    /// giving precedence to the live rename buffer if active, followed by the
    /// ECS [`ae_core::ecs::Name`] component, and falling back to a formatted entity ID.
    pub fn entity_display_name(&self, entity: hecs::Entity) -> String {
        self.active_rename_buffer
            .map(String::from)
            .or_else(|| {
                self.world
                    .get::<&ae_core::ecs::Name>(entity)
                    .ok()
                    .map(|n| n.0.clone())
            })
            .unwrap_or_else(|| format!("Entity ({:?})", entity))
    }
}

/// Dragging mode on the 2D HSV color picker.
///
/// Aliased directly to [`irisui::widgets::color_picker::ColorPickerDragMode`].
pub type InspectorColorDragMode = irisui::widgets::color_picker::ColorPickerDragMode;

pub use super::state::{InspectorPanelState, InspectorState};