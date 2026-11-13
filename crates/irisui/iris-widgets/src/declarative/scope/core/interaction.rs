// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Event & Interaction Evaluation
//!
//! Provides interaction state evaluation, gesture recognition, hover/pressed/focused queries,
//! and [`WidgetResponse`] generation across the active [`UiScope`].

use super::context::UiScope;
use iris_core::{InteractionEvent, Point, Rect, ResponseFlags, WidgetId, WidgetResponse};

impl<'a> UiScope<'a> {
    /// Checks the interaction state of an emitted widget against the active event stream.
    ///
    /// Resolves interaction states both against autonomous [`iris_core::dispatch::EventState`]
    /// tracked directly on [`iris_core::UiTree`] and against explicitly supplied frame interaction records.
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
                || (tag != 0 && self.tree.event_state.pointer_capture_tag == Some(tag))
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
    /// Evaluates clicks, hovers, double-clicks, and drag interactions against [`iris_core::UiTree::event_state`].
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
        let is_drag_released = self.tree.event_state.frame_drag_released_id == Some(id)
            || (tag != 0 && self.tree.event_state.frame_drag_released_tag == Some(tag))
            || self
                .tree
                .event_state
                .frame_drag_released_id
                .is_some_and(|cid| self.tree.is_direct_interaction_ancestor(cid, id));
        let is_dragged = drag_delta.is_some()
            || (self.tree.event_state.is_pointer_drag_active
                && (self.tree.event_state.pointer_capture == Some(id)
                    || (tag != 0 && self.tree.event_state.pointer_capture_tag == Some(tag))
                    || (tag != 0
                        && self
                            .tree
                            .event_state
                            .pointer_capture
                            .and_then(|cid| self.tree.get(cid))
                            .is_some_and(|n| n.tag == tag))));

        let rect = self.tree.get(id).map_or(Rect::ZERO, |n| n.computed_rect);

        let is_file_hovered = self
            .tree
            .event_state
            .hovered_file
            .as_ref()
            .is_some_and(|(_, pt)| {
                rect.contains_point(*pt)
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
                        .is_some_and(|hid| self.tree.is_direct_interaction_ancestor(hid, id))
            });

        let is_file_dropped = self
            .tree
            .event_state
            .frame_dropped_files
            .as_ref()
            .is_some_and(|(target_id, target_tag, _, pt)| {
                rect.contains_point(*pt)
                    || *target_id == Some(id)
                    || (tag != 0 && *target_tag == tag)
                    || target_id
                        .is_some_and(|tid| self.tree.is_direct_interaction_ancestor(tid, id))
            });

        let mut flags = ResponseFlags::empty();
        flags.set(ResponseFlags::CLICKED, clicked);
        flags.set(ResponseFlags::HOVERED, hovered);
        flags.set(ResponseFlags::DOUBLE_CLICKED, double_clicked);
        flags.set(ResponseFlags::SECONDARY_CLICKED, secondary_clicked);
        flags.set(ResponseFlags::DRAGGED, is_dragged);
        flags.set(ResponseFlags::DRAG_RELEASED, is_drag_released);
        flags.set(ResponseFlags::PRESSED, self.is_pressed(id));
        flags.set(ResponseFlags::FILE_HOVERED, is_file_hovered);
        flags.set(ResponseFlags::FILE_DROPPED, is_file_dropped);

        let pointer_pos = if is_dragged {
            self.tree
                .event_state
                .last_pointer_pos
                .or(self.tree.event_state.frame_clicked_pos)
        } else if clicked || double_clicked || secondary_clicked || is_drag_released {
            self.tree
                .event_state
                .frame_clicked_pos
                .or(self.tree.event_state.last_pointer_pos)
        } else if hovered {
            self.tree.event_state.last_pointer_pos
        } else {
            None
        };
        WidgetResponse::from_flags(id, rect, flags)
            .with_pointer_pos(pointer_pos)
            .with_drag_delta(drag_delta)
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
            || (tag != 0 && self.tree.event_state.pressed_tag == Some(tag))
    }

    /// Checks whether the specified widget node or semantic tag currently holds keyboard focus.
    ///
    /// Inspects both direct node ID equality and 64-bit semantic tag identity against [`iris_core::UiTree::event_state`].
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
    /// Resolves against autonomous event dispatch states in [`iris_core::UiTree::event_state`]
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
        if matches_focus
            && self.tree.event_state.frame_key_pressed
            && let Some(ref text) = self.tree.event_state.frame_text_input
        {
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
}