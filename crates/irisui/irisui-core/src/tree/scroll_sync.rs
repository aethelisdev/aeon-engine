// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scroll viewport state, content extent tracking, and post-layout scrollbar synchronization.

use crate::dispatch::engine::scrollbar::find_track_and_thumb_for_viewport;
use crate::geometry::{Rect, Size};
use crate::id::WidgetId;
use crate::node::WidgetRole;
use crate::tree::UiTree;

impl UiTree {
    /// Returns the persistent vertical scroll position for the given semantic `tag`.
    ///
    /// Defaults to `0.0` if this tag has never been scrolled.
    #[inline]
    #[must_use]
    pub fn scroll_offset(&self, tag: u64) -> f32 {
        self.scroll_offsets.get(&tag).copied().unwrap_or(0.0)
    }

    /// Sets the persistent vertical scroll position for the given semantic `tag`.
    #[inline]
    pub fn set_scroll_offset(&mut self, tag: u64, offset: f32) {
        self.scroll_offsets.insert(tag, offset.max(0.0));
    }

    /// Associates a domain `key` with the scrollable viewport `tag` and resets its scroll
    /// offset to `0.0` whenever `key` changes from a previously recorded key.
    ///
    /// Returns `true` when the key changed and the scroll offset was reset.
    #[inline]
    pub fn sync_scroll_key(&mut self, tag: u64, key: u64) -> bool {
        if tag == 0 {
            return false;
        }
        let prev = self.scroll_keys.insert(tag, key);
        if prev.is_some_and(|old| old != key) {
            self.scroll_offsets.insert(tag, 0.0);
            true
        } else {
            false
        }
    }

    /// Returns the resolved viewport size for the given semantic `tag` computed by layout.
    #[inline]
    #[must_use]
    pub fn viewport_size(&self, tag: u64) -> Option<Size> {
        self.viewport_sizes.get(&tag).copied()
    }

    /// Sets the resolved viewport size for the given semantic `tag`.
    #[inline]
    pub fn set_viewport_size(&mut self, tag: u64, size: Size) {
        self.viewport_sizes.insert(tag, size);
    }

    /// Returns the maximum vertical scroll extent for the given semantic `tag`.
    ///
    /// Defaults to `0.0` if this tag has no measured scrollable overflow content.
    #[inline]
    #[must_use]
    pub fn max_scroll(&self, tag: u64) -> f32 {
        self.max_scrolls.get(&tag).copied().unwrap_or(0.0)
    }

    /// Sets the content size for the node identified by the semantic `tag`.
    ///
    /// Updates the measured content bounds used for scroll normalization and bounds checking.
    #[inline]
    pub fn set_content_size(&mut self, tag: u64, size: Size) {
        if let Some(node) = self.find_node_by_tag(tag).and_then(|id| self.get_mut(id)) {
            node.content_size = size;
        }
    }

    /// Sets the maximum vertical scroll extent for the given semantic `tag`.
    #[inline]
    pub fn set_max_scroll(&mut self, tag: u64, max_scroll: f32) {
        self.max_scrolls.insert(tag, max_scroll.max(0.0));
    }

