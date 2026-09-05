// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Types & Parameters
//!
//! Provides data structures, action events, and interaction target collectors for the
//! Iris UI hardware SDF Viewport HUD subsystem.

use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use ae_editor::snapping::SnapSettings;
use ae_renderer::camera::{Camera, ProjectionMode};
use hecs::{Entity, World};
use irisui::prelude::*;

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
    /// Toggles wireframe overlay rendering mode.
    ToggleWireframe,
    /// Sets active transform gizmo operation mode.
    SetGizmoMode(GizmoMode),
    /// Toggles coordinate frame between World and Local space.
    ToggleGizmoSpace,
    /// Toggles entity translation/rotation snapping.
    ToggleSnapping,
    /// Selects an entity in the active scene.
    SelectEntity(Entity),
    /// Toggles a dropdown popup menu open or closed.
    ToggleDropdown(Option<ViewportHudDropdownId>),
}

/// Hit-test interaction target collection for Viewport HUD widgets.
#[derive(Default)]
pub struct ViewportHudTargets {
    /// Clickable buttons in the toolbar: `(Action, ScreenRect)`.
    pub buttons: Vec<(ViewportHudAction, Rect)>,
    /// Dropdown trigger buttons: `(DropdownId, ScreenRect)`.
    pub dropdown_triggers: Vec<(ViewportHudDropdownId, Rect)>,
    /// Items inside an open dropdown popup: `(Action, ItemRect, Label)`.
    pub active_dropdown_items: Vec<(ViewportHudAction, Rect, String)>,
    /// Bounding rectangle of the active open dropdown popup.
    pub active_dropdown_popup_rect: Option<Rect>,
    /// 3D compass axis snap knobs: `(Action, ScreenRect)`.
    pub compass_knobs: Vec<(ViewportHudAction, Rect)>,
    /// 3D projected billboard icons: `(Entity, ScreenRect)`.
    pub billboard_icons: Vec<(Entity, Rect)>,
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
    /// Active ECS world reference for billboard entity query.
    pub world: &'a World,
    /// Whether the editor is currently in Edit mode (vs Play mode).
    pub is_editing: bool,
}