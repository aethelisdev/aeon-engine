// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Inspector Type Definitions and Hit-Test Targets
//!
//! Provides the core data structures, actions, category definitions,
//! and hit-test target buffers for the Iris UI GPU SDF Inspector panel.

use irisui::prelude::*;

/// Number input field identifier inside the Inspector panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectorNumberInputId {
    /// Transform position X.
    PosX,
    /// Transform position Y.
    PosY,
    /// Transform position Z.
    PosZ,
    /// Transform rotation Euler X (degrees).
    RotX,
    /// Transform rotation Euler Y (degrees).
    RotY,
    /// Transform rotation Euler Z (degrees).
    RotZ,
    /// Transform scale X.
    ScaleX,
    /// Transform scale Y.
    ScaleY,
    /// Transform scale Z.
    ScaleZ,
    /// Collider Half-Height.
    ColliderHalfHeight,
    /// Collider Radius.
    ColliderRadius,
    /// Collider Center Y offset.
    ColliderCenterY,
    /// Collider Box Half-Extent X.
    ColliderBoxX,
    /// Collider Box Half-Extent Y.
    ColliderBoxY,
    /// Collider Box Half-Extent Z.
    ColliderBoxZ,
    /// Collider Friction coefficient.
    ColliderFriction,
    /// Collider Restitution coefficient.
    ColliderRestitution,
    /// Physics Material Friction.
    PhysMatFriction,
    /// Physics Material Restitution.
    PhysMatRestitution,
    /// Character controller height.
    CharacterHeight,
    /// Character controller radius.
    CharacterRadius,
    /// Character controller center Y offset.
    CharacterCenterY,
    /// Character controller max slope angle.
    CharacterMaxSlope,
    /// Character controller step height.
    CharacterStepHeight,
    /// CharacterAction speed or range.
    ActionSpeedRange,
    /// CharacterAction cooldown time (seconds).
    ActionCooldown,
    /// Velocity linear X.
    VelocityX,
    /// Velocity linear Y.
    VelocityY,
    /// Velocity linear Z.
    VelocityZ,
    /// Light intensity.
    LightIntensity,
    /// Light range.
    LightRange,
    /// RigidBody mass (kg).
    RigidBodyMass,
    /// RigidBody gravity scale.
    RigidBodyGravity,
    /// Camera field of view (degrees).
    CameraFov,
    /// Camera near plane.
    CameraNear,
    /// Camera far plane.
    CameraFar,
    /// AudioSource volume gain (0.0 to 2.0).
    AudioVolume,
    /// AudioSource pitch playback multiplier (0.1 to 3.0).
    AudioPitch,
    /// 2D Screen UI Element Offset X (px).
    UiOffsetX,
    /// 2D Screen UI Element Offset Y (px).
    UiOffsetY,
    /// 2D Screen UI Element Size Width (px).
    UiSizeW,
    /// 2D Screen UI Element Size Height (px).
    UiSizeH,
    /// 2D Screen UI Element Pivot X (0.0 to 1.0).
    UiPivotX,
    /// 2D Screen UI Element Pivot Y (0.0 to 1.0).
    UiPivotY,
    /// 2D Screen UI Element Z-Index layer ordering.
    UiZIndex,
    /// 2D Screen UI Element Opacity Alpha (0.0 to 1.0).
    UiAlpha,
    /// 2D Screen UI Text font size (pt).
    UiFontSize,
    /// 2D Screen UI Panel border width (px).
    UiBorderWidth,
    /// 2D Screen UI Panel corner radius (px).
    UiCornerRadius,
    /// 2D Screen UI ProgressBar minimum value.
    UiProgressMin,
    /// 2D Screen UI ProgressBar maximum value.
    UiProgressMax,
    /// 2D Screen UI ProgressBar current value.
    UiProgressVal,
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
            Self::LightIntensity | Self::LightRange => "Light",
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
    /// Prevents physics solver singularities, negative extents, negative mass, and invalid ranges.
    #[must_use]
    pub fn valid_range(self) -> (f32, f32) {
        match self {
            Self::PosX | Self::PosY | Self::PosZ => (-100_000.0, 100_000.0),
            Self::RotX | Self::RotY | Self::RotZ => (-36_000.0, 36_000.0),
            Self::ScaleX | Self::ScaleY | Self::ScaleZ => (-10_000.0, 10_000.0),
            Self::VelocityX | Self::VelocityY | Self::VelocityZ => (-100_000.0, 100_000.0),
            Self::ColliderBoxX | Self::ColliderBoxY | Self::ColliderBoxZ => (0.001, 10_000.0),
            Self::ColliderHalfHeight => (0.001, 10_000.0),
            Self::ColliderRadius => (0.001, 10_000.0),
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
            Self::LightIntensity => (0.0, 1_000_000.0),
            Self::LightRange => (0.01, 10_000.0),
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
    /// Active HEX color text input editing buffer (e.g. `"#ffffff"`).
    pub active_hex_buffer: Option<&'a str>,
    /// Live HSV color cache: `[hue (0..360), saturation (0..1), value (0..1)]`.
    pub inspector_hsv: [f32; 3],
    /// Caret blink phase indicator for text inputs.
    pub blink_caret: bool,
}

/// Hit-test target collections generated during Inspector layout construction.
#[derive(Default)]
pub struct InspectorPanelTargets {
    /// Inspected ECS entity associated with these hit-test targets.
    pub inspected_entity: Option<hecs::Entity>,
    /// Bounding rectangle of the scrollable component cards container.
    pub scroll_container_rect: Rect,
    /// Entity Name input box hit-test rect.
    pub name_input_rect: Rect,
    /// Transform reset buttons: `(AxisType, Rect)`.
    pub transform_reset_btns: Vec<(TransformAxisType, Rect)>,
    /// Numeric drag/input pill boxes: `(FieldId, Rect, Min, Max, CurrentValue)`.
    pub number_inputs: Vec<(InspectorNumberInputId, Rect, f32, f32, f32)>,
    /// String text input boxes: `(FieldId, Rect, CurrentValue)`.
    pub text_inputs: Vec<(InspectorTextInputId, Rect, String)>,
    /// Color swatch box hit-test rect on the Appearance card.
    pub color_swatch_rect: Option<Rect>,
    /// Hex input box hit-test rect on the Appearance card.
    pub hex_input_rect: Option<Rect>,
    /// Add current color to palette button hit-test rect.
    pub add_palette_btn_rect: Option<Rect>,
    /// Clear selected palette swatch button hit-test rect.
    pub clear_palette_btn_rect: Option<Rect>,
    /// Palette color swatch pills: `(SwatchIndex, Rect, Color)`.
    pub palette_swatches: Vec<(usize, Rect, Color)>,
    /// Floating Color Picker popup bounding box.
    pub color_picker_popup_rect: Option<Rect>,
    /// Floating Color Picker close `✖` button bounding box.
    pub color_picker_close_btn_rect: Option<Rect>,
    /// Floating Color Picker 2D Saturation-Value box bounding rect.
    pub color_picker_sv_box_rect: Option<Rect>,
    /// Floating Color Picker vertical Rainbow Hue bar bounding rect.
    pub color_picker_hue_bar_rect: Option<Rect>,
    /// Physics material preset reset button hit-test rect.
    pub preset_btn_rect: Option<Rect>,
    /// Dropdown trigger combo boxes: `(DropdownId, Rect, CurrentSelectedIndex)`.
    pub dropdowns: Vec<(InspectorDropdownId, Rect, usize)>,
    /// Active floating dropdown popup bounding box.
    pub active_dropdown_popup_rect: Option<Rect>,
    /// Interactive items inside an active dropdown popup: `(OptionIndex, Rect)`.
    pub dropdown_items: Vec<(usize, Rect)>,
    /// Component boolean checkboxes: `(CheckboxId, Rect, CurrentValue)`.
    pub checkboxes: Vec<(ComponentCheckboxId, Rect, bool)>,
    /// Component trash/delete buttons: `(ComponentName, Rect)`.
    pub component_delete_btns: Vec<(&'static str, Rect)>,
    /// `➕ Add Component` button bounding box.
    pub add_component_btn_rect: Rect,
    /// `💾 Save as Prefab` button bounding box.
    pub save_prefab_btn_rect: Rect,
    /// Bounding rectangle of the top-level Add Component menu.
    pub active_add_menu_rect: Option<Rect>,
    /// Interactive items inside the top-level Add Component menu: `(Category, Rect)`.
    pub add_menu_categories: Vec<(ComponentCategory, Rect)>,
    /// Bounding rectangle of the cascading Add Component submenu flyout.
    pub active_submenu_rect: Option<Rect>,
    /// Interactive items inside the cascading Add Component submenu: `(ComponentName, Rect)`.
    pub submenu_components: Vec<(&'static str, Rect)>,
    /// Audio file picker `📁` button hit-test rect.
    pub audio_pick_btn_rect: Option<Rect>,
    /// Audio play/stop preview toggle button hit-test rect.
    pub audio_play_btn_rect: Option<Rect>,
    /// Unparent `❌` button hit-test rect.
    pub unparent_btn_rect: Option<Rect>,
}

/// Parameter descriptor for rendering a compact numeric input row.
#[derive(Debug, Clone, Copy)]
pub struct CompactNumericRowParams {
    /// Label text displayed on the left.
    pub label: &'static str,
    /// Target numeric input ID for editing.
    pub input_id: InspectorNumberInputId,
    /// Current float value.
    pub val: f32,
    /// Vertical Y position within the card.
    pub row_y: f32,
    /// Width of the label column.
    pub label_w: f32,
    /// Width of the input pill box.
    pub box_w: f32,
    /// Optional suffix unit string (e.g. `m/s`, `s`, `°`).
    pub unit: Option<&'static str>,
}

/// Parameter descriptor for rendering a standard compact ComboBox row.
#[derive(Debug, Clone, Copy)]
pub struct ComboboxRowParams {
    /// Label text displayed on the left.
    pub label: &'static str,
    /// Currently selected option display text.
    pub selected_text: &'static str,
    /// Target dropdown identifier.
    pub dropdown_id: InspectorDropdownId,
    /// Width of the label column.
    pub label_w: f32,
    /// Vertical Y position within the card.
    pub row_y: f32,
}

/// Parameter descriptor for rendering a ComboBox row accompanied by an action button.
#[derive(Debug, Clone, Copy)]
pub struct ComboboxWithButtonParams {
    /// Label text displayed on the left.
    pub label: &'static str,
    /// Currently selected option display text.
    pub selected_text: &'static str,
    /// Target dropdown identifier.
    pub dropdown_id: InspectorDropdownId,
    /// Button label text (e.g. `↺ Preset`).
    pub btn_label: &'static str,
    /// Vertical Y position within the card.
    pub row_y: f32,
}