// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # In-Place Retained Live Text Updates (`UiTree`)
//!
//! Provides zero-layout-rebuild live text and color updates for nodes registered under
//! semantic 64-bit tags inside [`UiTree`].

use super::arena::UiTree;
use crate::DirtyFlags;

impl UiTree {
    /// Updates the retained text of the node tagged `tag` in place without mutating layout.
    ///
    /// Marks `DirtyFlags::PAINT | DirtyFlags::TEXT` and records live text + quad refresh flags
    /// only when the text actually changed. Returns `true` when the text was updated.
    pub fn update_live_text_by_tag(&mut self, tag: u64, text: &str) -> bool {
        self.update_live_text_inner(tag, text, None)
    }

    /// Updates both the retained text and foreground color of the node tagged `tag` in place
    /// without mutating layout.
    ///
    /// Marks `DirtyFlags::PAINT | DirtyFlags::TEXT` only when either text or color changed.
    pub fn update_live_text_colored_by_tag(
        &mut self,
        tag: u64,
        text: &str,
        color: crate::color::Color,
    ) -> bool {
        self.update_live_text_inner(tag, text, Some(color))
    }

    fn update_live_text_inner(
        &mut self,
        tag: u64,
        text: &str,
        color: Option<crate::color::Color>,
    ) -> bool {
        let Some(id) = self.node_tagged(tag).map(|n| n.id) else {
            return false;
        };
        if tag != 0 && self.live_node_tags.insert(tag) {
            self.pending_static_text_sync = true;
        }
        let Some(node) = self.nodes.get_mut(id) else {
            return false;
        };
        let text_same = node.text.as_deref() == Some(text);
        let color_same = color.is_none_or(|c| node.text_color == c);
        if text_same && color_same {
            return false;
        }
        if !text_same {
            if let Some(existing) = &mut node.text {
                existing.clear();
                existing.push_str(text);
            } else {
                node.text = Some(text.to_owned());
            }
        }
        if let Some(next_color) = color {
            node.text_color = next_color;
        }
        self.mark_dirty(id, DirtyFlags::PAINT | DirtyFlags::TEXT);
        if !text_same {
            self.record_live_text_update();
        }
        self.record_live_quad_update();
        true
    }
}