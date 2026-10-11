// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Retained Scroll Synchronization Subsystem
//!
//! Synchronizes panel scroll offsets and scrollbar indicators across the retained UI tree
//! by delegating directly to [`irisui::prelude::ScrollArea`] without clearing or rebuilding nodes.
//!

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
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
        if !self.has_any_scroll_changed() && !self.chrome.needs_scroll_sync {
            return;
        }

        // 1. Material Panel
        if (self.material.last_scroll_y - self.material.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.material.scroll_y;
            let max_scroll = self.material.max_scroll_y;
            if let Some(tree) = self.panel_tree_mut(PanelId::MaterialEditor) {
                ScrollArea::sync_scroll_in_place(
                    tree,
                    super::material::types::MATERIAL_TAG_VIEWPORT,
                    Some(super::material::types::MATERIAL_TAG_SCROLLBAR_TRACK),
                    Some(super::material::types::MATERIAL_TAG_SCROLLBAR_THUMB),
                    scroll_y,
                    max_scroll,
                );
            }
            self.material.last_scroll_y = scroll_y;
        }

        // 2. Preferences Dialog
        if (self.preferences.last_scroll_y - self.preferences.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.preferences.scroll_y;
            let max_scroll = self.preferences.max_scroll_y;
            ScrollArea::sync_scroll_in_place(
                self.overlay_tree.tree_mut(),
                super::preferences::types::PREF_TAG_CONTENT_VIEW,
                Some(super::preferences::types::PREF_TAG_SCROLLBAR_TRACK),
                Some(super::preferences::types::PREF_TAG_SCROLLBAR_THUMB),
                scroll_y,
                max_scroll,
            );
            self.preferences.last_scroll_y = scroll_y;
        }

        // 3. Stats Panel
        if (self.stats.last_scroll_y - self.stats.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.stats.scroll_y;
            if let Some(tree) = self.panel_tree_mut(PanelId::Stats) {
                ScrollArea::update_container_scroll_by_tag(
                    tree,
                    super::stats::types::STATS_TAG_VIEWPORT,
                    scroll_y,
                );
            }
            self.stats.last_scroll_y = scroll_y;
        }

        // 4. Inspector Panel
        if (self.inspector.interactions.last_scroll_y - self.inspector.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.inspector.scroll_y;
            if let Some(tree) = self.panel_tree_mut(PanelId::Inspector) {
                ScrollArea::update_container_scroll_by_tag(
                    tree,
                    super::inspector::tags::TAG_INSPECTOR_CARDS_CONTAINER,
                    scroll_y,
                );
            }
            self.inspector.interactions.last_scroll_y = scroll_y;
        }

        // 5. Hierarchy Panel
        if (self.hierarchy.last_scroll_y - self.hierarchy.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.hierarchy.scroll_y;
            let max_scroll = self.hierarchy.max_scroll;
            if let Some(tree) = self.panel_tree_mut(PanelId::Hierarchy) {
                ScrollArea::sync_scroll_in_place(
                    tree,
                    super::hierarchy::types::HIERARCHY_TAG_VIEWPORT,
                    Some(super::hierarchy::types::HIERARCHY_TAG_SCROLLBAR_TRACK),
                    Some(super::hierarchy::types::HIERARCHY_TAG_SCROLLBAR_THUMB),
                    scroll_y,
                    max_scroll,
                );
            }
            self.hierarchy.last_scroll_y = scroll_y;
        }

        // 6. Developer Console
        if (self.console.interactions.last_scroll_y - self.console.interactions.scroll_y).abs()
            > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.console.interactions.scroll_y;
            let max_scroll = self.console.max_scroll_y;
            if let Some(tree) = self.panel_tree_mut(PanelId::Console) {
                ScrollArea::sync_scroll_in_place(
                    tree,
                    super::console::types::CONSOLE_TAG_VIEWPORT,
                    Some(super::console::types::CONSOLE_TAG_SCROLLBAR_TRACK),
                    Some(super::console::types::CONSOLE_TAG_SCROLLBAR_THUMB),
                    scroll_y,
                    max_scroll,
                );
            }
            self.console.interactions.last_scroll_y = scroll_y;
        }

        // 7. Assets Browser
        if (self.assets.interactions.last_scroll_y - self.assets.scroll_y).abs() > 0.001
            || self.chrome.needs_scroll_sync
        {
            let scroll_y = self.assets.scroll_y;
            if let Some(tree) = self.panel_tree_mut(PanelId::Assets) {
                ScrollArea::update_container_scroll_by_tag(
                    tree,
                    super::assets::types::ASSETS_TAG_CONTENT_VIEWPORT,
                    scroll_y,
                );
            }
            self.assets.interactions.last_scroll_y = scroll_y;
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

    #[test]
    fn test_iris_retained_scroll_with_track_and_thumb_tags() {
        use irisui::prelude::{ScrollAreaStyle, ScrollBarGeometry};

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let vp_tag = super::super::hierarchy::types::HIERARCHY_TAG_VIEWPORT;
        let track_tag = super::super::hierarchy::types::HIERARCHY_TAG_SCROLLBAR_TRACK;
        let thumb_tag = super::super::hierarchy::types::HIERARCHY_TAG_SCROLLBAR_THUMB;

        {
            let mut scope = UiScope::new(&mut tree, root);
            let _c = scope.container_tagged(
                "HierarchyViewport",
                Style::new().width(260.0).height(400.0).flex_col(),
                WidgetRole::Default,
                vp_tag,
                |s| {
                    let _ = s.empty_box_passive_named("Item1", Style::new().height(800.0));
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 260.0, 400.0));

            // Create track and thumb nodes declaratively
            let style = ScrollAreaStyle::dark_default();
            let geom = ScrollBarGeometry::compute_vertical(
                Rect::new(0.0, 0.0, 260.0, 400.0),
                800.0,
                0.0,
                &style,
            )
            .unwrap();

            scope.scrollbar_vertical(
                geom,
                Rect::new(0.0, 0.0, 260.0, 400.0),
                false,
                None,
                track_tag,
                thumb_tag,
            );
        }

        let thumb_node_id = tree.find_node_by_tag(thumb_tag).expect("thumb must exist");
        let initial_thumb_y = tree.get(thumb_node_id).unwrap().computed_rect.y;

        // Perform in-place sync for 50% scroll (200px of 400px max scroll)
        let synced = ScrollArea::sync_scroll_in_place(
            &mut tree,
            vp_tag,
            Some(track_tag),
            Some(thumb_tag),
            200.0,
            400.0,
        );
        assert!(synced);

        let scrolled_thumb_y = tree.get(thumb_node_id).unwrap().computed_rect.y;
        assert!(scrolled_thumb_y > initial_thumb_y);
    }

    #[test]
    fn test_hierarchy_scrollbar_geometry_and_thumb_drag() {
        use irisui::prelude::{ScrollAreaStyle, ScrollBarGeometry};

        let vp_rect = Rect::new(0.0, 31.0, 260.0, 300.0);
        let content_h = 900.0;
        let style = ScrollAreaStyle::dark_default();

        let geom = ScrollBarGeometry::compute_vertical(vp_rect, content_h, 0.0, &style).unwrap();
        assert_eq!(geom.track_rect.x, 260.0 - style.thickness - style.inset);
        assert_eq!(geom.track_rect.y, 31.0 + style.inset);
        assert!(geom.thumb_rect.height >= style.thumb_min_length);
        assert!(geom.thumb_rect.height < geom.track_rect.height);

        let max_scroll = 600.0;
        let delta_scroll = ScrollBarGeometry::scroll_from_thumb_drag(
            50.0,
            geom.track_rect.height,
            geom.thumb_rect.height,
            max_scroll,
        );
        assert!(delta_scroll > 0.0);
    }
}