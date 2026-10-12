// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative TreeView and TreeRow Scope Primitives
//!
//! Exposes fluent declarative [`UiScope`] methods for rendering hierarchical tree rows
//! with branch lines, foldout chevrons, selection capsules, and virtualized list integration.
//!
//! ## Architectural Guarantees
//! - **Zero Per-Frame Allocation**: Connector line segments and item rows are emitted
//!   using POD geometries and stack closures.
//! - **Mathematical Alignment**: Child items, chevron buttons, and branch arms are aligned
//!   without visual overlap or manual coordinate math in host panels.
//!

use super::core::UiScope;
use crate::scroll_area::types::VirtualScrollConfig;
use crate::tree_view::model::{TreeIndentConfig, TreeState, for_each_tree_connector_line};
use crate::tree_view::navigation::TreeItemInfo;
use crate::tree_view::selection::{TreeRowStyle, tree_chevron_arrow_glyph};
use iris_core::Color;
use iris_core::geometry::Insets;
use iris_core::id::WidgetId;
use iris_core::node::WidgetRole;
use iris_core::style::{AlignItems, JustifyContent, Style, TextAlign};
use std::hash::Hash;

/// Configuration options customizing a declarative tree view and its rows.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeViewConfig {
    /// Debug name assigned to the container node.
    pub name: &'static str,
    /// Indentation step, margins, and branch line geometry.
    pub indent: TreeIndentConfig,
    /// Visual styling for normal, hovered, and active selection states.
    pub style: TreeRowStyle,
    /// Height of each tree row item in physical pixels.
    pub row_height: f32,
    /// Vertical gap between consecutive tree rows in physical pixels.
    pub row_gap: f32,
}

impl Default for TreeViewConfig {
    #[inline]
    fn default() -> Self {
        Self {
            name: "TreeView",
            indent: TreeIndentConfig::default(),
            style: TreeRowStyle::dark_default(),
            row_height: 24.0,
            row_gap: 2.0,
        }
    }
}

impl TreeViewConfig {
    /// Creates a new tree view configuration with engine default values.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the debug name.
    #[inline]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Overrides the indentation configuration.
    #[inline]
    pub fn with_indent(mut self, indent: TreeIndentConfig) -> Self {
        self.indent = indent;
        self
    }

    /// Overrides the visual row style.
    #[inline]
    pub fn with_style(mut self, style: TreeRowStyle) -> Self {
        self.style = style;
        self
    }

    /// Overrides the row height in physical pixels.
    #[inline]
    pub fn with_row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(1.0);
        self
    }

    /// Overrides the gap between consecutive rows in physical pixels.
    #[inline]
    pub fn with_row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap.max(0.0);
        self
    }

    /// Computes the total row stride including the row gap.
    #[inline]
    pub fn row_stride(&self) -> f32 {
        self.row_height + self.row_gap
    }
}

/// Descriptor parameter bundle configuring an individual tree row within a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeRowProps {
    /// Nesting depth level (0 = root).
    pub depth: u16,
    /// True if the item contains child branches.
    pub has_children: bool,
    /// True if the item is currently expanded.
    pub is_expanded: bool,
    /// True if the item is actively selected.
    pub is_selected: bool,
    /// Semantic hardware tag for row selection hit-testing.
    pub tag: u64,
    /// Semantic hardware tag for foldout chevron toggle hit-testing.
    pub foldout_tag: u64,
}

impl TreeRowProps {
    /// Creates a new tree row properties bundle.
    #[inline]
    pub const fn new(
        depth: u16,
        has_children: bool,
        is_expanded: bool,
        is_selected: bool,
        tag: u64,
        foldout_tag: u64,
    ) -> Self {
        Self {
            depth,
            has_children,
            is_expanded,
            is_selected,
            tag,
            foldout_tag,
        }
    }
}

/// Options bundle configuring a declarative virtualized tree view viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeViewProps {
    /// Semantic hardware tag for the virtual scroll viewport.
    pub tag: u64,
    /// Visible height of the viewport container in physical pixels.
    pub viewport_height: f32,
    /// Current vertical scroll position in physical pixels.
    pub scroll_y: f32,
}

impl TreeViewProps {
    /// Creates a new tree view viewport properties bundle.
    #[inline]
    pub const fn new(tag: u64, viewport_height: f32, scroll_y: f32) -> Self {
        Self {
            tag,
            viewport_height,
            scroll_y,
        }
    }
}

