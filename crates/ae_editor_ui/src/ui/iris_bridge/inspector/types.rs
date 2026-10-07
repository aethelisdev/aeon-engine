// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Type Definitions and Interaction State
//!
//! Provides the core data structures, actions, category definitions,
//! and interaction state models for the Iris UI GPU SDF Inspector panel.

use irisui::prelude::*;

/// Number input field identifier inside the Inspector panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum InspectorNumberInputId {
    /// Transform position X.
    PosX = 0,
    /// Transform position Y.
    PosY = 1,
    /// Transform position Z.
    PosZ = 2,
    /// Transform rotation Euler X (degrees).
    RotX = 3,
    /// Transform rotation Euler Y (degrees).
    RotY = 4,
    /// Transform rotation Euler Z (degrees).
    RotZ = 5,
    /// Transform scale X.
    ScaleX = 6,
    /// Transform scale Y.
    ScaleY = 7,
    /// Transform scale Z.
    ScaleZ = 8,
    /// Collider Half-Height.
    ColliderHalfHeight = 9,
    /// Collider Radius.
    ColliderRadius = 10,
    /// Collider Center Y offset.
    ColliderCenterY = 11,
    /// Collider Box Half-Extent X.
    ColliderBoxX = 12,
    /// Collider Box Half-Extent Y.
    ColliderBoxY = 13,
    /// Collider Box Half-Extent Z.
    ColliderBoxZ = 14,
    /// Collider Friction coefficient.
    ColliderFriction = 15,
    /// Collider Restitution coefficient.
    ColliderRestitution = 16,
    /// Physics Material Friction.
    PhysMatFriction = 17,
    /// Physics Material Restitution.
    PhysMatRestitution = 18,
    /// Character controller height.
    CharacterHeight = 19,
    /// Character controller radius.
    CharacterRadius = 20,
    /// Character controller center Y offset.
    CharacterCenterY = 21,
    /// Character max slope angle (degrees).
    CharacterMaxSlope = 22,
    /// Character controller step height.
    CharacterStepHeight = 23,
    /// CharacterAction speed or range.
    ActionSpeedRange = 24,
    /// CharacterAction cooldown time (seconds).
    ActionCooldown = 25,
    /// Velocity linear X.
    VelocityX = 26,
    /// Velocity linear Y.
    VelocityY = 27,
    /// Velocity linear Z.
    VelocityZ = 28,
    /// Rotator angular speed (rad/s).
    RotatorSpeed = 29,
    /// Rotator axis X, Y, Z components.
    RotatorAxisX = 30,
    RotatorAxisY = 31,
    RotatorAxisZ = 32,
    /// Light offset Y and color components.
    LightOffsetY = 33,
    LightColorR = 34,
    LightColorG = 35,
    /// RigidBody mass (kg).
    RigidBodyMass = 36,
    /// RigidBody gravity scale.
    RigidBodyGravity = 37,
    /// Camera field of view (degrees).
    CameraFov = 38,
    /// Camera near plane.
    CameraNear = 39,
    /// Camera far plane.
    CameraFar = 40,
    /// AudioSource volume gain (0.0 to 2.0).
    AudioVolume = 41,
    /// AudioSource pitch playback multiplier (0.1 to 3.0).
    AudioPitch = 42,
    /// 2D Screen UI Element Offset X (px).
    UiOffsetX = 43,
    /// 2D Screen UI Element Offset Y (px).
    UiOffsetY = 44,
    /// 2D Screen UI Element Size Width (px).
    UiSizeW = 45,
    /// 2D Screen UI Element Size Height (px).
    UiSizeH = 46,
    /// 2D Screen UI Element Pivot X (0.0 to 1.0).
    UiPivotX = 47,
    /// 2D Screen UI Element Pivot Y (0.0 to 1.0).
    UiPivotY = 48,
    /// 2D Screen UI Element Z-Index layer ordering.
    UiZIndex = 49,
    /// 2D Screen UI Element Opacity Alpha (0.0 to 1.0).
    UiAlpha = 50,
    /// 2D Screen UI Text font size (pt).
    UiFontSize = 51,
    /// 2D Screen UI Panel border width (px).
    UiBorderWidth = 52,
    /// 2D Screen UI Panel corner radius (px).
    UiCornerRadius = 53,
    /// 2D Screen UI ProgressBar minimum value.
    UiProgressMin = 54,
    /// 2D Screen UI ProgressBar maximum value.
    UiProgressMax = 55,
    /// 2D Screen UI ProgressBar current value.
    UiProgressVal = 56,
}

