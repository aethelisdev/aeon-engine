// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;

#[test]
fn test_text_undo_stack_push_undo_redo() {
    let mut stack = TextUndoStack::new();
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());

    // Initial buffer: ""
    let snap0 = TextSnapshot::new("", 0, None);
    stack.checkpoint(snap0);
    assert!(stack.can_undo());
    assert_eq!(stack.undo_count(), 1);

    // Edit 1: "Hello"
    let snap1 = TextSnapshot::new("Hello", 5, None);
    stack.checkpoint(snap1);
    assert_eq!(stack.undo_count(), 2);

    // Current state is "Hello World", 11
    let reverted = stack.undo("Hello World", 11, None).expect("Undo available");
    assert_eq!(reverted.text, "Hello");
    assert_eq!(reverted.cursor_byte_idx, 5);
    assert!(stack.can_redo());
    assert_eq!(stack.redo_count(), 1);

    // Undo again to initial state
    let reverted_initial = stack.undo("Hello", 5, None).expect("Undo available");
    assert_eq!(reverted_initial.text, "");
    assert_eq!(reverted_initial.cursor_byte_idx, 0);
    assert!(!stack.can_undo());
    assert_eq!(stack.redo_count(), 2);

    // Redo forward to "Hello"
    let redone = stack.redo("", 0, None).expect("Redo available");
    assert_eq!(redone.text, "Hello");
    assert_eq!(redone.cursor_byte_idx, 5);
    assert_eq!(stack.redo_count(), 1);

    // Redo forward to "Hello World"
    let redone_world = stack.redo("Hello", 5, None).expect("Redo available");
    assert_eq!(redone_world.text, "Hello World");
    assert_eq!(redone_world.cursor_byte_idx, 11);
    assert!(!stack.can_redo());
}

#[test]
fn test_text_undo_stack_in_place_helpers() {
    let mut stack = TextUndoStack::new();
    let mut text = "initial".to_string();
    let mut cursor = 7;
    let mut selection = None;

    stack.checkpoint(TextSnapshot::new(text.clone(), cursor, selection));

    text = "initial modified".to_string();
    cursor = 16;

    assert!(stack.undo_in_place(&mut text, &mut cursor, &mut selection));
    assert_eq!(text, "initial");
    assert_eq!(cursor, 7);

    assert!(stack.redo_in_place(&mut text, &mut cursor, &mut selection));
    assert_eq!(text, "initial modified");
    assert_eq!(cursor, 16);
}

#[test]
fn test_text_undo_stack_capacity_limit() {
    let mut stack = TextUndoStack::with_max_depth(3);
    assert_eq!(stack.max_depth(), 3);

    stack.checkpoint(TextSnapshot::new("0", 1, None));
    stack.checkpoint(TextSnapshot::new("1", 1, None));
    stack.checkpoint(TextSnapshot::new("2", 1, None));
    assert_eq!(stack.undo_count(), 3);

    // Adding 4th item must drop the oldest ("0")
    stack.checkpoint(TextSnapshot::new("3", 1, None));
    assert_eq!(stack.undo_count(), 3);

    let s3 = stack.undo("4", 1, None).unwrap();
    assert_eq!(s3.text, "3");
    let s2 = stack.undo("3", 1, None).unwrap();
    assert_eq!(s2.text, "2");
    let s1 = stack.undo("2", 1, None).unwrap();
    assert_eq!(s1.text, "1");

    // Oldest item "0" was dropped by capacity limit
    assert!(!stack.can_undo());
}

#[test]
fn test_text_undo_stack_branch_invalidation() {
    let mut stack = TextUndoStack::new();

    stack.checkpoint(TextSnapshot::new("Initial", 7, None));
    stack.checkpoint(TextSnapshot::new("Step 1", 6, None));

    // Undo step 1
    let _ = stack.undo("Step 2", 6, None);
    assert!(stack.can_redo());
    assert_eq!(stack.redo_count(), 1);

    // New editing branch invalidates redo stack
    stack.checkpoint(TextSnapshot::new("New Branch", 10, None));
    assert!(!stack.can_redo());
    assert_eq!(stack.redo_count(), 0);
}

#[test]
fn test_text_undo_stack_keystroke_coalescing() {
    let mut stack = TextUndoStack::new();

    // 1. Initial typing: 'a' pushes the pre-edit snapshot ""
    stack.record_edit(
        TextSnapshot::new("", 0, None),
        UndoActionKind::Insert,
        Some('a'),
    );
    assert_eq!(stack.undo_count(), 1);

    // 2. Immediate continuous typing: 'b', 'c', 'd' within timeout should coalesce (not push new snapshots)
    stack.record_edit(
        TextSnapshot::new("a", 1, None),
        UndoActionKind::Insert,
        Some('b'),
    );
    assert_eq!(stack.undo_count(), 1);

    stack.record_edit(
        TextSnapshot::new("ab", 2, None),
        UndoActionKind::Insert,
        Some('c'),
    );
    assert_eq!(stack.undo_count(), 1);

    stack.record_edit(
        TextSnapshot::new("abc", 3, None),
        UndoActionKind::Insert,
        Some('d'),
    );
    assert_eq!(stack.undo_count(), 1);

    // Undo must restore the initial pre-typing state ""
    let restored = stack.undo("abcd", 4, None).expect("Undo available");
    assert_eq!(restored.text, "");
    assert_eq!(restored.cursor_byte_idx, 0);
}

