// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frame-Scoped Semantic Tag Click Queries
//!
//! Exposes declarative, allocation-free queries over the completed click recorded by
//! [`iris_core::dispatch::EventState::frame_clicked_tag`] for the active frame.
//!
//! Retained subtrees allocate fresh [`iris_core::WidgetId`]s on every rebuild, so a click that
//! was dispatched before the rebuild can only be matched through the persistent 64-bit semantic
//! tag stored on the widget node. These queries let a consumer map such a click directly into
//! its own typed action value (`Option<T>`) without reading `UiTree::event_state` manually and
//! without intermediate `let` bindings in view code.

use super::core::UiScope;

impl UiScope<'_> {
    /// Returns `true` when the widget carrying the persistent semantic `tag` received a
    /// completed click (press and release) during the active frame.
    ///
    /// Tag `0` is the reserved "untagged" sentinel and never matches, which prevents every
    /// untagged widget from reporting a click. The query is `O(1)`, performs no allocation, and
    /// is valid both before and after the tagged widget is re-emitted in the current rebuild.
    #[inline]
    #[must_use]
    pub fn tag_clicked(&self, tag: u64) -> bool {
        tag != 0 && self.tree.event_state.frame_clicked_tag == Some(tag)
    }

    /// Maps a completed click on the semantic `tag` during the active frame into `Some(f())`.
    ///
    /// Returns `None` (and never invokes `f`) when the tag was not clicked or is `0`. Intended for
    /// declarative action mapping, e.g. `scope.map_tag_clicked(tag, || Action::Remove(id))`, so the
    /// consumer receives its own action type without the framework knowing that type.
    #[inline]
    pub fn map_tag_clicked<T>(&self, tag: u64, f: impl FnOnce() -> T) -> Option<T> {
        self.tag_clicked(tag).then(f)
    }

    /// Computes a deterministic non-zero 64-bit semantic tag for an action or identifier label.
    ///
    /// Uses FNV-1a hashing with a guaranteed non-zero result to avoid colliding with the
    /// untagged sentinel (`0`).
    #[inline]
    #[must_use]
    pub fn action_tag(label: &str) -> u64 {
        let tag = crate::declarative::types::hash_label(label);
        if tag == 0 { 1 } else { tag }
    }

    /// Returns `true` when the action or widget carrying the semantic label was clicked during the active frame.
    #[inline]
    #[must_use]
    pub fn action_clicked(&self, label: &str) -> bool {
        self.tag_clicked(Self::action_tag(label))
    }

    /// Maps a completed click on the semantic label during the active frame into `Some(f())`.
    #[inline]
    pub fn map_action_clicked<T>(&self, label: &str, f: impl FnOnce() -> T) -> Option<T> {
        self.action_clicked(label).then(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::declarative::scope::core::UiScope;
    use iris_core::UiTree;

    #[test]
    fn test_map_tag_clicked_matches_only_the_clicked_nonzero_tag() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        tree.event_state.frame_clicked_tag = Some(0xABCD);

        let scope = UiScope::new(&mut tree, root);
        assert!(scope.tag_clicked(0xABCD));
        assert!(!scope.tag_clicked(0x1234));
        assert_eq!(scope.map_tag_clicked(0xABCD, || 7_u8), Some(7));
        assert_eq!(scope.map_tag_clicked(0x1234, || 7_u8), None);
    }

    #[test]
    fn test_tag_clicked_rejects_zero_sentinel_and_empty_frame() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");

        assert!(!UiScope::new(&mut tree, root).tag_clicked(0));
        assert!(!UiScope::new(&mut tree, root).tag_clicked(0xABCD));

        tree.event_state.frame_clicked_tag = Some(0);
        assert!(!UiScope::new(&mut tree, root).tag_clicked(0));
        assert_eq!(
            UiScope::new(&mut tree, root).map_tag_clicked(0, || "never"),
            None
        );
    }

    #[test]
    fn test_action_tag_and_action_clicked() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let tag = UiScope::action_tag("DeleteComponent");
        assert_ne!(tag, 0);

        tree.event_state.frame_clicked_tag = Some(tag);
        let scope = UiScope::new(&mut tree, root);
        assert!(scope.action_clicked("DeleteComponent"));
        assert!(!scope.action_clicked("OtherAction"));
        assert_eq!(scope.map_action_clicked("DeleteComponent", || 42), Some(42));
        assert_eq!(scope.map_action_clicked("OtherAction", || 42), None);
    }
}