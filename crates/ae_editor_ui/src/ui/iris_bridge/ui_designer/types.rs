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
    /// Currently hovered 64-bit semantic tag resolved in the active frame.
    pub hovered_tag: Option<u64>,
}

// ── 64-Bit Hardware Semantic Tags ────────────────────────────────────────────

/// Tag for the UI Designer root panel container.
pub const UI_DESIGNER_TAG_PANEL_ROOT: u64 = 0x0070_0000_0000_0001;
/// Tag for the letterbox outer container.
pub const UI_DESIGNER_TAG_LETTERBOX: u64 = 0x0070_0000_0000_0002;
/// Tag for the virtual canvas board.
pub const UI_DESIGNER_TAG_CANVAS_BOARD: u64 = 0x0070_0000_0000_0003;
/// Tag for the 34px elevated top toolbar container.
pub const UI_DESIGNER_TAG_TOOLBAR: u64 = 0x0070_0000_0000_0004;

// Toolbar Button Tags
/// Tag for Aspect Ratio dropdown trigger button.
pub const UI_DESIGNER_TAG_ASPECT_BTN: u64 = 0x0070_0000_0000_0010;
/// Tag for Zoom Out button.
pub const UI_DESIGNER_TAG_ZOOM_OUT: u64 = 0x0070_0000_0000_0011;
/// Tag for Zoom Reset button.
pub const UI_DESIGNER_TAG_ZOOM_RESET: u64 = 0x0070_0000_0000_0012;
/// Tag for Zoom In button.
pub const UI_DESIGNER_TAG_ZOOM_IN: u64 = 0x0070_0000_0000_0013;
/// Tag for Grid Snap cycle button.
pub const UI_DESIGNER_TAG_SNAP_BTN: u64 = 0x0070_0000_0000_0014;
/// Tag for Anchor Guide lines toggle button.
pub const UI_DESIGNER_TAG_ANCHORS_BTN: u64 = 0x0070_0000_0000_0015;
/// Tag for visual Grid toggle button.
pub const UI_DESIGNER_TAG_GRID_BTN: u64 = 0x0070_0000_0000_0016;
/// Tag for Add Element palette button.
pub const UI_DESIGNER_TAG_ADD_ELEMENT_BTN: u64 = 0x0070_0000_0000_0017;

// Aspect Ratio Popup Item Tags
/// Tag base for Aspect Ratio dropdown menu items: `0x0070_0000_0000_0100`.
pub const UI_DESIGNER_TAG_ASPECT_ITEM_BASE: u64 = 0x0070_0000_0000_0100;
/// Tag mask for Aspect Ratio dropdown menu items.
pub const UI_DESIGNER_TAG_ASPECT_ITEM_MASK: u64 = 0xFFFF_FFFF_FFFF_FFF0;

/// Constructs a semantic tag for an aspect ratio preset item.
#[inline]
pub const fn make_aspect_item_tag(idx: usize) -> u64 {
    UI_DESIGNER_TAG_ASPECT_ITEM_BASE | (idx as u64)
}

/// Parses an aspect ratio preset index from a semantic tag.
#[inline]
pub const fn parse_aspect_item_tag(tag: u64) -> Option<usize> {
    if (tag & UI_DESIGNER_TAG_ASPECT_ITEM_MASK) == UI_DESIGNER_TAG_ASPECT_ITEM_BASE {
        Some((tag & 0xF) as usize)
    } else {
        None
    }
}

// Add Element Palette Item Tags
/// Tag base for Add Element palette dropdown menu items: `0x0070_0000_0000_0200`.
pub const UI_DESIGNER_TAG_ADD_ITEM_BASE: u64 = 0x0070_0000_0000_0200;
/// Tag mask for Add Element palette dropdown menu items.
pub const UI_DESIGNER_TAG_ADD_ITEM_MASK: u64 = 0xFFFF_FFFF_FFFF_FF00;

/// Constructs a semantic tag for an add element palette item.
#[inline]
pub const fn make_add_item_tag(idx: usize) -> u64 {
    UI_DESIGNER_TAG_ADD_ITEM_BASE | (idx as u64)
}

/// Parses an add element palette item index from a semantic tag.
#[inline]
pub const fn parse_add_item_tag(tag: u64) -> Option<usize> {
    if (tag & UI_DESIGNER_TAG_ADD_ITEM_MASK) == UI_DESIGNER_TAG_ADD_ITEM_BASE {
        Some((tag & 0xFF) as usize)
    } else {
        None
    }
}

// On-Canvas UI Element Tags
/// Base tag prefix for on-canvas UI elements: `0x0070_1000_0000_0000`.
pub const UI_DESIGNER_TAG_ELEMENT_BASE: u64 = 0x0070_1000_0000_0000;
/// Mask to isolate the element tag prefix.
pub const UI_DESIGNER_TAG_ELEMENT_MASK: u64 = 0xFFFF_F000_0000_0000;
/// Mask to isolate the element index payload.
pub const UI_DESIGNER_TAG_ELEMENT_PAYLOAD_MASK: u64 = 0x0000_0FFF_FFFF_FFFF;

