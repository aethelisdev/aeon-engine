// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::{ActionMap, ElementState, InputManager, KeyCode, MouseButton};

#[test]
fn test_key_and_mouse_three_state_lifecycle_and_repeat_suppression() {
    let mut input = InputManager::new();

    input.process_key_event(KeyCode::KeyW, ElementState::Pressed);
    input.process_mouse_button_event(MouseButton::Right, ElementState::Pressed);
    assert!(input.is_key_pressed(KeyCode::KeyW));
    assert!(input.is_key_just_pressed(KeyCode::KeyW));
    assert!(input.is_mouse_button_pressed(MouseButton::Right));
    assert!(input.is_mouse_button_just_pressed(MouseButton::Right));

    // Frame end clears transient flags while retaining continuous held state
    input.end_frame();
    assert!(input.is_key_pressed(KeyCode::KeyW));
    assert!(!input.is_key_just_pressed(KeyCode::KeyW));
    assert!(!input.is_mouse_button_just_pressed(MouseButton::Right));

    // Key repeat event while already held must not re-fire just_pressed
    input.process_key_event(KeyCode::KeyW, ElementState::Pressed);
    assert!(!input.is_key_just_pressed(KeyCode::KeyW));

    // Release triggers just_released for one frame then clears
    input.process_key_event(KeyCode::KeyW, ElementState::Released);
    input.process_mouse_button_event(MouseButton::Right, ElementState::Released);
    assert!(!input.is_key_pressed(KeyCode::KeyW));
    assert!(input.is_key_just_released(KeyCode::KeyW));
    assert!(input.is_mouse_button_just_released(MouseButton::Right));

    input.end_frame();
    assert!(!input.is_key_just_released(KeyCode::KeyW));
    assert!(!input.is_mouse_button_just_released(MouseButton::Right));
}

#[test]
fn test_clear_pressed_keys_resets_stuck_states_on_focus_loss() {
    let mut input = InputManager::new();
    input.process_key_event(KeyCode::ShiftLeft, ElementState::Pressed);
    input.process_mouse_button_event(MouseButton::Left, ElementState::Pressed);

    input.clear_pressed_keys();
    assert!(!input.is_key_pressed(KeyCode::ShiftLeft));
    assert!(!input.is_key_just_pressed(KeyCode::ShiftLeft));
    assert!(!input.is_mouse_button_pressed(MouseButton::Left));
    assert!(!input.is_mouse_button_just_pressed(MouseButton::Left));
}

#[test]
fn test_action_map_digital_actions_axes_rebind_and_clamping() {
    let mut input = InputManager::new();
    assert!(!input.is_action_pressed("Jump"));
    assert!(!input.is_action_pressed("MissingAction"));
    assert_eq!(input.get_axis("MoveForward"), 0.0);
    assert_eq!(input.get_axis("MissingAxis"), 0.0);

    input.process_key_event(KeyCode::Space, ElementState::Pressed);
    assert!(input.is_action_pressed("Jump"));
    assert!(input.is_action_just_pressed("Jump"));

    input.process_key_event(KeyCode::KeyW, ElementState::Pressed);
    assert_eq!(input.get_axis("MoveForward"), 1.0);
    // Holding ArrowUp as well must clamp at +1.0 instead of summing to +2.0
    input.process_key_event(KeyCode::ArrowUp, ElementState::Pressed);
    assert_eq!(input.get_axis("MoveForward"), 1.0);

    // Non-finite axis binding scale is ignored cleanly instead of poisoning output with NaN
    input
        .action_map
        .bind_axis("MoveForward", KeyCode::KeyW, f32::NAN);
    assert_eq!(input.get_axis("MoveForward"), 1.0);

    // Rebinding missing key/action returns false cleanly; valid rebind switches key
    assert!(
        !input
            .action_map
            .rebind_action("Jump", KeyCode::KeyX, KeyCode::KeyJ)
    );
    assert!(
        !input
            .action_map
            .rebind_action("Missing", KeyCode::Space, KeyCode::KeyJ)
    );
    assert!(
        input
            .action_map
            .rebind_action("Jump", KeyCode::Space, KeyCode::KeyJ)
    );
    input.process_key_event(KeyCode::KeyJ, ElementState::Pressed);
    assert!(input.is_action_pressed("Jump"));
}

#[test]
fn test_action_map_json_roundtrip_and_invalid_input_error() {
    let map = ActionMap::default_gameplay();
    let json = map.to_json().expect("Failed to serialize ActionMap");
    let restored = ActionMap::from_json(&json).expect("Failed to deserialize ActionMap");

    assert_eq!(map.name, restored.name);
    assert!(restored.actions.contains_key("Jump"));
    assert!(restored.actions.contains_key("Pause"));
    assert!(restored.axes.contains_key("MoveForward"));
    assert!(ActionMap::from_json("{malformed json").is_err());
}

#[test]
fn test_cursor_motion_wheel_modifiers_and_non_finite_guards() {
    let mut input = InputManager::new();

    // First cursor move initializes cursor_pos with zero synthetic jump
    input.process_cursor_moved(500.0, 300.0);
    assert_eq!(input.cursor_pos(), (500.0, 300.0));
    assert_eq!(input.cursor_delta(), (0.0, 0.0));

    // Subsequent move accumulates cursor_delta
    input.process_cursor_moved(512.5, 290.0);
    assert_eq!(input.cursor_pos(), (512.5, 290.0));
    assert_eq!(input.cursor_delta(), (12.5, -10.0));

    // Cursor warp (set_cursor_pos) must not add delta
    input.set_cursor_pos(640.0, 360.0);
    assert_eq!(input.cursor_pos(), (640.0, 360.0));
    assert_eq!(input.cursor_delta(), (12.5, -10.0));

    // Raw mouse motion and scroll wheel accumulate and reject non-finite values
    input.add_mouse_delta(3.5, -2.0);
    input.add_mouse_delta(f32::NAN, 1.0);
    assert_eq!(input.mouse_delta(), (3.5, -2.0));

    input.process_mouse_wheel(0.0, 1.5);
    input.process_mouse_wheel(f32::INFINITY, 0.0);
    assert_eq!(input.scroll_delta(), (0.0, 1.5));

    input.process_cursor_moved(f64::NAN, 100.0);
    assert_eq!(input.cursor_pos(), (640.0, 360.0));

    // Modifiers work for either left or right side keys
    input.process_key_event(KeyCode::ControlRight, ElementState::Pressed);
    input.process_key_event(KeyCode::ShiftLeft, ElementState::Pressed);
    input.process_key_event(KeyCode::AltRight, ElementState::Pressed);
    assert!(input.is_ctrl_pressed());
    assert!(input.is_shift_pressed());
    assert!(input.is_alt_pressed());

    // Frame end resets all per-frame motion and scroll deltas while retaining cursor_pos
    input.end_frame();
    assert_eq!(input.cursor_pos(), (640.0, 360.0));
    assert_eq!(input.cursor_delta(), (0.0, 0.0));
    assert_eq!(input.mouse_delta(), (0.0, 0.0));
    assert_eq!(input.scroll_delta(), (0.0, 0.0));
}