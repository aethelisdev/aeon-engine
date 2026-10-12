// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Autonomous Numeric Dragging Unit Tests
//!
//! Verifies that Inspector numeric scrubbing correctly delegates to the framework-level
//! [`NumericDragState`], honoring movement thresholds, Shift/Ctrl modifier multipliers,
//! in-place click detection, and Escape key cancellations.

use super::types::{InspectorNumberDragState, InspectorNumberInputId};

#[test]
fn test_inspector_number_drag_autonomous_delta_and_clamping() {
    let entity = hecs::Entity::DANGLING;
    let num_id = InspectorNumberInputId::PosX;

    let mut drag_state = InspectorNumberDragState::new(
        entity, num_id, 100.0, // start_x
        10.0,  // start_val
        -50.0, // min_val
        50.0,  // max_val
        0.5,   // sensitivity
    );

    assert_eq!(drag_state.start_val(), 10.0);
    assert!(!drag_state.drag.has_dragged);

    // 1. Below 2.0px threshold -> no drag activation
    let res_sub = drag_state.drag.update(101.0, false, false);
    assert!(!res_sub.has_dragged);
    assert_eq!(res_sub.value, None);

    // 2. Beyond threshold (delta = +20px) -> 10.0 + (20 * 0.5) = 20.0
    let res = drag_state.drag.update(120.0, false, false);
    assert!(res.has_dragged);
    assert_eq!(res.value, Some(20.0));

    // 3. Clamping at upper bound (delta = +200px) -> clamped to 50.0
    let res_clamp = drag_state.drag.update(300.0, false, false);
    assert_eq!(res_clamp.value, Some(50.0));
}

#[test]
fn test_inspector_number_drag_modifiers_shift_and_ctrl() {
    let entity = hecs::Entity::DANGLING;
    let num_id = InspectorNumberInputId::ScaleX;

    // Shift -> fine 0.1x multiplier
    let mut fine_drag = InspectorNumberDragState::new(entity, num_id, 100.0, 1.0, 0.0, 100.0, 1.0);
    let res_fine = fine_drag.drag.update(110.0, true, false);
    assert!(res_fine.has_dragged);
    // delta = 10.0 * 1.0 * 0.1 = 1.0 -> 1.0 + 1.0 = 2.0
    assert_eq!(res_fine.value, Some(2.0));

    // Ctrl -> coarse 10.0x multiplier
    let mut coarse_drag =
        InspectorNumberDragState::new(entity, num_id, 100.0, 1.0, 0.0, 1000.0, 1.0);
    let res_coarse = coarse_drag.drag.update(110.0, false, true);
    assert!(res_coarse.has_dragged);
    // delta = 10.0 * 1.0 * 10.0 = 100.0 -> 1.0 + 100.0 = 101.0
    assert_eq!(res_coarse.value, Some(101.0));
}

#[test]
fn test_inspector_number_drag_click_in_place_without_motion() {
    let entity = hecs::Entity::DANGLING;
    let num_id = InspectorNumberInputId::PosY;

    let mut drag_state =
        InspectorNumberDragState::new(entity, num_id, 200.0, 5.0, -100.0, 100.0, 0.1);

    // Mouse released at exactly start coordinate or < 2.0px motion
    let res = drag_state.drag.finish();
    assert!(
        res.is_click_in_place,
        "Stationary mouse click must trigger click-in-place for text editing"
    );
    assert!(!res.has_dragged);
}

#[test]
fn test_inspector_number_drag_escape_cancellation() {
    let entity = hecs::Entity::DANGLING;
    let num_id = InspectorNumberInputId::RotY;

    let mut drag_state =
        InspectorNumberDragState::new(entity, num_id, 100.0, 45.0, -360.0, 360.0, 1.0);

    // Active drag modifies value
    let res = drag_state.drag.update(150.0, false, false);
    assert_eq!(res.value, Some(95.0));

    // Escape cancellation resets to initial starting value
    let cancel_res = drag_state.drag.cancel();
    assert!(cancel_res.is_cancelled);
    assert_eq!(cancel_res.value, Some(45.0));
    assert_eq!(drag_state.drag.current_val, 45.0);
}