impl InspectorNumberInputId {
    /// Returns the canonical ComponentRegistry type name associated with this numeric field.
    #[must_use]
    pub fn component_name(self) -> &'static str {
        match self {
            Self::PosX | Self::PosY | Self::PosZ => "Position",
            Self::RotX | Self::RotY | Self::RotZ => "Rotation",
            Self::ScaleX | Self::ScaleY | Self::ScaleZ => "Scale",
            Self::VelocityX | Self::VelocityY | Self::VelocityZ => "Velocity",
            Self::ColliderHalfHeight
            | Self::ColliderRadius
            | Self::ColliderCenterY
            | Self::ColliderBoxX
            | Self::ColliderBoxY
            | Self::ColliderBoxZ
            | Self::ColliderFriction
            | Self::ColliderRestitution => "Collider",
            Self::PhysMatFriction | Self::PhysMatRestitution => "PhysicsMaterial",
            Self::CharacterHeight
            | Self::CharacterRadius
            | Self::CharacterCenterY
            | Self::CharacterMaxSlope
            | Self::CharacterStepHeight => "CharacterController",
            Self::ActionSpeedRange | Self::ActionCooldown => "CharacterAction",
            Self::RotatorSpeed | Self::RotatorAxisX | Self::RotatorAxisY | Self::RotatorAxisZ => {
                "Rotator"
            }
            Self::LightOffsetY | Self::LightColorR | Self::LightColorG => "Light",
            Self::RigidBodyMass | Self::RigidBodyGravity => "RigidBody",
            Self::CameraFov | Self::CameraNear | Self::CameraFar => "Camera",
            Self::AudioVolume | Self::AudioPitch => "AudioSource",
            Self::UiOffsetX
            | Self::UiOffsetY
            | Self::UiSizeW
            | Self::UiSizeH
            | Self::UiPivotX
            | Self::UiPivotY
            | Self::UiZIndex
            | Self::UiAlpha => "UiElement",
            Self::UiFontSize => "UiText",
            Self::UiBorderWidth | Self::UiCornerRadius => "UiPanel",
            Self::UiProgressMin | Self::UiProgressMax | Self::UiProgressVal => "UiProgressBar",
        }
    }

    /// Returns the valid numerical range `[min, max]` allowed for this property.
    ///
    /// Prevents physics solver singularities, negative extents, negative mass, and invalid ranges.
    #[must_use]
    pub fn valid_range(self) -> (f32, f32) {
        match self {
            Self::PosX | Self::PosY | Self::PosZ => (-100_000.0, 100_000.0),
            Self::RotX | Self::RotY | Self::RotZ => (-36_000.0, 36_000.0),
            Self::ScaleX | Self::ScaleY | Self::ScaleZ => (-10_000.0, 10_000.0),
            Self::VelocityX | Self::VelocityY | Self::VelocityZ => (-100_000.0, 100_000.0),
            Self::ColliderBoxX | Self::ColliderBoxY | Self::ColliderBoxZ => (0.001, 10_000.0),
            Self::ColliderHalfHeight | Self::ColliderRadius => (0.001, 10_000.0),
            Self::ColliderCenterY => (-10_000.0, 10_000.0),
            Self::ColliderFriction | Self::PhysMatFriction => (0.0, 100.0),
            Self::ColliderRestitution | Self::PhysMatRestitution => (0.0, 1.0),
            Self::CharacterHeight => (0.05, 1_000.0),
            Self::CharacterRadius => (0.01, 500.0),
            Self::CharacterCenterY => (-1_000.0, 1_000.0),
            Self::CharacterMaxSlope => (0.0, 89.9),
            Self::CharacterStepHeight => (0.0, 100.0),
            Self::ActionSpeedRange => (0.0, 10_000.0),
            Self::ActionCooldown => (0.0, 3_600.0),
            Self::RotatorSpeed => (-100.0, 100.0),
            Self::RotatorAxisX | Self::RotatorAxisY | Self::RotatorAxisZ => (-1.0, 1.0),
            Self::LightOffsetY => (-1_000.0, 1_000.0),
            Self::LightColorR | Self::LightColorG => (0.0, 100.0),
            Self::RigidBodyMass => (0.001, 100_000.0),
            Self::RigidBodyGravity => (-100.0, 100.0),
            Self::CameraFov => (1.0, 179.0),
            Self::CameraNear => (0.001, 1_000.0),
            Self::CameraFar => (0.01, 100_000.0),
            Self::AudioVolume => (0.0, 10.0),
            Self::AudioPitch => (0.05, 5.0),
            Self::UiOffsetX | Self::UiOffsetY => (-10_000.0, 10_000.0),
            Self::UiSizeW | Self::UiSizeH => (1.0, 10_000.0),
            Self::UiPivotX | Self::UiPivotY => (0.0, 1.0),
            Self::UiZIndex => (-1_000.0, 1_000.0),
            Self::UiAlpha => (0.0, 1.0),
            Self::UiFontSize => (1.0, 500.0),
            Self::UiBorderWidth => (0.0, 200.0),
            Self::UiCornerRadius => (0.0, 200.0),
            Self::UiProgressMin | Self::UiProgressMax | Self::UiProgressVal => {
                (-100_000.0, 100_000.0)
            }
        }
    }

    /// Clamps an incoming numeric value to the safe range prescribed by this input identifier.
    #[must_use]
    pub fn clamp_value(self, val: f32) -> f32 {
        let (min_val, max_val) = self.valid_range();
        val.clamp(min_val, max_val)
    }

    /// Converts this numeric field identifier to a zero-based tag index.
    ///
    /// Serves as the canonical Single Source of Truth (SSOT) mapping between
    /// variant order and persistent 64-bit semantic tags.
    #[inline]
    #[must_use]
    pub const fn to_tag_index(self) -> usize {
        self as usize
    }

    /// Resolves a zero-based tag index back into its corresponding numeric field identifier.
    ///
    /// Serves as the canonical Single Source of Truth (SSOT) mapping from
    /// semantic tag offsets to [`InspectorNumberInputId`] variants.
    #[inline]
    #[must_use]
    pub const fn from_tag_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::PosX),
            1 => Some(Self::PosY),
            2 => Some(Self::PosZ),
            3 => Some(Self::RotX),
            4 => Some(Self::RotY),
            5 => Some(Self::RotZ),
            6 => Some(Self::ScaleX),
            7 => Some(Self::ScaleY),
            8 => Some(Self::ScaleZ),
            9 => Some(Self::ColliderHalfHeight),
            10 => Some(Self::ColliderRadius),
            11 => Some(Self::ColliderCenterY),
            12 => Some(Self::ColliderBoxX),
            13 => Some(Self::ColliderBoxY),
            14 => Some(Self::ColliderBoxZ),
            15 => Some(Self::ColliderFriction),
            16 => Some(Self::ColliderRestitution),
            17 => Some(Self::PhysMatFriction),
            18 => Some(Self::PhysMatRestitution),
            19 => Some(Self::CharacterHeight),
            20 => Some(Self::CharacterRadius),
            21 => Some(Self::CharacterCenterY),
            22 => Some(Self::CharacterMaxSlope),
            23 => Some(Self::CharacterStepHeight),
            24 => Some(Self::ActionSpeedRange),
            25 => Some(Self::ActionCooldown),
            26 => Some(Self::VelocityX),
            27 => Some(Self::VelocityY),
            28 => Some(Self::VelocityZ),
            29 => Some(Self::RotatorSpeed),
            30 => Some(Self::RotatorAxisX),
            31 => Some(Self::RotatorAxisY),
            32 => Some(Self::RotatorAxisZ),
            33 => Some(Self::LightOffsetY),
            34 => Some(Self::LightColorR),
            35 => Some(Self::LightColorG),
            36 => Some(Self::RigidBodyMass),
            37 => Some(Self::RigidBodyGravity),
            38 => Some(Self::CameraFov),
            39 => Some(Self::CameraNear),
            40 => Some(Self::CameraFar),
            41 => Some(Self::AudioVolume),
            42 => Some(Self::AudioPitch),
            43 => Some(Self::UiOffsetX),
            44 => Some(Self::UiOffsetY),
            45 => Some(Self::UiSizeW),
            46 => Some(Self::UiSizeH),
            47 => Some(Self::UiPivotX),
            48 => Some(Self::UiPivotY),
            49 => Some(Self::UiZIndex),
            50 => Some(Self::UiAlpha),
            51 => Some(Self::UiFontSize),
            52 => Some(Self::UiBorderWidth),
            53 => Some(Self::UiCornerRadius),
            54 => Some(Self::UiProgressMin),
            55 => Some(Self::UiProgressMax),
            56 => Some(Self::UiProgressVal),
            _ => None,
        }
    }
}

