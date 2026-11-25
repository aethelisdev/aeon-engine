// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use irisui_core::{KeyCode, WidgetRole};

#[test]
fn test_text_system_internal_caret_blink_and_subms_accumulation() {
    let mut sys = TextSystem::new();
    assert!(sys.is_caret_visible());
    assert_eq!(sys.caret_blink_interval_ms(), 530);

    sys.set_caret_blink_interval_ms(100);
    // Sub-millisecond frames (250 frames x 400us = 100ms) must toggle cleanly
    let mut flipped = false;
    for _ in 0..250 {
        if sys.advance_caret_blink(Duration::from_micros(400)) {
            flipped = true;
        }
    }
    assert!(
        flipped,
        "Sub-millisecond frames must accumulate to flip caret"
    );
    assert!(!sys.is_caret_visible());

    // Disabling blink forces steady visibility
    sys.set_caret_blink_enabled(false);
    assert!(sys.is_caret_visible());
    assert!(!sys.advance_caret_blink(Duration::from_millis(500)));
    assert!(sys.is_caret_visible());
}

#[test]
fn test_text_system_sync_caret_blink_focus_and_keystroke_reset() {
    let mut sys = TextSystem::new();
    sys.set_caret_blink_interval_ms(200);

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let input_id = tree.create_node();
    if let Some(node) = tree.get_mut(input_id) {
        node.role = WidgetRole::TextInput;
        node.tag = 42;
    }
    tree.add_child(root, input_id).expect("add child");

    // 1. Unfocused tree -> not visible, no redraw
    let s0 = sys.sync_caret_blink_with_delta(&tree, Duration::from_millis(100));
    assert!(!s0.has_text_focus);
    assert!(!s0.is_visible);
    assert!(!s0.needs_redraw());

    // 2. Gain focus -> visible + focus_changed
    tree.event_state.focused = Some(input_id);
    tree.event_state.focused_tag = Some(42);
    let s1 = sys.sync_caret_blink_with_delta(&tree, Duration::from_millis(16));
    assert!(s1.has_text_focus);
    assert!(s1.is_visible);
    assert!(s1.focus_changed);
    assert!(s1.needs_redraw());

    // 3. Advance past half-period -> flips hidden
    let s2 = sys.sync_caret_blink_with_delta(&tree, Duration::from_millis(210));
    assert!(!s2.is_visible);
    assert!(s2.blink_changed);
    assert!(s2.needs_redraw());

    // 4. Keystroke while hidden -> immediately resets caret visible
    tree.event_state.frame_key = Some(KeyCode::Character('a'));
    tree.event_state.frame_key_pressed = true;
    let s3 = sys.sync_caret_blink_with_delta(&tree, Duration::from_millis(16));
    assert!(s3.is_visible);
    assert!(s3.blink_changed);
    assert!(s3.needs_redraw());
}

#[cfg(feature = "bundled-sans")]
#[test]
fn test_shape_text_indexed_reuses_buffer_without_cloning() {
    let mut sys = TextSystem::new();
    let params = TextShapeParams {
        font_size: 14.0,
        line_height: 18.0,
        bounds_width: 200.0,
        bounds_height: 40.0,
        align: TextAlign::Left,
        wrap: TextWrap::None,
    };
    let idx1 = sys.shape_text_indexed("StaticLabel", params);
    let ptr1 = sys.shaped_buffer(idx1).expect("shaped buffer must exist") as *const Buffer;
    let idx2 = sys.shape_text_indexed("StaticLabel", params);
    let ptr2 = sys.shape_text_ref("StaticLabel", params) as *const Buffer;

    assert_eq!(idx1, idx2);
    assert_eq!(ptr1, ptr2);
}

#[test]
fn test_font_loading_clears_measurement_caches_and_tolerates_malformed_bytes() {
    let mut sys = TextSystem::new();
    let params = TextShapeParams {
        font_size: 14.0,
        line_height: 18.0,
        bounds_width: 200.0,
        bounds_height: 40.0,
        align: TextAlign::Left,
        wrap: TextWrap::None,
    };
    let initial = sys.measure_text("CacheInvalidation", 14.0, 18.0, None);
    if cfg!(feature = "bundled-sans") {
        assert!(initial.width > 0.0);
        assert_eq!(initial.height, 18.0);
    } else {
        assert_eq!(initial, Size::ZERO);
        assert_eq!(sys.measure_advance("CacheInvalidation", 14.0), 0.0);
        assert_eq!(sys.index_for_position("CacheInvalidation", 14.0, 12.0), 0);
        let idx = sys.shape_text_indexed("CacheInvalidation", params);
        assert!(sys.shaped_buffer(idx).is_some());
    }

    // Negative / malformed font bytes must never panic and must safely invalidate caches.
    sys.load_font_bytes(b"not-a-valid-ttf-header".to_vec());
    sys.load_static_font(&[]);

    // Repeated measurement after cache clear must succeed deterministically without panic.
    let after = sys.measure_text("CacheInvalidation", 14.0, 18.0, None);
    assert_eq!(after, initial);
}