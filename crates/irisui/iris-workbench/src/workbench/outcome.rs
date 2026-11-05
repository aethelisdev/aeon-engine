// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Event Outcome and Interaction Results
//!
//! Encapsulates the complete result of dispatching an interaction event through
//! the autonomous [`Workbench`] coordinator, including event consumption,
//! requested platform cursor, and emitted type-safe application actions.

use iris_core::node::WidgetCursor;

/// Summarized outcome returned after processing an event in [`crate::workbench::Workbench::handle_event`].
///
/// Designed to provide complete event telemetry to the hosting application without requiring
/// any internal knowledge of window borders, title bar dragging, dock splitters, or modal focus traps.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkbenchOutcome<Action> {
    /// Whether the event was intercepted and consumed by the workbench shell,
    /// stopping propagation to underlying game engines, 3D canvases, or custom tools.
    pub consumed: bool,

    /// Requested platform cursor icon override (e.g. `ResizeNorth`, `ColResize`, `Grabbing`, `Pointer`),
    /// to be synchronized with the OS window via [`iris_winit::frameless::WindowInterface::set_widget_cursor`].
    pub cursor: Option<WidgetCursor>,

    /// Whether the docking hierarchy, splitter ratios, or floating window boundaries changed,
    /// signaling that a layout rebuild or visual refresh is required.
    pub layout_changed: bool,

    /// List of decoupled application actions emitted during this event cycle
    /// (e.g. menu triggers, modal confirmations, titlebar minimize/maximize requests).
    pub actions: Vec<Action>,

    /// Dropped files intercepted during a native drag-and-drop file ingest event.
    pub dropped_files: Option<(Vec<std::path::PathBuf>, iris_core::geometry::Point)>,

    /// UI zoom scale factor update requested via keyboard shortcuts (e.g. Ctrl +/-/0).
    pub zoom_changed: Option<f32>,

    /// Toggle preferences modal dialog requested via keyboard shortcuts (e.g. Ctrl+,).
    pub toggle_preferences: bool,
}

impl<Action> Default for WorkbenchOutcome<Action> {
    #[inline]
    fn default() -> Self {
        Self {
            consumed: false,
            cursor: None,
            layout_changed: false,
            actions: Vec::new(),
            dropped_files: None,
            zoom_changed: None,
            toggle_preferences: false,
        }
    }
}

impl<Action> WorkbenchOutcome<Action> {
    /// Constructs an unconsumed, empty event outcome with no requested actions or layout alterations.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Constructs a consumed event outcome with an optional cursor override.
    #[inline]
    #[must_use]
    pub fn consumed(cursor: Option<WidgetCursor>) -> Self {
        Self {
            consumed: true,
            cursor,
            layout_changed: false,
            actions: Vec::new(),
            dropped_files: None,
            zoom_changed: None,
            toggle_preferences: false,
        }
    }

    /// Attaches an emitted action to the outcome, marking the event as consumed.
    #[inline]
    pub fn with_action(mut self, action: Action) -> Self {
        self.consumed = true;
        self.actions.push(action);
        self
    }
}