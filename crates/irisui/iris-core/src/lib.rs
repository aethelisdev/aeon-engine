// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Core (`iris-core`)
//!
//! Generational arena-based UI graph, geometry primitives, dirty-state caching,
//! and fluent styling attributes for Iris UI.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod dirty;
pub mod error;
pub mod event;
pub mod geometry;
pub mod id;
pub mod node;
pub mod style;
pub mod tree;

pub use color::Color;
pub use dirty::DirtyFlags;
pub use error::IrisCoreError;
pub use event::{
    EventDispatcher, FocusManager, HitTestResult, ImeEvent, InteractionEvent, KeyCode, MouseButton,
    UiEvent, WidgetState,
};
pub use geometry::{Border, BoxShadow, CornerRadii, Insets, Point, Rect, Size};
pub use id::WidgetId;
pub use node::WidgetNode;
pub use style::{AlignItems, FlexDirection, JustifyContent, Style, TextAlign};
pub use tree::UiTree;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation_and_reparenting() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root should be created");
        let child1 = tree.create_node();
        let child2 = tree.create_node();

        assert_eq!(tree.len(), 3);
        assert!(tree.add_child(root, child1).is_ok());
        assert!(tree.add_child(root, child2).is_ok());

        assert_eq!(tree.get(root).unwrap().children.len(), 2);
        assert_eq!(tree.get(child1).unwrap().parent, Some(root));
        assert_eq!(tree.get(child2).unwrap().parent, Some(root));
    }

    #[test]
    fn test_circular_reference_prevention() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let child = tree.create_node();
        let grand_child = tree.create_node();

        assert!(tree.add_child(root, child).is_ok());
        assert!(tree.add_child(child, grand_child).is_ok());

        // Attempting to make root a child of grand_child must fail with CircularHierarchy error
        let result = tree.add_child(grand_child, root);
        assert!(matches!(
            result,
            Err(IrisCoreError::CircularHierarchy { .. })
        ));
    }

    #[test]
    fn test_color_hex_and_lerp() {
        let col = Color::hex("#ff0000");
        assert_eq!(col, Color::RED);

        let black = Color::BLACK;
        let white = Color::WHITE;
        let mid = black.lerp(white, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_dirty_flag_propagation() {
        let mut node = WidgetNode::new(WidgetId::default());
        assert!(node.dirty.contains(DirtyFlags::ALL));

        node.clear_dirty(DirtyFlags::ALL);
        assert!(!node.dirty.intersects(DirtyFlags::ALL));

        node.mark_dirty(DirtyFlags::LAYOUT);
        assert!(node.dirty.contains(DirtyFlags::LAYOUT));
        assert!(!node.dirty.contains(DirtyFlags::PAINT));
    }

    #[test]
    fn test_focus_manager_and_event_dispatcher() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        if let Some(node) = tree.get_mut(root) {
            node.interactive = false;
            node.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);
        }
        let btn1 = tree.create_node();
        let btn2 = tree.create_node();

        if let Some(node) = tree.get_mut(btn1) {
            node.interactive = true;
            node.computed_rect = Rect::new(0.0, 0.0, 100.0, 30.0);
        }
        if let Some(node) = tree.get_mut(btn2) {
            node.interactive = true;
            node.computed_rect = Rect::new(0.0, 40.0, 100.0, 30.0);
        }

        let _ = tree.add_child(root, btn1);
        let _ = tree.add_child(root, btn2);

        let mut focus = FocusManager::new();

        // Advance focus with Tab key
        focus.advance_focus(&tree, false);
        assert_eq!(focus.focused, Some(btn1));

        focus.advance_focus(&tree, false);
        assert_eq!(focus.focused, Some(btn2));

        focus.advance_focus(&tree, false);
        assert_eq!(focus.focused, Some(btn1));

        // Clear focus and test mouse click event generation
        focus.clear_focus();

        let events_down = EventDispatcher::dispatch(
            &mut tree,
            &mut focus,
            UiEvent::MouseDown {
                button: MouseButton::Left,
                point: Point::new(50.0, 15.0),
            },
        );
        assert!(
            events_down
                .iter()
                .any(|(id, e)| *id == btn1 && *e == InteractionEvent::FocusGained)
        );

        let events_up = EventDispatcher::dispatch(
            &mut tree,
            &mut focus,
            UiEvent::MouseUp {
                button: MouseButton::Left,
                point: Point::new(50.0, 15.0),
            },
        );
        assert!(events_up.iter().any(|(id, e)| *id == btn1
            && *e
                == InteractionEvent::Click {
                    button: MouseButton::Left
                }));
    }

    #[test]
    fn test_ime_event_dispatching() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let input_id = tree.create_node();

        if let Some(node) = tree.get_mut(input_id) {
            node.interactive = true;
            node.computed_rect = Rect::new(10.0, 10.0, 200.0, 30.0);
        }
        let _ = tree.add_child(root, input_id);

        let mut focus = FocusManager::new();
        focus.set_focus(input_id);

        assert_eq!(
            focus.get_ime_cursor_area(&tree),
            Some(Rect::new(10.0, 10.0, 200.0, 30.0))
        );

        // Preedit event
        let events_preedit = EventDispatcher::dispatch(
            &mut tree,
            &mut focus,
            UiEvent::Ime(ImeEvent::Preedit("にほん".into(), Some((0, 3)))),
        );
        assert_eq!(
            events_preedit,
            vec![(
                input_id,
                InteractionEvent::ImePreedit {
                    text: "にほん".into(),
                    cursor: Some((0, 3)),
                }
            )]
        );

        // Commit event
        let events_commit = EventDispatcher::dispatch(
            &mut tree,
            &mut focus,
            UiEvent::Ime(ImeEvent::Commit("日本".into())),
        );
        assert_eq!(
            events_commit,
            vec![(
                input_id,
                InteractionEvent::ImeCommit {
                    text: "日本".into()
                }
            )]
        );
    }

    #[test]
    fn test_node_texture_uv_and_tint() {
        let mut tree = UiTree::new();
        let id1 = tree.create_node();
        let id2 = tree.create_node();

        let node = WidgetNode::new(id1)
            .with_texture_uv([0.25, 0.0, 0.50, 1.0])
            .with_texture_tint(Color::rgba(0.9, 0.9, 1.0, 0.8));

        assert_eq!(node.texture_uv, Some([0.25, 0.0, 0.50, 1.0]));
        assert_eq!(node.texture_tint, Some(Color::rgba(0.9, 0.9, 1.0, 0.8)));

        let mut node2 = WidgetNode::new(id2);
        node2.set_texture_uv([0.5, 0.0, 0.75, 1.0]);
        node2.set_texture_tint(Color::WHITE);
        assert_eq!(node2.texture_uv, Some([0.5, 0.0, 0.75, 1.0]));
        assert_eq!(node2.texture_tint, Some(Color::WHITE));
    }
}