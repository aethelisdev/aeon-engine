// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Types & Parameters
//!
//! Provides semantic [`UiTag`] constants, parameter descriptors, and live-cache
//! state for the Iris UI Viewport HUD subsystem without intermediate action queues.

pub use ae_gizmos::SceneViewSnap;
use ae_renderer::camera::{Camera, ProjectionMode};
use irisui::prelude::*;

/// Semantic tag for the Viewport Camera Mode dropdown trigger button.
pub const TAG_VIEWPORT_CAMERA_MODE: u64 = UiTag::from_str("viewport.hud.camera_mode").raw();
/// Semantic tag for the Viewport Shading Mode dropdown trigger button.
pub const TAG_VIEWPORT_SHADING_MODE: u64 = UiTag::from_str("viewport.hud.shading_mode").raw();
/// Semantic tag for the Gizmo Select tool button.
pub const TAG_GIZMO_SELECT: u64 = UiTag::from_str("viewport.hud.gizmo.select").raw();
/// Semantic tag for the Gizmo Translate tool button.
pub const TAG_GIZMO_TRANSLATE: u64 = UiTag::from_str("viewport.hud.gizmo.translate").raw();
/// Semantic tag for the Gizmo Rotate tool button.
pub const TAG_GIZMO_ROTATE: u64 = UiTag::from_str("viewport.hud.gizmo.rotate").raw();
/// Semantic tag for the Gizmo Scale tool button.
pub const TAG_GIZMO_SCALE: u64 = UiTag::from_str("viewport.hud.gizmo.scale").raw();
/// Semantic tag for the Gizmo Coordinate Space toggle button.
pub const TAG_GIZMO_SPACE_TOGGLE: u64 = UiTag::from_str("viewport.hud.gizmo.space").raw();
/// Semantic tag for the Viewport Grid Snap toggle button.
pub const TAG_SNAP_GRID_TOGGLE: u64 = UiTag::from_str("viewport.hud.snap.toggle").raw();
/// Semantic tag for the Viewport Grid Snap size dropdown trigger button.
pub const TAG_SNAP_GRID_DROPDOWN: u64 = UiTag::from_str("viewport.hud.snap.dropdown").raw();
/// Semantic tag for the Compass 3D orientation canvas background and axis lines.
pub const TAG_COMPASS_CANVAS: u64 = UiTag::from_str("viewport.hud.compass").raw();
/// Semantic tag for the Viewport camera telemetry info HUD immediate canvas.
pub const TAG_CAMERA_HUD: u64 = UiTag::from_str("viewport.hud.camera_info").raw();
/// Semantic tag for the In-Game Pause Resume Game button.
pub const TAG_PLAY_RESUME: u64 = ae_runtime::hud::TAG_PLAY_RESUME;
/// Semantic tag for the In-Game Pause Exit to Editor button.
pub const TAG_PLAY_EXIT: u64 = ae_runtime::hud::TAG_PLAY_EXIT;
/// Semantic tag for the Play Mode aiming crosshair canvas.
pub const TAG_PLAY_CROSSHAIR: u64 = UiTag::from_str("viewport.hud.play.crosshair").raw();
/// Semantic tag for the Play Mode bottom-left quick controls guide badge.
pub const TAG_PLAY_GUIDE_BADGE: u64 = UiTag::from_str("viewport.hud.play.guide_badge").raw();

/// Root tag namespace for Camera Mode dropdown option rows.
pub const TAG_POPUP_CAMERA_GROUP: UiTag = UiTag::from_str("viewport.hud.popup.camera");
/// Root tag namespace for Shading Mode dropdown option rows.
pub const TAG_POPUP_SHADING_GROUP: UiTag = UiTag::from_str("viewport.hud.popup.shading");
/// Root tag namespace for Grid Snap dropdown option rows.
pub const TAG_POPUP_SNAP_GRID_GROUP: UiTag = UiTag::from_str("viewport.hud.popup.snap_grid");

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
            _ => TAG_VIEWPORT_CAMERA_MODE,
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

