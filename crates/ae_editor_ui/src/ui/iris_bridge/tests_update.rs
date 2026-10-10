// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Retained UI update loop, always-rebuild and typography collection tests.
//!

use super::IrisEditorOverlay;
use super::types::IrisChromeState;
use irisui::prelude::{Color, FocusManager, Rect, UiScope, UiTree};

#[test]
fn test_always_rebuild_default_and_setter() {
    let mut chrome = IrisChromeState::default();
    assert!(
        !chrome.always_rebuild,
        "always_rebuild must default to false for retained multi-tree architecture"
    );
    chrome.always_rebuild = true;
    assert!(chrome.always_rebuild);
}

#[test]
fn test_collect_text_sections_with_hover_state() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn_tag = 888u64;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.button_tagged("Export", btn_tag);
        scope.finish_layout(Rect::new(0.0, 0.0, 800.0, 600.0));
    }

    // Without hover
    let normal = IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &[], &[]);
    assert_eq!(normal.len(), 1);
    assert_eq!(normal[0].color, Color::rgba(0.0, 0.88, 1.0, 1.0));

    // With hover on btn_tag
    let hovered = IrisEditorOverlay::collect_text_sections_from_tree_with_hover(
        &tree,
        &[],
        &[],
        &[],
        Some(btn_tag),
    );
    assert_eq!(hovered.len(), 1);
    assert_eq!(hovered[0].color, Color::WHITE);
}

#[test]
fn test_focus_manager_text_input_focus() {
    use super::hierarchy::HIERARCHY_TAG_SEARCH_INPUT;

    let mut focus_mgr = FocusManager::new();
    assert!(!focus_mgr.has_focus());
    assert_eq!(focus_mgr.focused_tag(), None);

    focus_mgr.set_focus_tag(HIERARCHY_TAG_SEARCH_INPUT);
    assert!(focus_mgr.has_focus());
    assert_eq!(focus_mgr.focused_tag(), Some(HIERARCHY_TAG_SEARCH_INPUT));
    assert!(focus_mgr.is_tag_focused(HIERARCHY_TAG_SEARCH_INPUT));
    assert!(!focus_mgr.is_tag_focused(99999));

    focus_mgr.clear_focus();
    assert!(!focus_mgr.has_focus());
    assert_eq!(focus_mgr.focused_tag(), None);
}