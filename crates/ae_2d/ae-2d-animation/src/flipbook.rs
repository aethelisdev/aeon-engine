// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use serde::{Deserialize, Serialize};

/// 2D Sprite Flipbook / Sprite Sheet animation component.
/// Advances UV sub-rectangles across a sequence of frames at a configurable frame rate.
/// Can be used standalone or synchronized directly with a `SpriteRenderer` component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAnimation {
    /// Ordered sequence of UV coordinate bounding boxes `[u_min, v_min, u_max, v_max]` for each animation frame.
    pub frames: Vec<[f32; 4]>,

    /// Playback speed in frames per second (fps). Must be positive non-zero (`>= 0.001`).
    pub frame_rate: f32,

    /// Accumulated playback time in seconds within the current frame.
    pub elapsed_time: f32,

    /// Zero-based index of the currently active frame in `frames`.
    pub current_frame: usize,

    /// Whether playback loops back to frame 0 after reaching the final frame.
    pub looping: bool,

    /// Playback active state. When false, time does not advance.
    pub is_playing: bool,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            frame_rate: 12.0,
            elapsed_time: 0.0,
            current_frame: 0,
            looping: true,
            is_playing: true,
        }
    }
}

impl SpriteAnimation {
    /// Creates a new sprite animation with a list of UV frames and a playback frame rate.
    pub fn new(frames: Vec<[f32; 4]>, frame_rate: f32, looping: bool) -> Self {
        Self {
            frames,
            frame_rate: frame_rate.max(0.001),
            elapsed_time: 0.0,
            current_frame: 0,
            looping,
            is_playing: true,
        }
    }

    /// Creates a grid-based uniform sprite sheet animation given column and row counts.
    /// Generates UV rectangles sliced across a standard `[0.0, 0.0, 1.0, 1.0]` texture.
    pub fn from_grid(columns: usize, rows: usize, total_frames: usize, fps: f32) -> Self {
        let cols = columns.max(1);
        let rws = rows.max(1);
        let max_frames = (cols * rws).min(total_frames.max(1));

        let tile_w = 1.0 / cols as f32;
        let tile_h = 1.0 / rws as f32;

        let mut frames = Vec::with_capacity(max_frames);
        for i in 0..max_frames {
            let col = i % cols;
            let row = i / cols;

            let u_min = col as f32 * tile_w;
            let v_min = row as f32 * tile_h;
            let u_max = u_min + tile_w;
            let v_max = v_min + tile_h;

            frames.push([u_min, v_min, u_max, v_max]);
        }

        Self::new(frames, fps, true)
    }

    /// Advances the animation by `delta_time` seconds and returns the currently active UV rectangle.
    pub fn update(&mut self, delta_time: f32) -> Option<[f32; 4]> {
        if self.frames.is_empty() {
            return None;
        }

        if self.is_playing && delta_time > 0.0 {
            self.elapsed_time += delta_time;
            let frame_duration = 1.0 / self.frame_rate.max(0.001);

            while self.elapsed_time >= frame_duration {
                self.elapsed_time -= frame_duration;
                if self.current_frame + 1 < self.frames.len() {
                    self.current_frame += 1;
                } else if self.looping {
                    self.current_frame = 0;
                } else {
                    self.is_playing = false;
                    break;
                }
            }
        }

        self.frames.get(self.current_frame).copied()
    }

    /// Resets animation playback to frame 0 and restarts time accumulation.
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed_time = 0.0;
        self.is_playing = true;
    }
}