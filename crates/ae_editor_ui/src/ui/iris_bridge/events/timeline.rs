// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Window Event Routing
//!
//! Dispatches mouse clicks, playhead dragging via direct hardware hit-testing, and
//! transport controls via semantic tags for the Iris UI Animation Timeline Studio panel.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, IrisOverlayEventResult};
use irisui::prelude::*;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

impl IrisEditorOverlay {
    /// Routes window events to the Animation Timeline Studio panel when active.
    ///
    /// Evaluates clicks and playhead dragging directly via hardware hit-testing
    /// ([`UiTree::hit_test_target`]) and persistent semantic tags without relying
    /// on retained coordinate target descriptors.
    pub(crate) fn handle_timeline_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        let _ = self.timeline.panel_rect?;
        let mut result = IrisOverlayEventResult::default();

        // 1. Mouse Click handling
        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: WinitMouseButton::Left,
            ..
        } = event
        {
            let click_point = self.cursor_pos();

            if let Some(hit) = self.tree.hit_test_target(click_point) {
                // A. Scrubber Track or Playhead Cap hit -> initiate scrubbing drag
                if hit.tag == TIMELINE_TAG_SCRUBBER_TRACK || hit.tag == TIMELINE_TAG_PLAYHEAD_CAP {
                    let track_x = hit.rect.x;
                    let track_w = hit.rect.width.max(1.0);
                    self.timeline.active_scrubber_track = Some((track_x, track_w));
                    self.timeline.is_dragging = true;

                    let scrub_t = super::super::timeline::compute_scrub_timestamp(
                        click_point.x,
                        track_x,
                        track_w,
                        self.timeline.clip_duration,
                    );
                    self.timeline
                        .actions
                        .push(super::super::timeline::TimelineAction::ScrubTo(scrub_t));
                    self.notifier.tag_all();
                    self.chrome.needs_layout_rebuild = true;
                    result.consumed = true;
                    return Some(result);
                }

                // B. Tagged Transport Controls
                self.timeline.pending_interaction_events.push((
                    hit.tag,
                    InteractionEvent::Click {
                        button: MouseButton::Left,
                    },
                ));

                if let Some(action) = super::super::timeline::handle_timeline_click(
                    hit.tag,
                    self.timeline.selected_entity,
                ) {
                    self.timeline.actions.push(action);
                    result.consumed = true;
                    return Some(result);
                }

                if irisui::prelude::is_timeline_tag(hit.tag) {
                    result.consumed = true;
                    return Some(result);
                }
            }
        }

        // 2. Mouse Release handling (terminate playhead dragging)
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: WinitMouseButton::Left,
            ..
        } = event
            && self.timeline.is_dragging
        {
            self.timeline.is_dragging = false;
            self.timeline.active_scrubber_track = None;
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        // 3. Cursor movement while dragging scrubber
        if let WindowEvent::CursorMoved { .. } = event
            && self.timeline.is_dragging
            && let Some((track_x, track_w)) = self.timeline.active_scrubber_track
        {
            let cursor_x = self.cursor_pos().x;
            let scrub_t = super::super::timeline::compute_scrub_timestamp(
                cursor_x,
                track_x,
                track_w,
                self.timeline.clip_duration,
            );
            self.timeline
                .actions
                .push(super::super::timeline::TimelineAction::ScrubTo(scrub_t));
            self.notifier.tag_all();
            self.chrome.needs_layout_rebuild = true;
            result.consumed = true;
            return Some(result);
        }

        None
    }
}