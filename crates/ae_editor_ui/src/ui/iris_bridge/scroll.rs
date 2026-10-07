// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Retained Scroll Synchronization Subsystem
//!
//! Synchronizes panel scroll offsets and scrollbar indicators across the retained UI tree
//! by delegating directly to [`irisui::prelude::ScrollArea`] without clearing or rebuilding nodes.
//!

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::ScrollArea;

impl IrisEditorOverlay {
    /// Determines whether any panel's vertical scroll offset has changed since the last synchronization.
    pub fn has_any_scroll_changed(&self) -> bool {
        (self.material.last_scroll_y - self.material.scroll_y).abs() > 0.001
            || (self.preferences.last_scroll_y - self.preferences.scroll_y).abs() > 0.001
            || (self.stats.last_scroll_y - self.stats.scroll_y).abs() > 0.001
            || (self.inspector.interactions.last_scroll_y - self.inspector.scroll_y).abs() > 0.001
            || (self.hierarchy.last_scroll_y - self.hierarchy.scroll_y).abs() > 0.001
            || (self.console.interactions.last_scroll_y - self.console.interactions.scroll_y).abs()
                > 0.001
            || (self.assets.interactions.last_scroll_y - self.assets.scroll_y).abs() > 0.001
    }

    /// Performs in-place scroll synchronization across all active scrollable containers in the retained tree,
    /// delegating to [`ScrollArea::sync_scroll_in_place`] and recompiling GPU draw commands without
    /// clearing or rebuilding widget nodes.
    pub fn sync_tree_scroll_offsets(&mut self, params: &OverlayUpdateParams<'_>) {
        if self.tree.is_empty() {
            return;
        }

        // 1. Material Panel
        if (self.material.last_scroll_y - self.material.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::sync_scroll_in_place(
                &mut self.tree,
                super::material::types::MATERIAL_TAG_VIEWPORT,
                Some(super::material::types::MATERIAL_TAG_SCROLLBAR_TRACK),
                Some(super::material::types::MATERIAL_TAG_SCROLLBAR_THUMB),
                self.material.scroll_y,
                self.material.max_scroll_y,
            );
            self.material.last_scroll_y = self.material.scroll_y;
        }

        // 2. Preferences Dialog
        if (self.preferences.last_scroll_y - self.preferences.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::sync_scroll_in_place(
                &mut self.tree,
                super::preferences::types::PREF_TAG_CONTENT_VIEW,
                Some(super::preferences::types::PREF_TAG_SCROLLBAR_TRACK),
                Some(super::preferences::types::PREF_TAG_SCROLLBAR_THUMB),
                self.preferences.scroll_y,
                self.preferences.max_scroll_y,
            );
            self.preferences.last_scroll_y = self.preferences.scroll_y;
        }

        // 3. Stats Panel
        if (self.stats.last_scroll_y - self.stats.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::update_container_scroll_by_tag(
                &mut self.tree,
                super::stats::types::STATS_TAG_VIEWPORT,
                self.stats.scroll_y,
            );
            self.stats.last_scroll_y = self.stats.scroll_y;
        }

        // 4. Inspector Panel
        if (self.inspector.interactions.last_scroll_y - self.inspector.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::update_container_scroll_by_tag(
                &mut self.tree,
                super::inspector::tags::TAG_INSPECTOR_CARDS_CONTAINER,
                self.inspector.scroll_y,
            );
            self.inspector.interactions.last_scroll_y = self.inspector.scroll_y;
        }

        // 5. Hierarchy Panel
        if (self.hierarchy.last_scroll_y - self.hierarchy.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::update_container_scroll_by_tag(
                &mut self.tree,
                super::hierarchy::types::HIERARCHY_TAG_VIEWPORT,
                self.hierarchy.scroll_y,
            );
            self.hierarchy.last_scroll_y = self.hierarchy.scroll_y;
        }

        // 6. Developer Console
        if (self.console.interactions.last_scroll_y - self.console.interactions.scroll_y).abs()
            > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::sync_scroll_in_place(
                &mut self.tree,
                super::console::types::CONSOLE_TAG_VIEWPORT,
                Some(super::console::types::CONSOLE_TAG_SCROLLBAR_TRACK),
                Some(super::console::types::CONSOLE_TAG_SCROLLBAR_THUMB),
                self.console.interactions.scroll_y,
                self.console.max_scroll_y,
            );
            self.console.interactions.last_scroll_y = self.console.interactions.scroll_y;
        }

        // 7. Assets Browser
        if (self.assets.interactions.last_scroll_y - self.assets.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            ScrollArea::update_container_scroll_by_tag(
                &mut self.tree,
                super::assets::types::ASSETS_TAG_CONTENT_VIEWPORT,
                self.assets.scroll_y,
            );
            self.assets.interactions.last_scroll_y = self.assets.scroll_y;
        }

        // Recompile GPU draw command list from updated layout nodes
        self.command_list.clear();
        if let Some(root) = self.tree.root() {
            self.populate_draw_commands(root, None, Some(params.telemetry.frame_pacing));
        }

        self.chrome.needs_scroll_sync = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::prelude::{Rect, Style, UiScope, UiTree, WidgetRole};

    #[test]
    fn test_iris_retained_scroll_delegation_with_editor_tags() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let vp_tag = super::super::inspector::tags::TAG_INSPECTOR_CARDS_CONTAINER;
        let (_cid, ch1) = {
            let mut scope = UiScope::new(&mut tree, root);
            let mut child = None;
            let c = scope.container_tagged(
                "InspectorCardsContainer",
                Style::new().width(200.0).height(400.0).flex_col(),
                WidgetRole::Default,
                vp_tag,
                |s| {
                    child = Some(s.empty_box_passive_named("Card", Style::new().height(50.0)));
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 200.0, 400.0));
            (c, child.unwrap())
        };

        assert_eq!(tree.get(ch1).unwrap().computed_rect.y, 0.0);

        // Update container scroll via ScrollArea API
        let success = ScrollArea::update_container_scroll_by_tag(&mut tree, vp_tag, 20.0);
        assert!(success);

        // Child should be shifted upwards by 20px
        assert_eq!(tree.get(ch1).unwrap().computed_rect.y, -20.0);
    }
}