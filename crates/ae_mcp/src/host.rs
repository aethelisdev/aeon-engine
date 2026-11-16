// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Host abstraction decoupling `ae_mcp` from any specific engine or editor crate.

use irisui::core::{Rect, UiEvent, UiTree};
use serde_json::{Map, Value};
use winit::keyboard::KeyCode as WinitKeyCode;

/// Floating dock window descriptor reported by `ui.tree_dump`.
#[derive(Debug, Clone)]
pub struct McpFloatingWindow {
    /// Unique floating window identifier.
    pub id: u64,
    /// Display window title.
    pub title: String,
    /// Outer floating window bounding rectangle.
    pub rect: Rect,
    /// Inner dock content bounding rectangle.
    pub content_rect: Rect,
}

/// Window, DPI, mode, selection, and UI input capture telemetry returned by `health`.
#[derive(Debug, Clone)]
pub struct McpHealthSnapshot {
    /// Physical framebuffer dimensions `(width, height)`.
    pub physical_size: (u32, u32),
    /// Active UI scale / DPI factor.
    pub scale_factor: f32,
    /// Human-readable active engine mode (e.g. `"Edit"`, `"Play"`).
    pub mode: String,
    /// Currently selected entity bits, if any.
    pub selected_entity_bits: Option<u64>,
    /// Whether the UI currently captures keyboard input (focused text box).
    pub wants_keyboard_input: bool,
}

/// Summary descriptor for an entity in the host's scene graph.
#[derive(Debug, Clone)]
pub struct McpEntityInfo {
    /// Encoded `u64` entity identifier bits.
    pub id_bits: u64,
    /// Optional entity display name.
    pub name: Option<String>,
}

/// Structured host logger entry exposed via `engine.logs`.
#[derive(Debug, Clone)]
pub struct McpLogEntry {
    /// Formatted timestamp string.
    pub timestamp: String,
    /// Log severity string (`"ERROR"`, `"WARN"`, `"INFO"`, `"DEBUG"`, `"TRACE"`).
    pub level: String,
    /// Log source module target.
    pub target: String,
    /// Formatted log payload message.
    pub message: String,
}

/// Borrowed WGPU device/queue/format handles required for overlay rendering and screenshot readback.
pub struct McpGpuHandles<'a> {
    /// Logical GPU device.
    pub device: &'a wgpu::Device,
    /// Hardware command submission queue.
    pub queue: &'a wgpu::Queue,
    /// Target surface texture format.
    pub format: wgpu::TextureFormat,
    /// Current physical surface width and height.
    pub physical_size: (u32, u32),
}

/// Host application contract implemented by `ae_engine` (or any Iris UI desktop host)
/// to expose live UI inspection, input injection, GPU readback, and scene queries to [`crate::McpSession`].
///
/// Keeping all engine/ECS concrete types behind this trait allows `ae_mcp` to remain 100%
/// independent of internal engine crates.
pub trait McpHost {
    /// Returns live window, scale, mode, and focus telemetry.
    fn health_snapshot(&self) -> McpHealthSnapshot;

    /// Returns read-only access to the host's retained Iris UI tree.
    fn ui_tree(&self) -> &UiTree;

    /// Returns active floating dock window bounds for layout introspection.
    fn floating_windows(&self) -> Vec<McpFloatingWindow>;

    /// Returns the workbench's current logical viewport extent `(width, height)`, if known.
    fn logical_screen_size(&self) -> Option<[f32; 2]>;

    /// Dispatches a synthetic [`UiEvent`] through the host's normal UI arbiter.
    fn dispatch_ui_event(&mut self, event: &UiEvent) -> bool;

    /// Feeds a hardware key state change into the host's global shortcut/input tracker.
    fn inject_shortcut_key(&mut self, key: WinitKeyCode, pressed: bool);

    /// Flags retained UI panels for rebuild (`mark_all = true` forces full shell/panel refresh) and requests a redraw.
    fn invalidate_ui(&mut self, mark_all: bool);

    /// Requests a platform window redraw for the next frame.
    fn request_redraw(&self);

    /// Returns borrowed WGPU device, queue, and surface metadata for offscreen capture.
    fn gpu_handles(&self) -> McpGpuHandles<'_>;

    /// Renders the host UI into an offscreen target `view` during screenshot capture.
    fn render_ui_to_view(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_size: (u32, u32),
        scale_factor: f32,
    );

    /// Lists all entities in the active scene along with the active selection bits.
    fn list_entities(&self) -> (Option<u64>, Vec<McpEntityInfo>);

    /// Selects an entity by `id_bits` or `name` and flags relevant UI panels dirty.
    ///
    /// # Errors
    /// Returns an error string when no matching entity exists.
    fn select_entity(&mut self, id_bits: Option<u64>, name: Option<&str>) -> Result<u64, String>;

    /// Serializes all components of the target (or currently selected) entity into JSON fields.
    ///
    /// # Errors
    /// Returns an error string when no matching entity exists.
    fn inspect_entity(
        &self,
        id_bits: Option<u64>,
        name: Option<&str>,
    ) -> Result<(u64, Map<String, Value>), String>;

    /// Fetches up to `limit` recent log entries optionally filtered by severity `level`.
    ///
    /// # Errors
    /// Returns an error string when the host log buffer cannot be accessed.
    fn recent_logs(
        &self,
        limit: usize,
        level: Option<&str>,
    ) -> Result<(u64, Vec<McpLogEntry>), String>;
}