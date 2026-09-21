// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bottom status and diagnostics utility bar construction routines.

use irisui::prelude::*;

/// Height of the bottom status bar in physical pixels.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// Diagnostic parameters required to build the bottom status bar.
pub struct StatusBarParams<'a> {
    /// Screen width in physical pixels.
    pub screen_width: f32,
    /// Screen height in physical pixels.
    pub screen_height: f32,
    /// Optional status notification message spans with text color.
    pub status_spans: Option<&'a [(String, Color)]>,
}

/// Builds the bottom status bar widget tree matching the clean standard utility bar.
pub fn build_bottom_status_bar(tree: &mut UiTree, params: StatusBarParams<'_>) -> WidgetId {
    let mut bar_builder = StatusBarBuilder::new(tree, params.screen_width, STATUS_BAR_HEIGHT);

    // 1. Left side: Engine status message or "● Ready" indicator
    if let Some(spans) = params.status_spans
        && !spans.is_empty()
    {
        for (text, color) in spans {
            bar_builder.add_status_indicator(text, *color);
        }
    } else {
        bar_builder.add_status_indicator("● Ready", Color::hex("#46be78"));
    }

    // 2. Right side: Engine Version
    let version_text = format!("Aeon Engine v{}", env!("CARGO_PKG_VERSION"));
    bar_builder.add_right_label(&version_text, Color::hex("#646470"));

    bar_builder.build()
}