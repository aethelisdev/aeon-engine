// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Types, Cache & Semantic Tags
//!
//! Provides deterministic `UiTag` identifiers, cached keyframe marker extraction
//! (`TimelineClipCache`), panel view parameters, and helper player mutation emitters.

use crate::ui::types::EngineUiAction;
use irisui::prelude::{ScrubberMarker, UiTag};
use irisui::workbench::ActionBus;

/// Default height in logical pixels allocated for the top ruler ticks section.
pub const DEFAULT_RULER_HEIGHT: f32 = 18.0;

/// Default height in logical pixels allocated for the interactive scrubber track.
pub const DEFAULT_SCRUBBER_HEIGHT: f32 = 36.0;

/// Width of the left channel property badge column in logical pixels.
pub const TIMELINE_SIDEBAR_WIDTH: f32 = 118.0;

/// Semantic tag for the timeline panel root container background.
pub const TIMELINE_TAG_PANEL_ROOT: u64 = UiTag::from_str("timeline.panel.root").raw();
/// Semantic tag for the timeline step back button.
pub const TIMELINE_TAG_STEP_BACK: u64 = UiTag::from_str("timeline.transport.step_back").raw();
/// Semantic tag for the timeline play/pause toggle button.
pub const TIMELINE_TAG_PLAY_PAUSE: u64 = UiTag::from_str("timeline.transport.play_pause").raw();
/// Semantic tag for the timeline stop playback button.
pub const TIMELINE_TAG_STOP: u64 = UiTag::from_str("timeline.transport.stop").raw();
/// Semantic tag for the timeline step forward button.
pub const TIMELINE_TAG_STEP_FWD: u64 = UiTag::from_str("timeline.transport.step_fwd").raw();
/// Semantic tag for the timeline loop toggle pill.
pub const TIMELINE_TAG_LOOP: u64 = UiTag::from_str("timeline.transport.loop").raw();
/// Semantic tag for the timeline ruler bar container.
pub const TIMELINE_TAG_RULER_ROOT: u64 = UiTag::from_str("timeline.ruler.root").raw();
/// Semantic tag for the timeline scrubber track interactive region.
pub const TIMELINE_TAG_SCRUBBER_TRACK: u64 = UiTag::from_str("timeline.scrubber.track").raw();
/// Semantic tag for the timeline playhead draggable needle cap handle.
pub const TIMELINE_TAG_PLAYHEAD_CAP: u64 = UiTag::from_str("timeline.scrubber.cap").raw();
/// Semantic tag for the clip title badge container.
pub const TIMELINE_TAG_CLIP_BADGE: u64 = UiTag::from_str("timeline.transport.clip_badge").raw();
/// Semantic tag for the time readout badge container.
pub const TIMELINE_TAG_TIME_READOUT: u64 = UiTag::from_str("timeline.transport.time_readout").raw();
/// Semantic tag for the volatile in-place time readout text node updated at 60 FPS during playback.
pub const TIMELINE_TAG_TIME_READOUT_TEXT: u64 =
    UiTag::from_str("timeline.transport.time_readout.text").raw();
/// Semantic tag for the non-interactive live scrubber playhead overlay canvas.
pub const TIMELINE_TAG_SCRUBBER_PLAYHEAD: u64 = UiTag::from_str("timeline.scrubber.playhead").raw();
/// Semantic tag for the non-interactive live dope-sheet vertical playhead overlay canvas.
pub const TIMELINE_TAG_LANES_PLAYHEAD: u64 = UiTag::from_str("timeline.lanes.playhead").raw();

/// Pure domain cache extracting, deduplicating, and sorting keyframe timestamps outside view rendering.
///
/// Prevents per-frame `Vec` allocations and sorting passes when inspecting unchanged clips.
#[derive(Debug, Default, Clone)]
pub struct TimelineClipCache {
    cached_clip_name: String,
    cached_duration: f32,
    cached_channel_count: usize,
    keyframe_markers: Vec<ScrubberMarker>,
}

impl TimelineClipCache {
    /// Synchronizes cached keyframe markers from `player`, skipping work when clip metadata matches.
    pub fn sync(&mut self, player: Option<&ae_animation::AnimationPlayer>) {
        let Some(clip) = player.and_then(|p| p.current_clip.as_ref()) else {
            self.cached_clip_name.clear();
            self.cached_duration = 0.0;
            self.cached_channel_count = 0;
            self.keyframe_markers.clear();
            return;
        };

        if self.cached_clip_name == clip.name
            && (self.cached_duration - clip.duration).abs() < 1e-5
            && self.cached_channel_count == clip.channels.len()
        {
            return;
        }

        self.cached_clip_name.clear();
        self.cached_clip_name.push_str(&clip.name);
        self.cached_duration = clip.duration;
        self.cached_channel_count = clip.channels.len();
        self.keyframe_markers.clear();

        for channel in &clip.channels {
            if let Some(ref track) = channel.vector_track {
                for kf in &track.keyframes {
                    if !self
                        .keyframe_markers
                        .iter()
                        .any(|m| (m.time - kf.time).abs() < 0.02)
                    {
                        self.keyframe_markers.push(ScrubberMarker::new(kf.time));
                    }
                }
            }
            if let Some(ref track) = channel.rotation_track {
                for kf in &track.keyframes {
                    if !self
                        .keyframe_markers
                        .iter()
                        .any(|m| (m.time - kf.time).abs() < 0.02)
                    {
                        self.keyframe_markers.push(ScrubberMarker::new(kf.time));
                    }
                }
            }
        }

        self.keyframe_markers
            .sort_by(|a, b| a.time.total_cmp(&b.time));
    }