/// Dropdown selector identifier inside the Inspector panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorDropdownId {
    /// RigidBody Body Type (Dynamic, Kinematic, Static).
    RigidBodyType,
    /// Collider Shape (Box, Sphere, Capsule, Cylinder).
    ColliderShape,
    /// Physics Material Surface Type (Flesh, Concrete, Metal, Wood, Ice, Rubber, Glass).
    SurfaceType,
    /// Light Type (Point, Directional, Spot).
    LightType,
    /// Camera Projection (Perspective, Orthographic).
    CameraProjection,
    /// 3D Shape Type (Cube, Sphere, Cylinder, Capsule, Torus, Plane).
    ShapeType,
    /// 2D Screen UI Element Anchor Preset.
    UiAnchor,
    /// 2D Screen UI Text horizontal alignment.
    UiTextAlignment,
}

impl InspectorDropdownId {
    /// Returns the canonical ComponentRegistry type name associated with this dropdown selector.
    #[must_use]
    pub fn component_name(self) -> &'static str {
        match self {
            Self::RigidBodyType => "RigidBody",
            Self::ColliderShape => "Collider",
            Self::SurfaceType => "PhysicsMaterial",
            Self::LightType => "Light",
            Self::CameraProjection => "Camera",
            Self::ShapeType => "Shape",
            Self::UiAnchor => "UiElement",
            Self::UiTextAlignment => "UiText",
        }
    }
}

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
#[derive(Debug, Clone)]
pub enum InspectorAction {
    /// Renames the specified entity in the ECS world.
    RenameEntity(hecs::Entity, String),
    /// Focuses the entity rename text field.
    FocusRename,
    /// Resets a specific transform axis (Position, Rotation, or Scale) on an entity.
    ResetTransform(hecs::Entity, TransformAxisType),
    /// Applies a direct numeric value change to a component property on an entity.
    SetNumberValue(hecs::Entity, InspectorNumberInputId, f32),
    /// Applies an object color change from the Appearance card to an entity.
    SetObjectColor(hecs::Entity, Color),
    /// Signals that interactive color picker dragging has begun on an entity, snapshotting pre-edit color.
    StartColorEdit(hecs::Entity),
    /// Live modifies object color in ECS for real-time viewport preview without spamming undo history.
    LiveSetObjectColor(hecs::Entity, Color),
    /// Signals that interactive color picker dragging has finished on an entity, committing a single atomic undo entry.
    CommitColorEdit(hecs::Entity),
    /// Focuses the HEX color text input for typing.
    FocusHexInput,
    /// Toggles the floating Color Picker popup.
    ToggleColorPicker,
    /// Adds the current object color into the user's saved palette.
    AddColorToPalette(Color),
    /// Clears custom user-saved swatches from the palette.
    ClearCustomPalette,
    /// Removes a color swatch from the saved palette by index.
    RemoveColorFromPalette(usize),
    /// Selects a dropdown combo option on an entity.
    SelectDropdown(hecs::Entity, InspectorDropdownId, usize),
    /// Toggles a boolean checkbox on a component of an entity.
    ToggleCheckbox(hecs::Entity, ComponentCheckboxId),
    /// Removes an entire component from the specified entity.
    RemoveComponent(hecs::Entity, &'static str),
    /// Adds a new component to the specified entity from the Add Menu.
    AddComponent(hecs::Entity, &'static str),
    /// Saves the specified entity and its entire hierarchy as a reusable Prefab file.
    SaveAsPrefab(hecs::Entity),
    /// Opens the 8-category Add Component cascading menu.
    OpenAddComponentMenu(Point),
    /// Closes the Add Component menu.
    CloseAddComponentMenu,
    /// Opens an Add Component subcategory flyout menu.
    OpenAddSubmenu(ComponentCategory),
    /// Closes the active Add Component subcategory flyout menu.
    CloseAddSubmenu,
    /// Resets Physics Material properties to the active SurfaceType preset on an entity.
    ResetPhysMatPreset(hecs::Entity),
    /// Signals that numeric input editing has begun on an entity, capturing pre-edit component state.
    StartNumberEdit(hecs::Entity, InspectorNumberInputId),
    /// Signals that numeric input editing has completed on an entity, committing undo history if value changed.
    CommitNumberEdit(hecs::Entity, InspectorNumberInputId),
    /// Opens native file dialog to select sound asset for AudioSource on an entity.
    PickAudioFile(hecs::Entity),
    /// Toggles play/stop preview playback for AudioSource on an entity.
    ToggleAudioPlayback(hecs::Entity),
    /// Unparents the specified entity from its parent.
    Unparent(hecs::Entity),
    /// Sets a string text property on a component of an entity.
    SetTextValue(hecs::Entity, InspectorTextInputId, String),
}

/// String text input field identifier inside the Inspector panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorTextInputId {
    /// UiText component string content.
    UiTextContent,
    /// UiTextInput component placeholder string.
    UiTextInputPlaceholder,
}

impl InspectorTextInputId {
    /// Returns the canonical ComponentRegistry type name associated with this text field.
    #[must_use]
    pub fn component_name(self) -> &'static str {
        match self {
            Self::UiTextContent => "UiText",
            Self::UiTextInputPlaceholder => "UiTextInput",
        }
    }
}

