// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Bottom Status & Diagnostics Utility Bar
//!
//! Declarative utility bar positioned at the bottom of the editor shell,
//! presenting engine status diagnostics and version metadata exclusively via
//! [`irisui::workbench::StatusBarDescriptor`].

use irisui::prelude::*;

/// Height of the bottom status bar in physical pixels.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// Constructs a declarative [`irisui::workbench::StatusBarDescriptor`] for the editor shell.
///
/// Uses `"● Ready"` when `status_spans` is absent or empty; otherwise forwards all status spans.
/// Written as a pure declarative expression with zero intermediate bindings.
#[must_use]
pub fn build_workbench_statusbar_descriptor(
    status_spans: Option<&[(String, Color)]>,
) -> irisui::workbench::StatusBarDescriptor {
    match status_spans.filter(|spans| !spans.is_empty()) {
        Some(spans) => irisui::workbench::StatusBarDescriptor::new()
            .with_height(STATUS_BAR_HEIGHT)
            .with_right_text(format!("Aeon Engine v{}", env!("CARGO_PKG_VERSION")))
            .with_status_spans(spans.to_vec()),
        None => irisui::workbench::StatusBarDescriptor::new()
            .with_height(STATUS_BAR_HEIGHT)
            .with_right_text(format!("Aeon Engine v{}", env!("CARGO_PKG_VERSION")))
            .with_left_status("● Ready"),
    }
}

/// Renders the bottom status bar directly into the active [`UiScope`] via [`irisui::workbench::StatusBarDescriptor`].
pub fn render_workbench_status_bar(
    scope: &mut UiScope<'_>,
    screen_width: f32,
    screen_height: f32,
    status_spans: Option<&[(String, Color)]>,
) {
    build_workbench_statusbar_descriptor(status_spans).render(scope, screen_width, screen_height);
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::workbench::StatusBarDescriptor;

    #[test]
    fn test_status_bar_default_ready() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Failed to create root");

        {
            let mut scope = UiScope::new(&mut tree, root);
            render_workbench_status_bar(&mut scope, 1920.0, 1080.0, None);
        }

        let bar_id = tree
            .find_node_by_tag(StatusBarDescriptor::STATUS_BAR_TAG)
            .expect("Status bar node must exist");
        let bar_node = tree.get(bar_id).expect("Status bar node must exist");

        assert_eq!(bar_node.style.height, Some(STATUS_BAR_HEIGHT));
        assert_eq!(bar_node.style.width, Some(1920.0));
        assert_eq!(bar_node.style.justify_content, JustifyContent::SpaceBetween);

        let root_node = tree.get(root).expect("Root node must exist");
        assert!(root_node.children.contains(&bar_id));
        assert_eq!(bar_node.children.len(), 2);

        let left_group = tree.get(bar_node.children[0]).expect("Left group");
        let default_label = tree.get(left_group.children[0]).expect("Default label");
        assert_eq!(default_label.text.as_deref(), Some("● Ready"));
    }

    #[test]
    fn test_status_bar_custom_spans() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Failed to create root");

        let custom_spans = vec![
            ("Compiling Shader...".to_string(), Color::YELLOW),
            ("100%".to_string(), Color::GREEN),
        ];

        {
            let mut scope = UiScope::new(&mut tree, root);
            render_workbench_status_bar(&mut scope, 1280.0, 720.0, Some(&custom_spans));
        }

        let bar_id = tree
            .find_node_by_tag(StatusBarDescriptor::STATUS_BAR_TAG)
            .expect("Status bar node must exist");
        let bar_node = tree.get(bar_id).expect("Status bar node must exist");

        let left_group = tree
            .get(bar_node.children[0])
            .expect("Left group must exist");
        assert_eq!(left_group.children.len(), 2);

        let first_span = tree.get(left_group.children[0]).expect("First span");
        assert_eq!(first_span.text.as_deref(), Some("Compiling Shader..."));
    }
}