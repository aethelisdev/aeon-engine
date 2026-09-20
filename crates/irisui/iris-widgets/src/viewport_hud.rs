// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport HUD Widget Subsystem (`iris-widgets::viewport_hud`)
//!
//! Provides floating interactive overlay toolbars for 3D Viewports:
//! - Transform manipulation tools (Select, Translate, Rotate, Scale, Coordinate Space, Snapping, Grid).
//! - Engine simulation controls (Play, Pause, Stop).
//! - Viewport diagnostics and render modes (Camera Projection, Wireframe/Lit Shading, FPS diagnostics).
//!

use iris_core::color::Color;
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetCursor, WidgetRole};
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;

/// Tag constant for Gizmo Select tool button.
pub const VIEWPORT_HUD_TAG_GIZMO_SELECT: u64 = 2001;
/// Tag constant for Gizmo Translate tool button.
pub const VIEWPORT_HUD_TAG_GIZMO_TRANSLATE: u64 = 2002;
/// Tag constant for Gizmo Rotate tool button.
pub const VIEWPORT_HUD_TAG_GIZMO_ROTATE: u64 = 2003;
/// Tag constant for Gizmo Scale tool button.
pub const VIEWPORT_HUD_TAG_GIZMO_SCALE: u64 = 2004;
/// Tag constant for Gizmo Coordinate Space toggle button.
pub const VIEWPORT_HUD_TAG_GIZMO_SPACE: u64 = 2005;
/// Tag constant for Viewport Snapping toggle button.
pub const VIEWPORT_HUD_TAG_SNAPPING: u64 = 2006;
/// Tag constant for Viewport Grid toggle button.
pub const VIEWPORT_HUD_TAG_GRID: u64 = 2007;
/// Tag constant for Play / Pause simulation toggle button.
pub const VIEWPORT_HUD_TAG_PLAY_PAUSE: u64 = 2008;
/// Tag constant for Stop simulation button.
pub const VIEWPORT_HUD_TAG_STOP: u64 = 2009;
/// Tag constant for Camera Projection toggle button.
pub const VIEWPORT_HUD_TAG_CAMERA_PROJECTION: u64 = 2010;
/// Tag constant for Wireframe / Lit shading mode toggle button.
pub const VIEWPORT_HUD_TAG_WIREFRAME: u64 = 2011;

/// Active transform gizmo manipulation tool in the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportGizmoMode {
    /// Object selection mode without transform handles.
    #[default]
    Select,
    /// 3-axis position translation mode.
    Translate,
    /// 3-axis orientation rotation mode.
    Rotate,
    /// 3-axis dimension scale mode.
    Scale,
}

/// Active coordinate space frame for gizmo orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportGizmoSpace {
    /// Coordinates aligned with global universe axes.
    World,
    /// Coordinates aligned with entity local orientation.
    #[default]
    Local,
}

/// Active camera optical projection mode in the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportCameraMode {
    /// Standard 3D perspective projection with vanishing point.
    #[default]
    Perspective,
    /// Orthographic 2D/isometric parallel projection without foreshortening.
    Orthographic,
}

/// Active engine simulation lifecycle mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportEngineMode {
    /// Standstill editor authoring mode.
    #[default]
    Editing,
    /// Real-time tick and physics simulation mode.
    Playing,
    /// Simulation paused with active runtime scene state preserved.
    Paused,
}

/// Dispatched user interaction action originating from Viewport HUD clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportHudAction {
    /// Switch gizmo manipulation mode to the specified tool.
    SetGizmoMode(ViewportGizmoMode),
    /// Toggles coordinate frame between World and Local space.
    ToggleGizmoSpace,
    /// Toggles transform translation/rotation snapping.
    ToggleSnapping,
    /// Toggles reference ground grid visibility in the viewport.
    ToggleGrid,
    /// Toggles between Play and Pause engine runtime states.
    TogglePlayPause,
    /// Stops runtime simulation and reverts to editor state.
    StopSimulation,
    /// Toggles camera projection between Perspective and Orthographic.
    ToggleCameraProjection,
    /// Toggles wireframe overlay shading mode.
    ToggleWireframe,
}

