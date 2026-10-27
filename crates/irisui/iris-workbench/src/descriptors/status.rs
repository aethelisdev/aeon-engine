// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Status Bar and Telemetry Pill Descriptors
//!
//! Provides declarative descriptors for configuring the bottom application status bar,
//! activity status messages, and right-aligned telemetry capsules (FPS, memory, entity counts).

/// Visual theme and emphasis style for status bar telemetry pills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusPillStyle {
    /// Subdued dark badge with translucent background.
    #[default]
    Default,

    /// Vibrant accent highlight (e.g. active editing mode or selection count).
    Accent,

    /// Amber warning pill indicating performance throttling or dropped frames.
    Warning,

    /// Crimson alert pill indicating critical engine errors or shader compile faults.
    Danger,

    /// Emerald success pill indicating synchronized state or successful builds.
    Success,
}

/// Describes an indicator badge or telemetry capsule rendered in the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPillDescriptor {
    /// Text content displayed inside the pill (e.g. `"144 FPS"`, `"12,450 Entities"`).
    pub label: String,

    /// Semantic interaction tag for click and hover handling.
    pub tag: u64,

    /// Visual style applied to the pill container.
    pub style: StatusPillStyle,

    /// Optional tooltip text displayed when hovering over the pill.
    pub tooltip: Option<String>,
}

impl StatusPillDescriptor {
    /// Creates a new status pill with default styling.
    ///
    /// # Arguments
    /// * `label` - Text content displayed inside the capsule.
    /// * `tag` - Semantic tag identifier.
    #[inline]
    #[must_use]
    pub fn new(label: impl Into<String>, tag: u64) -> Self {
        Self {
            label: label.into(),
            tag,
            style: StatusPillStyle::Default,
            tooltip: None,
        }
    }

    /// Sets the visual emphasis style via fluent chaining.
    #[inline]
    #[must_use]
    pub const fn with_style(mut self, style: StatusPillStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets an optional hover tooltip via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

/// Declarative configuration descriptor for the application status bar at the bottom of the shell.
///
/// Supports left-aligned status/progress reporting and right-aligned telemetry capsules.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusBarDescriptor {
    /// Left-aligned general status or activity message (e.g. `"Ready"`, `"Compiling Shaders..."`).
    pub left_status: String,

    /// Right-aligned telemetry capsules, profiler meters, and indicator pills.
    pub pills: Vec<StatusPillDescriptor>,

    /// Height of the status bar in logical pixels (defaults to `24.0`).
    pub height: f32,
}

impl Default for StatusBarDescriptor {
    fn default() -> Self {
        Self {
            left_status: "Ready".to_string(),
            pills: Vec::new(),
            height: 24.0,
        }
    }
}

impl StatusBarDescriptor {
    /// Creates a new status bar descriptor with `"Ready"` status and default height.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the left status message via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_left_status(mut self, status: impl Into<String>) -> Self {
        self.left_status = status.into();
        self
    }

    /// Sets the bar height in logical pixels via fluent chaining.
    #[inline]
    #[must_use]
    pub const fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Appends a telemetry pill via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_pill(mut self, pill: StatusPillDescriptor) -> Self {
        self.pills.push(pill);
        self
    }

    /// Appends a telemetry pill directly to the status bar.
    #[inline]
    pub fn add_pill(&mut self, pill: StatusPillDescriptor) {
        self.pills.push(pill);
    }
}