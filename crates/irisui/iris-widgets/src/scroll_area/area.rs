// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ScrollArea Fluent Builder & Frame (`iris-widgets::scroll_area::area`)
//!
//! Renders hardware-clipped container viewports and proportional scrollbar indicators.
//!

use super::scroll_bar::ScrollBarGeometry;
use super::style::ScrollAreaStyle;
use super::types::ScrollBarVisibility;
use iris_core::WidgetRole;
use iris_core::color::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::style::Style;
use iris_core::tree::UiTree;

/// Layout and hit-testing targets returned after constructing a scroll area.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollAreaFrame {
    /// ID of the container node where scrollable child elements should be attached.
    pub container_id: WidgetId,
    /// Bounding rectangle of the visible viewport window.
    pub viewport_rect: Rect,
    /// Total virtual or measured height of all contained items.
    pub content_height: f32,
    /// Maximum vertical scroll offset before reaching the bottom edge.
    pub max_scroll_y: f32,
    /// Geometric layout of the rendered vertical scrollbar, if active.
    pub scrollbar: Option<ScrollBarGeometry>,
}

impl ScrollAreaFrame {
    /// Clamps an arbitrary scroll offset to the valid range `[0.0, max_scroll_y]`.
    #[must_use]
    pub fn clamp_scroll_y(&self, scroll: f32) -> f32 {
        scroll.clamp(0.0, self.max_scroll_y)
    }

    /// Computes the new scroll position given a mouse wheel delta and step size.
    ///
    /// Wheel deltas are typically negative for downwards scrolling, so delta is subtracted.
    #[must_use]
    pub fn scroll_step_wheel(&self, current_scroll: f32, wheel_delta: f32, step_size: f32) -> f32 {
        let delta = wheel_delta * step_size;
        self.clamp_scroll_y(current_scroll - delta)
    }
}

/// Fluent builder for constructing a generic scroll area container with optional scrollbars.
pub struct ScrollAreaBuilder<'a> {
    name: &'a str,
    viewport_rect: Rect,
    content_height: f32,
    scroll_y: f32,
    style: ScrollAreaStyle,
    visibility: ScrollBarVisibility,
    cursor_pos: Option<Point>,
    is_dragging: bool,
    background: Option<Color>,
}

impl<'a> ScrollAreaBuilder<'a> {
    /// Creates a new scroll area builder with the given viewport rectangle and total content height.
    #[must_use]
    pub fn new(viewport_rect: Rect, content_height: f32) -> Self {
        Self {
            name: "ScrollArea",
            viewport_rect,
            content_height: content_height.max(0.0),
            scroll_y: 0.0,
            style: ScrollAreaStyle::default(),
            visibility: ScrollBarVisibility::Auto,
            cursor_pos: None,
            is_dragging: false,
            background: None,
        }
    }

    /// Sets the debug and telemetry node name for the container.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    /// Sets the current vertical scroll offset in physical pixels.
    #[must_use]
    pub fn scroll_y(mut self, scroll_y: f32) -> Self {
        self.scroll_y = scroll_y.max(0.0);
        self
    }

    /// Overrides the visual styling configuration for scrollbars.
    #[must_use]
    pub fn style(mut self, style: ScrollAreaStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the visibility policy for scrollbars.
    #[must_use]
    pub fn visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets the mouse cursor position for thumb hover evaluation.
    #[must_use]
    pub fn cursor_pos(mut self, cursor_pos: Option<Point>) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Sets whether the scrollbar thumb is actively being dragged by the mouse.
    #[must_use]
    pub fn is_dragging(mut self, is_dragging: bool) -> Self {
        self.is_dragging = is_dragging;
        self
    }

    /// Sets an explicit background color on the viewport container node.
    ///
    /// When specified, applies the color to the viewport container style, enabling
    /// opaque or semi-transparent background rendering under scrolled content.
    #[must_use]
    pub fn background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    /// Builds the clipped viewport container and conditional scrollbar into the UI tree.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> ScrollAreaFrame {
        let max_scroll_y = (self.content_height - self.viewport_rect.height).max(0.0);
        let safe_scroll_y = self.scroll_y.clamp(0.0, max_scroll_y);

        // 1. Create viewport container with hardware child clipping
        let container_id = tree.create_node();
        if let Some(node) = tree.get_mut(container_id) {
            node.set_name(self.name);
            node.computed_rect = self.viewport_rect;
            node.role = WidgetRole::Default;
            let mut style = Style::new().clip_children(true);
            if let Some(bg) = self.background {
                style = style.background(bg);
            }
            node.style = style;
        }
        let _ = tree.add_child(parent_id, container_id);

        // 2. Conditionally compute and build scrollbar indicators
        let scrollbar_geom = match self.visibility {
            ScrollBarVisibility::Hidden => None,
            ScrollBarVisibility::Auto | ScrollBarVisibility::Always => {
                ScrollBarGeometry::compute_vertical(
                    self.viewport_rect,
                    self.content_height,
                    safe_scroll_y,
                    &self.style,
                )
            }
        };

        if let Some(geom) = scrollbar_geom {
            // Track background node
            let track_id = tree.create_node();
            if let Some(node) = tree.get_mut(track_id) {
                node.set_name(format!("{}_Track", self.name));
                node.computed_rect = geom.track_rect;
                node.style = Style::new()
                    .background(self.style.track_bg)
                    .border_radius(self.style.track_border_radius);
            }
            let _ = tree.add_child(parent_id, track_id);

            // Draggable thumb node
            let is_thumb_hovered = self
                .cursor_pos
                .is_some_and(|pos| geom.thumb_rect.contains_point(pos));

            let thumb_bg = if self.is_dragging {
                self.style.thumb_bg_drag
            } else if is_thumb_hovered {
                self.style.thumb_bg_hover
            } else {
                self.style.thumb_bg_idle
            };

            let thumb_id = tree.create_node();
            if let Some(node) = tree.get_mut(thumb_id) {
                node.set_name(format!("{}_Thumb", self.name));
                node.computed_rect = geom.thumb_rect;
                node.style = Style::new()
                    .background(thumb_bg)
                    .border_radius(self.style.thumb_border_radius);
            }
            let _ = tree.add_child(parent_id, thumb_id);
        }

        ScrollAreaFrame {
            container_id,
            viewport_rect: self.viewport_rect,
            content_height: self.content_height,
            max_scroll_y,
            scrollbar: scrollbar_geom,
        }
    }
}