/// Evaluates an Iris UI node tag and maps it into a semantic [`ViewportHudAction`].
#[must_use]
pub fn evaluate_viewport_hud_tag(tag: u64) -> Option<ViewportHudAction> {
    match tag {
        VIEWPORT_HUD_TAG_GIZMO_SELECT => {
            Some(ViewportHudAction::SetGizmoMode(ViewportGizmoMode::Select))
        }
        VIEWPORT_HUD_TAG_GIZMO_TRANSLATE => Some(ViewportHudAction::SetGizmoMode(
            ViewportGizmoMode::Translate,
        )),
        VIEWPORT_HUD_TAG_GIZMO_ROTATE => {
            Some(ViewportHudAction::SetGizmoMode(ViewportGizmoMode::Rotate))
        }
        VIEWPORT_HUD_TAG_GIZMO_SCALE => {
            Some(ViewportHudAction::SetGizmoMode(ViewportGizmoMode::Scale))
        }
        VIEWPORT_HUD_TAG_GIZMO_SPACE => Some(ViewportHudAction::ToggleGizmoSpace),
        VIEWPORT_HUD_TAG_SNAPPING => Some(ViewportHudAction::ToggleSnapping),
        VIEWPORT_HUD_TAG_GRID => Some(ViewportHudAction::ToggleGrid),
        VIEWPORT_HUD_TAG_PLAY_PAUSE => Some(ViewportHudAction::TogglePlayPause),
        VIEWPORT_HUD_TAG_STOP => Some(ViewportHudAction::StopSimulation),
        VIEWPORT_HUD_TAG_CAMERA_PROJECTION => Some(ViewportHudAction::ToggleCameraProjection),
        VIEWPORT_HUD_TAG_WIREFRAME => Some(ViewportHudAction::ToggleWireframe),
        _ => None,
    }
}

/// Visual styling configuration for Viewport HUD overlay bars and buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportHudStyle {
    /// Translucent background color for floating capsule bars.
    pub bar_bg: Color,
    /// Outline border stroke color for capsule bars.
    pub bar_border: Color,
    /// Rounded corner radius for floating capsule containers.
    pub corner_radius: f32,
    /// Background color for currently active/selected tool buttons.
    pub button_active_bg: Color,
    /// Background color for idle/unselected tool buttons.
    pub button_idle_bg: Color,
    /// Text color for active buttons.
    pub text_active: Color,
    /// Text color for idle buttons.
    pub text_idle: Color,
    /// Good diagnostics text color (FPS >= 55).
    pub fps_good: Color,
    /// Warning diagnostics text color (30 <= FPS < 55).
    pub fps_warn: Color,
    /// Poor diagnostics text color (FPS < 30).
    pub fps_poor: Color,
}

impl Default for ViewportHudStyle {
    fn default() -> Self {
        Self {
            bar_bg: Color::rgba(0.08, 0.09, 0.13, 0.88),
            bar_border: Color::rgba(1.0, 1.0, 1.0, 0.10),
            corner_radius: 6.0,
            button_active_bg: Color::rgba(0.20, 0.45, 0.90, 0.85),
            button_idle_bg: Color::rgba(1.0, 1.0, 1.0, 0.04),
            text_active: Color::WHITE,
            text_idle: Color::rgba(0.85, 0.88, 0.92, 0.88),
            fps_good: Color::hex("#22c55e"),
            fps_warn: Color::hex("#eab308"),
            fps_poor: Color::hex("#ef4444"),
        }
    }
}

