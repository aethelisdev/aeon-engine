// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Z-order layered hit testing, occlusion calculations, and cursor evaluation for the UI tree.

use crate::geometry::{Point, Rect};
use crate::id::WidgetId;
use crate::node::{UiLayer, WidgetCursor, WidgetRole};

use super::arena::UiTree;

/// Comprehensive metadata and interaction properties of a hit-tested widget.
///
/// Returned by [`UiTree::hit_test_target`] to supply the host application with
/// the widget's identifier, effective stacking layer, functional role, user-defined tag,
/// layout boundary rectangle, and optional debug name in a single query.
#[derive(Debug, Clone, PartialEq)]
pub struct HitTargetInfo {
    /// Generational unique identifier of the hit widget.
    pub id: WidgetId,
    /// Effective stacking context layer computed from hierarchy inheritance.
    pub layer: UiLayer,
    /// Semantic functional role assigned to the widget.
    pub role: WidgetRole,
    /// Active hardware mouse cursor shape requested by this widget or its functional role.
    pub cursor: Option<WidgetCursor>,
    /// User-defined numeric tag or action identifier associated with the widget.
    pub tag: u64,
    /// Absolute computed screen-space rectangle of the widget.
    pub rect: Rect,
    /// Optional debug name of the widget node.
    pub name: Option<String>,
}

impl UiTree {
    /// Performs screen-space hit testing to find the deepest interactive node under `point`.
    pub fn hit_test(&self, point: Point) -> Option<WidgetId> {
        let root_id = self.root?;
        self.hit_test_recursive(root_id, point)
    }

    fn hit_test_recursive(&self, current_id: WidgetId, point: Point) -> Option<WidgetId> {
        let node = self.nodes.get(current_id)?;
        if !node.visible || !node.computed_rect.contains_point(point) {
            return None;
        }

        // Iterate children in reverse order (top-most z-order first)
        for &child_id in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(child_id, point) {
                return Some(hit);
            }
        }

