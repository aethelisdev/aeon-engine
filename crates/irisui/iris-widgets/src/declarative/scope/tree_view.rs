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
use iris_core::geometry::{Insets, Point};
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

/// Descriptor parameter bundle configuring a high-level tree node synchronized with [`TreeState`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeNodeProps<'a> {
    /// Debug name assigned to the row container.
    pub name: &'static str,
    /// Nesting depth level (0 = root).
    pub depth: u16,
    /// True if the item contains child branches.
    pub has_children: bool,
    /// Visual styling and indentation configuration.
    pub config: &'a TreeViewConfig,
}

impl<'a> TreeNodeProps<'a> {
    /// Creates a new tree node descriptor bundle.
    #[inline]
    pub const fn new(
        name: &'static str,
        depth: u16,
        has_children: bool,
        config: &'a TreeViewConfig,
    ) -> Self {
        Self {
            name,
            depth,
            has_children,
            config,
        }
    }
}

/// Comprehensive interaction response produced by emitting a [`UiScope::tree_row`].
///
/// Encapsulates separate click/hover states for both the row body and the foldout chevron button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeRowResponse {
    /// Allocated widget node ID for the row container.
    pub id: WidgetId,
    /// Allocated widget node ID for the foldout chevron button, if present.
    pub foldout_id: Option<WidgetId>,
    /// Whether the row container itself was clicked during this frame.
    pub clicked: bool,
    /// Whether the foldout chevron button was clicked during this frame.
    pub foldout_clicked: bool,
    /// Whether the row container is currently hovered by the pointer.
    pub hovered: bool,
    /// Whether the foldout chevron button is currently hovered.
    pub foldout_hovered: bool,
    /// Whether the row container was secondary (right) clicked during this frame.
    pub secondary_clicked: bool,
    /// Whether the row is currently marked as selected.
    pub is_selected: bool,
    /// Whether the branch is currently marked as expanded.
    pub is_expanded: bool,
    /// Absolute cursor screen position when this row interaction occurred, if captured.
    pub pointer_pos: Option<Point>,
}

impl TreeRowResponse {
    /// Returns the allocated widget node ID of the row container.
    #[inline]
    pub const fn id(&self) -> WidgetId {
        self.id
    }

    /// Returns the allocated widget node ID of the foldout chevron button, if present.
    #[inline]
    pub const fn foldout_id(&self) -> Option<WidgetId> {
        self.foldout_id
    }

