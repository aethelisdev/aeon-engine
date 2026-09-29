// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Subsystem (`iris_bridge::timeline`)
//!
//! Provides the 100% GPU SDF hardware-accelerated Animation Timeline Studio panel
//! for Aeon Engine, utilizing decoupled Retained UI trees, responsive transport controls,
//! adaptive time rulers, and interactive scrubbing.
//!

pub mod events;
pub mod lanes;
pub mod panel;
pub mod ruler;
#[cfg(test)]
mod tests;
pub mod transport;
pub mod types;

pub use events::{compute_scrub_timestamp, handle_timeline_click};
pub use lanes::{CHANNEL_LANE_HEIGHT, MAX_VISIBLE_LANES, build_dope_sheet_lanes};
pub use panel::build_timeline_panel;
pub use ruler::{
    RULER_HEIGHT, RULER_TOTAL_HEIGHT, SCRUBBER_TRACK_HEIGHT, build_ruler_and_scrubber,
    render_ruler_bar, render_scrubber_track,
};
pub use transport::{
    SPEED_PRESETS, TRANSPORT_TOOLBAR_HEIGHT, build_transport_toolbar, render_clip_badge,
    render_loop_pill, render_speed_pill, render_time_readout, render_transport_button,
};
pub use types::{
    DEFAULT_RULER_HEIGHT, DEFAULT_SCRUBBER_HEIGHT, MediaTransportAction, MediaTransportStyle,
    TIMELINE_SIDEBAR_WIDTH, TIMELINE_TAG_LOOP, TIMELINE_TAG_PANEL_ROOT, TIMELINE_TAG_PLAY_PAUSE,
    TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_SCRUBBER_TRACK, TIMELINE_TAG_SPEED_BASE,
    TIMELINE_TAG_STEP_BACK, TIMELINE_TAG_STEP_FWD, TIMELINE_TAG_STOP, TimelineAction,
    TimelineKeyframeMarker, TimelinePanelParams, TimelinePanelState, TimelineRulerStyle,
    evaluate_timeline_transport_tag, is_timeline_tag,
};