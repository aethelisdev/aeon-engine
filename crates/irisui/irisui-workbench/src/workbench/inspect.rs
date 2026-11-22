// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Read-Only UI Inspection & Automation Query API
//!
//! Exposes high-level inspection snapshots ([`UiInspectionSummary`], [`UiNodeDump`],
//! [`UiTreeDump`]) and bounding rectangle lookup helpers on [`Workbench`] and
//! [`WorkbenchHost`](super::WorkbenchHost) so external automation, accessibility,
//! and test harnesses inspect retained UI state without ever borrowing [`irisui_core::UiTree`].

use std::fmt::Debug;
use std::hash::Hash;

use irisui_core::{Rect, UiLayer, UiTree};

use super::runner::Workbench;

/// Concise telemetry summary describing active retained UI nodes and popup overlays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiInspectionSummary {
    /// Total widget node count currently allocated in the retained tree.
    pub node_count: usize,
    /// `true` when any context menu, dropdown, or registered popup node is active.
    pub popup_active: bool,
    /// Active combobox dropdown semantic tag, if one is currently expanded.
    pub active_combobox_tag: Option<u64>,
    /// Semantic tags of all currently active popup layers and menus.
    pub active_popup_tags: Vec<u64>,
}

/// Snapshot of a single retained widget node serialized for external UI inspection tools.
#[derive(Debug, Clone, PartialEq)]
pub struct UiNodeDump {
    /// Arena slot debug identifier.
    pub id: String,
    /// Optional developer-assigned widget debug name.
    pub name: Option<String>,
    /// Optional debug name of the immediate parent node.
    pub parent_name: Option<String>,
    /// 64-bit semantic interaction tag (`0` when untagged).
    pub tag: u64,
    /// Widget semantic role name.
    pub role: String,
    /// Active paint layer name.
    pub layer: String,
    /// Resolved physical/logical bounding rectangle in UI space.
    pub computed_rect: Rect,
    /// Flexbox positioning mode (`Relative` / `Absolute`).
    pub position: String,
    /// Absolute/relative left inset in logical pixels.
    pub inset_left: Option<f32>,
    /// Absolute/relative top inset in logical pixels.
    pub inset_top: Option<f32>,
    /// Whether pointer/keyboard hit testing considers this node interactive.
    pub interactive: bool,
    /// Optional rendered text label attached to the node.
    pub text: Option<String>,
}

/// Filtered snapshot of retained widget nodes, cached tag rectangles, and active popup tags.
#[derive(Debug, Clone, PartialEq)]
pub struct UiTreeDump {
    /// Matched widget node snapshots.
    pub nodes: Vec<UiNodeDump>,
    /// Cached bounding rectangles indexed by semantic tag.
    pub tag_rects: Vec<(u64, Rect)>,
    /// Semantic tags of currently active popup containers.
    pub active_popup_tags: Vec<u64>,
    /// Semantic tag of the currently open combobox dropdown, if any.
    pub active_combobox_tag: Option<u64>,
}

/// Computes a read-only [`UiInspectionSummary`] from a retained [`UiTree`].
#[must_use]
pub(crate) fn summarize_ui_tree(tree: &UiTree) -> UiInspectionSummary {
    UiInspectionSummary {
        node_count: tree.len(),
        popup_active: tree.has_active_popup_nodes(),
        active_combobox_tag: tree.active_combobox_tag,
        active_popup_tags: tree.active_popup_tags.iter().copied().collect(),
    }
}

/// Serializes filtered widget snapshots and tag rectangles from a retained [`UiTree`].
#[must_use]
pub(crate) fn dump_ui_tree(
    tree: &UiTree,
    tag_filter: Option<u64>,
    name_filter: Option<&str>,
    only_popups: bool,
) -> UiTreeDump {
    let mut nodes = Vec::new();
    for (id, node) in tree.iter() {
        if only_popups && node.layer < UiLayer::Floating {
            continue;
        }
        if let Some(target_tag) = tag_filter
            && node.tag != target_tag
        {
            continue;
        }
        if let Some(substr) = name_filter {
            let name_match = node.name.as_deref().is_some_and(|n| n.contains(substr));
            let text_match = node.text.as_deref().is_some_and(|t| t.contains(substr));
            if !name_match && !text_match {
                continue;
            }
        }
        let parent_name = node
            .parent
            .and_then(|pid| tree.get(pid))
            .and_then(|p| p.name.as_deref())
            .map(str::to_owned);
        nodes.push(UiNodeDump {
            id: format!("{id:?}"),
            name: node.name.clone(),
            parent_name,
            tag: node.tag,
            role: format!("{:?}", node.role),
            layer: format!("{:?}", node.layer),
            computed_rect: node.computed_rect,
            position: format!("{:?}", node.style.position),
            inset_left: node.style.inset_left,
            inset_top: node.style.inset_top,
            interactive: node.interactive,
            text: node.text.clone(),
        });
    }

    let tag_rects = tree
        .tag_rects
        .iter()
        .filter(|(tag, _)| tag_filter.is_none_or(|t| t == **tag))
        .map(|(tag, rect)| (*tag, *rect))
        .collect();

    UiTreeDump {
        nodes,
        tag_rects,
        active_popup_tags: tree.active_popup_tags.iter().copied().collect(),
        active_combobox_tag: tree.active_combobox_tag,
    }
}

