// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Types & Target Coordinates
//!
//! Exposes input parameters, hit-testing targets, and command actions for the 2D
//! in-game HUD and canvas designer studio rendered in 100% native Iris UI GPU SDF.
//!

pub use ae_uidesign::{CanvasAspectRatio, UiDesignerState, UiDragState, UiElementType};
use irisui::prelude::*;

/// Input parameters supplied to the UI Designer panel builder each frame.
pub struct UiDesignerPanelParams<'a> {
    /// Screen-space bounding rectangle allocated for the UI Designer panel.
    pub panel_rect: Rect,
    /// Read-only reference to the active ECS world.
    pub world: &'a hecs::World,
    /// Currently selected entity in the editor hierarchy or scene.
    pub selected_entity: Option<hecs::Entity>,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Persistent UI Designer state (aspect ratio, zoom, pan offset, grid snap).
    pub state: &'a UiDesignerState,
    /// Whether the Aspect Ratio dropdown popup is currently open.
    pub is_aspect_dropdown_open: bool,
    /// Whether the `➕ Add Element` palette popup is currently open.
    pub is_add_menu_open: bool,
}

/// Interactive hit target for an on-canvas UI element.
#[derive(Debug, Clone, Copy)]
pub struct UiElementHitTarget {
    /// Entity handle of the UI element in the ECS world.
    pub entity: hecs::Entity,
    /// Bounding box of the UI element in screen coordinates.
    pub rect: Rect,
    /// Anchor origin in virtual canvas coordinates: `[x, y]`.
    pub anchor_origin: [f32; 2],
    /// Current element offset: `[x, y]`.
    pub initial_offset: [f32; 2],
}

/// Hit-testing bounding boxes recorded during panel layout for mouse interactions.
#[derive(Debug, Clone, Default)]
pub struct UiDesignerPanelTargets {
    /// Total bounding box of the UI Designer panel.
    pub panel_rect: Rect,
    /// Virtual canvas bounding box in screen pixels.
    pub canvas_rect: Rect,
    /// Computed base scale factor converting virtual canvas pixels to screen pixels.
    pub base_scale: f32,
    /// Virtual canvas reference resolution `[width, height]`.
    pub resolution: [f32; 2],
    /// Current canvas zoom factor cached for toolbar and scrolling adjustments.
    pub current_zoom: f32,
    /// Active grid snap spacing in pixels, if enabled.
    pub snap_grid: Option<f32>,

    // ── Toolbar Targets ───────────────────────────────────────────────────────
    /// Hit target for the Aspect Ratio selector button.
    pub btn_aspect: Option<Rect>,
    /// Hit target for Zoom Out `-` button.
    pub btn_zoom_out: Option<Rect>,
    /// Hit target for Zoom Reset `100%` button.
    pub btn_zoom_reset: Option<Rect>,
    /// Hit target for Zoom In `+` button.
    pub btn_zoom_in: Option<Rect>,
    /// Hit target for Grid Snap cycle button (`Snap: 8px`).
    pub btn_snap: Option<Rect>,
    /// Hit target for visual Anchor Guide lines toggle.
    pub btn_anchors: Option<Rect>,
    /// Hit target for visual background Grid toggle.
    pub btn_grid: Option<Rect>,
    /// Hit target for `➕ Add Element` palette button.
    pub btn_add_element: Option<Rect>,

    // ── Popup Targets ─────────────────────────────────────────────────────────
    /// Full bounding box of the Aspect Ratio dropdown popup window if open.
    pub aspect_popup_rect: Option<Rect>,
    /// Hit targets for individual Aspect Ratio options: `(preset, rect)`.
    pub aspect_dropdown_options: Vec<(CanvasAspectRatio, Rect)>,
    /// Full bounding box of the Add Element palette popup window if open.
    pub add_popup_rect: Option<Rect>,
    /// Hit targets for individual UI element spawn options: `(element_type, rect)`.
    pub add_menu_options: Vec<(UiElementType, Rect)>,

    // ── Canvas Element Hit Targets ────────────────────────────────────────────
    /// Hit targets for on-canvas UI elements: `(entity, screen_rect)`.
    pub element_rects: Vec<(hecs::Entity, Rect)>,
    /// Detailed hit targets with anchor and offset data for drag operations.
    pub element_targets: Vec<UiElementHitTarget>,
}

/// Dispatched user interaction actions emitted by the UI Designer panel.
#[derive(Debug, Clone, PartialEq)]
pub enum UiDesignerAction {
    /// Requests spawning a new UI element of the specified type into the scene.
    SpawnElement(UiElementType),
    /// Selects or deselects an entity on the canvas.
    SelectEntity(Option<hecs::Entity>),
    /// Updates the 2D offset vector of a UI element during dragging.
    UpdateElementOffset {
        /// Target entity being repositioned.
        entity: hecs::Entity,
        /// New virtual canvas offset `[x, y]`.
        offset: [f32; 2],
    },
    /// Changes the active virtual canvas aspect ratio.
    SetAspectRatio(CanvasAspectRatio),
    /// Updates the virtual canvas zoom factor.
    SetZoom(f32),
    /// Toggles the background grid rendering.
    ToggleGrid,
    /// Toggles visual anchor pins and distance guidelines.
    ToggleAnchorGuides,
    /// Cycles through grid snap presets (Free -> 8px -> 16px -> 32px).
    CycleGridSnap,
    /// Resets the canvas zoom to 1.0x and centers the view.
    ResetView,
    /// Adjusts the 2D pan offset of the virtual canvas.
    PanCanvas([f32; 2]),
    /// Opens or closes the Aspect Ratio dropdown popup.
    ToggleAspectDropdown,
    /// Opens or closes the `➕ Add Element` palette popup.
    ToggleAddMenu,
    /// Closes any open dropdown popups.
    ClosePopups,
}