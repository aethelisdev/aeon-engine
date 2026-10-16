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

/// Persistent 64-bit semantic tag for the status bar container.
pub const STATUS_BAR_TAG: u64 = 0x5354_4154_5553_5F31;

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
) {
    let mut scope = UiScope::new(tree, parent_id);

    scope
        .row_with()
        .tagged(STATUS_BAR_TAG, WidgetRole::Default)
        .width(params.screen_width)
        .height(STATUS_BAR_HEIGHT)
        .space_between()
        .padding_insets(Insets::new(0.0, 10.0, 0.0, 10.0))
        .background_themed(|t| t.toolbar_bg)
        .show(|bar| {
            // 1. Left side: Engine status message or "● Ready" indicator
            bar.row_spaced(10.0, |left| {
                if let Some(spans) = params.status_spans
                    && !spans.is_empty()
                {
                    for (text, color) in spans {
                        left.colored_label(*color, text);
                    }
                } else {
                    left.colored_label(left.theme().success, "● Ready");
                }
            });

            // 2. Right side: Engine Version
            bar.row_spaced(8.0, |right| {
                right.colored_label(
                    right.theme().text_muted,
                    format!("Aeon Engine v{}", env!("CARGO_PKG_VERSION")),
                );
            });
        });
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

        build_bottom_status_bar(&mut tree, root, &params);
        let bar_id = tree
            .find_node_by_tag(STATUS_BAR_TAG)
            .expect("Status bar node must exist");
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

        build_bottom_status_bar(&mut tree, root, &params);
        let bar_id = tree
            .find_node_by_tag(STATUS_BAR_TAG)
            .expect("Status bar node must exist");
        let bar_node = tree.get(bar_id).expect("Status bar node must exist");

        let left_group_id = bar_node.children[0];
        let left_group = tree.get(left_group_id).expect("Left group must exist");

        // Should have 2 children for the 2 spans
        assert_eq!(left_group.children.len(), 2);

        let first_span = tree.get(left_group.children[0]).expect("First span");
        assert_eq!(first_span.text.as_deref(), Some("Compiling Shader..."));
    }
}