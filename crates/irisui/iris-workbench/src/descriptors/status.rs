// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Status Bar and Telemetry Pill Descriptors
//!
//! Provides declarative descriptors for configuring the bottom application status bar,
//! activity status messages, and right-aligned telemetry capsules (FPS, memory, entity counts).

use iris_core::Color;
use iris_core::WidgetRole;
use iris_core::geometry::Insets;
use iris_core::style::Style;
use iris_widgets::declarative::UiScope;

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

    /// Optional styled text spans for rich left status reporting (e.g. colored phase names).
    pub status_spans: Vec<(String, Color)>,

    /// Right-aligned telemetry capsules, profiler meters, and indicator pills.
    pub pills: Vec<StatusPillDescriptor>,

    /// Optional right-aligned engine metadata label (e.g. `"Aeon Engine v0.9.0"`).
    pub right_text: Option<String>,

    /// Height of the status bar in logical pixels (defaults to `22.0`).
    pub height: f32,
}

impl Default for StatusBarDescriptor {
    fn default() -> Self {
        Self {
            left_status: "Ready".to_string(),
            status_spans: Vec::new(),
            pills: Vec::new(),
            right_text: None,
            height: 22.0,
        }
    }
}

impl StatusBarDescriptor {
    /// Persistent 64-bit semantic tag for the status bar container.
    pub const STATUS_BAR_TAG: u64 = 0x5354_4154_5553_5F31;

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

    /// Sets rich colored status spans via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_status_spans(mut self, spans: Vec<(String, Color)>) -> Self {
        self.status_spans = spans;
        self
    }

    /// Sets right-aligned metadata text (e.g. version) via fluent chaining.
    #[inline]
    #[must_use]
    pub fn with_right_text(mut self, text: impl Into<String>) -> Self {
        self.right_text = Some(text.into());
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

    /// Declaratively renders the bottom application status bar and top divider into [`UiScope`].
    ///
    /// # Arguments
    /// * `scope` - Active declarative UI scope attached to the root container.
    /// * `screen_width` - Available width in physical pixels.
    /// * `screen_height` - Available window height in physical pixels.
    pub fn render(&self, scope: &mut UiScope<'_>, screen_width: f32, screen_height: f32) {
        // 1. Baseline divider above StatusBar (positioned absolutely with 1.0px height)
        scope.empty_box_passive_named(
            "IrisBottomStatusBarDivider",
            Style::new()
                .position_absolute()
                .left(0.0)
                .top(screen_height - self.height)
                .width(screen_width)
                .height(1.0)
                .background(Color::hex("#1c1e24")),
        );

        // 2. Status bar flex row container
        scope
            .row_with()
            .tagged(Self::STATUS_BAR_TAG, WidgetRole::Default)
            .width(screen_width)
            .height(self.height)
            .space_between()
            .padding_insets(Insets::new(0.0, 10.0, 0.0, 10.0))
            .background_themed(|t| t.toolbar_bg)
            .show(|bar| {
                // Left group: status spans or fallback left status message
                bar.row_spaced(10.0, |left| {
                    if !self.status_spans.is_empty() {
                        for (text, color) in &self.status_spans {
                            left.colored_label(*color, text);
                        }
                    } else {
                        left.colored_label(left.theme().success, &self.left_status);
                    }
                });

                // Right group: telemetry pills and optional version label
                bar.row_spaced(8.0, |right| {
                    for pill in &self.pills {
                        let color = match pill.style {
                            StatusPillStyle::Default => right.theme().text_muted,
                            StatusPillStyle::Accent => right.theme().accent,
                            StatusPillStyle::Warning => Color::hex("#f59e0b"),
                            StatusPillStyle::Danger => Color::hex("#ef4444"),
                            StatusPillStyle::Success => right.theme().success,
                        };
                        right.colored_label(color, &pill.label);
                    }

                    if let Some(version) = &self.right_text {
                        right.colored_label(right.theme().text_muted, version);
                    }
                });
            });
    }
}