/// Output layout frame returned after assembling the Viewport HUD overlay hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportHudFrame {
    /// Root overlay container widget ID.
    pub root_id: WidgetId,
    /// Top-left transform tools capsule bounding rectangle.
    pub transform_bar_rect: Rect,
    /// Top-center simulation playback capsule bounding rectangle.
    pub play_bar_rect: Rect,
    /// Top-right diagnostics and render modes capsule bounding rectangle.
    pub diagnostics_bar_rect: Rect,
}

/// Fluent builder for constructing interactive hardware SDF Viewport HUD overlays.
///
/// Generates top-left transform controls, top-center transport controls, and top-right
/// render diagnostics on `UiLayer::Floating`, assigning unique semantic tags for zero-overhead
/// layered hit-testing.
pub struct ViewportHudBuilder {
    viewport_rect: Rect,
    gizmo_mode: ViewportGizmoMode,
    gizmo_space: ViewportGizmoSpace,
    snapping_enabled: bool,
    grid_enabled: bool,
    wireframe_enabled: bool,
    camera_mode: ViewportCameraMode,
    engine_mode: ViewportEngineMode,
    fps: f32,
    frame_ms: f32,
    style: ViewportHudStyle,
}

impl ViewportHudBuilder {
    /// Creates a new `ViewportHudBuilder` bound to the designated viewport area.
    #[must_use]
    pub fn new(viewport_rect: Rect) -> Self {
        Self {
            viewport_rect,
            gizmo_mode: ViewportGizmoMode::Select,
            gizmo_space: ViewportGizmoSpace::Local,
            snapping_enabled: false,
            grid_enabled: true,
            wireframe_enabled: false,
            camera_mode: ViewportCameraMode::Perspective,
            engine_mode: ViewportEngineMode::Editing,
            fps: 60.0,
            frame_ms: 16.67,
            style: ViewportHudStyle::default(),
        }
    }

    /// Sets the active transform gizmo mode.
    #[must_use]
    pub fn gizmo_mode(mut self, mode: ViewportGizmoMode) -> Self {
        self.gizmo_mode = mode;
        self
    }

    /// Sets the active coordinate space frame.
    #[must_use]
    pub fn gizmo_space(mut self, space: ViewportGizmoSpace) -> Self {
        self.gizmo_space = space;
        self
    }

    /// Configures whether transform snapping is active.
    #[must_use]
    pub fn snapping(mut self, enabled: bool) -> Self {
        self.snapping_enabled = enabled;
        self
    }

    /// Configures whether the viewport grid is visible.
    #[must_use]
    pub fn grid(mut self, enabled: bool) -> Self {
        self.grid_enabled = enabled;
        self
    }

    /// Configures whether wireframe shading is enabled.
    #[must_use]
    pub fn wireframe(mut self, enabled: bool) -> Self {
        self.wireframe_enabled = enabled;
        self
    }

    /// Sets the active camera projection mode.
    #[must_use]
    pub fn camera_mode(mut self, mode: ViewportCameraMode) -> Self {
        self.camera_mode = mode;
        self
    }

    /// Sets the active engine lifecycle simulation mode.
    #[must_use]
    pub fn engine_mode(mut self, mode: ViewportEngineMode) -> Self {
        self.engine_mode = mode;
        self
    }

    /// Sets real-time frame rate diagnostics.
    #[must_use]
    pub fn diagnostics(mut self, fps: f32, frame_ms: f32) -> Self {
        self.fps = fps;
        self.frame_ms = frame_ms;
        self
    }

    /// Applies custom styling parameters.
    #[must_use]
    pub fn style(mut self, style: ViewportHudStyle) -> Self {
        self.style = style;
        self
    }

