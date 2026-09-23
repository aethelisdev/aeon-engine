// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Types & Parameters
//!
//! Provides data structures, action events, and interaction target collectors for the
//! Iris UI hardware SDF Viewport HUD subsystem.

use ae_editor::gizmo::{GizmoMode, GizmoSpace};
pub use ae_editor::scene_gizmo::SceneViewSnap;
use ae_editor::snapping::SnapSettings;
use ae_renderer::camera::{Camera, ProjectionMode};
use hecs::{Entity, World};
use irisui::prelude::*;

/// Semantic tag for the Viewport Camera Mode dropdown trigger button.
pub const TAG_VIEWPORT_CAMERA_MODE: u64 = 2001;
/// Semantic tag for the Viewport Shading Mode dropdown trigger button.
pub const TAG_VIEWPORT_SHADING_MODE: u64 = 2002;
/// Semantic tag for the Gizmo Select tool button.
pub const TAG_GIZMO_SELECT: u64 = 2010;
/// Semantic tag for the Gizmo Translate tool button.
pub const TAG_GIZMO_TRANSLATE: u64 = 2011;
/// Semantic tag for the Gizmo Rotate tool button.
pub const TAG_GIZMO_ROTATE: u64 = 2012;
/// Semantic tag for the Gizmo Scale tool button.
pub const TAG_GIZMO_SCALE: u64 = 2013;
/// Semantic tag for the Gizmo Coordinate Space toggle button.
pub const TAG_GIZMO_SPACE_TOGGLE: u64 = 2020;
/// Semantic tag for the Compass +X axis knob.
pub const TAG_COMPASS_POS_X: u64 = 2030;
/// Semantic tag for the Compass +Y axis knob.
pub const TAG_COMPASS_POS_Y: u64 = 2031;
/// Semantic tag for the Compass +Z axis knob.
pub const TAG_COMPASS_POS_Z: u64 = 2032;
/// Semantic tag for the Compass -X axis dot.
pub const TAG_COMPASS_NEG_X: u64 = 2033;
/// Semantic tag for the Compass -Y axis dot.
pub const TAG_COMPASS_NEG_Y: u64 = 2034;
/// Semantic tag for the Compass -Z axis dot.
pub const TAG_COMPASS_NEG_Z: u64 = 2035;
/// Semantic tag for the Compass 3D orientation canvas background and axis lines.
pub const TAG_COMPASS_CANVAS: u64 = 2036;
/// Semantic tag for the In-Game Pause Resume Game button.
pub const TAG_PLAY_RESUME: u64 = 2040;
/// Semantic tag for the In-Game Pause Exit to Editor button.
pub const TAG_PLAY_EXIT: u64 = 2041;
/// Semantic tag for the Play Mode aiming crosshair canvas.
pub const TAG_PLAY_CROSSHAIR: u64 = 2050;

/// Identifies active floating dropdown menus in the Viewport HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportHudDropdownId {
    /// Camera projection mode selector (Perspective, Top, Front, Right, Ortho).
    CameraMode,
    /// Shading / wireframe render mode selector (Lit, Wireframe).
    ShadingMode,
}

/// Dispatched user interaction actions originating from the Viewport HUD.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewportHudAction {
    /// Sets camera projection mode.
    SetCameraMode(ProjectionMode),
    /// Toggles camera projection mode between 3D Perspective and 2D Orthographic.
    ToggleCameraProjection,
    /// Sets camera orientation angles and eye position, optionally switching projection mode.
    SetCameraTransform {
        /// Camera pitch in radians.
        pitch: cgmath::Rad<f32>,
        /// Camera yaw in radians.
        yaw: cgmath::Rad<f32>,
        /// Camera position in world space.
        position: cgmath::Point3<f32>,
        /// Optional camera projection mode to switch into synchronously.
        mode: Option<ProjectionMode>,
    },
    /// Snaps camera to a cardinal view direction (Top, Bottom, Front, Back, Left, Right).
    SnapCamera(SceneViewSnap),
    /// Toggles wireframe overlay rendering mode.
    ToggleWireframe,
    /// Sets active transform gizmo operation mode.
    SetGizmoMode(GizmoMode),
    /// Toggles coordinate frame between World and Local space.
    ToggleGizmoSpace,
    /// Toggles entity translation/rotation snapping.
    ToggleSnapping,
    /// Toggles a dropdown popup menu open or closed.
    ToggleDropdown(Option<ViewportHudDropdownId>),
    /// Dispatches selection of an item within an active dropdown menu.
    SelectDropdownItem(ViewportHudDropdownId, usize),
    /// Resumes active in-game gameplay from the pause overlay.
    ResumeGame,
    /// Exits in-game play mode and returns to editor mode.
    ExitToEditor,
}

