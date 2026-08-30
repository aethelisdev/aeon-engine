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
    /// Renames the selected entity in the ECS world.
    RenameEntity(String),
    /// Focuses the entity rename text field.
    FocusRename,
    /// Resets a specific transform axis (Position, Rotation, or Scale).
    ResetTransform(TransformAxisType),
    /// Applies a direct numeric value change to a component property.
    SetNumberValue(InspectorNumberInputId, f32),
    /// Applies an object color change from the Appearance card.
    SetObjectColor(Color),
    /// Adds the current object color into the user's saved palette.
    AddColorToPalette(Color),
    /// Removes a color swatch from the saved palette by index.
    RemoveColorFromPalette(usize),
    /// Selects a dropdown combo option.
    SelectDropdown(InspectorDropdownId, usize),
    /// Toggles a boolean checkbox on a component.
    ToggleCheckbox(ComponentCheckboxId),
    /// Removes an entire component from the selected entity.
    RemoveComponent(&'static str),
    /// Adds a new component to the selected entity from the Add Menu.
    AddComponent(&'static str),
    /// Saves the selected entity and its entire hierarchy as a reusable Prefab file.
    SaveAsPrefab,
    /// Opens the 8-category Add Component cascading menu.
    OpenAddComponentMenu(Point),
    /// Closes the Add Component menu.
    CloseAddComponentMenu,
    /// Opens an Add Component subcategory flyout menu.
    OpenAddSubmenu(ComponentCategory),
    /// Closes the active Add Component subcategory flyout menu.
    CloseAddSubmenu,
    /// Resets Physics Material properties to the active SurfaceType preset.
    ResetPhysMatPreset,
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
    /// Light shadow casting flag.
    LightCastShadows,
    /// UI Button interactable state.
    UiInteractable,
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
    /// Active numeric input field and its text editing buffer: `(FieldId, BufferText)`.
    pub active_number_input: Option<(InspectorNumberInputId, &'a str)>,
    /// Active entity rename text buffer if currently being edited.
    pub active_rename_buffer: Option<&'a str>,
    /// Caret blink phase indicator for text inputs.
    pub blink_caret: bool,
}

/// Hit-test target collections generated during Inspector layout construction.
#[derive(Default)]
pub struct InspectorPanelTargets {
    /// Bounding rectangle of the scrollable component cards container.
    pub scroll_container_rect: Rect,
    /// Entity Name input box hit-test rect.
    pub name_input_rect: Rect,
    /// Transform reset buttons: `(AxisType, Rect)`.
    pub transform_reset_btns: Vec<(TransformAxisType, Rect)>,
    /// Numeric drag/input pill boxes: `(FieldId, Rect, Min, Max, CurrentValue)`.
    pub number_inputs: Vec<(InspectorNumberInputId, Rect, f32, f32, f32)>,
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