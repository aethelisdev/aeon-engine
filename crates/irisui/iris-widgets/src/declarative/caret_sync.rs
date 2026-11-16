// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Caret Blink Synchronization & Render Options Coordinator (`iris-widgets::declarative::caret_sync`)
//!
//! Connects [`iris_text::TextSystem`]'s internal caret blink animation timer with
//! retained [`iris_core::UiTree`] widgets, [`iris_wgpu::TreeCompilerOptions`], and
//! [`iris_text::TextCollectionOptions`] without forcing host applications to compute
//! manual frame-level modulo blink booleans or rebuild UI trees on blink transitions.

use iris_core::UiTree;
use iris_text::{CaretBlinkUpdate, TextCollectionOptions, TextSystem};
use iris_wgpu::TreeCompilerOptions;
use std::time::Duration;

/// Synchronizes internal caret blink timing inside `text_system` against `tree` using
/// the system monotonic clock (`Instant::now()`).
#[inline]
pub fn sync_caret_blink_for_tree(tree: &UiTree, text_system: &mut TextSystem) -> CaretBlinkUpdate {
    text_system.sync_caret_blink(tree)
}

/// Synchronizes internal caret blink timing inside `text_system` against `tree` using
/// an explicit frame `delta`.
#[inline]
pub fn sync_caret_blink_with_delta_for_tree(
    tree: &UiTree,
    text_system: &mut TextSystem,
    delta: Duration,
) -> CaretBlinkUpdate {
    text_system.sync_caret_blink_with_delta(tree, delta)
}

/// Builds GPU command [`TreeCompilerOptions`] wired directly to `tree` focus state and
/// `text_system`'s live caret blink visibility.
#[must_use]
#[inline]
pub fn build_caret_aware_compile_options<'a>(
    tree: &UiTree,
    text_system: &TextSystem,
    hovered_tag: Option<u64>,
) -> TreeCompilerOptions<'a> {
    TreeCompilerOptions::new()
        .with_tree_focus(tree)
        .with_hovered_tag(hovered_tag)
        .with_blink_caret(text_system.is_caret_visible())
}

/// Builds typography [`TextCollectionOptions`] wired directly to `tree` focus state and
/// `text_system`'s live caret blink visibility.
#[must_use]
#[inline]
pub fn build_caret_aware_text_options(
    tree: &UiTree,
    text_system: &TextSystem,
    hovered_tag: Option<u64>,
) -> TextCollectionOptions {
    text_system
        .text_collection_options(tree)
        .with_hovered_tag(hovered_tag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::{InputBoxProps, UiScope};
    use iris_core::{Color, Rect, UiTree};
    use iris_wgpu::compile_tree_draw_commands;

    #[test]
    fn test_widget_caret_blinks_from_text_system_timer_without_tree_rebuild() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut text_system = TextSystem::new();
        text_system.set_caret_blink_interval_ms(200);
        let tag = 0x7701;

        // Build input box ONCE using autonomous focused constructor (no manual blink bool)
        tree.event_state
            .start_text_edit(None, tag, "SceneName", 9, None);
        {
            let mut scope = UiScope::new(&mut tree, root);
            let props = InputBoxProps::focused("SceneName", "Name...", 200.0, true);
            let _ = scope.input_box_tagged(&props, tag);
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 40.0));
        }

        let base_quads = {
            let mut off_opts = TreeCompilerOptions::new()
                .with_tree_focus(&tree)
                .with_blink_caret(false);
            compile_tree_draw_commands(&tree, root, &mut off_opts)
                .quads
                .len()
        };

        // 1. Initial sync -> focused, caret visible -> +1 caret quad
        let u1 = sync_caret_blink_with_delta_for_tree(
            &tree,
            &mut text_system,
            Duration::from_millis(16),
        );
        assert!(u1.has_text_focus);
        assert!(u1.is_visible);
        assert!(u1.focus_changed);

        let mut opts_on = build_caret_aware_compile_options(&tree, &text_system, None);
        let cmds_on = compile_tree_draw_commands(&tree, root, &mut opts_on);
        assert_eq!(cmds_on.quads.len(), base_quads + 1);

        // 2. Advance internal TextSystem timer past half-cycle without rebuilding UI tree
        let u2 = sync_caret_blink_with_delta_for_tree(
            &tree,
            &mut text_system,
            Duration::from_millis(220),
        );
        assert!(u2.has_text_focus);
        assert!(!u2.is_visible);
        assert!(u2.blink_changed);

        let mut opts_off = build_caret_aware_compile_options(&tree, &text_system, None);
        let cmds_off = compile_tree_draw_commands(&tree, root, &mut opts_off);
        assert_eq!(cmds_off.quads.len(), base_quads);
    }

    #[test]
    fn test_custom_input_box_style_and_scope_sync_helpers() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut text_system = TextSystem::new();
        text_system.set_caret_blink_interval_ms(150);
        let custom_caret = Color::rgba(0.95, 0.35, 0.10, 1.0);
        let style = crate::declarative::InputBoxStyle::default().with_caret_color(custom_caret);

        let tag = 0x7702;
        tree.event_state.start_text_edit(None, tag, "Abc", 3, None);
        {
            let mut scope = UiScope::new(&mut tree, root);
            let props = InputBoxProps::focused("Abc", "Hint", 180.0, true).with_style(style);
            let _ = scope.input_box_tagged(&props, tag);
            let update =
                scope.sync_caret_blink_with_delta(&mut text_system, Duration::from_millis(16));
            assert!(update.is_visible);
            assert!(scope.is_caret_visible(&text_system));
        }

        let box_id = tree.find_node_by_tag(tag).expect("box node");
        let caret_id = tree
            .get(box_id)
            .expect("box")
            .children
            .iter()
            .copied()
            .find(|&id| {
                tree.get(id)
                    .is_some_and(|n| n.role == iris_core::WidgetRole::TextInputCaret)
            })
            .expect("caret node");
        assert_eq!(
            tree.get(caret_id).unwrap().style.background_color,
            custom_caret
        );
    }
}