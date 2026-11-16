// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Type-Safe `ActionBus`
//!
//! Verifies FIFO sequencing, capacity invariants, drainage, clear behavior,
//! thread-safety bounds, and simulated high-throughput widget emissions.

use super::ActionBus;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    SaveProject,
    OpenPreferences,
    TogglePanel(&'static str),
    SetZoom(i32),
    CustomCommand(u64),
}

#[test]
fn test_action_bus_new_and_defaults() {
    let bus: ActionBus<TestAction> = ActionBus::new();
    assert!(bus.is_empty());
    assert_eq!(bus.len(), 0);
    assert!(bus.capacity() >= ActionBus::<TestAction>::DEFAULT_CAPACITY);
    assert_eq!(bus.peek(), None);

    let default_bus: ActionBus<TestAction> = ActionBus::default();
    assert!(default_bus.is_empty());
    assert_eq!(default_bus.len(), 0);
    assert!(default_bus.capacity() >= ActionBus::<TestAction>::DEFAULT_CAPACITY);
}

#[test]
fn test_action_bus_with_capacity() {
    let bus: ActionBus<TestAction> = ActionBus::with_capacity(128);
    assert!(bus.is_empty());
    assert!(bus.capacity() >= 128);
}

#[test]
fn test_action_bus_fifo_ordering() {
    let mut bus = ActionBus::new();
    bus.emit(TestAction::SaveProject);
    bus.emit(TestAction::OpenPreferences);
    bus.emit(TestAction::TogglePanel("Hierarchy"));

    assert_eq!(bus.len(), 3);
    assert!(!bus.is_empty());
    assert_eq!(bus.peek(), Some(&TestAction::SaveProject));

    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(
        drained,
        vec![
            TestAction::SaveProject,
            TestAction::OpenPreferences,
            TestAction::TogglePanel("Hierarchy"),
        ]
    );

    assert!(bus.is_empty());
    assert_eq!(bus.len(), 0);
    assert_eq!(bus.peek(), None);
}

#[test]
fn test_action_bus_emit_batch() {
    let mut bus = ActionBus::new();
    let batch = vec![
        TestAction::SetZoom(100),
        TestAction::SetZoom(125),
        TestAction::SetZoom(150),
    ];
    bus.emit_batch(batch);

    assert_eq!(bus.len(), 3);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(
        drained,
        vec![
            TestAction::SetZoom(100),
            TestAction::SetZoom(125),
            TestAction::SetZoom(150),
        ]
    );
}

#[test]
fn test_action_bus_clear_retains_capacity() {
    let mut bus = ActionBus::with_capacity(256);
    let initial_cap = bus.capacity();

    for i in 0..50 {
        bus.emit(TestAction::CustomCommand(i));
    }
    assert_eq!(bus.len(), 50);

    bus.clear();
    assert!(bus.is_empty());
    assert_eq!(bus.len(), 0);
    assert_eq!(bus.capacity(), initial_cap);
}

#[test]
fn test_action_bus_drain_retains_capacity_for_next_frame() {
    let mut bus = ActionBus::new();
    let initial_cap = bus.capacity();

    // Frame 1
    bus.emit(TestAction::SaveProject);
    bus.emit(TestAction::TogglePanel("Inspector"));
    let _ = bus.drain().count();

    // Frame 2
    assert_eq!(bus.capacity(), initial_cap);
    bus.emit(TestAction::OpenPreferences);
    assert_eq!(bus.len(), 1);
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained, vec![TestAction::OpenPreferences]);
}

#[test]
fn test_action_bus_100_widget_emission_stress() {
    let mut bus = ActionBus::new();

    // Simulate 100 distinct UI controls / widgets emitting actions in a frame
    for widget_id in 0..100 {
        match widget_id % 4 {
            0 => bus.emit(TestAction::CustomCommand(widget_id)),
            1 => bus.emit(TestAction::SetZoom(widget_id as i32)),
            2 => bus.emit(TestAction::TogglePanel("Console")),
            _ => bus.emit(TestAction::SaveProject),
        }
    }

    assert_eq!(bus.len(), 100);

    // Consume all actions and verify exact FIFO sequence
    let drained: Vec<TestAction> = bus.drain().collect();
    assert_eq!(drained.len(), 100);

    for (widget_id, action) in drained.into_iter().enumerate() {
        let expected = match widget_id % 4 {
            0 => TestAction::CustomCommand(widget_id as u64),
            1 => TestAction::SetZoom(widget_id as i32),
            2 => TestAction::TogglePanel("Console"),
            _ => TestAction::SaveProject,
        };
        assert_eq!(action, expected, "Mismatch at widget emission #{widget_id}");
    }

    assert!(bus.is_empty());
}

#[test]
fn test_action_bus_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ActionBus<TestAction>>();
}

#[test]
fn test_action_bus_clone_and_equality() {
    let mut bus1 = ActionBus::new();
    bus1.emit(TestAction::SaveProject);
    bus1.emit(TestAction::SetZoom(200));

    let bus2 = bus1.clone();
    assert_eq!(bus1, bus2);

    let items1: Vec<TestAction> = bus1.into_iter().collect();
    let items2: Vec<TestAction> = bus2.into_iter().collect();
    assert_eq!(items1, items2);
}

#[test]
fn test_action_bus_reserve() {
    let mut bus = ActionBus::<TestAction>::with_capacity(10);
    bus.emit(TestAction::SaveProject);
    bus.reserve(50);
    assert!(bus.capacity() >= 51);
}

#[test]
fn test_action_bus_debug_formatting() {
    let mut bus = ActionBus::<TestAction>::new();
    bus.emit(TestAction::SaveProject);
    let debug_str = format!("{bus:?}");
    assert!(debug_str.contains("ActionBus"));
    assert!(debug_str.contains("len: 1"));
}