/// High-level retained scroll synchronization routines.
///
/// Enables in-place updates to container scroll offsets, local subtree flow recalculation,
/// and scrollbar thumb geometry translation for existing [`UiTree`] nodes without
/// destroying, clearing, or rebuilding the retained widget hierarchy.
pub struct ScrollArea;

impl ScrollArea {
    /// Updates the vertical scroll offset on the container node in-place and recalculates its subtree layout.
    ///
    /// Mutates `style.scroll_offset_y` via [`UiTree::set_scroll_offset_y`] and triggers
    /// a local [`crate::declarative::layout_subtree`] pass without affecting sibling nodes.
    ///
    /// Returns `true` if the container was found and its layout subtree was refreshed.
    pub fn update_container_scroll_in_place(
        tree: &mut UiTree,
        container_id: WidgetId,
        scroll_y: f32,
    ) -> bool {
        if !tree.set_scroll_offset_y(container_id, scroll_y) {
            return false;
        }
        let bounds = tree
            .get(container_id)
            .map_or(Rect::ZERO, |n| n.computed_rect);
        crate::declarative::layout_subtree(tree, container_id, bounds);
        true
    }

    /// Updates the vertical scroll offset on a container node identified by semantic tag in-place.
    ///
    /// Resolves the node via [`UiTree::find_node_by_tag`] and runs in-place subtree layout.
    ///
    /// Returns `true` if the container node matching `tag` was found and refreshed.
    pub fn update_container_scroll_by_tag(
        tree: &mut UiTree,
        container_tag: u64,
        scroll_y: f32,
    ) -> bool {
        if let Some(cid) = tree.find_node_by_tag(container_tag) {
            Self::update_container_scroll_in_place(tree, cid, scroll_y)
        } else {
            false
        }
    }

    /// Synchronizes a scrollbar thumb node's Y coordinate in-place based on its track node geometry.
    ///
    /// Computes the proportional vertical thumb offset within the track:
    /// `thumb_y = track_y + (scroll_y / max_scroll_y).clamp(0.0, 1.0) * (track_h - thumb_h)`
    /// and mutates the thumb's computed rectangle Y coordinate via [`UiTree::set_computed_rect_y`].
    ///
    /// Returns `true` if both track and thumb nodes were found and the thumb was updated.
    pub fn sync_thumb_in_place(
        tree: &mut UiTree,
        track_id: WidgetId,
        thumb_id: WidgetId,
        scroll_y: f32,
        max_scroll_y: f32,
    ) -> bool {
        let (track_rect, thumb_h) = {
            let Some(track_node) = tree.get(track_id) else {
                return false;
            };
            let Some(thumb_node) = tree.get(thumb_id) else {
                return false;
            };
            (track_node.computed_rect, thumb_node.computed_rect.height)
        };

        let thumb_y = ScrollBarGeometry::compute_thumb_y(
            track_rect.y,
            track_rect.height,
            thumb_h,
            scroll_y,
            max_scroll_y,
        );
        tree.set_computed_rect_y(thumb_id, thumb_y)
    }

    /// Synchronizes a scrollbar thumb node's position in-place using semantic tags for track and thumb.
    ///
    /// Returns `true` if both nodes were resolved by tag and the thumb was updated.
    pub fn sync_thumb_by_tags_in_place(
        tree: &mut UiTree,
        track_tag: u64,
        thumb_tag: u64,
        scroll_y: f32,
        max_scroll_y: f32,
    ) -> bool {
        let Some(track_id) = tree.find_node_by_tag(track_tag) else {
            return false;
        };
        let Some(thumb_id) = tree.find_node_by_tag(thumb_tag) else {
            return false;
        };
        Self::sync_thumb_in_place(tree, track_id, thumb_id, scroll_y, max_scroll_y)
    }

