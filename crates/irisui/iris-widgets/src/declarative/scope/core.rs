// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Core Lifecycle & Subtree Layout
//!
//! Provides the core [`UiScope`] struct definition, lifecycle initialization,
//! interaction query evaluation, and recursive layout completion routines.
//!

use super::undeclarative::UndeclarativeScope;
use crate::declarative::layout::layout_subtree;
use iris_core::{
    InteractionEvent, Point, Rect, ResponseFlags, Size, UiTree, WidgetId, WidgetResponse,
};

/// Declarative UI hierarchical builder scope.
///
/// Encapsulates a reference to the generational [`UiTree`] arena and the current
/// parent container [`WidgetId`]. All widgets created within this scope are automatically
/// appended as children to the active parent with appropriate flexbox layout properties.
pub struct UiScope<'a> {
    pub(crate) tree: &'a mut UiTree,
    pub(crate) parent: WidgetId,
    pub(crate) events: &'a [(WidgetId, InteractionEvent)],
    pub(crate) tagged_events: &'a [(u64, InteractionEvent)],
    pub(crate) hovered_id: Option<WidgetId>,
    pub(crate) hovered_tag: Option<u64>,
    pub(crate) seed: u64,
    pub(crate) active_text_input: Option<(u64, &'a str, bool)>,
}

impl<'a> UiScope<'a> {
    /// Creates a new declarative scope attached to the given parent node with empty interactions.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node arena.
    /// * `parent` - Target parent widget node receiving emitted children.
    pub fn new(tree: &'a mut UiTree, parent: WidgetId) -> Self {
        Self {
            tree,
            parent,
            events: &[],
            tagged_events: &[],
            hovered_id: None,
            hovered_tag: None,
            seed: 0,
            active_text_input: None,
        }
    }

    /// Creates a new declarative scope configured with live frame interaction events and hover state.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node arena.
    /// * `parent` - Target parent widget node receiving emitted children.
    /// * `events` - Frame interaction events mapped by [`WidgetId`].
    /// * `hovered_id` - Currently hovered node identifier resolved in the previous frame.
    pub fn with_interactions(
        tree: &'a mut UiTree,
        parent: WidgetId,
        events: &'a [(WidgetId, InteractionEvent)],
        hovered_id: Option<WidgetId>,
    ) -> Self {
        Self {
            tree,
            parent,
            events,
            tagged_events: &[],
            hovered_id,
            hovered_tag: None,
            seed: 0,
            active_text_input: None,
        }
    }

    /// Creates a new declarative scope configured with persistent tagged interaction events and hover state.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node arena.
    /// * `parent` - Target parent widget node receiving emitted children.
    /// * `events` - Frame interaction events mapped by persistent 64-bit semantic tags.
    /// * `hovered_tag` - Currently hovered semantic tag resolved in the previous frame.
    pub fn with_tagged_interactions(
        tree: &'a mut UiTree,
        parent: WidgetId,
        events: &'a [(u64, InteractionEvent)],
        hovered_tag: Option<u64>,
    ) -> Self {
        Self {
            tree,
            parent,
            events: &[],
            tagged_events: events,
            hovered_id: None,
            hovered_tag,
            seed: 0,
            active_text_input: None,
        }
    }

    /// Attaches the active text input session containing focused widget tag, current buffer text, and selection state.
    ///
    /// Automatically passed down to child scopes, enabling all inline numeric text boxes to render active
    /// cursor and selection highlights without manual per-widget boilerplate.
    pub fn with_active_text_input(mut self, active_input: Option<(u64, &'a str, bool)>) -> Self {
        self.active_text_input = active_input;
        self
    }

    /// Creates a child scope for a nested widget container, preserving active events, tags, and text input state.
    ///
    /// Enables sub-systems and custom container builders to emit child nodes into an existing
    /// container while maintaining consistent event mapping, focus states, and typography context.
    #[inline]
    pub fn child_scope(&mut self, parent: WidgetId) -> UiScope<'_> {
        UiScope {
            tree: self.tree,
            parent,
            events: self.events,
            tagged_events: self.tagged_events,
            hovered_id: self.hovered_id,
            hovered_tag: self.hovered_tag,
            seed: self.seed,
            active_text_input: self.active_text_input,
        }
    }

    /// Adds a configurable widget to this scope using the unified builder pattern.
    ///
    /// Enables declarative expression chaining without combinatorial method proliferation:
    /// ```ignore
    /// scope.add(Button::new("Save").fill(Color::GREEN).tag(SAVE_TAG));
    /// ```
    #[inline]
    pub fn add<W: crate::declarative::widget::Widget>(&mut self, widget: W) -> W::Response {
        widget.ui(self)
    }

