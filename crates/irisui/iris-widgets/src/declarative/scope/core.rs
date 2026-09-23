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
        }
    }

    /// Returns the target [`WidgetId`] currently acting as the parent for emitted widgets.
    #[inline]
    pub fn parent(&self) -> WidgetId {
        self.parent
    }

    /// Returns a mutable reference to the underlying [`UiTree`] arena.
    #[inline]
    pub fn tree_mut(&mut self) -> &mut UiTree {
        self.tree
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