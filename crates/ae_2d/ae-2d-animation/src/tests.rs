// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::SpriteAnimation;

#[test]
fn test_sprite_animation_grid_and_playback() {
    let mut anim = SpriteAnimation::from_grid(4, 2, 8, 10.0);
    assert_eq!(anim.frames.len(), 8);
    assert_eq!(anim.frames[0], [0.0, 0.0, 0.25, 0.5]);
    assert_eq!(anim.current_frame, 0);

    // 10 FPS = 0.1s per frame
    anim.update(0.05);
    assert_eq!(anim.current_frame, 0);

    anim.update(0.06); // total 0.11s -> advances to frame 1
    assert_eq!(anim.current_frame, 1);

    // Advance past all frames
    anim.update(1.0);
    assert_eq!(anim.current_frame, 3); // 1 + 10 = 11 -> 11 % 8 = 3 (looping)

    anim.reset();
    assert_eq!(anim.current_frame, 0);
    assert_eq!(anim.elapsed_time, 0.0);
}