/// Transform axis group for reset actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformAxisType {
    /// Resets Position to (0, 0, 0).
    Position,
    /// Resets Rotation Euler to (0, 0, 0) degrees.
    Rotation,
    /// Resets Scale to (1, 1, 1).
    Scale,
}

/// Checkbox field identifier on components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentCheckboxId {
    /// Collider IsSensor trigger flag.
    ColliderIsSensor,
    /// AudioSource loop flag.
    AudioLoop,
    /// AudioSource spatial 3D audio flag.
    AudioSpatial,
    /// AudioSource play on start flag.
    AudioPlayOnStart,
    /// Light shadow casting flag.
    LightCastShadows,
    /// UI Button interactable state.
    UiInteractable,
    /// UI Element visible state flag.
    UiVisible,
}

impl ComponentCheckboxId {
    /// Returns the canonical ComponentRegistry type name associated with this boolean checkbox.
    #[must_use]
    pub fn component_name(self) -> &'static str {
        match self {
            Self::ColliderIsSensor => "Collider",
            Self::AudioLoop | Self::AudioSpatial | Self::AudioPlayOnStart => "AudioSource",
            Self::LightCastShadows => "Light",
            Self::UiInteractable => "UiButton",
            Self::UiVisible => "UiElement",
        }
    }
}

