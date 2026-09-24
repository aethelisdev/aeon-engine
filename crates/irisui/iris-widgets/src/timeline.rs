// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio & Media Playback Widgets (`iris-widgets::timeline`)
//!
//! Exposes timeline constants, styling, semantic tags, and playback transport actions.
//!

pub mod style;
pub mod transport;
pub mod types;

pub use style::{MediaTransportStyle, TimelineRulerStyle};
pub use transport::{
    DEFAULT_SPEED_PRESETS, TIMELINE_TAG_LOOP, TIMELINE_TAG_PANEL_ROOT, TIMELINE_TAG_PLAY_PAUSE,
    TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_SCRUBBER_TRACK, TIMELINE_TAG_SPEED_BASE,
    TIMELINE_TAG_STEP_BACK, TIMELINE_TAG_STEP_FWD, TIMELINE_TAG_STOP,
    evaluate_timeline_transport_tag, is_timeline_tag,
};
pub use types::{
    DEFAULT_RULER_HEIGHT, DEFAULT_SCRUBBER_HEIGHT, MediaTransportAction, TimelineKeyframeMarker,
};