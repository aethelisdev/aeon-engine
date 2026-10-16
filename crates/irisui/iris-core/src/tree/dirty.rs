// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Invalidation and dirty flag tracking mechanisms for the UI tree.

use crate::dirty::DirtyFlags;
use crate::id::WidgetId;

use super::arena::UiTree;

impl UiTree {
    /// Recursively marks dirty flags on a node and all of its descendants with zero heap allocation.
    pub fn mark_dirty_subtree(&mut self, id: WidgetId, flags: DirtyFlags) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.mark_dirty(flags);
        }
        let child_count = self.nodes.get(id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            let child_id = self.nodes[id].children[i];
            self.mark_dirty_subtree(child_id, flags);
        }
    }

    /// Checks if any node in the tree currently has any of the specified dirty flags set.
    pub fn has_dirty_nodes(&self, flags: DirtyFlags) -> bool {
        self.nodes.values().any(|n| n.dirty.intersects(flags))
    }
}