/// Active numeric input field display state passed to Inspector rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveNumberInputState<'a> {
    /// Field identifier.
    pub id: InspectorNumberInputId,
    /// Live text buffer being edited.
    pub buffer: &'a str,
    /// Cursor index within the buffer for caret rendering.
    pub cursor_idx: usize,
    /// Whether all text in the field is selected.
    pub is_all_selected: bool,
}

impl<'a> ActiveNumberInputState<'a> {
    /// Converts this active number input state to an Iris UI [`NumericInputEditState`].
    #[must_use]
    pub fn to_edit_state(&self, blink_caret: bool) -> NumericInputEditState<'a> {
        NumericInputEditState {
            buffer: self.buffer,
            cursor_idx: self.cursor_idx,
            is_all_selected: self.is_all_selected,
            blink_caret,
        }
    }
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
    /// Vertical scroll offset in pixels.
    pub scroll_y: f32,
    /// Active open dropdown identifier (if any).
    pub active_dropdown: Option<InspectorDropdownId>,
    /// Active open Add Component category flyout (if any).
    pub active_submenu: Option<ComponentCategory>,
    /// Whether the top-level Add Component menu is open.
    pub is_add_menu_open: bool,
    /// Whether the floating Color Picker popup is open.
    pub is_color_picker_open: bool,
    /// Active numeric input field display and cursor state if currently focused.
    pub active_number_input: Option<ActiveNumberInputState<'a>>,
    /// Active string text input field and its buffer: `(FieldId, BufferText)`.
    pub active_text_input: Option<(InspectorTextInputId, &'a str)>,
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
    /// Currently hovered widget semantic tag for real-time hover styling.
    pub hovered_tag: Option<u64>,
}

