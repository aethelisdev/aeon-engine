// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Time Ruler, Interactive Scrubber & Track Lanes (`irisui-widgets::declarative::scope::time_ruler`)
//!
//! Pure layout-adaptive time-line ruler ticks, interactive playhead scrubbing tracks,
//! and multi-channel marker lane rows built on [`ImmediateCanvas`](crate::canvas::ImmediateCanvas)
//! and `StackString` formatting with zero engine or domain coupling.

pub mod lane;
pub mod ruler;
pub mod scrubber;

pub use lane::{TrackLaneGroupSpec, TrackLaneSpec};
pub use ruler::{TimeRulerSpec, TimeRulerStyle, tick_step_for_duration};
pub use scrubber::{ScrubberMarker, ScrubberResponse, ScrubberTrackSpec};