/// Returns the exact required pixel width of the View Modes group container (including 2px border and 1px divider).
#[inline]
#[must_use]
pub fn view_modes_group_width(camera: &Camera, wireframe_enabled: bool, is_2d: bool) -> f32 {
    CameraViewOrientation::from_camera(camera, is_2d).width()
        + if is_2d {
            2.0
        } else {
            1.0 + ShadingViewMode::from_wireframe(wireframe_enabled).width() + 2.0
        }
}

/// Returns the total required width for the floating toolbar strip.
#[inline]
#[must_use]
pub fn toolbar_total_width(camera: &Camera, wireframe_enabled: bool, is_2d: bool) -> f32 {
    view_modes_group_width(camera, wireframe_enabled, is_2d) + 8.0 + 140.0 + 8.0 + 32.0 + 8.0 + 84.0
}

#[cfg(test)]
pub(super) fn with_test_frame_data<R>(
    (viewport_rect, is_editing, is_2d_mode): (Rect, bool, bool),
    camera: &Camera,
    editor_state: &ae_editor::editor_state::EditorState,
    snapping_settings: &ae_editor::snapping::SnapSettings,
    world: &ae_ecs::World,
    f: impl FnOnce(&crate::ui::types::EditorFrameData<'_>) -> R,
) -> R {
    use crate::ui::types::{EditorFrameData, EditorTelemetrySnapshot, ExportDialogState};
    use ae_assets::types::AssetBrowserState;
    use ae_editor::editor_state::EditorConfig;
    use ae_renderer::asset::AssetStorage;
    use ae_renderer::graphics_settings::GraphicsSettings;
    use ae_uidesign::UiDesignerState;
    use std::collections::HashSet;

    let gs = GraphicsSettings::default();
    let cfg = EditorConfig::default();
    let enabled = HashSet::new();
    let telemetry = EditorTelemetrySnapshot::default();
    let mut asset_browser = AssetBrowserState::default();
    let mut ui_designer_state = UiDesignerState::default();
    let mut export_modal = ExportDialogState::default();
    let textures = AssetStorage::new();
    let models = AssetStorage::new();

    let frame = EditorFrameData {
        is_editing,
        is_2d_mode,
        view_panel_mask: 0,
        can_undo: false,
        can_redo: false,
        has_viewport_texture: false,
        viewport_rect,
        camera,
        world,
        editor_state,
        graphics_settings: &gs,
        snapping_settings,
        editor_config: &cfg,
        enable_live_updates: false,
        enabled_modules: &enabled,
        telemetry: &telemetry,
        gpu_adapter_name: "TestGPU",
        gpu_backend: "Vulkan",
        asset_browser: &mut asset_browser,
        ui_designer_state: &mut ui_designer_state,
        export_modal: &mut export_modal,
        textures: &textures,
        models: &models,
    };
    f(&frame)
}

/// Persistent live-tracking cache for the 3D Viewport HUD overlay.
#[derive(Debug, Default, Clone)]
pub struct ViewportHudState {
    /// Active 3D camera Euler angles `(pitch_rad, yaw_rad)` for live telemetry synchronization.
    pub camera_angles: (f32, f32),
    /// Active 3D camera world position `(x, y, z)` for live telemetry synchronization.
    pub camera_pos: (f32, f32, f32),
    /// Cached quantized signature `(pos_tenths, pitch_deg, yaw_deg, viewport_rect)` for skipping unchanged Camera HUD updates.
    pub last_camera_hud_key: Option<([i32; 3], i32, i32, Rect)>,
    /// Cached quantized signature `(pitch_mrad, yaw_mrad, viewport_rect)` for skipping unchanged Compass updates.
    pub last_compass_key: Option<(i32, i32, Rect)>,
}

impl ViewportHudState {
    /// Clears cached live-canvas telemetry signatures so the next `InPlaceRefresh` pass re-evaluates them.
    #[inline]
    pub fn reset_live_cache(&mut self) {
        self.last_camera_hud_key = None;
        self.last_compass_key = None;
    }
}