/// Active horizontal mouse drag state for interactive Inspector numeric inputs.
#[derive(Debug, Clone, Copy)]
pub struct InspectorNumberDragState {
    /// Inspected target ECS entity.
    pub entity: hecs::Entity,
    /// Target numeric input identifier.
    pub id: InspectorNumberInputId,
    /// Starting X coordinate of the cursor when mouse was pressed.
    pub start_x: f32,
    /// Starting value of the numeric field before dragging began.
    pub start_val: f32,
    /// Lower clamp bound.
    pub min_val: f32,
    /// Upper clamp bound.
    pub max_val: f32,
    /// Value delta per dragged pixel.
    pub sensitivity: f32,
    /// Whether mouse has dragged beyond threshold.
    pub has_dragged: bool,
}

/// Dragging mode on the 2D HSV color picker.
///
/// Aliased directly to [`irisui::prelude::ColorPickerDragMode`].
pub type InspectorColorDragMode = irisui::prelude::ColorPickerDragMode;

/// Active numeric text input session in Inspector.
#[derive(Debug, Clone)]
pub struct InspectorNumberInputSession {
    /// Target entity being modified.
    pub entity: hecs::Entity,
    /// Identifier of the specific numeric field being edited.
    pub id: InspectorNumberInputId,
    /// Text buffer containing the current expression or number.
    pub buffer: String,
    /// Byte cursor index within the buffer for caret rendering and insertion.
    pub cursor_idx: usize,
    /// Whether the text in the buffer is fully selected.
    pub is_all_selected: bool,
    /// Initial baseline value before editing started.
    pub initial_val: f32,
    /// Minimum allowed value for clamping.
    pub min_val: f32,
    /// Maximum allowed value for clamping.
    pub max_val: f32,
}

pub use super::state::{InspectorPanelState, InspectorState};