impl<'a> UiScope<'a> {
    /// Emits a single hierarchical tree row with branch connector lines, optional foldout chevron,
    /// selection capsule styling, and custom content.
    ///
    /// # Arguments
    /// - `name`: Debug name assigned to the row container.
    /// - `props`: Grouped row properties (depth, children, expansion, selection, tags).
    /// - `config`: Visual styling and indentation configuration.
    /// - `content`: Closure rendering the node icon, label, and custom controls.
    pub fn tree_row<F>(
        &mut self,
        name: &'static str,
        props: TreeRowProps,
        config: &TreeViewConfig,
        content: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let depth = props.depth;
        let padding_left = if depth == 0 {
            config.indent.base_padding
        } else {
            let last_depth = (depth - 1) as f32;
            let stem_x =
                config.indent.base_padding + 6.0 + (last_depth * config.indent.indent_step);
            stem_x + config.indent.line_arm_length + config.indent.line_arm_gap
        };

        let is_selected = props.is_selected;
        let mut row_style = config
            .style
            .build_style(is_selected)
            .flex_row()
            .align_items(AlignItems::Center)
            .height(config.row_height)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, padding_left))
            .gap(4.0);

        if !is_selected {
            row_style = row_style
                .hover_background(config.style.hover_bg)
                .hover_border(config.style.border_width, config.style.hover_border);
        }

        self.container_tagged(name, row_style, WidgetRole::Button, props.tag, |row_s| {
            // 0. Hierarchy Connector Lines (when depth > 0)
            if depth > 0 {
                let line_color = config.indent.line_color;
                for_each_tree_connector_line(depth, &config.indent, |v_line, h_line| {
                    let rel_x = v_line.x - padding_left;

                    let v_style = Style::new()
                        .position_absolute()
                        .left(rel_x)
                        .top(0.0)
                        .width(v_line.thickness)
                        .height(v_line.height)
                        .background(line_color);
                    row_s.empty_box(v_style);

                    if let Some(h) = h_line {
                        let h_style = Style::new()
                            .position_absolute()
                            .left(rel_x)
                            .top(h.y)
                            .width(h.width)
                            .height(h.thickness)
                            .background(line_color);
                        row_s.empty_box(h_style);
                    }
                });
            }

            // 1. Foldout Chevron Button (if item has children)
            if props.has_children {
                let chevron_glyph = tree_chevron_arrow_glyph(props.is_expanded);
                let chevron_col = config.style.resolve_chevron_color(is_selected, false);

                let fold_style = Style::new()
                    .flex_row()
                    .width(12.0)
                    .height(config.row_height)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

                row_s.container_tagged(
                    "TreeFoldoutButton",
                    fold_style,
                    WidgetRole::Button,
                    props.foldout_tag,
                    |fold_s| {
                        let txt_id = fold_s.label_with_width(
                            chevron_glyph,
                            12.0,
                            9.0,
                            chevron_col,
                            TextAlign::Center,
                        );
                        fold_s.set_hover_text_color(txt_id, config.style.chevron_hover_color);
                    },
                );
            }

            // 2. Custom Row Content (Icon, Label, Actions)
            content(row_s);
        })
    }

    /// Emits a virtualized, frustum-culled tree view rendering a slice of visible nodes.
    ///
    /// Automatically manages $O(1)$ windowing, overscan buffers, sub-pixel scroll clamping,
    /// and branch line rendering.
    ///
    /// Returns the computed maximum scroll limit `max_scroll_y` in physical pixels.
    pub fn tree_view<Id, FTag, FContent>(
        &mut self,
        props: TreeViewProps,
        items: &[TreeItemInfo<Id>],
        state: &TreeState<Id>,
        config: &TreeViewConfig,
        mut make_tags: FTag,
        mut row_content: FContent,
    ) -> f32
    where
        Id: Eq + Hash + Clone,
        FTag: FnMut(&Id, usize) -> (u64, u64),
        FContent: FnMut(&mut UiScope<'_>, &TreeItemInfo<Id>, usize, bool),
    {
        if items.is_empty() {
            return 0.0;
        }

        let stride = config.row_stride();
        let scroll_config = VirtualScrollConfig::fixed(items.len(), stride)
            .with_overscan(2)
            .with_bg(Color::TRANSPARENT);

        self.virtual_scroll_area(
            config.name,
            props.tag,
            props.viewport_height,
            props.scroll_y,
            scroll_config,
            |row_scope, item_idx| {
                let item = &items[item_idx];
                let is_selected = state.is_selected(&item.id);
                let is_expanded = state.is_expanded(&item.id);
                let (row_tag, foldout_tag) = make_tags(&item.id, item_idx);

                let row_props = TreeRowProps::new(
                    item.depth,
                    item.has_children,
                    is_expanded,
                    is_selected,
                    row_tag,
                    foldout_tag,
                );

                row_scope.tree_row("TreeItemRow", row_props, config, |content_s| {
                    row_content(content_s, item, item_idx, is_selected);
                });
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_declarative_tree_row_and_tree_view() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root widget node");
        let config = TreeViewConfig::new();

        let items = vec![
            TreeItemInfo::new(1u32, 0, true),
            TreeItemInfo::new(11u32, 1, false),
            TreeItemInfo::new(2u32, 0, false),
        ];
        let mut state = TreeState::<u32>::new();
        state.select_single(11);
        state.set_expanded(1, true);

        let mut scope = UiScope::new(&mut tree, root);
        let props = TreeViewProps::new(1001, 300.0, 0.0);
        let max_scroll = scope.tree_view(
            props,
            &items,
            &state,
            &config,
            |id, idx| (*id as u64 * 10, idx as u64 + 1),
            |row_s, item, _idx, is_sel| {
                row_s.label(
                    if is_sel { "Selected" } else { "Normal" },
                    12.0,
                    Color::WHITE,
                    TextAlign::Left,
                );
                assert!(item.depth <= 1);
            },
        );

        // All items fit within 300px viewport, so max_scroll is 0.0
        assert_eq!(max_scroll, 0.0);
        assert!(tree.len() > 3);
    }
}