    /// Returns `true` if the row body was clicked during this frame.
    #[inline]
    pub const fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` if the row body was secondary (right) clicked during this frame.
    #[inline]
    pub const fn secondary_clicked(&self) -> bool {
        self.secondary_clicked
    }

    /// Returns `true` if the foldout chevron button was clicked during this frame.
    #[inline]
    pub const fn foldout_clicked(&self) -> bool {
        self.foldout_clicked
    }

    /// Returns `true` if the pointer is currently hovering over the foldout chevron button.
    #[inline]
    pub const fn foldout_hovered(&self) -> bool {
        self.foldout_hovered
    }

    /// Returns `true` if the pointer is currently hovering over the row body.
    #[inline]
    pub const fn hovered(&self) -> bool {
        self.hovered
    }

    /// Returns `true` if this tree item is currently marked as selected.
    #[inline]
    pub const fn selected(&self) -> bool {
        self.is_selected
    }

    /// Returns `true` if this tree branch is currently expanded.
    #[inline]
    pub const fn expanded(&self) -> bool {
        self.is_expanded
    }

    /// Returns the absolute screen position of the cursor when this interaction occurred, if any.
    ///
    /// Useful for anchoring context menus and popups without manual engine coordinate propagation.
    #[inline]
    pub const fn interact_pointer_pos(&self) -> Option<Point> {
        self.pointer_pos
    }

    /// Executes the provided closure if the foldout chevron button was clicked during this frame.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_foldout(self, f: impl FnOnce()) -> Self {
        if self.foldout_clicked {
            f();
        }
        self
    }

    /// Executes the provided closure if the tree row body was clicked during this frame.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        if self.clicked {
            f();
        }
        self
    }

    /// Executes the provided closure if the tree row body was secondary (right) clicked during this frame.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_secondary_click(self, f: impl FnOnce()) -> Self {
        if self.secondary_clicked {
            f();
        }
        self
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
    /// Evaluates autonomous click and hover states on both the row container and the foldout button,
    /// returning a [`TreeRowResponse`].
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
    ) -> TreeRowResponse
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

        let resp = self.container_tagged(name, row_style, WidgetRole::Button, props.tag, |row_s| {
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
            let foldout_id = if props.has_children {
                let chevron_glyph = tree_chevron_arrow_glyph(props.is_expanded);
                let chevron_col = config.style.resolve_chevron_color(is_selected, false);

                let fold_style = Style::new()
                    .flex_row()
                    .width(12.0)
                    .height(config.row_height)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center);

                let fid = row_s
                    .container_tagged(
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
                    )
                    .response
                    .id;
                Some(fid)
            } else {
                None
            };

            // 2. Custom Row Content (Icon, Label, Actions)
            content(row_s);
            foldout_id
        });

        let row_id = resp.response.id;
        let foldout_id = resp.inner;

        let row_resp = resp.response;
        let row_clicked = row_resp.clicked();
        let row_secondary_clicked = row_resp.secondary_clicked();
        let row_hovered = row_resp.hovered();
        let pointer_pos = row_resp.interact_pointer_pos();

        let (foldout_clicked, foldout_hovered) = if let Some(fid) = foldout_id {
            let (fc, fh, _) = self.check_interaction(fid);
            (fc, fh)
        } else {
            (false, false)
        };

        TreeRowResponse {
            id: row_id,
            foldout_id,
            clicked: row_clicked,
            secondary_clicked: row_secondary_clicked,
            foldout_clicked,
            hovered: row_hovered,
            foldout_hovered,
            is_selected: props.is_selected,
            is_expanded: props.is_expanded,
            pointer_pos,
        }
    }

    /// Emits a hierarchical tree row and mutates the `is_expanded` state boolean in-place
    /// when the foldout chevron button is clicked.
    ///
    /// # Arguments
    /// - `name`: Debug name assigned to the row container.
    /// - `props`: Grouped row properties.
    /// - `is_expanded`: Mutable reference to the boolean state toggled upon chevron click.
    /// - `config`: Visual styling and indentation configuration.
    /// - `content`: Custom content closure.
    pub fn tree_row_toggle<F>(
        &mut self,
        name: &'static str,
        mut props: TreeRowProps,
        is_expanded: &mut bool,
        config: &TreeViewConfig,
        content: F,
    ) -> TreeRowResponse
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        props.is_expanded = *is_expanded;
        let resp = self.tree_row(name, props, config, content);
        if resp.foldout_clicked {
            *is_expanded = !*is_expanded;
        }
        resp
    }

    /// Emits a hierarchical tree node fully integrated with [`TreeState`].
    ///
    /// Automatically inspects and synchronizes expansion and selection states in [`TreeState`],
    /// toggling expansion upon chevron click and selecting the item upon row click.
    ///
    /// # Arguments
    /// - `state`: Mutable reference to the active [`TreeState`] manager.
    /// - `id`: Reference to the item's domain identifier.
    /// - `props`: Node descriptor specifying name, depth, child presence, and configuration.
    /// - `content`: Custom content closure.
    pub fn tree_node<Id, F>(
        &mut self,
        state: &mut TreeState<Id>,
        id: &Id,
        props: TreeNodeProps<'_>,
        content: F,
    ) -> TreeRowResponse
    where
        Id: Eq + Hash + Clone,
        F: FnOnce(&mut UiScope<'_>),
    {
        let is_expanded = state.is_expanded(id);
        let is_selected = state.is_selected(id);
        let tag = self.tag_for(props.name);
        let foldout_tag = tag.wrapping_add(1);
        let row_props = TreeRowProps::new(
            props.depth,
            props.has_children,
            is_expanded,
            is_selected,
            tag,
            foldout_tag,
        );
        let resp = self.tree_row(props.name, row_props, props.config, content);
        if resp.foldout_clicked {
            state.toggle_expanded(id);
        }
        if resp.clicked {
            state.select_single(id.clone());
        }
        resp
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
        let mut scroll_config = VirtualScrollConfig::fixed(items.len(), stride)
            .with_overscan(2)
            .with_bg(Color::TRANSPARENT)
            .with_scroll_y(props.scroll_y);

        if props.viewport_height > 0.0 {
            scroll_config = scroll_config.with_viewport_height(props.viewport_height);
        }

        let resp = self.virtual_scroll_area(
            config.name,
            props.tag,
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
        );

        resp.max_scroll_y
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
                row_s.label(if is_sel { "Selected" } else { "Normal" });
                assert!(item.depth <= 1);
            },
        );

        // All items fit within 300px viewport, so max_scroll is 0.0
        assert_eq!(max_scroll, 0.0);
        assert!(tree.len() > 3);
    }
}