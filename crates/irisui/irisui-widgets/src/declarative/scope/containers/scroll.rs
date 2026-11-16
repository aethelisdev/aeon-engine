// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scroll Container Synchronization Primitives
//!
//! Provides in-place scroll offset mutation and layout synchronization on active [`UiScope`]
//! trees without triggering destructive container rebuilds.

use super::super::core::UiScope;

impl<'a> UiScope<'a> {
    /// Synchronizes a scrollable container's offset and scrollbar thumb in-place on the active tree without rebuilding.
    ///
    /// Delegates directly to [`crate::scroll_area::ScrollArea::sync_scroll_in_place`].
    ///
    /// # Arguments
    /// * `container_tag` - Semantic tag of the scroll container viewport.
    /// * `track_tag` - Optional semantic tag of the scrollbar track.
    /// * `thumb_tag` - Optional semantic tag of the draggable scrollbar thumb.
    /// * `scroll_y` - Current vertical scroll offset in physical pixels.
    /// * `max_scroll_y` - Maximum scroll limit in physical pixels.
    ///
    /// Returns `true` if any nodes were found and updated.
    #[inline]
    pub fn sync_scroll_in_place(
        &mut self,
        container_tag: u64,
        track_tag: Option<u64>,
        thumb_tag: Option<u64>,
        scroll_y: f32,
        max_scroll_y: f32,
    ) -> bool {
        crate::scroll_area::ScrollArea::sync_scroll_in_place(
            self.tree,
            container_tag,
            track_tag,
            thumb_tag,
            scroll_y,
            max_scroll_y,
        )
    }

    /// Updates a container node's scroll offset and re-layouts its subtree in-place on the active tree.
    ///
    /// Delegates directly to [`crate::scroll_area::ScrollArea::update_container_scroll_by_tag`].
    ///
    /// # Arguments
    /// * `container_tag` - Semantic tag of the target container.
    /// * `scroll_y` - New vertical scroll offset in physical pixels.
    ///
    /// Returns `true` if the node was found and updated.
    #[inline]
    pub fn update_container_scroll_by_tag(&mut self, container_tag: u64, scroll_y: f32) -> bool {
        crate::scroll_area::ScrollArea::update_container_scroll_by_tag(
            self.tree,
            container_tag,
            scroll_y,
        )
    }
}