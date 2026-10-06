// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Core Lifecycle & Subtree Layout
//!
//! Provides the core [`UiScope`] struct definition, lifecycle initialization,
//! interaction query evaluation, and recursive layout completion routines.
//!

use crate::declarative::layout::layout_subtree;
use crate::modal::{
    MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_DANGER, MODAL_TAG_SCRIM,
};
use iris_core::{Color, InteractionEvent, Point, Rect, UiTree, WidgetId, WidgetRole};

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

    /// Creates a child scope for a nested widget container, preserving active events and scope seed.
    #[inline]
    pub(crate) fn child_scope(&mut self, parent: WidgetId) -> UiScope<'_> {
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
    /// Matches either directly by allocated [`WidgetId`] or by persistent [`WidgetNode::tag`].
    ///
    /// Returns `(clicked, hovered, drag_delta)`.
    pub fn check_interaction(&self, id: WidgetId) -> (bool, bool, Option<Point>) {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);
        let mut clicked = false;
        let mut drag_delta = None;
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
        let hovered = self.hovered_id == Some(id)
            || (tag != 0 && self.hovered_tag == Some(tag))
            || (tag != 0
                && self
                    .hovered_id
                    .and_then(|hid| self.tree.get(hid))
                    .is_some_and(|n| n.tag == tag));
        (clicked, hovered, drag_delta)
    }

    /// Checks if a committed text input interaction occurred for the given widget node or semantic tag.
    ///
    /// Resolves against both local widget allocations and tagged event records in the active frame stream.
    pub fn check_interaction_text(&self, id: WidgetId) -> Option<&str> {
        let tag = self.tree.get(id).map_or(0, |n| n.tag);
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
    pub fn finish_layout(&mut self, bounds: Rect) {
        layout_subtree(self.tree, self.parent, bounds);
    }

    /// Calculates and applies recursive flow layout across `bounds`, then immediately updates
    /// visual hover styling on interactive descendants based on the real-time `cursor_pos`.
    ///
    /// Ensures 0ms hover latency on newly built declarative modal hierarchies without
    /// waiting for subsequent frame reconciliation cycles.
    ///
    /// # Arguments
    /// * `bounds` - Bounding rectangle constraining the subtree layout.
    /// * `cursor_pos` - Current hardware mouse pointer position in physical screen coordinates.
    pub fn finish_layout_with_hover(&mut self, bounds: Rect, cursor_pos: Point) {
        layout_subtree(self.tree, self.parent, bounds);
        update_hover_styles_recursive(self.tree, self.parent, cursor_pos);
    }

    /// Sets the vertical scroll offset in pixels on the active parent container node.
    #[inline]
    pub fn set_scroll_offset_y(&mut self, offset_y: f32) {
        if let Some(node) = self.tree.get_mut(self.parent) {
            node.style.scroll_offset_y = offset_y;
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
}

/// Recursively updates hover styling for interactive modal buttons and links in a computed subtree.
///
/// Evaluates `computed_rect.contains_point(cursor_pos)` against known semantic tags and roles,
/// applying clean ~10% background brightness lifts without neon glows or shadow halos.
fn update_hover_styles_recursive(tree: &mut UiTree, current: WidgetId, cursor_pos: Point) {
    let (is_hovered, tag, role, interactive, children) = {
        let Some(node) = tree.get(current) else {
            return;
        };
        (
            node.computed_rect.contains_point(cursor_pos),
            node.tag,
            node.role,
            node.interactive,
            node.children.clone(),
        )
    };

    if let Some(node) = tree.get_mut(current) {
        if tag == MODAL_TAG_CLOSE {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.60, 0.64, 0.72, 0.85)
            };
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.85, 0.22, 0.22, 0.28)
                } else {
                    Color::TRANSPARENT
                })
                .border(0.0, Color::TRANSPARENT);
            node.style.box_shadow = None;
        } else if tag == MODAL_TAG_CONFIRM {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.92, 0.96, 1.0, 1.0)
            };
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.18, 0.48, 0.68, 1.0)
                } else {
                    Color::rgba(0.14, 0.40, 0.58, 1.0)
                })
                .border(
                    1.0,
                    if is_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 0.25)
                    } else {
                        Color::rgba(1.0, 1.0, 1.0, 0.15)
                    },
                );
            node.style.box_shadow = None;
        } else if tag == MODAL_TAG_DANGER {
            node.text_color = Color::WHITE;
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.75, 0.20, 0.20, 1.0)
                } else {
                    Color::rgba(0.63, 0.14, 0.14, 1.0)
                })
                .border(
                    1.0,
                    if is_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 0.25)
                    } else {
                        Color::rgba(1.0, 1.0, 1.0, 0.15)
                    },
                );
            node.style.box_shadow = None;
        } else if tag == MODAL_TAG_CANCEL {
            node.text_color = if is_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.75, 0.78, 0.84, 1.0)
            };
            node.style = node
                .style
                .background(if is_hovered {
                    Color::rgba(0.20, 0.22, 0.26, 1.0)
                } else {
                    Color::rgba(0.14, 0.15, 0.18, 1.0)
                })
                .border(
                    1.0,
                    if is_hovered {
                        Color::rgba(1.0, 1.0, 1.0, 0.18)
                    } else {
                        Color::rgba(1.0, 1.0, 1.0, 0.10)
                    },
                );
            node.style.box_shadow = None;
        } else if role == WidgetRole::MenuBarItem {
            if node.text_color != Color::hex("#00e5ff") {
                node.text_color = if is_hovered {
                    Color::WHITE
                } else {
                    Color::hex("#dcdce2")
                };
                node.style = node.style.background(if is_hovered {
                    Color::hex("#222634")
                } else {
                    Color::TRANSPARENT
                });
            }
        } else if role == WidgetRole::DropdownItem && interactive {
            node.style = node.style.background(if is_hovered {
                Color::hex("#222634")
            } else {
                Color::TRANSPARENT
            });
        } else if role == WidgetRole::Button && (2010..=2025).contains(&tag) {
            let is_active = node.style.background_color == Color::rgba(0.06, 0.46, 0.92, 1.0);
            if !is_active {
                let (bg, border) = if is_hovered {
                    (
                        Color::rgba(0.20, 0.23, 0.30, 0.95),
                        Color::rgba(0.35, 0.40, 0.50, 0.90),
                    )
                } else {
                    (
                        Color::rgba(0.12, 0.13, 0.16, 0.92),
                        Color::rgba(0.24, 0.26, 0.32, 0.85),
                    )
                };
                node.style = node.style.background(bg).border(1.0, border);
                for cid in &children {
                    if let Some(child) = tree.get_mut(*cid) {
                        child.set_texture_tint(if is_hovered {
                            Color::rgba(0.95, 0.98, 1.0, 1.0)
                        } else {
                            Color::rgba(0.80, 0.84, 0.90, 0.90)
                        });
                    }
                }
            }
        } else if role == WidgetRole::Button && (tag == 2001 || tag == 2002) {
            let is_open = node.style.background_color == Color::rgba(0.20, 0.23, 0.30, 0.90);
            if !is_open {
                node.style = node.style.background(if is_hovered {
                    Color::rgba(0.20, 0.23, 0.30, 0.90)
                } else {
                    Color::TRANSPARENT
                });
                let tc = if is_hovered {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                } else {
                    Color::rgba(0.85, 0.88, 0.94, 1.0)
                };
                for cid in &children {
                    if let Some(child) = tree.get_mut(*cid) {
                        if child.text.is_some() {
                            child.text_color = tc;
                        } else {
                            child.set_texture_tint(tc);
                        }
                    }
                }
            }
        } else if role == WidgetRole::Button && (2030..=2035).contains(&tag) {
            let is_knob = (2030..=2032).contains(&tag);
            if is_knob {
                let base_color = match tag {
                    2030 => Color::rgba(0.92, 0.22, 0.22, 1.0),
                    2031 => Color::rgba(0.22, 0.82, 0.32, 1.0),
                    2032 => Color::rgba(0.24, 0.52, 0.95, 1.0),
                    _ => Color::WHITE,
                };
                node.style =
                    node.style
                        .background(if is_hovered { Color::WHITE } else { base_color });
                for cid in &children {
                    if let Some(child) = tree.get_mut(*cid) {
                        child.text_color = if is_hovered {
                            Color::BLACK
                        } else {
                            Color::WHITE
                        };
                    }
                }
            } else {
                let base_color = match tag {
                    2033 => Color::rgba(0.92 * 0.45, 0.22 * 0.45, 0.22 * 0.45, 0.75),
                    2034 => Color::rgba(0.22 * 0.45, 0.82 * 0.45, 0.32 * 0.45, 0.75),
                    2035 => Color::rgba(0.24 * 0.45, 0.52 * 0.45, 0.95 * 0.45, 0.75),
                    _ => Color::rgba(0.5, 0.5, 0.5, 0.75),
                };
                node.style =
                    node.style
                        .background(if is_hovered { Color::WHITE } else { base_color });
            }
        } else if role == WidgetRole::Button && tag != 0 && tag != MODAL_TAG_SCRIM {
            node.text_color = if is_hovered {
                Color::rgba(0.25, 0.88, 1.0, 1.0)
            } else {
                Color::rgba(0.0, 0.80, 0.95, 1.0)
            };
        }
    }

    if role == WidgetRole::DropdownItem && interactive {
        for cid in &children {
            if let Some(child) = tree.get_mut(*cid) {
                if child.role == WidgetRole::DropdownLabel || child.role == WidgetRole::DropdownIcon
                {
                    child.text_color = if is_hovered {
                        Color::WHITE
                    } else {
                        Color::hex("#dcdce2")
                    };
                } else if child.role == WidgetRole::DropdownShortcut
                    && child.text.as_deref() != Some("✓")
                {
                    child.text_color = if is_hovered {
                        Color::hex("#b0b0be")
                    } else {
                        Color::hex("#828292")
                    };
                }
            }
        }
    }

    for child_id in children {
        update_hover_styles_recursive(tree, child_id, cursor_pos);
    }
}