/// Parameter block passed into the Viewport HUD builder.
pub struct ViewportHudParams<'a> {
    /// Screen rectangle bounding the 3D viewport canvas.
    pub viewport_rect: Rect,
    /// Reference to the active 3D camera.
    pub camera: &'a Camera,
    /// Whether wireframe rendering is currently enabled.
    pub wireframe_enabled: bool,
    /// Currently active gizmo manipulation mode (Translate, Rotate, Scale).
    pub gizmo_mode: GizmoMode,
    /// Currently active gizmo coordinate space (World, Local).
    pub gizmo_space: GizmoSpace,
    /// Snapping settings configuration.
    pub snapping: &'a SnapSettings,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Active dropdown menu currently open in the HUD.
    pub active_dropdown: Option<ViewportHudDropdownId>,
    /// Currently selected entity in the editor, if any.
    pub selected_entity: Option<Entity>,
    /// Active ECS world reference for gameplay HUD and runtime entity queries.
    pub world: &'a World,
    /// Whether the editor is currently in Edit mode (vs Play mode).
    pub is_editing: bool,
    /// Whether the active project / dimension mode is 2D.
    pub is_2d: bool,
}

/// Persistent interactive state and dirty-tracking cache for the 3D Viewport HUD overlay.
#[derive(Debug, Default, Clone)]
pub struct ViewportHudState {
    /// Whether the 3D Viewport HUD overlay is active and receiving input.
    pub is_active: bool,
    /// Currently open dropdown menu in Viewport HUD.
    pub dropdown: Option<ViewportHudDropdownId>,
    /// Dispatched action queue for Viewport HUD interactions.
    pub actions: Vec<ViewportHudAction>,
    /// Active search filter text query in Viewport Add Object popup.
    pub search_query: String,
    /// Whether search input box is focused in Viewport Add Object popup.
    pub is_search_focused: bool,
    /// Active 3D camera Euler angles `(pitch_rad, yaw_rad)` for compass line rendering.
    pub camera_angles: (f32, f32),
    /// Cached camera position snapshot `(x, y, z)` for retained-mode dirty tracking.
    pub last_camera_pos: (f32, f32, f32),
    /// Cached camera rotation snapshot `(pitch_rad, yaw_rad)` for retained-mode dirty tracking.
    pub last_camera_rot: (f32, f32),
    /// Cached gizmo mode selection for retained-mode dirty tracking.
    pub last_gizmo_mode: Option<GizmoMode>,
    /// Cached gizmo coordinate space selection for retained-mode dirty tracking.
    pub last_gizmo_space: Option<GizmoSpace>,
    /// Cached wireframe mode toggle for retained-mode dirty tracking.
    pub last_wireframe: bool,
    /// Cached edit/play mode flag for retained-mode dirty tracking.
    pub last_is_editing: bool,
    /// Cached 2D/3D dimension mode flag for retained-mode dirty tracking.
    pub last_is_2d: bool,
    /// Cached selected entity for retained-mode dirty tracking.
    pub last_selected_entity: Option<Entity>,
    /// Cached viewport bounding box for retained-mode dirty tracking.
    pub last_viewport_rect: Rect,
}

impl ViewportHudState {
    /// Consumes and returns all pending dispatched Viewport HUD actions.
    pub fn take_actions(&mut self) -> Vec<ViewportHudAction> {
        std::mem::take(&mut self.actions)
    }
}