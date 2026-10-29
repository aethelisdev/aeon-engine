// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Operating System File Drop Target Widget (`DropTarget`)
//!
//! Provides the declarative [`DropTargetId`], [`DropTargetResponse`], and fluent builder methods
//! on [`UiScope`], enabling components to register custom file drag-and-drop landing zones,
//! display reactive hover highlights, and consume dropped OS file paths without engine-level shims.

use std::ops::Deref;
use std::path::PathBuf;

use iris_core::{Rect, Style, WidgetResponse, WidgetRole};

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::hash_label_with_seed;

/// Magic seed constant used when deriving persistent tags for named drop targets.
pub const DROP_TARGET_SEED: u64 = 0x4452_4F50_5441_5247; // "DROPTARG"

/// Identifier assigned to a file drop target, either as an explicit 64-bit semantic tag or a string name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DropTargetId {
    /// Explicit 64-bit persistent semantic tag.
    Tag(u64),
    /// String name from which a deterministic persistent tag and debug name are derived.
    Named(String),
}

impl From<&str> for DropTargetId {
    #[inline]
    fn from(s: &str) -> Self {
        Self::Named(s.to_string())
    }
}

impl From<String> for DropTargetId {
    #[inline]
    fn from(s: String) -> Self {
        Self::Named(s)
    }
}

impl From<u64> for DropTargetId {
    #[inline]
    fn from(tag: u64) -> Self {
        Self::Tag(tag)
    }
}

impl From<u32> for DropTargetId {
    #[inline]
    fn from(tag: u32) -> Self {
        Self::Tag(tag as u64)
    }
}

impl DropTargetId {
    /// Resolves the persistent semantic tag for this drop target identifier.
    #[must_use]
    #[inline]
    pub fn resolve_tag(&self) -> u64 {
        match self {
            Self::Tag(tag) => *tag,
            Self::Named(name) => hash_label_with_seed(DROP_TARGET_SEED, name),
        }
    }

    /// Returns the string representation or debug name of this drop target.
    #[must_use]
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Self::Named(name) => name.as_str(),
            Self::Tag(_) => "DropTarget",
        }
    }
}

/// Interaction response produced by evaluating a file drop target container.
///
/// Bundles the inner closure value, the container's [`WidgetResponse`], active file hover status,
/// and any dropped file paths delivered during this interaction frame.
#[derive(Debug, Clone)]
pub struct DropTargetResponse<R> {
    /// Value returned by the inner child scope builder closure.
    pub inner: R,
    /// Interaction response emitted by the drop target container node.
    pub response: WidgetResponse,
    /// Whether one or more OS files are actively hovered over this drop target area.
    pub is_hovered: bool,
    /// Paths of files dropped onto this target during the current frame, if any.
    pub dropped_paths: Option<Vec<PathBuf>>,
}

impl<R> DropTargetResponse<R> {
    /// Executes the provided callback if files were dropped onto this drop target during this frame.
    ///
    /// Returns `self` to support fluent declarative chaining without intermediate `let` bindings.
    ///
    /// # Arguments
    /// * `callback` - Closure receiving a slice of dropped file paths.
    #[inline]
    pub fn on_drop<F>(self, callback: F) -> Self
    where
        F: FnOnce(&[PathBuf]),
    {
        if let Some(ref paths) = self.dropped_paths {
            callback(paths);
        }
        self
    }

    /// Returns `true` if an OS file is actively hovering over this drop target.
    #[must_use]
    #[inline]
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    /// Returns a reference to the dropped file paths, if any were dropped during this frame.
    #[must_use]
    #[inline]
    pub fn dropped_paths(&self) -> Option<&[PathBuf]> {
        self.dropped_paths.as_deref()
    }
}

impl<R> Deref for DropTargetResponse<R> {
    type Target = R;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative file drop target container executing a child builder closure.
    ///
    /// Passes `is_hovered: bool` to the inner closure so the child content can adaptively
    /// style its borders, background tint, or hint labels when an OS file is dragged overhead.
    ///
    /// Supports the fluent `.on_drop(|paths| ...)` continuation.
    ///
    /// # Arguments
    /// * `id` - Unique identifier ([`DropTargetId`]: string name or 64-bit persistent tag).
    /// * `f` - Child scope closure emitting the container content `|scope, is_hovered| ...`.
    ///
    /// # Example
    /// ```ignore
    /// scope.drop_target("AssetDropZone", |zone, is_hovered| {
    ///     let border_color = if is_hovered { Color::hex("#007ACC") } else { Color::TRANSPARENT };
    ///     zone.text("Drop assets here...");
    /// }).on_drop(|paths| {
    ///     println!("Dropped: {:?}", paths);
    /// });
    /// ```
    pub fn drop_target<ID, R, F>(&mut self, id: ID, f: F) -> DropTargetResponse<R>
    where
        ID: Into<DropTargetId>,
        F: FnOnce(&mut UiScope<'_>, bool) -> R,
    {
        let default_style = Style::new().flex_col().flex_grow(1.0);
        self.drop_target_styled(id, default_style, f)
    }

    /// Emits a declarative file drop target container with explicit flexbox styling.
    ///
    /// # Arguments
    /// * `id` - Unique identifier ([`DropTargetId`]: string name or 64-bit persistent tag).
    /// * `style` - Base flexbox and visual styling applied to the container node.
    /// * `f` - Child scope closure emitting the container content `|scope, is_hovered| ...`.
    pub fn drop_target_styled<ID, R, F>(
        &mut self,
        id: ID,
        style: Style,
        f: F,
    ) -> DropTargetResponse<R>
    where
        ID: Into<DropTargetId>,
        F: FnOnce(&mut UiScope<'_>, bool) -> R,
    {
        let target_id = id.into();
        let tag = target_id.resolve_tag();
        let name = target_id.name();

        // 1. Pre-layout hover check: check if an OS file is currently hovering over this area
        // using the previously computed rect for this persistent tag
        let prev_rect = self.tree.find_node_by_tag(tag).map_or(Rect::ZERO, |nid| {
            self.tree.get(nid).map_or(Rect::ZERO, |n| n.computed_rect)
        });

        let mut is_hovered = self
            .tree
            .event_state
            .hovered_file
            .as_ref()
            .is_some_and(|(_, pt)| {
                prev_rect.contains_point(*pt)
                    || self
                        .tree
                        .event_state
                        .hovered
                        .and_then(|hid| self.tree.get(hid))
                        .is_some_and(|n| n.tag == tag)
            });

        // 2. Allocate container node in generational arena
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.set_tag(tag);
            node.set_style(style);
            node.interactive = true;
            node.role = WidgetRole::Default;
        }
        let _ = self.tree.add_child(self.parent, node_id);

        // 3. Build nested child scope
        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope, is_hovered);

        // 4. Query widget response and synchronize post-build hover state
        let response = self.response(node_id);
        if response.file_hovered() {
            is_hovered = true;
        }

        // 5. Extract dropped paths if this drop target was the hit target or spatial recipient
        let dropped_paths = if let Some((target_id, target_tag, ref paths, pt)) =
            self.tree.event_state.frame_dropped_files
        {
            let matched = target_id == Some(node_id)
                || (tag != 0 && target_tag == tag)
                || target_id
                    .is_some_and(|tid| self.tree.is_direct_interaction_ancestor(tid, node_id))
                || response.rect.contains_point(pt)
                || prev_rect.contains_point(pt);

            if matched { Some(paths.clone()) } else { None }
        } else {
            None
        };

        DropTargetResponse {
            inner,
            response,
            is_hovered,
            dropped_paths,
        }
    }
}