    /// High-level single-call retained scroll synchronizer.
    ///
    /// In a single pass, updates the container viewport's scroll offset and re-layouts its subtree,
    /// and optionally updates the scrollbar thumb geometry if track and thumb tags are supplied.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the retained [`UiTree`].
    /// * `container_tag` - Semantic tag of the scroll container viewport.
    /// * `track_tag` - Optional semantic tag of the scrollbar track.
    /// * `thumb_tag` - Optional semantic tag of the draggable scrollbar thumb.
    /// * `scroll_y` - Current vertical scroll offset in physical pixels.
    /// * `max_scroll_y` - Maximum scroll limit in physical pixels.
    ///
    /// Returns `true` if any nodes were found and updated.
    pub fn sync_scroll_in_place(
        tree: &mut UiTree,
        container_tag: u64,
        track_tag: Option<u64>,
        thumb_tag: Option<u64>,
        scroll_y: f32,
        max_scroll_y: f32,
    ) -> bool {
        let container_updated = Self::update_container_scroll_by_tag(tree, container_tag, scroll_y);
        let thumb_updated = match (track_tag, thumb_tag) {
            (Some(trk), Some(thm)) => {
                Self::sync_thumb_by_tags_in_place(tree, trk, thm, scroll_y, max_scroll_y)
            }
            _ => false,
        };
        container_updated || thumb_updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_area_no_scrollbar_when_content_fits() {
        let mut tree = UiTree::new();
        let parent = tree.create_node();
        let vp = Rect::new(0.0, 0.0, 300.0, 400.0);

        let frame = ScrollAreaBuilder::new(vp, 250.0).build(&mut tree, parent);

        assert_eq!(frame.max_scroll_y, 0.0);
        assert!(frame.scrollbar.is_none());
        assert!(tree.get(frame.container_id).is_some());
        assert!(tree.get(frame.container_id).unwrap().style.clip_children);
    }

    #[test]
    fn test_scroll_area_with_scrollbar_when_overflows() {
        let mut tree = UiTree::new();
        let parent = tree.create_node();
        let vp = Rect::new(10.0, 20.0, 300.0, 400.0);

        let frame = ScrollAreaBuilder::new(vp, 1000.0)
            .scroll_y(200.0)
            .is_dragging(true)
            .build(&mut tree, parent);

        assert_eq!(frame.max_scroll_y, 600.0);
        assert!(frame.scrollbar.is_some());

        // Test scroll clamp
        assert_eq!(frame.clamp_scroll_y(-50.0), 0.0);
        assert_eq!(frame.clamp_scroll_y(800.0), 600.0);
        assert_eq!(frame.clamp_scroll_y(300.0), 300.0);

        // Test wheel step
        let stepped = frame.scroll_step_wheel(100.0, 1.0, 24.0);
        assert_eq!(stepped, 76.0);
    }

    #[test]
    fn test_scroll_area_retained_in_place_sync() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root must exist");

        let container_tag = 0xAA01;
        let track_tag = 0xAA02;
        let thumb_tag = 0xAA03;

        // Construct container with children via UiScope
        let (_cid, ch1) = {
            let mut scope = crate::declarative::UiScope::new(&mut tree, root);
            let mut child_id = None;
            let c = scope.container_tagged(
                "TestScrollContainer",
                Style::new().width(200.0).height(200.0).flex_col(),
                iris_core::WidgetRole::Default,
                container_tag,
                |s| {
                    child_id = Some(
                        s.empty_box_passive_named("Child1", Style::new().width(200.0).height(60.0)),
                    );
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 200.0, 200.0));
            (c, child_id.unwrap())
        };

        // Construct track and thumb
        let trk = tree.create_node();
        tree.set_tag(trk, track_tag);
        tree.set_computed_rect(trk, Rect::new(190.0, 0.0, 10.0, 200.0));
        let _ = tree.add_child(root, trk);

        let thm = tree.create_node();
        tree.set_tag(thm, thumb_tag);
        tree.set_computed_rect(thm, Rect::new(190.0, 0.0, 10.0, 40.0));
        let _ = tree.add_child(root, thm);

        // Child should start at y = 0
        assert_eq!(tree.get(ch1).unwrap().computed_rect.y, 0.0);
        assert_eq!(tree.get(thm).unwrap().computed_rect.y, 0.0);

        // Sync scroll to 50.0 (max_scroll = 100.0)
        let updated = ScrollArea::sync_scroll_in_place(
            &mut tree,
            container_tag,
            Some(track_tag),
            Some(thumb_tag),
            50.0,
            100.0,
        );
        assert!(updated);

        // Child should be shifted upwards by 50px
        assert_eq!(tree.get(ch1).unwrap().computed_rect.y, -50.0);
        // Thumb travel = 200 - 40 = 160. Halfway = 80.0
        assert_eq!(tree.get(thm).unwrap().computed_rect.y, 80.0);
    }
}