    /// Returns the active visual theme tokens configured on the underlying tree.
    #[inline]
    #[must_use]
    pub fn theme(&self) -> iris_core::Theme {
        self.tree.theme()
    }

    /// Creates a child scope for a nested widget container with an explicitly isolated seed.
    ///
    /// Preserves active events, tags, and text input state while establishing a new hierarchical
    /// namespace seed derived via [`combine_seeds`](crate::declarative::types::combine_seeds).
    ///
    /// # Arguments
    /// * `parent` - Target parent widget node receiving emitted children.
    /// * `seed` - Combined 64-bit seed used for deterministic child tag hashing.
    #[inline]
    pub fn child_scope_seeded(&mut self, parent: WidgetId, seed: u64) -> UiScope<'_> {
        UiScope {
            tree: self.tree,
            parent,
            events: self.events,
            tagged_events: self.tagged_events,
            hovered_id: self.hovered_id,
            hovered_tag: self.hovered_tag,
            seed,
            active_text_input: self.active_text_input,
        }
    }

    /// Returns the active scope seed used to isolate tag hashing.
    #[inline]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Computes a deterministic 64-bit tag for the given label within the current scope hierarchy.
    ///
    /// Combines [`split_label_id`] and [`hash_label_with_seed`] using the scope's active seed.
    /// If the label contains the `##` separator (e.g. `"Offset##collider"`), the entire string
    /// is used for hashing while preserving the visual prefix for UI text rendering.
    #[inline]
    pub fn tag_for(&self, label: &str) -> u64 {
        let (_visible, tag_source) = crate::declarative::types::split_label_id(label);
        crate::declarative::types::hash_label_with_seed(self.seed, tag_source)
    }

    /// Executes a closure within a nested sub-scope seeded with the given identifier.
    ///
    /// Any widgets created within this sub-scope inherit a combined seed ([`combine_seeds`]),
    /// preventing tag collisions between identical labels across different entities, inspector cards,
    /// or repeated list items.
    ///
    /// # Arguments
    /// * `id` - Scope discriminator seed implementing [`ScopeId`] (e.g. Entity ID, component name `&str`, or loop index `usize`).
    /// * `f` - Closure receiving the seeded sub-scope.
    pub fn with_id<I: crate::declarative::types::ScopeId, R, F: FnOnce(&mut UiScope<'_>) -> R>(
        &mut self,
        id: I,
        f: F,
    ) -> R {
        let child_seed = crate::declarative::types::combine_seeds(self.seed, id.into_seed());
        let mut sub_scope = UiScope {
            tree: self.tree,
            parent: self.parent,
            events: self.events,
            tagged_events: self.tagged_events,
            hovered_id: self.hovered_id,
            hovered_tag: self.hovered_tag,
            seed: child_seed,
            active_text_input: self.active_text_input,
        };
        f(&mut sub_scope)
    }

    /// Returns the target [`WidgetId`] currently acting as the parent for emitted widgets.
    #[inline]
    pub fn parent(&self) -> WidgetId {
        self.parent
    }

    /// Returns an immutable reference to the underlying [`UiTree`] arena.
    #[inline]
    pub fn tree(&self) -> &UiTree {
        self.tree
    }

    /// Returns a mutable reference to the underlying [`UiTree`] arena.
    #[inline]
    pub fn tree_mut(&mut self) -> &mut UiTree {
        self.tree
    }

    /// Measures the horizontal layout advance width of a string in physical pixels at the given font size.
    ///
    /// Dynamically evaluates font glyph advances via the underlying typography engine.
    ///
    /// # Arguments
    /// * `text` - Text string to measure.
    /// * `font_size` - Font size in physical pixels.
    ///
    /// # Returns
    /// Total horizontal advance width in pixels.
    #[inline]
    pub fn measure_text_advance(&self, text: &str, font_size: f32) -> f32 {
        iris_text::measure_text_advance(text, font_size)
    }

    /// Configures the default font family name for thread-local dynamic text layout measurements.
    ///
    /// # Arguments
    /// * `family` - Target font family name (e.g. `"Inter"`, `"Roboto"`, `"sans-serif"`).
    #[inline]
    pub fn set_default_font_family(&self, family: &str) {
        iris_text::set_default_text_family(family);
    }

    /// Returns the computed bounding rectangle of an emitted widget after layout resolution.
    #[inline]
    pub fn computed_rect(&self, id: WidgetId) -> Rect {
        self.tree
            .get(id)
            .map(|node| node.computed_rect)
            .unwrap_or(Rect::ZERO)
    }

    /// Checks the interaction state of an emitted widget against the active event stream.
    ///
    /// Resolves interaction states both against autonomous [`iris_core::dispatch::EventState`]
    /// tracked directly on [`UiTree`] and against explicitly supplied frame interaction records.
    ///
    /// Returns `(clicked, hovered, drag_delta)`.
    pub fn check_interaction(&self, id: WidgetId) -> (bool, bool, Option<Point>) {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);
        let mut clicked = false;
        let mut drag_delta = None;

        // 1. Autonomous dispatch event integration from UiTree::event_state
        if self.tree.event_state.frame_clicked_id == Some(id)
            || (tag != 0 && self.tree.event_state.frame_clicked_tag == Some(tag))
            || self
                .tree
                .event_state
                .frame_clicked_id
                .is_some_and(|cid| self.tree.is_direct_interaction_ancestor(cid, id))
        {
            clicked = true;
        }

        if let Some(delta) = self.tree.event_state.frame_drag_delta {
            let captured_matches = self.tree.event_state.pointer_capture == Some(id)
                || (tag != 0
                    && self
                        .tree
                        .event_state
                        .pointer_capture
                        .and_then(|cid| self.tree.get(cid))
                        .is_some_and(|n| n.tag == tag));
            if captured_matches {
                drag_delta = Some(delta);
            }
        }

        // 2. Compatibility check for explicitly supplied frame interaction events
        for (ev_id, ev) in self.events {
            let matches_id = *ev_id == id;
            let matches_tag = tag != 0 && self.tree.get(*ev_id).is_some_and(|n| n.tag == tag);
            if matches_id || matches_tag {
                match ev {
                    InteractionEvent::Click { .. } => clicked = true,
                    InteractionEvent::Drag { delta } => drag_delta = Some(*delta),
                    _ => {}
                }
            }
        }
        for (ev_tag, ev) in self.tagged_events {
            if tag != 0 && *ev_tag == tag {
                match ev {
                    InteractionEvent::Click { .. } => clicked = true,
                    InteractionEvent::Drag { delta } => drag_delta = Some(*delta),
                    _ => {}
                }
            }
        }

        // 3. Resolve hovered state combining local scope flags and autonomous tree event_state
        let hovered = self.hovered_id == Some(id)
            || (tag != 0 && self.hovered_tag == Some(tag))
            || (tag != 0
                && self
                    .hovered_id
                    .and_then(|hid| self.tree.get(hid))
                    .is_some_and(|n| n.tag == tag))
            || self.tree.event_state.hovered == Some(id)
            || (tag != 0
                && self
                    .tree
                    .event_state
                    .hovered
                    .and_then(|hid| self.tree.get(hid))
                    .is_some_and(|n| n.tag == tag))
            || self
                .tree
                .event_state
                .hovered
                .is_some_and(|hid| self.tree.is_descendant_of(hid, id));

        (clicked, hovered, drag_delta)
    }

    /// Returns a [`WidgetResponse`] describing the interaction state of the given widget node.
    ///
    /// Evaluates clicks, hovers, double-clicks, and drag interactions against [`UiTree::event_state`].
    pub fn response(&self, id: WidgetId) -> WidgetResponse {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);
        let (clicked, hovered, drag_delta) = self.check_interaction(id);
        let double_clicked = self.tree.event_state.frame_double_clicked_id == Some(id)
            || (tag != 0 && self.tree.event_state.frame_double_clicked_tag == Some(tag))
            || self
                .tree
                .event_state
                .frame_double_clicked_id
                .is_some_and(|cid| self.tree.is_direct_interaction_ancestor(cid, id));
        let secondary_clicked = self.tree.event_state.frame_secondary_clicked_id == Some(id)
            || (tag != 0 && self.tree.event_state.frame_secondary_clicked_tag == Some(tag))
            || self
                .tree
                .event_state
                .frame_secondary_clicked_id
                .is_some_and(|cid| self.tree.is_direct_interaction_ancestor(cid, id));
        let is_dragged = drag_delta.is_some()
            || self.tree.event_state.pointer_capture == Some(id)
            || (tag != 0
                && self
                    .tree
                    .event_state
                    .pointer_capture
                    .and_then(|cid| self.tree.get(cid))
                    .is_some_and(|n| n.tag == tag));

        let mut flags = ResponseFlags::empty();
        if clicked {
            flags = flags.union(ResponseFlags::CLICKED);
        }
        if hovered {
            flags = flags.union(ResponseFlags::HOVERED);
        }
        if double_clicked {
            flags = flags.union(ResponseFlags::DOUBLE_CLICKED);
        }
        if secondary_clicked {
            flags = flags.union(ResponseFlags::SECONDARY_CLICKED);
        }
        if is_dragged {
            flags = flags.union(ResponseFlags::DRAGGED);
        }

        let pointer_pos = if clicked || double_clicked || secondary_clicked {
            self.tree
                .event_state
                .frame_clicked_pos
                .or(self.tree.event_state.last_pointer_pos)
        } else if hovered {
            self.tree.event_state.last_pointer_pos
        } else {
            None
        };
        let rect = self.tree.get(id).map_or(Rect::ZERO, |n| n.computed_rect);
        WidgetResponse::from_flags(id, rect, flags).with_pointer_pos(pointer_pos)
    }

    /// Checks whether the given widget node or its semantic tag is actively in a pressed state.
    ///
    /// Evaluates against [`iris_core::dispatch::EventState`] where an active mouse press gesture
    /// is held down on this specific widget.
    #[inline]
    pub fn is_pressed(&self, id: WidgetId) -> bool {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);
        self.tree.event_state.pressed == Some(id)
            || (tag != 0
                && self
                    .tree
                    .event_state
                    .pressed
                    .and_then(|pid| self.tree.get(pid))
                    .is_some_and(|n| n.tag == tag))
    }

    /// Checks whether the specified widget node or semantic tag currently holds keyboard focus.
    ///
    /// Inspects both direct node ID equality and 64-bit semantic tag identity against [`UiTree::event_state`].
    pub fn is_focused(&self, id: WidgetId) -> bool {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);
        self.tree.event_state.focused == Some(id)
            || (tag != 0
                && self
                    .tree
                    .event_state
                    .focused
                    .and_then(|fid| self.tree.get(fid))
                    .is_some_and(|n| n.tag == tag))
            || (tag != 0 && self.tree.event_state.focused_tag == Some(tag))
    }

    /// Checks if a committed text input interaction occurred for the given widget node or semantic tag.
    ///
    /// Resolves against autonomous event dispatch states in [`UiTree::event_state`]
    /// as well as tagged event records in the active frame stream.
    pub fn check_interaction_text(&self, id: WidgetId) -> Option<&str> {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);

        // 1. Autonomous dispatch keyboard integration from UiTree::event_state
        let matches_focus = self.tree.event_state.focused == Some(id)
            || (tag != 0 && self.tree.event_state.focused_tag == Some(tag))
            || (tag != 0
                && self
                    .tree
                    .event_state
                    .focused
                    .and_then(|fid| self.tree.get(fid))
                    .is_some_and(|n| n.tag == tag));
        if matches_focus && let Some(ref text) = self.tree.event_state.frame_text_input {
            return Some(text.as_str());
        }

        // 2. Compatibility check for explicitly supplied frame interaction events
        for (ev_id, ev) in self.events {
            let matches_id = *ev_id == id;
            let matches_tag = tag != 0 && self.tree.get(*ev_id).is_some_and(|n| n.tag == tag);
            if (matches_id || matches_tag)
                && let InteractionEvent::TextInput { text } = ev
            {
                return Some(text.as_str());
            }
        }
        for (ev_tag, ev) in self.tagged_events {
            if tag != 0
                && *ev_tag == tag
                && let InteractionEvent::TextInput { text } = ev
            {
                return Some(text.as_str());
            }
        }
        None
    }

    /// Returns the semantic tag currently hovered in the active interaction session.
    #[inline]
    pub fn hovered_tag(&self) -> Option<u64> {
        self.hovered_tag
    }

    /// Checks if a persistent 64-bit semantic tag is currently hovered in the active interaction session.
    ///
    /// Evaluates direct match with `hovered_tag` as well as node tag on `hovered_id`.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic identifier to inspect.
    pub fn is_tag_hovered(&self, tag: u64) -> bool {
        if tag == 0 {
            return false;
        }
        self.hovered_tag == Some(tag)
            || self
                .hovered_id
                .and_then(|hid| self.tree.get(hid))
                .is_some_and(|n| n.tag == tag)
    }

    /// Calculates and applies recursive flow layout for this scope's subtree across `bounds`.
    /// Automatically reconciles visual hover styling on interactive descendants if a pointer
    /// position is registered in the UI tree, eliminating manual pointer coordinate plumbing.
    pub fn finish_layout(&mut self, bounds: Rect) {
        layout_subtree(self.tree, self.parent, bounds);
        if let Some(cursor) = self.tree.cursor_position() {
            super::hover::update_hover_styles_recursive(self.tree, self.parent, cursor);
        }
    }

    /// Executes a declarative UI builder closure within a scoped hierarchy on `parent`,
    /// automatically executing recursive layout finalization across `bounds` upon closure exit.
    ///
    /// Eliminates boilerplate manual calls to `finish_layout(bounds)` across panels and viewports.
    ///
    /// # Arguments
    /// * `tree` - Mutable reference to the UI node hierarchy.
    /// * `parent` - Target parent widget under which nodes are constructed.
    /// * `bounds` - Bounding layout rectangle constraining the subtree.
    /// * `f` - Declarative builder closure.
    pub fn scoped<F, R>(tree: &mut UiTree, parent: WidgetId, bounds: Rect, f: F) -> R
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let mut scope = UiScope::new(tree, parent);
        let result = f(&mut scope);
        layout_subtree(tree, parent, bounds);
        if let Some(cursor) = tree.cursor_position() {
            super::hover::update_hover_styles_recursive(tree, parent, cursor);
        }
        result
    }

    /// Sets the vertical scroll offset in pixels on the active parent container node.
    #[inline]
    pub fn set_scroll_offset_y(&mut self, offset_y: f32) {
        if let Some(node) = self.tree.get_mut(self.parent) {
            node.style.scroll_offset_y = offset_y;
        }
    }

    /// Sets the scrollable content extent size in physical pixels on the specified container node.
    ///
    /// Configures virtual scrolling boundaries without leaking direct [`iris_core::UiNode`] mutable access.
    ///
    /// # Arguments
    /// * `widget_id` - Identifier of the container node whose content size is configured.
    /// * `size` - Content extent size in physical pixels.
    #[inline]
    pub fn set_content_size(&mut self, widget_id: WidgetId, size: Size) {
        if let Some(node) = self.tree.get_mut(widget_id) {
            node.content_size = size;
        }
    }

    /// Configures the active parent container's debug name, layout style, and interactivity.
    ///
    /// Provides declarative setup for root nodes and container scopes without
    /// raw `UiNode` imperative manipulation in the consumer application layer.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the container node.
    /// * `style` - Flexbox layout constraints and visual properties applied to the container.
    /// * `interactive` - Whether the container node responds to pointer interactions.
    #[inline]
    pub fn configure_container(
        &mut self,
        name: impl Into<String>,
        style: iris_core::Style,
        interactive: bool,
    ) {
        if let Some(node) = self.tree.get_mut(self.parent) {
            node.set_name(name);
            node.interactive = interactive;
            node.set_style(style);
        }
    }

    /// Returns the available bounding layout rectangle of the active parent container.
    ///
    /// Evaluates `computed_rect`, `style` width/height constraints, or cached `viewport_sizes`.
    pub fn available_rect(&self) -> Rect {
        if let Some(node) = self.tree.get(self.parent) {
            let w = if node.computed_rect.width > 0.0 {
                node.computed_rect.width
            } else if let Some(sw) = node.style.width {
                sw
            } else if node.tag != 0 {
                self.tree
                    .viewport_size(node.tag)
                    .map(|s| s.width)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let h = if node.computed_rect.height > 0.0 {
                node.computed_rect.height
            } else if let Some(sh) = node.style.height {
                sh
            } else if node.tag != 0 {
                self.tree
                    .viewport_size(node.tag)
                    .map(|s| s.height)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            if w > 0.0 && h > 0.0 {
                return Rect::new(0.0, 0.0, w, h);
            }
        }
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    /// Enters an explicit imperative escape hatch scope for raw tree mutations and identifier extraction.
    ///
    /// This method serves as an intentional architectural checkpoint. Code executing within this
    /// closure explicitly opts into imperative UI handling and raw [`WidgetId`] manipulation via
    /// [`UndeclarativeScope`].
    ///
    /// # Arguments
    /// * `f` - Closure executing imperative operations on the [`UndeclarativeScope`].
    ///
    /// # Returns
    /// The arbitrary result `R` returned by the closure.
    pub fn undeclarative<R>(&mut self, f: impl FnOnce(&mut UndeclarativeScope<'a, '_>) -> R) -> R {
        let mut raw = UndeclarativeScope::new(self);
        f(&mut raw)
    }
}