/// Encodes an element cache index into a 64-bit hardware tag.
#[inline]
pub const fn encode_element_tag(idx: usize) -> u64 {
    UI_DESIGNER_TAG_ELEMENT_BASE | (idx as u64 & UI_DESIGNER_TAG_ELEMENT_PAYLOAD_MASK)
}

/// Decodes an element cache index from a 64-bit hardware tag.
#[inline]
pub const fn parse_element_tag(tag: u64) -> Option<usize> {
    if (tag & UI_DESIGNER_TAG_ELEMENT_MASK) == UI_DESIGNER_TAG_ELEMENT_BASE {
        Some((tag & UI_DESIGNER_TAG_ELEMENT_PAYLOAD_MASK) as usize)
    } else {
        None
    }
}

// Tag domain constants
/// Tag family domain prefix for all 2D Visual UI Designer widgets: `0x0070_0000_0000_0000`.
pub const UI_DESIGNER_TAG_DOMAIN: u64 = 0x0070_0000_0000_0000;
/// Tag domain mask to isolate subsystem ownership: `0xFFFF_0000_0000_0000`.
pub const UI_DESIGNER_TAG_DOMAIN_MASK: u64 = 0xFFFF_0000_0000_0000;

/// Returns true if the given 64-bit hardware tag belongs to the 2D Visual UI Designer subsystem.
///
/// Evaluates whether the upper 16 bits match the domain prefix `0x0070`.
#[inline]
pub const fn is_ui_designer_tag(tag: u64) -> bool {
    (tag & UI_DESIGNER_TAG_DOMAIN_MASK) == UI_DESIGNER_TAG_DOMAIN
}

/// Initial drag context captured for an on-canvas UI element when rendered.
///
/// Contains pure state data required to initiate smooth drag operations in virtual canvas space.
/// Does NOT contain screen pixel rectangles; hit detection is 100% delegated to UiTree GPU SDF tags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiElementDragContext {
    /// Entity handle of the UI element in the ECS world.
    pub entity: hecs::Entity,
    /// Anchor origin in virtual canvas coordinates: `[x, y]`.
    pub anchor_origin: [f32; 2],
    /// Current element offset relative to anchor: `[x, y]`.
    pub initial_offset: [f32; 2],
}

/// Calculated virtual canvas metrics and coordinates for the 2D UI Designer.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UiDesignerCanvasMetrics {
    /// Screen-space bounding rectangle allocated for the total UI Designer panel.
    pub panel_rect: Rect,
    /// Screen-space bounding rectangle of the virtual canvas board.
    pub canvas_rect: Rect,
    /// Reference resolution of the virtual canvas in virtual pixels: `[width, height]`.
    pub resolution: [f32; 2],
    /// Current display scale factor from virtual coordinates to screen pixels.
    pub base_scale: f32,
    /// Current canvas zoom factor.
    pub current_zoom: f32,
    /// Active grid snap spacing in pixels, if enabled.
    pub snap_grid: Option<f32>,
}

impl UiDesignerCanvasMetrics {
    /// Converts a screen-space coordinate point into virtual canvas space.
    #[inline]
    pub fn screen_to_canvas(&self, screen_pt: Point) -> [f32; 2] {
        if self.canvas_rect.width <= 0.0 || self.canvas_rect.height <= 0.0 {
            return [0.0, 0.0];
        }
        let rel_x = (screen_pt.x - self.canvas_rect.x) / self.canvas_rect.width;
        let rel_y = (screen_pt.y - self.canvas_rect.y) / self.canvas_rect.height;
        [rel_x * self.resolution[0], rel_y * self.resolution[1]]
    }

    /// Converts a virtual canvas coordinate into screen-space pixel position.
    #[inline]
    pub fn canvas_to_screen(&self, canvas_pt: [f32; 2]) -> Point {
        Point::new(
            self.canvas_rect.x + (canvas_pt[0] / self.resolution[0]) * self.canvas_rect.width,
            self.canvas_rect.y + (canvas_pt[1] / self.resolution[1]) * self.canvas_rect.height,
        )
    }
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

/// Persistent interactive state for the 2D Visual UI Designer panel overlay.
#[derive(Debug, Default, Clone)]
pub struct UiDesignerPanelState {
    /// Common panel interaction state (actions, events).
    pub interactions: crate::ui::iris_bridge::types::PanelInteractionState<(), UiDesignerAction>,
    /// Calculated virtual canvas metrics and coordinates from the last render pass.
    pub canvas_metrics: UiDesignerCanvasMetrics,
    /// Cached on-canvas UI element drag contexts for O(1) drag initialization.
    pub drag_contexts: Vec<UiElementDragContext>,
    /// Whether the Aspect Ratio dropdown popup is open in the UI Designer.
    pub is_aspect_open: bool,
    /// Whether the Add Element palette popup is open in the UI Designer.
    pub is_add_menu_open: bool,
    /// Active element dragging state in the UI Designer.
    pub drag_state: Option<ae_uidesign::UiDragState>,
    /// Whether user is currently panning the UI Designer virtual canvas.
    pub is_panning: bool,
    /// Last mouse cursor coordinates recorded during UI Designer dragging or panning.
    pub last_cursor: Point,
}

impl std::ops::Deref for UiDesignerPanelState {
    type Target = crate::ui::iris_bridge::types::PanelInteractionState<(), UiDesignerAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for UiDesignerPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}