#[test]
fn test_text_undo_stack_word_boundary_coalescing() {
    let mut stack = TextUndoStack::new();

    // Type "hello"
    stack.record_edit(
        TextSnapshot::new("", 0, None),
        UndoActionKind::Insert,
        Some('h'),
    );
    stack.record_edit(
        TextSnapshot::new("h", 1, None),
        UndoActionKind::Insert,
        Some('e'),
    );
    stack.record_edit(
        TextSnapshot::new("he", 2, None),
        UndoActionKind::Insert,
        Some('l'),
    );
    stack.record_edit(
        TextSnapshot::new("hel", 3, None),
        UndoActionKind::Insert,
        Some('l'),
    );
    stack.record_edit(
        TextSnapshot::new("hell", 4, None),
        UndoActionKind::Insert,
        Some('o'),
    );
    assert_eq!(stack.undo_count(), 1);

    // Space triggers word boundary checkpoint
    stack.record_edit(
        TextSnapshot::new("hello", 5, None),
        UndoActionKind::Insert,
        Some(' '),
    );
    assert_eq!(stack.undo_count(), 2);

    // Type "world"
    stack.record_edit(
        TextSnapshot::new("hello ", 6, None),
        UndoActionKind::Insert,
        Some('w'),
    );
    stack.record_edit(
        TextSnapshot::new("hello w", 7, None),
        UndoActionKind::Insert,
        Some('o'),
    );
    assert_eq!(stack.undo_count(), 3);

    // Undo 1: Reverts to word boundary snapshot "hello "
    let u1 = stack.undo("hello wo", 8, None).expect("Undo 1");
    assert_eq!(u1.text, "hello ");

    // Undo 2: Reverts to word boundary snapshot "hello"
    let u2 = stack.undo("hello ", 6, None).expect("Undo 2");
    assert_eq!(u2.text, "hello");

    // Undo 3: Reverts to initial snapshot ""
    let u3 = stack.undo("hello", 5, None).expect("Undo 3");
    assert_eq!(u3.text, "");
}

#[test]
fn test_text_undo_stack_action_kind_transition() {
    let mut stack = TextUndoStack::new();

    // Typing inserts
    stack.record_edit(
        TextSnapshot::new("", 0, None),
        UndoActionKind::Insert,
        Some('a'),
    );
    stack.record_edit(
        TextSnapshot::new("a", 1, None),
        UndoActionKind::Insert,
        Some('b'),
    );
    assert_eq!(stack.undo_count(), 1);

    // Transition from Insert to Delete forces a new snapshot
    stack.record_edit(
        TextSnapshot::new("ab", 2, None),
        UndoActionKind::Delete,
        None,
    );
    assert_eq!(stack.undo_count(), 2);

    // Continuous deletions coalesce
    stack.record_edit(
        TextSnapshot::new("a", 1, None),
        UndoActionKind::Delete,
        None,
    );
    assert_eq!(stack.undo_count(), 2);
}

#[test]
fn test_text_undo_stack_atomic_action_isolation() {
    let mut stack = TextUndoStack::new();

    // Paste action (Atomic) always forces an independent snapshot
    stack.record_edit(
        TextSnapshot::new("Base ", 5, None),
        UndoActionKind::Atomic,
        None,
    );
    assert_eq!(stack.undo_count(), 1);

    stack.record_edit(
        TextSnapshot::new("Base Pasted ", 12, None),
        UndoActionKind::Atomic,
        None,
    );
    assert_eq!(stack.undo_count(), 2);
}

#[test]
fn test_text_undo_stack_selection_preservation() {
    let mut stack = TextUndoStack::new();

    let snap_with_sel = TextSnapshot::new("Selected Text", 13, Some((0, 8)));
    stack.checkpoint(snap_with_sel);

    let undone = stack.undo("Replacement", 11, None).expect("Undo");
    assert_eq!(undone.text, "Selected Text");
    assert_eq!(undone.cursor_byte_idx, 13);
    assert_eq!(undone.selection, Some((0, 8)));
}

#[test]
fn test_text_undo_stack_dynamic_capacity_resize() {
    let mut stack = TextUndoStack::with_max_depth(10);
    for i in 0..10 {
        stack.checkpoint(TextSnapshot::new(format!("Item {i}"), i, None));
    }
    assert_eq!(stack.undo_count(), 10);

    // Shrink capacity down to 3
    stack.set_max_depth(3);
    assert_eq!(stack.max_depth(), 3);
    assert_eq!(stack.undo_count(), 3);

    // Should retain the 3 most recent entries (7, 8, 9)
    let s9 = stack.undo("Current", 0, None).unwrap();
    assert_eq!(s9.text, "Item 9");
    let s8 = stack.undo("Item 9", 0, None).unwrap();
    assert_eq!(s8.text, "Item 8");
    let s7 = stack.undo("Item 8", 0, None).unwrap();
    assert_eq!(s7.text, "Item 7");
    assert!(!stack.can_undo());
}