    /// Builds the complete Viewport HUD hierarchy into the provided [`UiTree`].
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> ViewportHudFrame {
        let root_id = tree.create_node();
        if let Some(node) = tree.get_mut(root_id) {
            node.name = Some("ViewportHudRoot".to_string());
            node.computed_rect = self.viewport_rect;
            node.layer = UiLayer::Floating;
            node.interactive = false;
        }
        let _ = tree.add_child(parent_id, root_id);

        let pad = 10.0;
        let bar_h = 30.0;
        let btn_h = 24.0;
        let btn_y_off = 3.0;

        // -------------------------------------------------------------
        // 1. TOP-LEFT: Transform Tools Capsule Bar
        // -------------------------------------------------------------
        let transform_bar_w = 445.0;
        let transform_bar_rect = Rect::new(
            self.viewport_rect.x + pad,
            self.viewport_rect.y + pad,
            transform_bar_w,
            bar_h,
        );
        let trans_bar_node = tree.create_node();
        if let Some(node) = tree.get_mut(trans_bar_node) {
            node.name = Some("ViewportHud_TransformBar".to_string());
            node.computed_rect = transform_bar_rect;
            node.layer = UiLayer::Floating;
            node.interactive = true;
            node.style = Style::new()
                .background(self.style.bar_bg)
                .border(1.0, self.style.bar_border)
                .border_radius(self.style.corner_radius);
        }
        let _ = tree.add_child(root_id, trans_bar_node);

        let mut btn_x = transform_bar_rect.x + 4.0;
        let btn_gap = 4.0;

        // Select Tool
        let is_sel = self.gizmo_mode == ViewportGizmoMode::Select;
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_GIZMO_SELECT,
            "Select",
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 54.0, btn_h),
            is_sel,
        );
        btn_x += 54.0 + btn_gap;

        // Translate Tool
        let is_trans = self.gizmo_mode == ViewportGizmoMode::Translate;
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_GIZMO_TRANSLATE,
            "Move",
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 48.0, btn_h),
            is_trans,
        );
        btn_x += 48.0 + btn_gap;

        // Rotate Tool
        let is_rot = self.gizmo_mode == ViewportGizmoMode::Rotate;
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_GIZMO_ROTATE,
            "Rotate",
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 52.0, btn_h),
            is_rot,
        );
        btn_x += 52.0 + btn_gap;

        // Scale Tool
        let is_scale = self.gizmo_mode == ViewportGizmoMode::Scale;
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_GIZMO_SCALE,
            "Scale",
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 48.0, btn_h),
            is_scale,
        );
        btn_x += 48.0 + btn_gap + 4.0;

        // Space Toggle (World / Local)
        let space_label = match self.gizmo_space {
            ViewportGizmoSpace::World => "World",
            ViewportGizmoSpace::Local => "Local",
        };
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_GIZMO_SPACE,
            space_label,
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 54.0, btn_h),
            false,
        );
        btn_x += 54.0 + btn_gap;

        // Snapping Toggle
        let snap_label = if self.snapping_enabled {
            "Snap: ON"
        } else {
            "Snap: OFF"
        };
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_SNAPPING,
            snap_label,
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 78.0, btn_h),
            self.snapping_enabled,
        );
        btn_x += 78.0 + btn_gap;

        // Grid Toggle
        let grid_label = if self.grid_enabled {
            "Grid: ON"
        } else {
            "Grid: OFF"
        };
        self.add_hud_button(
            tree,
            trans_bar_node,
            VIEWPORT_HUD_TAG_GRID,
            grid_label,
            Rect::new(btn_x, transform_bar_rect.y + btn_y_off, 74.0, btn_h),
            self.grid_enabled,
        );

        // -------------------------------------------------------------
        // 2. TOP-CENTER: Play / Simulation Controls Capsule Bar
        // -------------------------------------------------------------
        let play_bar_w = 156.0;
        let center_x = self.viewport_rect.x + (self.viewport_rect.width - play_bar_w) * 0.5;
        let play_bar_rect = Rect::new(center_x, self.viewport_rect.y + pad, play_bar_w, bar_h);
        let play_bar_node = tree.create_node();
        if let Some(node) = tree.get_mut(play_bar_node) {
            node.name = Some("ViewportHud_PlayBar".to_string());
            node.computed_rect = play_bar_rect;
            node.layer = UiLayer::Floating;
            node.interactive = true;
            node.style = Style::new()
                .background(self.style.bar_bg)
                .border(1.0, self.style.bar_border)
                .border_radius(self.style.corner_radius);
        }
        let _ = tree.add_child(root_id, play_bar_node);

        let play_label = match self.engine_mode {
            ViewportEngineMode::Editing => "▶ Play",
            ViewportEngineMode::Playing => "⏸ Pause",
            ViewportEngineMode::Paused => "▶ Resume",
        };
        let is_playing = self.engine_mode != ViewportEngineMode::Editing;
        self.add_hud_button(
            tree,
            play_bar_node,
            VIEWPORT_HUD_TAG_PLAY_PAUSE,
            play_label,
            Rect::new(
                play_bar_rect.x + 4.0,
                play_bar_rect.y + btn_y_off,
                76.0,
                btn_h,
            ),
            is_playing,
        );

        self.add_hud_button(
            tree,
            play_bar_node,
            VIEWPORT_HUD_TAG_STOP,
            "⏹ Stop",
            Rect::new(
                play_bar_rect.x + 84.0,
                play_bar_rect.y + btn_y_off,
                68.0,
                btn_h,
            ),
            false,
        );

        // -------------------------------------------------------------
        // 3. TOP-RIGHT: Render Diagnostics & Mode Capsule Bar
        // -------------------------------------------------------------
        let diag_bar_w = 345.0;
        let right_x = (self.viewport_rect.right() - diag_bar_w - pad).max(self.viewport_rect.x);
        let diag_bar_rect = Rect::new(right_x, self.viewport_rect.y + pad, diag_bar_w, bar_h);
        let diag_bar_node = tree.create_node();
        if let Some(node) = tree.get_mut(diag_bar_node) {
            node.name = Some("ViewportHud_DiagnosticsBar".to_string());
            node.computed_rect = diag_bar_rect;
            node.layer = UiLayer::Floating;
            node.interactive = true;
            node.style = Style::new()
                .background(self.style.bar_bg)
                .border(1.0, self.style.bar_border)
                .border_radius(self.style.corner_radius);
        }
        let _ = tree.add_child(root_id, diag_bar_node);

        let mut diag_x = diag_bar_rect.x + 4.0;

        // Camera Projection Button
        let cam_label = match self.camera_mode {
            ViewportCameraMode::Perspective => "Perspective",
            ViewportCameraMode::Orthographic => "Orthographic",
        };
        self.add_hud_button(
            tree,
            diag_bar_node,
            VIEWPORT_HUD_TAG_CAMERA_PROJECTION,
            cam_label,
            Rect::new(diag_x, diag_bar_rect.y + btn_y_off, 94.0, btn_h),
            false,
        );
        diag_x += 94.0 + btn_gap;

        // Wireframe Shading Button
        let wire_label = if self.wireframe_enabled {
            "Wireframe"
        } else {
            "Lit Shaded"
        };
        self.add_hud_button(
            tree,
            diag_bar_node,
            VIEWPORT_HUD_TAG_WIREFRAME,
            wire_label,
            Rect::new(diag_x, diag_bar_rect.y + btn_y_off, 86.0, btn_h),
            self.wireframe_enabled,
        );
        diag_x += 86.0 + btn_gap;

        // FPS Badge Pill
        let fps_badge_node = tree.create_node();
        let fps_color = if self.fps >= 55.0 {
            self.style.fps_good
        } else if self.fps >= 30.0 {
            self.style.fps_warn
        } else {
            self.style.fps_poor
        };
        let fps_text = format!("{:.0} FPS · {:.1}ms", self.fps, self.frame_ms);
        if let Some(node) = tree.get_mut(fps_badge_node) {
            node.name = Some("ViewportHud_FpsPill".to_string());
            node.computed_rect = Rect::new(diag_x, diag_bar_rect.y + btn_y_off, 148.0, btn_h);
            node.layer = UiLayer::Floating;
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.0, 0.0, 0.35))
                .border_radius(4.0);
            node.text = Some(fps_text);
            node.text_color = fps_color;
            node.font_size = 11.5;
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(diag_bar_node, fps_badge_node);

        ViewportHudFrame {
            root_id,
            transform_bar_rect,
            play_bar_rect,
            diagnostics_bar_rect: diag_bar_rect,
        }
    }

    /// Private helper to assemble a styled button node with semantic tag, cursor, and role.
    fn add_hud_button(
        &self,
        tree: &mut UiTree,
        parent: WidgetId,
        tag: u64,
        label: &str,
        rect: Rect,
        is_active: bool,
    ) {
        let btn = tree.create_node();
        if let Some(node) = tree.get_mut(btn) {
            node.name = Some(format!("ViewportHud_Btn_{}", label));
            node.computed_rect = rect;
            node.layer = UiLayer::Floating;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.tag = tag;
            node.interactive = true;

            let bg = if is_active {
                self.style.button_active_bg
            } else {
                self.style.button_idle_bg
            };
            let fg = if is_active {
                self.style.text_active
            } else {
                self.style.text_idle
            };

            node.style = Style::new().background(bg).border_radius(4.0);
            node.text = Some(label.to_string());
            node.text_color = fg;
            node.font_size = 11.5;
            node.text_align = TextAlign::Center;
        }
        let _ = tree.add_child(parent, btn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_hud_builder_and_tag_assignment() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        }
        let _ = tree.set_root(root);

        let frame = ViewportHudBuilder::new(Rect::new(0.0, 0.0, 1920.0, 1080.0))
            .gizmo_mode(ViewportGizmoMode::Translate)
            .gizmo_space(ViewportGizmoSpace::World)
            .snapping(true)
            .grid(true)
            .wireframe(false)
            .camera_mode(ViewportCameraMode::Perspective)
            .engine_mode(ViewportEngineMode::Playing)
            .diagnostics(60.0, 16.67)
            .build(&mut tree, root);

        assert!(tree.get(frame.root_id).is_some());
        assert_eq!(frame.transform_bar_rect.x, 10.0);
        assert_eq!(frame.transform_bar_rect.y, 10.0);

        // Verify layered hit testing correctly resolves HUD buttons by position
        let translate_btn_pos = iris_core::geometry::Point::new(75.0, 20.0);
        let hit = tree.hit_test_target(translate_btn_pos);
        assert!(hit.is_some());
        let info = hit.unwrap();
        assert_eq!(info.tag, VIEWPORT_HUD_TAG_GIZMO_TRANSLATE);
        assert_eq!(info.cursor, Some(WidgetCursor::Pointer));
        assert_eq!(info.role, WidgetRole::Button);

        // Verify action evaluation from tag
        let action = evaluate_viewport_hud_tag(info.tag);
        assert_eq!(
            action,
            Some(ViewportHudAction::SetGizmoMode(
                ViewportGizmoMode::Translate
            ))
        );
    }

    #[test]
    fn test_viewport_hud_actions_mapping() {
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_GIZMO_SELECT),
            Some(ViewportHudAction::SetGizmoMode(ViewportGizmoMode::Select))
        );
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_GIZMO_SCALE),
            Some(ViewportHudAction::SetGizmoMode(ViewportGizmoMode::Scale))
        );
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_GIZMO_SPACE),
            Some(ViewportHudAction::ToggleGizmoSpace)
        );
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_PLAY_PAUSE),
            Some(ViewportHudAction::TogglePlayPause)
        );
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_STOP),
            Some(ViewportHudAction::StopSimulation)
        );
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_CAMERA_PROJECTION),
            Some(ViewportHudAction::ToggleCameraProjection)
        );
        assert_eq!(
            evaluate_viewport_hud_tag(VIEWPORT_HUD_TAG_WIREFRAME),
            Some(ViewportHudAction::ToggleWireframe)
        );
        assert_eq!(evaluate_viewport_hud_tag(999999), None);
    }
}