        if node.interactive {
            Some(current_id)
        } else {
            None
        }
    }

    /// Returns the effective stacking layer for a given node, taking parent layer inheritance into account.
    pub fn effective_layer(&self, mut current_id: WidgetId) -> UiLayer {
        let mut max_layer = UiLayer::Background;
        while let Some(node) = self.nodes.get(current_id) {
            if node.layer > max_layer {
                max_layer = node.layer;
            }
            match node.parent {
                Some(parent_id) => current_id = parent_id,
                None => break,
            }
        }
        if max_layer == UiLayer::Background {
            UiLayer::Content
        } else {
            max_layer
        }
    }

    /// Checks if any visible modal window currently exists within the tree.
    ///
    /// When true, interaction with underlying background or docked panels is blocked.
    pub fn is_modal_active(&self) -> bool {
        self.nodes
            .values()
            .any(|n| n.visible && n.layer == UiLayer::Modal)
    }

    /// Performs layered screen-space hit testing, prioritizing higher `UiLayer` stacking contexts.
    ///
    /// Traversal priority:
    /// 1. `UiLayer::Tooltip`
    /// 2. `UiLayer::Popup`
    /// 3. `UiLayer::Modal`
    /// 4. `UiLayer::Floating`
    /// 5. `UiLayer::Content`
    /// 6. `UiLayer::Background`
    ///
    /// If an active `UiLayer::Modal` is visible on screen, hits on lower layers
    /// (`Floating`, `Content`, `Background`) are blocked with zero heap allocations.
    pub fn hit_test_layered(&self, point: Point) -> Option<WidgetId> {
        let root_id = self.root?;
        let mut layer_hits = [None; 6];
        let mut has_active_modal = false;

        self.hit_test_layered_recursive(
            root_id,
            point,
            UiLayer::Background,
            None,
            &mut layer_hits,
            &mut has_active_modal,
        );

        // Tooltip (index 5)
        if let Some(hit) = layer_hits[UiLayer::Tooltip.index()] {
            return Some(hit);
        }
        // Popup (index 4)
        if let Some(hit) = layer_hits[UiLayer::Popup.index()] {
            return Some(hit);
        }
        // Modal (index 3)
        if let Some(hit) = layer_hits[UiLayer::Modal.index()] {
            return Some(hit);
        }

        // If an active modal is displayed on screen, lower layers cannot receive events
        if has_active_modal {
            return None;
        }

        // Floating (index 2)
        if let Some(hit) = layer_hits[UiLayer::Floating.index()] {
            return Some(hit);
        }
        // Content (index 1)
        if let Some(hit) = layer_hits[UiLayer::Content.index()] {
            return Some(hit);
        }
        // Background (index 0)
        layer_hits[UiLayer::Background.index()]
    }

    fn hit_test_layered_recursive(
        &self,
        current_id: WidgetId,
        point: Point,
        inherited_layer: UiLayer,
        clip_rect: Option<Rect>,
        layer_hits: &mut [Option<WidgetId>; 6],
        has_active_modal: &mut bool,
    ) {
        let Some(node) = self.nodes.get(current_id) else {
            return;
        };
        if !node.visible {
            return;
        }

        let effective_layer = if node.layer > inherited_layer {
            node.layer
        } else {
            inherited_layer
        };

        if effective_layer == UiLayer::Modal {
            *has_active_modal = true;
        }

        // Elevated overlay layers break out of parent scissor boundaries
        let effective_clip = if effective_layer > inherited_layer {
            None
        } else {
            clip_rect
        };

        let child_clip = if node.style.clip_children {
            match effective_clip {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            effective_clip
        };

        // Traverse children in reverse order (top-most sibling first)
        for &child_id in node.children.iter().rev() {
            self.hit_test_layered_recursive(
                child_id,
                point,
                effective_layer,
                child_clip,
                layer_hits,
                has_active_modal,
            );
        }

        let is_within_clip = effective_clip.is_none_or(|c| c.contains_point(point));
        if node.interactive && is_within_clip && node.computed_rect.contains_point(point) {
            let idx = effective_layer.index();
            if layer_hits[idx].is_none() {
                layer_hits[idx] = Some(current_id);
            }
        }
    }

    /// Returns the highest `UiLayer` present under the given screen-space point.
    pub fn layer_at(&self, point: Point) -> Option<UiLayer> {
        let root_id = self.root?;
        let mut highest: Option<UiLayer> = None;
        self.find_layer_at_recursive(root_id, point, UiLayer::Background, None, &mut highest);
        highest
    }

    fn find_layer_at_recursive(
        &self,
        current_id: WidgetId,
        point: Point,
        inherited_layer: UiLayer,
        clip_rect: Option<Rect>,
        highest: &mut Option<UiLayer>,
    ) {
        let Some(node) = self.nodes.get(current_id) else {
            return;
        };
        if !node.visible {
            return;
        }
        let effective_layer = if node.layer > inherited_layer {
            node.layer
        } else {
            inherited_layer
        };

        let effective_clip = if effective_layer > inherited_layer {
            None
        } else {
            clip_rect
        };

        let child_clip = if node.style.clip_children {
            match effective_clip {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            effective_clip
        };

        let is_within_clip = effective_clip.is_none_or(|c| c.contains_point(point));
        if is_within_clip && node.computed_rect.contains_point(point) {
            *highest = match *highest {
                Some(prev) => Some(prev.max(effective_layer)),
                None => Some(effective_layer),
            };
        }

        for &child_id in &node.children {
            self.find_layer_at_recursive(child_id, point, effective_layer, child_clip, highest);
        }
    }

    /// Checks whether `ancestor` is the direct consuming target for interactions originating at `descendant`.
    ///
    /// An `ancestor` only inherits interaction events from a `descendant` if there are no intermediate
    /// tagged or intrinsically interactive widgets between `descendant` and `ancestor`.
    /// Furthermore, containers serving strictly as scroll viewports do not inherit interactions from their children.
    pub fn is_direct_interaction_ancestor(&self, descendant: WidgetId, ancestor: WidgetId) -> bool {
        if descendant == ancestor {
            return true;
        }

        // If the descendant itself is tagged or intrinsically interactive,
        // it consumed the interaction directly; parent containers must not bubble or steal it.
        if let Some(desc_node) = self.nodes.get(descendant) {
            if desc_node.tag != 0 || desc_node.role.is_interactive() {
                return false;
            }
            // Floating popup layers never bubble interactions to lower-layer containers
            if desc_node.layer == UiLayer::Popup
                && let Some(anc_node) = self.nodes.get(ancestor)
                && anc_node.layer < UiLayer::Popup
            {
                return false;
            }
        }

        // Viewports and passive panel containers never claim clicks originating from child widgets
        if let Some(anc_node) = self.nodes.get(ancestor)
            && anc_node.role == crate::node::WidgetRole::ScrollAreaViewport
        {
            return false;
        }

        let mut curr = self.nodes.get(descendant).and_then(|n| n.parent);
        while let Some(id) = curr {
            if id == ancestor {
                return true;
            }
            if let Some(node) = self.nodes.get(id) {
                // If an intermediate ancestor is tagged or intrinsically interactive, it consumed the event
                if node.tag != 0 || node.role.is_interactive() {
                    return false;
                }
                curr = node.parent;
            } else {
                break;
            }
        }
        false
    }

    /// Walks up the ancestor hierarchy starting from `start_id` to locate the nearest non-zero semantic tag.
    ///
    /// This enables interactive child widgets (such as text labels, vector icons, or inner padding wrappers)
    /// to transparently delegate their interaction events and hit-testing targets to their enclosing tagged container.
    ///
    /// Returns the resolved semantic tag, or `0` if no ancestor defines a non-zero tag.
    pub fn resolve_ancestor_tag(&self, start_id: WidgetId) -> u64 {
        let mut curr = Some(start_id);
        while let Some(id) = curr {
            if let Some(node) = self.nodes.get(id) {
                if node.tag != 0 {
                    return node.tag;
                }
                curr = node.parent;
            } else {
                break;
            }
        }
        0
    }

    /// Performs layered hit-testing and returns detailed metadata for the hit widget.
    ///
    /// Evaluates the highest priority [`UiLayer`] under `point` via [`UiTree::hit_test_layered`],
    /// then extracts the node's properties, role, effective layer, and tag with zero heap allocations for numeric queries.
    /// If the hit leaf node does not define a tag (`tag == 0`), resolves the tag from the nearest enclosing
    /// ancestor via [`UiTree::resolve_ancestor_tag`].
    pub fn hit_test_target(&self, point: Point) -> Option<HitTargetInfo> {
        let hit_id = self.hit_test_layered(point)?;
        let node = self.nodes.get(hit_id)?;
        let layer = self.effective_layer(hit_id);
        let cursor = node.cursor.or_else(|| node.role.default_cursor());
        let effective_tag = if node.tag != 0 {
            node.tag
        } else {
            self.resolve_ancestor_tag(hit_id)
        };
        Some(HitTargetInfo {
            id: hit_id,
            layer,
            role: node.role,
            cursor,
            tag: effective_tag,
            rect: node.computed_rect,
            name: node.name.clone(),
        })
    }

    /// Returns true if the specified screen coordinate intersects an interactive widget,
    /// a tagged component, an active popup/context menu, or an active modal overlay.
    ///
    /// Enables host applications to determine whether the UI captures pointer input
    /// without manual coordinate calculations or widget role filtering.
    #[must_use]
    pub fn wants_pointer_at(&self, point: Point) -> bool {
        if self.event_state.pointer_capture.is_some()
            || self.event_state.active_context_menu.is_some()
        {
            return true;
        }

        let Some(hit_id) = self.hit_test_layered(point) else {
            return false;
        };

        let Some(node) = self.nodes.get(hit_id) else {
            return false;
        };

        let layer = self.effective_layer(hit_id);
        if layer == UiLayer::Modal || layer == UiLayer::Popup || layer == UiLayer::Tooltip {
            return true;
        }

        if node.role.is_interactive() || node.tag != 0 {
            return true;
        }

        self.resolve_ancestor_tag(hit_id) != 0
    }

    /// Evaluates the active hardware mouse cursor shape for the widget currently under `point`.
    ///
    /// Resolves the highest-priority interactive widget through [`UiTree::hit_test_layered`].
    /// If the hit widget has an explicit cursor override ([`WidgetNode::cursor`]), it is returned.
    /// Otherwise, resolves the role's canonical cursor ([`WidgetRole::default_cursor`]).
    /// If neither is defined, inspects the parent hierarchy until a cursor hint is discovered
    /// or defaults to [`WidgetCursor::Default`].
    ///
    /// Respects layer priority (e.g. modal dialogs and dropdown popups occlude underlying splitters and tabs).
    pub fn cursor_at(&self, point: Point) -> WidgetCursor {
        let Some(hit_id) = self.hit_test_layered(point) else {
            return WidgetCursor::Default;
        };

        let mut curr = Some(hit_id);
        while let Some(id) = curr {
            if let Some(node) = self.nodes.get(id) {
                if let Some(c) = node.cursor {
                    return c;
                }
                if let Some(c) = node.role.default_cursor() {
                    return c;
                }
                curr = node.parent;
            } else {
                break;
            }
        }

        WidgetCursor::Default
    }
}