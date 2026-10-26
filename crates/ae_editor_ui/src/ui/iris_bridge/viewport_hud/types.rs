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
/// Semantic tag for the Viewport camera telemetry info HUD immediate canvas.
pub const TAG_CAMERA_HUD: u64 = 2038;
/// Semantic tag for the In-Game Pause Resume Game button.
pub const TAG_PLAY_RESUME: u64 = 2040;
/// Semantic tag for the In-Game Pause Exit to Editor button.
pub const TAG_PLAY_EXIT: u64 = 2041;
/// Semantic tag for the Play Mode aiming crosshair canvas.
pub const TAG_PLAY_CROSSHAIR: u64 = 2050;
/// Base semantic tag for camera mode dropdown popup items (2100..2104).
pub const TAG_POPUP_CAMERA_BASE: u64 = 2100;
/// Base semantic tag for shading mode dropdown popup items (2200..2201).
pub const TAG_POPUP_SHADING_BASE: u64 = 2200;

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
    /// Sets explicit wireframe overlay rendering state.
    SetWireframe(bool),
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

/// Cardinal and projection-mode classifications for camera display presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraViewOrientation {
    /// 2D project or dimension mode.
    TwoDimensional,
    /// 3D Perspective projection mode.
    Perspective,
    /// Orthographic Top cardinal projection.
    Top,
    /// Orthographic Front cardinal projection.
    Front,
    /// Orthographic Right cardinal projection.
    Right,
    /// General custom Orthographic 3D projection.
    Orthographic,
}

impl CameraViewOrientation {
    /// Classifies active camera orientation and projection mode.
    #[must_use]
    pub fn from_camera(camera: &Camera, is_2d: bool) -> Self {
        if is_2d {
            Self::TwoDimensional
        } else if camera.mode == ProjectionMode::Perspective {
            Self::Perspective
        } else if camera.pitch.0 < -1.5 {
            Self::Top
        } else if camera.pitch.0.abs() < 0.1 && camera.yaw.0 > 1.5 {
            Self::Front
        } else if camera.pitch.0.abs() < 0.1 && camera.yaw.0.abs() < 0.1 {
            Self::Right
        } else {
            Self::Orthographic
        }
    }

    /// Returns UI caption text for this orientation.
    #[inline]
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::TwoDimensional => "2D",
            Self::Perspective => "Perspective",
            Self::Top => "Top",
            Self::Front => "Front",
            Self::Right => "Right",
            Self::Orthographic => "Orthographic",
        }
    }

    /// Returns explicit button width in logical points.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> f32 {
        match self {
            Self::TwoDimensional => 44.0,
            Self::Top => 54.0,
            Self::Front | Self::Right => 58.0,
            Self::Perspective | Self::Orthographic => 116.0,
        }
    }

    /// Returns optional icon UV coordinates for leading cube icon.
    #[inline]
    #[must_use]
    pub const fn icon_uv(&self) -> Option<[f32; 4]> {
        match self {
            Self::Perspective => Some(crate::ui::iris_bridge::icons::ICON_CUBE),
            _ => None,
        }
    }

    /// Returns semantic tag for the camera mode dropdown trigger.
    #[inline]
    #[must_use]
    pub const fn tag(&self) -> u64 {
        match self {
            Self::TwoDimensional => 0,
            _ => crate::ui::iris_bridge::viewport_hud::TAG_VIEWPORT_CAMERA_MODE,
        }
    }

    /// Returns corner radii for the camera mode button within the segmented view modes container.
    #[inline]
    #[must_use]
    pub const fn corner_radii(&self) -> CornerRadii {
        match self {
            Self::TwoDimensional => CornerRadii::all(3.0),
            _ => CornerRadii::new(3.0, 0.0, 0.0, 3.0),
        }
    }
}

/// Shading display mode presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadingViewMode {
    /// Lit PBR shading mode.
    Lit,
    /// Wireframe overlay mode.
    Wireframe,
}

impl ShadingViewMode {
    /// Constructs shading mode presentation from wireframe active flag.
    #[inline]
    #[must_use]
    pub const fn from_wireframe(wireframe_enabled: bool) -> Self {
        if wireframe_enabled {
            Self::Wireframe
        } else {
            Self::Lit
        }
    }

    /// Returns caption text.
    #[inline]
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Lit => "Lit",
            Self::Wireframe => "Wireframe",
        }
    }

    /// Returns button width in logical points.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> f32 {
        match self {
            Self::Lit => 62.0,
            Self::Wireframe => 96.0,
        }
    }

    /// Returns mode icon UV coordinates.
    #[inline]
    #[must_use]
    pub const fn icon_uv(&self) -> [f32; 4] {
        match self {
            Self::Lit => crate::ui::iris_bridge::icons::ICON_LIGHT,
            Self::Wireframe => crate::ui::iris_bridge::icons::ICON_WIREFRAME,
        }
    }

    /// Returns corner radii for the shading mode button (rounded on right corners).
    #[inline]
    #[must_use]
    pub const fn corner_radii(&self) -> CornerRadii {
        CornerRadii::new(0.0, 3.0, 3.0, 0.0)
    }
}

impl<'a> ViewportHudParams<'a> {
    /// Returns camera view orientation classification.
    #[inline]
    #[must_use]
    pub fn camera_orientation(&self) -> CameraViewOrientation {
        CameraViewOrientation::from_camera(self.camera, self.is_2d)
    }

    /// Returns shading view mode classification.
    #[inline]
    #[must_use]
    pub const fn shading_mode(&self) -> ShadingViewMode {
        ShadingViewMode::from_wireframe(self.wireframe_enabled)
    }

    /// Returns the exact required pixel width of the View Modes group container (including 2px border and 1px divider).
    #[inline]
    #[must_use]
    pub fn view_modes_group_width(&self) -> f32 {
        let cam_w = self.camera_orientation().width();
        if self.is_2d {
            cam_w + 2.0
        } else {
            cam_w + 1.0 + self.shading_mode().width() + 2.0
        }
    }

    /// Returns the total required width for the floating toolbar strip.
    #[inline]
    #[must_use]
    pub fn toolbar_total_width(&self) -> f32 {
        self.view_modes_group_width() + 8.0 + 140.0 + 8.0 + 32.0
    }
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
    /// Active 3D camera Euler angles `(pitch_rad, yaw_rad)` for compass line rendering.
    pub camera_angles: (f32, f32),
    /// Active 3D camera world position `(x, y, z)` for real-time telemetry HUD updating.
    pub camera_pos: (f32, f32, f32),
}

impl ViewportHudState {
    /// Dispatches a user interaction action directly onto the pending Viewport HUD action queue.
    #[inline]
    pub fn dispatch(&mut self, action: ViewportHudAction) {
        self.actions.push(action);
    }

    /// Consumes and returns all pending dispatched Viewport HUD actions.
    pub fn take_actions(&mut self) -> Vec<ViewportHudAction> {
        std::mem::take(&mut self.actions)
    }
}