    /// Returns the deduplicated, chronologically sorted keyframe markers for the active clip.
    #[inline]
    #[must_use]
    pub fn markers(&self) -> &[ScrubberMarker] {
        &self.keyframe_markers
    }
}

/// Emits a serialized `AnimationPlayer` component update onto [`ActionBus`] after applying `mutate`.
pub(crate) fn emit_player_mutation(
    entity: hecs::Entity,
    player: &ae_animation::AnimationPlayer,
    action_bus: &mut ActionBus<EngineUiAction>,
    mutate: impl FnOnce(&mut ae_animation::AnimationPlayer),
) {
    let mut updated = player.clone();
    mutate(&mut updated);
    action_bus.emit(EngineUiAction::modify_component(
        entity,
        "AnimationPlayer",
        &updated,
    ));
}

/// Runtime parameters passed to the Animation Timeline Studio builder each frame.
pub struct TimelinePanelParams<'a> {
    /// Currently selected entity in the scene, if any.
    pub entity: Option<hecs::Entity>,
    /// Active animation player component borrowed from the ECS world, if present.
    pub animation_player: Option<&'a ae_animation::AnimationPlayer>,
}

/// Computes a zero-allocation 64-bit structural fingerprint of the selected entity and animation
/// transport/clip metadata for `UiScope::record_retained_revision` and `sync_retained_revision`.
pub(crate) fn timeline_structural_hash(
    entity: Option<hecs::Entity>,
    player: Option<&ae_animation::AnimationPlayer>,
) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mix = |acc: &mut u64, val: u64| {
        *acc ^= val;
        *acc = acc.wrapping_mul(0x100000001b3);
    };
    mix(&mut h, entity.map_or(0, |e| e.to_bits().get()));
    mix(&mut h, u64::from(player.is_some()));
    if let Some(p) = player {
        let state_code = match p.state {
            ae_animation::AnimationState::Stopped => 0,
            ae_animation::AnimationState::Playing => 1,
            ae_animation::AnimationState::Paused => 2,
        };
        mix(&mut h, state_code);
        mix(&mut h, u64::from(p.looping));
        mix(&mut h, u64::from(p.speed.to_bits()));
        if let Some(clip) = p.current_clip.as_ref() {
            mix(&mut h, clip.channels.len() as u64);
            for &b in clip.name.as_bytes() {
                mix(&mut h, u64::from(b));
            }
        }
    }
    h
}

/// Computes a 64-bit fingerprint of the fast-changing playback time and duration for `InPlaceRefresh`.
#[inline]
pub(crate) fn timeline_live_hash(player: Option<&ae_animation::AnimationPlayer>) -> u64 {
    let Some(p) = player else {
        return 0;
    };
    let duration = p.current_clip.as_ref().map_or(1.0, |c| c.duration.max(0.1));
    (u64::from(p.current_time.to_bits()) << 32) | u64::from(duration.to_bits())
}

/// Runtime persistent state for the Animation Timeline Studio panel.
#[derive(Debug, Default, Clone)]
pub struct TimelinePanelState {
    /// Deduplicated keyframe marker cache for the active animation clip.
    pub cache: TimelineClipCache,
    /// Selected entity handle cached for timeline interactions.
    pub selected_entity: Option<hecs::Entity>,
    /// Duration of the currently active animation clip in seconds.
    pub clip_duration: f32,
}

/// Semantic alias for [`TimelinePanelState`].
pub type TimelineState = TimelinePanelState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_cache_deduplicates_and_sorts_keyframes() {
        let mut clip = ae_animation::AnimationClip::new("Walk", 3.0);
        clip.channels.push(ae_animation::clip::Channel {
            joint_index: 0,
            target_property: ae_animation::clip::TargetProperty::Translation,
            vector_track: Some(ae_animation::clip::VectorTrack {
                keyframes: vec![
                    ae_animation::clip::Keyframe {
                        time: 2.0,
                        value: Default::default(),
                    },
                    ae_animation::clip::Keyframe {
                        time: 0.5,
                        value: Default::default(),
                    },
                    ae_animation::clip::Keyframe {
                        time: 0.505, // Within 0.02s deduplication window
                        value: Default::default(),
                    },
                ],
                interpolation: ae_animation::clip::Interpolation::Linear,
            }),
            rotation_track: None,
        });
        let player = ae_animation::AnimationPlayer {
            current_clip: Some(clip),
            ..Default::default()
        };

        let mut cache = TimelineClipCache::default();
        cache.sync(Some(&player));
        assert_eq!(cache.markers().len(), 2);
        assert_eq!(cache.markers()[0].time, 0.5);
        assert_eq!(cache.markers()[1].time, 2.0);
    }
}