    /// Evaluates whether the scroll container associated with the specified semantic tag was scrolled in the current frame.
    ///
    /// Checks [`crate::dispatch::EventState::frame_scrolled_tag`] matching `tag`.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled(&self, tag: u64) -> bool {
        tag != 0 && self.event_state.frame_scrolled_tag == Some(tag)
    }

    /// Returns the mouse wheel or scrollbar displacement delta `(delta_x, delta_y)` for the specified container tag, if scrolled.
    ///
    /// Returns `None` if the container was not scrolled during the current dispatch cycle.
    #[inline]
    #[must_use]
    pub fn tag_scroll_delta(&self, tag: u64) -> Option<(f32, f32)> {
        if self.is_tag_scrolled(tag) {
            self.event_state.frame_scroll_delta
        } else {
            None
        }
    }

    /// Evaluates whether an upward scroll gesture (`delta_y > 0.0`) occurred on the specified tag during the current frame.
    ///
    /// In typical mouse wheel scrolling, an upward wheel roll produces positive `delta_y`,
    /// moving viewport content towards the top (earlier entries) and decreasing scroll offset.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled_up(&self, tag: u64) -> bool {
        self.tag_scroll_delta(tag).is_some_and(|(_, dy)| dy > 0.0)
    }

    /// Evaluates whether a downward scroll gesture (`delta_y < 0.0`) occurred on the specified tag during the current frame.
    ///
    /// In typical mouse wheel scrolling, a downward wheel roll produces negative `delta_y`,
    /// moving viewport content towards the bottom (later entries) and increasing scroll offset.
    #[inline]
    #[must_use]
    pub fn is_tag_scrolled_down(&self, tag: u64) -> bool {
        self.tag_scroll_delta(tag).is_some_and(|(_, dy)| dy < 0.0)
    }

    /// Synchronizes cached viewport dimensions, scroll clamping, and sibling/child
    /// scrollbar geometry right after layout assigns `viewport_id`'s `computed_rect`.
    ///
    /// Ensures scrollbars align accurately on Frame 1 and after dock splits or sibling
    /// layout changes without requiring pre-seeded viewport bounds.
    pub fn sync_scroll_viewport_after_layout(&mut self, viewport_id: WidgetId) {
        let Some((vp_rect, content_h, vp_tag, old_scroll_y)) = self
            .get(viewport_id)
            .filter(|n| n.role == WidgetRole::ScrollAreaViewport)
            .map(|n| {
                (
                    n.computed_rect,
                    n.content_size.height,
                    n.tag,
                    n.style.scroll_offset_y,
                )
            })
        else {
            return;
        };

        if vp_rect.width <= 0.0 || vp_rect.height <= 0.0 {
            return;
        }

        if vp_tag != 0 {
            self.set_viewport_size(vp_tag, vp_rect.size());
            self.set_tag_rect(vp_tag, vp_rect);
        }

        if content_h <= 0.0 {
            return;
        }

        let max_scroll = (content_h - vp_rect.height).max(0.0);
        if vp_tag != 0 {
            self.set_max_scroll(vp_tag, max_scroll);
        }

        let clamped_scroll = old_scroll_y.clamp(0.0, max_scroll);
        if (clamped_scroll - old_scroll_y).abs() > 1e-4 {
            if let Some(vp) = self.get_mut(viewport_id) {
                vp.style.scroll_offset_y = clamped_scroll;
            }
            if vp_tag != 0 {
                self.set_scroll_offset(vp_tag, clamped_scroll);
            }
            let shift_y = old_scroll_y - clamped_scroll;
            self.shift_descendants_y(viewport_id, shift_y);
        }

        let Some((track_id, thumb_id)) = find_track_and_thumb_for_viewport(self, viewport_id)
        else {
            return;
        };

        if max_scroll <= 0.0 {
            for bar_id in [track_id, thumb_id] {
                if let Some(node) = self.get_mut(bar_id) {
                    node.computed_rect = Rect::ZERO;
                    node.interactive = false;
                    node.style.width = Some(0.0);
                    node.style.height = Some(0.0);
                }
            }
            return;
        }

        let (thickness, inset_r, inset_v, host_origin) = {
            let track = self.get(track_id);
            let thickness = track
                .and_then(|n| n.style.width)
                .filter(|&w| w > 0.0)
                .unwrap_or(4.0);
            let inset_r = track.and_then(|n| n.style.inset_right).unwrap_or(3.0);
            let inset_v = track
                .and_then(|n| n.style.inset_top)
                .filter(|&v| (0.0..=16.0).contains(&v))
                .unwrap_or(inset_r);
            let host_origin = track
                .and_then(|n| n.parent)
                .and_then(|pid| self.get(pid))
                .map_or(vp_rect.origin(), |p| p.computed_rect.origin());
            (thickness, inset_r, inset_v, host_origin)
        };

        let track_h = (vp_rect.height - inset_v * 2.0).max(0.0);
        if track_h <= 0.0 {
            return;
        }
        let track_x = vp_rect.right() - thickness - inset_r;
        let track_y = vp_rect.y + inset_v;
        let track_rect = Rect::new(track_x, track_y, thickness, track_h);

        let min_thumb_len = 24.0_f32.min(track_h);
        let thumb_h = ((vp_rect.height / content_h) * track_h).clamp(min_thumb_len, track_h);
        let travel = (track_h - thumb_h).max(0.0);
        let ratio = (clamped_scroll / max_scroll).clamp(0.0, 1.0);
        let thumb_y = track_y + ratio * travel;
        let thumb_rect = Rect::new(track_x, thumb_y, thickness, thumb_h);

        if let Some(track) = self.get_mut(track_id) {
            track.computed_rect = track_rect;
            track.interactive = true;
            track.style.inset_left = Some(track_rect.x - host_origin.x);
            track.style.inset_top = Some(track_rect.y - host_origin.y);
            track.style.width = Some(thickness);
            track.style.height = Some(track_h);
        }
        if let Some(thumb) = self.get_mut(thumb_id) {
            thumb.computed_rect = thumb_rect;
            thumb.interactive = true;
            thumb.style.inset_left = Some(thumb_rect.x - host_origin.x);
            thumb.style.inset_top = Some(thumb_rect.y - host_origin.y);
            thumb.style.width = Some(thickness);
            thumb.style.height = Some(thumb_h);
        }
    }

    fn shift_descendants_y(&mut self, parent_id: WidgetId, delta_y: f32) {
        if delta_y.abs() <= 1e-4 {
            return;
        }
        let children = self
            .get(parent_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for child_id in children {
            let role = self.get(child_id).map(|n| n.role);
            if matches!(
                role,
                Some(WidgetRole::ScrollBarTrack | WidgetRole::ScrollBarThumb)
            ) {
                continue;
            }
            if let Some(child) = self.get_mut(child_id) {
                child.computed_rect.y += delta_y;
            }
            self.shift_descendants_y(child_id, delta_y);
        }
    }
}