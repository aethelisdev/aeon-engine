// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Core Lifecycle & Context Hierarchy
//!
//! Provides the primary [`UiScope`] struct definition, arena reference encapsulation,
//! lifecycle initialization, hierarchical sub-scope scoping, and seed-based tag isolation.

use crate::declarative::types::{ScopeId, combine_seeds, hash_label_with_seed, split_label_id};
use crate::declarative::widget::Widget;
use crate::declarative::widget::chip::Chip;
use iris_core::{InteractionEvent, Theme, UiTree, WidgetId};

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
        let (_visible, tag_source) = split_label_id(label);
        hash_label_with_seed(self.seed, tag_source)
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
    pub fn with_id<I: ScopeId, R, F: FnOnce(&mut UiScope<'_>) -> R>(&mut self, id: I, f: F) -> R {
        let child_seed = combine_seeds(self.seed, id.into_seed());
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

    /// Returns an immutable reference to the underlying [`UiTree`] arena.
    #[inline]
    pub fn tree(&self) -> &UiTree {
        self.tree
    }

    /// Returns the active parent node identifier of this scope.
    #[inline]
    pub fn parent(&self) -> WidgetId {
        self.parent
    }

    /// Returns a mutable reference to the underlying [`UiTree`] arena.
    #[inline]
    pub fn tree_mut(&mut self) -> &mut UiTree {
        self.tree
    }

    /// Returns the active visual theme tokens configured on the underlying tree.
    #[inline]
    #[must_use]
    pub fn theme(&self) -> Theme {
        self.tree.theme()
    }

    /// Adds a configurable widget to this scope using the unified builder pattern.
    ///
    /// Enables declarative expression chaining without combinatorial method proliferation:
    /// ```ignore
    /// scope.add(Button::new("Save").fill(Color::GREEN).tag(SAVE_TAG));
    /// ```
    #[inline]
    pub fn add<W: Widget>(&mut self, widget: W) -> W::Response {
        widget.ui(self)
    }

    /// Returns a fluent [`Chip`] builder for emitting a categorized filter badge.
    ///
    /// # Arguments
    /// * `label` - Text displayed on the filter badge.
    #[inline]
    #[must_use]
    pub fn chip<'w>(&self, label: &'w str) -> Chip<'w> {
        Chip::new(label)
    }
}