/// Resolves the resolved non-zero bounding rectangle of the `index`-th widget matching `name`.
#[must_use]
pub(crate) fn resolve_widget_rect_in_tree(tree: &UiTree, name: &str, index: usize) -> Option<Rect> {
    tree.iter()
        .filter(|(_, n)| {
            n.name.as_deref() == Some(name) || n.text.as_deref().is_some_and(|t| t == name)
        })
        .nth(index)
        .map(|(_, n)| n.computed_rect)
        .filter(|r| r.width > 0.0 && r.height > 0.0)
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Returns a concise read-only telemetry summary of the retained UI without exposing [`UiTree`].
    #[inline]
    #[must_use]
    pub fn ui_inspection_summary(&self) -> UiInspectionSummary {
        summarize_ui_tree(&self.tree)
    }

    /// Captures a filtered node & tag-bounds snapshot of the active UI for automation or diagnostics.
    #[must_use]
    pub fn inspect_ui_nodes(
        &self,
        tag_filter: Option<u64>,
        name_filter: Option<&str>,
        only_popups: bool,
    ) -> UiTreeDump {
        dump_ui_tree(&self.tree, tag_filter, name_filter, only_popups)
    }

    /// Resolves the bounding rectangle for a semantic widget `tag`, if laid out with non-zero size.
    #[inline]
    #[must_use]
    pub fn resolve_tag_rect(&self, tag: u64) -> Option<Rect> {
        self.tree.resolve_anchor_rect(tag)
    }

    /// Resolves the non-zero bounding rectangle for the `index`-th widget matching `name` or text.
    #[inline]
    #[must_use]
    pub fn resolve_widget_rect(&self, name: &str, index: usize) -> Option<Rect> {
        resolve_widget_rect_in_tree(&self.tree, name, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptors::WorkbenchDescriptor;
    use irisui_core::Style;
    use irisui_widgets::UiScope;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Tab {
        Main,
    }

    #[test]
    fn test_workbench_inspection_summary_and_queries() {
        let mut wb = Workbench::<Tab, (), ()>::new(WorkbenchDescriptor::new(
            "Test",
            irisui_dock::DockState::from_layout(irisui_dock::DockLayoutNode::tabs(vec![Tab::Main])),
        ));
        let root = wb.tree.create_root().expect("root");
        let rect = Rect::new(12.0, 24.0, 120.0, 32.0);
        {
            let mut scope = UiScope::new(&mut wb.tree, root);
            scope.container_named(
                "TargetButton",
                Style::new()
                    .position_absolute()
                    .left(rect.x)
                    .top(rect.y)
                    .width(rect.width)
                    .height(rect.height),
                |inner| {
                    inner.text("ClickMe");
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 800.0, 600.0));
        }

        if let Some(id) = wb.tree.find_node_by_name("TargetButton")
            && let Some(node) = wb.tree.get_mut(id)
        {
            node.tag = 0x5555;
        }
        wb.tree.open_popup(0x6666);

        let summary = wb.ui_inspection_summary();
        assert!(summary.node_count >= 2);
        assert_eq!(summary.active_popup_tags, vec![0x6666]);

        let dump = wb.inspect_ui_nodes(Some(0x5555), None, false);
        assert_eq!(dump.nodes.len(), 1);
        assert_eq!(dump.nodes[0].name.as_deref(), Some("TargetButton"));
        assert_eq!(wb.resolve_widget_rect("TargetButton", 0), Some(rect));
        assert_eq!(wb.resolve_tag_rect(0x5555), Some(rect));
    }
}