// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Application Status Bar Builder Routines
//!
//! Provides factory functions constructing the bottom application status and diagnostics bar
//! utilizing [`iris_widgets::StatusBarBuilder`].
//!

use crate::ui::statusbar::types::{STATUS_BAR_HEIGHT, StatusBarParams};
use iris_widgets::StatusBarBuilder;
use irisui::prelude::{Color, UiTree, WidgetId};

/// Color constant for the default idle status dot indicator ("● Ready", `#46be78`).
pub const COLOR_READY_GREEN: Color = Color::rgba(0.2745, 0.7451, 0.4706, 1.0);

/// Color constant for the dim engine version label text on the right side (`#646470`).
pub const COLOR_VERSION_LABEL: Color = Color::rgba(0.3922, 0.3922, 0.4392, 1.0);

/// Builds the bottom application status bar widget tree matching the clean legacy editor design.
///
/// Constructs a full-width bottom status bar with a solid black background, containing:
/// - Left side: Status notification spans or default "● Ready" indicator in `#46be78`.
/// - Right side: Engine version text label in `#646470`.
///
/// # Arguments
/// * `tree` - Mutable reference to the UI widget tree.
/// * `parent_id` - Optional parent widget node (e.g. root canvas) to attach the status bar to.
/// * `params` - Sizing, status notification spans, and optional version label parameters.
///
/// # Returns
/// The root [`WidgetId`] of the generated status bar container.
pub fn build_bottom_status_bar(
    tree: &mut UiTree,
    parent_id: Option<WidgetId>,
    params: StatusBarParams<'_>,
) -> WidgetId {
    let mut bar_builder = StatusBarBuilder::new(tree, params.screen_width, STATUS_BAR_HEIGHT)
        .with_background(Color::rgba(0.0, 0.0, 0.0, 1.0))
        .with_border(1.0, Color::rgba(0.18, 0.21, 0.28, 0.85));

    // 1. Left Side: Active status notification spans or default "● Ready" indicator
    if let Some(spans) = params.status_spans
        && !spans.is_empty()
    {
        for (text, color) in spans {
            bar_builder.add_status_indicator(text, *color);
        }
    } else {
        bar_builder.add_status_indicator("● Ready", COLOR_READY_GREEN);
    }

    // 2. Right Side: Engine Version label
    if let Some(version) = params.version_text {
        bar_builder.add_right_label(version, COLOR_VERSION_LABEL);
    } else {
        bar_builder.add_right_label(
            concat!("Aeon Engine v", env!("CARGO_PKG_VERSION")),
            COLOR_VERSION_LABEL,
        );
    }

    bar_builder.attach_to(parent_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_bottom_status_bar_ready_state() {
        let mut tree = UiTree::new();
        let root = tree.create_node();

        let params = StatusBarParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            status_spans: None,
            version_text: Some("Aeon Engine v0.9.0"),
        };

        let bar_id = build_bottom_status_bar(&mut tree, Some(root), params);

        let root_node = tree.get(root).expect("Root node must exist");
        assert!(root_node.children.contains(&bar_id));

        let bar_node = tree.get(bar_id).expect("Status bar container must exist");
        assert_eq!(bar_node.name.as_deref(), Some("BottomStatusBar"));
        assert_eq!(
            bar_node.style.background_color,
            Color::rgba(0.0, 0.0, 0.0, 1.0)
        );

        // Left group contains Ready indicator
        let left_group = tree
            .get(bar_node.children[0])
            .expect("Left group must exist");
        let indicator = tree
            .get(left_group.children[0])
            .expect("Indicator must exist");
        assert_eq!(indicator.text.as_deref(), Some("● Ready"));
        assert_eq!(indicator.text_color, COLOR_READY_GREEN);

        // Right group contains only version label
        let right_group = tree
            .get(bar_node.children[1])
            .expect("Right group must exist");
        assert_eq!(right_group.children.len(), 1);
        let version_label = tree
            .get(right_group.children[0])
            .expect("Version label must exist");
        assert_eq!(version_label.text.as_deref(), Some("Aeon Engine v0.9.0"));
        assert_eq!(version_label.text_color, COLOR_VERSION_LABEL);
    }

    #[test]
    fn test_build_bottom_status_bar_custom_spans() {
        let mut tree = UiTree::new();
        let root = tree.create_node();

        let spans = vec![("Saving scene...".to_string(), Color::YELLOW)];
        let params = StatusBarParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            status_spans: Some(&spans),
            version_text: None,
        };

        let bar_id = build_bottom_status_bar(&mut tree, Some(root), params);
        let bar_node = tree.get(bar_id).expect("Status bar container must exist");

        let left_group = tree
            .get(bar_node.children[0])
            .expect("Left group must exist");
        let indicator = tree
            .get(left_group.children[0])
            .expect("Indicator must exist");
        assert_eq!(indicator.text.as_deref(), Some("Saving scene..."));
        assert_eq!(indicator.text_color, Color::YELLOW);

        // Right group contains only default version label
        let right_group = tree
            .get(bar_node.children[1])
            .expect("Right group must exist");
        assert_eq!(right_group.children.len(), 1);
        let version_label = tree
            .get(right_group.children[0])
            .expect("Version label must exist");
        assert_eq!(
            version_label.text.as_deref(),
            Some(concat!("Aeon Engine v", env!("CARGO_PKG_VERSION")))
        );
        assert_eq!(version_label.text_color, COLOR_VERSION_LABEL);
    }
}