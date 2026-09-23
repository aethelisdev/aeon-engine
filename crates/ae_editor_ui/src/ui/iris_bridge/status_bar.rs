// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Bottom Status & Diagnostics Utility Bar
//!
//! Declarative utility bar positioned at the bottom of the editor viewport,
//! presenting engine status diagnostics, operational ready indicators, and
//! engine version metadata purely via [`UiScope`].
//!

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

/// Builds the bottom status bar widget tree directly using declarative [`UiScope`].
///
/// Attaches the root horizontal flex container to `parent_id` with `SpaceBetween`
/// alignment, placing the status indicators on the left and engine version metadata
/// on the right.
pub fn build_bottom_status_bar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &StatusBarParams<'_>,
) -> WidgetId {
    let mut scope = UiScope::new(tree, parent_id);

    let bar_style = Style::new()
        .flex_row()
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::Center)
        .width(params.screen_width)
        .height(STATUS_BAR_HEIGHT)
        .padding_insets(Insets::new(0.0, 10.0, 0.0, 10.0))
        .background(Color::rgba(0.071, 0.082, 0.122, 1.0));

    scope.container(bar_style, |bar| {
        // 1. Left side: Engine status message or "● Ready" indicator
        let left_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(10.0);

        bar.container(left_style, |left| {
            if let Some(spans) = params.status_spans
                && !spans.is_empty()
            {
                for (text, color) in spans {
                    left.label(text, 11.0, *color, TextAlign::Left);
                }
            } else {
                left.label("● Ready", 11.0, Color::hex("#46be78"), TextAlign::Left);
            }
        });

        // 2. Right side: Engine Version
        let right_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(8.0);

        bar.container(right_style, |right| {
            let version_text = format!("Aeon Engine v{}", env!("CARGO_PKG_VERSION"));
            right.label(version_text, 10.5, Color::hex("#646470"), TextAlign::Right);
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_default_ready() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Failed to create root");

        let params = StatusBarParams {
            screen_width: 1920.0,
            screen_height: 1080.0,
            status_spans: None,
        };

        let bar_id = build_bottom_status_bar(&mut tree, root, &params);
        let bar_node = tree.get(bar_id).expect("Status bar node must exist");

        // Verify dimensions and layout style
        assert_eq!(bar_node.style.height, Some(STATUS_BAR_HEIGHT));
        assert_eq!(bar_node.style.width, Some(1920.0));
        assert_eq!(bar_node.style.justify_content, JustifyContent::SpaceBetween);

        // Verify root has the bar as child
        let root_node = tree.get(root).expect("Root node must exist");
        assert!(root_node.children.contains(&bar_id));

        // Verify 2 groups: left group and right group
        assert_eq!(bar_node.children.len(), 2);
    }

    #[test]
    fn test_status_bar_custom_spans() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Failed to create root");

        let custom_spans = vec![
            ("Compiling Shader...".to_string(), Color::YELLOW),
            ("100%".to_string(), Color::GREEN),
        ];

        let params = StatusBarParams {
            screen_width: 1280.0,
            screen_height: 720.0,
            status_spans: Some(&custom_spans),
        };

        let bar_id = build_bottom_status_bar(&mut tree, root, &params);
        let bar_node = tree.get(bar_id).expect("Status bar node must exist");

        let left_group_id = bar_node.children[0];
        let left_group = tree.get(left_group_id).expect("Left group must exist");

        // Should have 2 children for the 2 spans
        assert_eq!(left_group.children.len(), 2);

        let first_span = tree.get(left_group.children[0]).expect("First span");
        assert_eq!(first_span.text.as_deref(), Some("Compiling Shader..."));
    }
}