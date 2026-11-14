// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Volatile Overlay Hooks & Interactive Overlay Layer
//!
//! Verifies overlay anchoring geometry, lifecycle predicates, pointer hit-testing,
//! outside-click and Escape key dismissals, and action bus dispatching.

use crate::action_bus::ActionBus;
use crate::dialogs::InfoCardDialogDescriptor;
use crate::hooks::overlay::{
    ClosureOverlayHook, OverlayAnchor, OverlayHook, OverlayHost, OverlayId,
};
use crate::hooks::ui_ext::WorkbenchUiScopeExt;
use iris_core::event::{KeyCode, MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;
use iris_widgets::declarative::UiScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestAction {
    ToolbarClicked(u32),
    OverlayDismissed,
}

struct TestAppState {
    pub show_hud: bool,
    pub show_picker: bool,
    pub active_tool: u32,
    pub dismiss_count: u32,
}

impl Default for TestAppState {
    fn default() -> Self {
        Self {
            show_hud: true,
            show_picker: false,
            active_tool: 1,
            dismiss_count: 0,
        }
    }
}

#[test]
fn test_overlay_id_and_anchor_geometry() {
    let id = OverlayId::new(101);
    assert_eq!(id.value(), 101);
    assert_eq!(*id, 101);

    let workspace = Rect::new(10.0, 20.0, 800.0, 600.0);

    // Absolute
    let abs_rect = Rect::new(50.0, 50.0, 100.0, 200.0);
    assert_eq!(
        OverlayAnchor::Absolute(abs_rect).resolve_rect(workspace),
        abs_rect
    );

    // Offset
    let offset_anchor = OverlayAnchor::Offset {
        offset: Point::new(15.0, 25.0),
        size: (200.0, 100.0),
    };
    assert_eq!(
        offset_anchor.resolve_rect(workspace),
        Rect::new(25.0, 45.0, 200.0, 100.0)
    );

    // Centered
    let centered_anchor = OverlayAnchor::Centered {
        size: (400.0, 300.0),
    };
    assert_eq!(
        centered_anchor.resolve_rect(workspace),
        Rect::new(210.0, 170.0, 400.0, 300.0)
    );

    // Full
    assert_eq!(OverlayAnchor::Full.resolve_rect(workspace), workspace);
}

#[test]
fn test_closure_overlay_hook_builder_and_predicates() {
    let hook = ClosureOverlayHook::new(
        OverlayId::new(1),
        |scope, _bounds, state: &mut TestAppState, bus| {
            bus.emit(TestAction::ToolbarClicked(state.active_tool));
            scope.row(|_| {});
        },
    )
    .with_anchor(OverlayAnchor::Offset {
        offset: Point::new(10.0, 10.0),
        size: (120.0, 40.0),
    })
    .with_active(|state: &TestAppState| state.show_hud)
    .with_dismiss_on_outside_click(true)
    .with_dismiss_on_escape(true)
    .with_on_dismiss(|state: &mut TestAppState, bus| {
        state.dismiss_count += 1;
        bus.emit(TestAction::OverlayDismissed);
    });

    assert_eq!(hook.id(), OverlayId::new(1));
    assert!(hook.dismiss_on_outside_click());
    assert!(hook.dismiss_on_escape());

    let mut state = TestAppState::default();
    assert!(hook.is_active(&state));

    state.show_hud = false;
    assert!(!hook.is_active(&state));
}

#[test]
fn test_overlay_host_lifecycle_and_registration() {
    let mut host = OverlayHost::<TestAppState, TestAction>::new();
    assert!(host.is_empty());
    assert_eq!(host.len(), 0);

    let hook1 = ClosureOverlayHook::new(1, |_, _, _, _| {});
    let hook2 = ClosureOverlayHook::new(2, |_, _, _, _| {});

    host.register(hook1);
    host.register(hook2);

    assert_eq!(host.len(), 2);
    assert!(host.contains(OverlayId::new(1)));
    assert!(host.contains(OverlayId::new(2)));
    assert!(!host.contains(OverlayId::new(3)));

    // Re-registering with same ID replaces existing
    let hook1_v2 = ClosureOverlayHook::new(1, |_, _, _, _| {});
    host.register(hook1_v2);
    assert_eq!(host.len(), 2);

    // Remove
    let removed = host.remove(OverlayId::new(1));
    assert!(removed.is_some());
    assert_eq!(host.len(), 1);
    assert!(!host.contains(OverlayId::new(1)));

    host.clear();
    assert!(host.is_empty());
}

#[test]
fn test_overlay_host_pointer_hit_testing() {
    let mut host = OverlayHost::<TestAppState, TestAction>::new();
    let workspace = Rect::new(0.0, 0.0, 1000.0, 800.0);
    let state = TestAppState {
        show_hud: true,
        show_picker: false,
        ..Default::default()
    };

    let hud_hook = ClosureOverlayHook::new(10, |_, _, _, _| {})
        .with_anchor(OverlayAnchor::Offset {
            offset: Point::new(100.0, 100.0),
            size: (200.0, 50.0),
        })
        .with_active(|s: &TestAppState| s.show_hud);

    let picker_hook = ClosureOverlayHook::new(20, |_, _, _, _| {})
        .with_anchor(OverlayAnchor::Offset {
            offset: Point::new(400.0, 400.0),
            size: (150.0, 150.0),
        })
        .with_active(|s: &TestAppState| s.show_picker);

    host.register(hud_hook);
    host.register(picker_hook);

    // Inside HUD bounds [100..300, 100..150]
    assert!(host.wants_pointer_at(Point::new(150.0, 120.0), workspace, &state));
    // Outside HUD bounds
    assert!(!host.wants_pointer_at(Point::new(50.0, 50.0), workspace, &state));

    // Inside picker bounds [400..550, 400..550], but picker is inactive
    assert!(!host.wants_pointer_at(Point::new(450.0, 450.0), workspace, &state));

    // When picker becomes active
    let active_picker_state = TestAppState {
        show_picker: true,
        ..state
    };
    assert!(host.wants_pointer_at(Point::new(450.0, 450.0), workspace, &active_picker_state));
}

#[test]
fn test_overlay_host_escape_and_outside_click_dismissal() {
    let mut host = OverlayHost::<TestAppState, TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();
    let mut state = TestAppState::default();
    let workspace = Rect::new(0.0, 0.0, 1000.0, 800.0);

    let dropdown_hook = ClosureOverlayHook::new(1, |_, _, _, _| {})
        .with_anchor(OverlayAnchor::Offset {
            offset: Point::new(200.0, 200.0),
            size: (100.0, 100.0),
        })
        .with_dismiss_on_escape(true)
        .with_dismiss_on_outside_click(true)
        .with_on_dismiss(|s: &mut TestAppState, bus| {
            s.dismiss_count += 1;
            bus.emit(TestAction::OverlayDismissed);
        });

    host.register(dropdown_hook);

    // Click inside overlay bounds -> NOT dismissed
    let inside_click = UiEvent::MouseDown {
        point: Point::new(250.0, 250.0),
        button: iris_core::event::MouseButton::Left,
    };
    host.handle_event(&inside_click, workspace, &mut state, &mut bus);
    assert_eq!(state.dismiss_count, 0);
    assert_eq!(bus.len(), 0);

    // Click outside overlay bounds -> triggers on_dismiss
    let outside_click = UiEvent::MouseDown {
        point: Point::new(50.0, 50.0),
        button: iris_core::event::MouseButton::Left,
    };
    host.handle_event(&outside_click, workspace, &mut state, &mut bus);
    assert_eq!(state.dismiss_count, 1);
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::OverlayDismissed]);

    // Press Escape key -> consumes event and triggers on_dismiss
    let escape_key = UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        text: None,
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let consumed = host.handle_event(&escape_key, workspace, &mut state, &mut bus);
    assert!(consumed);
    assert_eq!(state.dismiss_count, 2);
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::OverlayDismissed]);
}

#[test]
fn test_overlay_host_render_and_ui_scope_ext() {
    let mut host = OverlayHost::<TestAppState, TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();
    let mut state = TestAppState {
        active_tool: 7,
        ..Default::default()
    };
    let workspace = Rect::new(0.0, 0.0, 800.0, 600.0);

    let hud = ClosureOverlayHook::new(1, |_scope, _bounds, s: &mut TestAppState, b| {
        b.emit(TestAction::ToolbarClicked(s.active_tool));
    });
    host.register(hud);

    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    scope.overlay_layer(&mut host, workspace, &mut state, &mut bus);

    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions, vec![TestAction::ToolbarClicked(7)]);
}

#[test]
fn test_workbench_ui_scope_ext_dialog_host() {
    let mut host = crate::dialogs::DialogHost::<TestAction>::new();
    let mut bus = ActionBus::<TestAction>::new();
    let workspace = Rect::new(0.0, 0.0, 1024.0, 768.0);

    host.open_confirm(
        "Confirm Action",
        "Are you sure you want to proceed?",
        TestAction::ToolbarClicked(42),
    );
    assert!(host.is_active());

    let mut tree = UiTree::new();
    let root = tree.create_node();
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.dialog_host(&mut host, workspace, false, &mut bus);
    }

    // Verify modal layer nodes were emitted into the tree
    let has_modal_nodes = tree
        .iter()
        .any(|(_, n)| n.layer == iris_core::UiLayer::Modal);
    assert!(
        has_modal_nodes,
        "Dialog host must construct nodes on UiLayer::Modal"
    );

    // Dismiss dialog via Escape simulation
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.dialog_host(&mut host, workspace, true, &mut bus);
    }
    assert!(
        !host.is_active(),
        "Escape key must dismiss cancellable modal dialog"
    );
}

#[test]
fn test_workbench_ui_scope_ext_info_modal_tagged() {
    let workspace = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let desc = InfoCardDialogDescriptor::new("About Dialog")
        .with_app_name("Test App")
        .with_version("1.0.0");

    let mut tree = UiTree::new();
    let root = tree.create_node();
    const MODAL_TAG: u64 = 0x5001;

    // 1. Initially closed
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(outcome.is_active());
    }
    assert!(!tree.is_modal_open(MODAL_TAG));

    // 2. Open modal via tag
    tree.open_modal(MODAL_TAG);
    assert!(tree.is_modal_open(MODAL_TAG));

    // 3. Render modal tagged; modal nodes should be emitted
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(outcome.is_active());
    }
    let has_modal_nodes = tree
        .iter()
        .any(|(_, n)| n.layer == iris_core::UiLayer::Modal);
    assert!(
        has_modal_nodes,
        "info_modal_tagged must emit nodes on UiLayer::Modal when open"
    );

    // 4. Simulate Escape to close modal
    tree.event_state.frame_key = Some(iris_core::KeyCode::Escape);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(!outcome.is_active());
    }
    assert!(
        !tree.is_modal_open(MODAL_TAG),
        "info_modal_tagged must close modal on Escape"
    );
}

#[test]
fn test_two_consecutive_clicks_inside_info_modal() {
    use iris_core::event::{MouseButton, UiEvent};
    use iris_core::geometry::Point;

    let workspace = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let desc = InfoCardDialogDescriptor::new("About Dialog")
        .with_app_name("Test App")
        .with_version("1.0.0");

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    tree.get_mut(root).unwrap().computed_rect = workspace;
    const MODAL_TAG: u64 = 0x5001;

    // 1. Open modal via tag
    tree.open_modal(MODAL_TAG);

    // Initial render
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(outcome.is_active());
    }

    let center = Point::new(960.0, 540.0);

    // Click 1: Mouse down + mouse up
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: center,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: center,
    });

    // Render after Click 1
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(
            outcome.is_active(),
            "Click 1 inside modal must keep modal active"
        );
        assert!(
            tree.is_modal_open(MODAL_TAG),
            "Click 1 inside modal must keep modal open"
        );
    }

    // Click 2: Mouse down + mouse up
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: center,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: center,
    });

    // Render after Click 2
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(
            outcome.is_active(),
            "Click 2 inside modal must keep modal active"
        );
        assert!(
            tree.is_modal_open(MODAL_TAG),
            "Click 2 inside modal must keep modal open"
        );
    }
}

#[test]
fn test_two_consecutive_clicks_inside_info_modal_with_tree_clear() {
    let workspace = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let desc = InfoCardDialogDescriptor::new("About Aeon Engine")
        .with_icon("ℹ")
        .with_app_name("Aeon Engine")
        .with_version("v0.9.0")
        .with_size(480.0, 250.0);

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    tree.get_mut(root).unwrap().computed_rect = workspace;
    const MODAL_TAG: u64 = 0x5001;

    tree.open_modal(MODAL_TAG);

    // Initial render
    {
        let mut scope = UiScope::new(&mut tree, root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(outcome.is_active());
    }

    let center = Point::new(960.0, 540.0);

    // Click 1: Mouse down + mouse up inside the card
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: center,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: center,
    });

    // Simulate shell rebuild with tree.clear()
    tree.clear();
    let new_root = tree.create_root().unwrap();
    tree.get_mut(new_root).unwrap().computed_rect = workspace;

    // Render after Click 1 with reconstructed tree
    {
        let mut scope = UiScope::new(&mut tree, new_root);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(
            outcome.is_active(),
            "Click 1 with tree.clear() must keep modal active"
        );
        assert!(
            tree.is_modal_open(MODAL_TAG),
            "Click 1 with tree.clear() must keep modal open"
        );
    }

    // Click 2: Second click inside the card (e.g. on label or empty area)
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: center,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: center,
    });

    // Simulate shell rebuild with tree.clear() again
    tree.clear();
    let new_root2 = tree.create_root().unwrap();
    tree.get_mut(new_root2).unwrap().computed_rect = workspace;

    // Render after Click 2
    {
        let mut scope = UiScope::new(&mut tree, new_root2);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(
            outcome.is_active(),
            "Click 2 with tree.clear() must keep modal active"
        );
        assert!(
            tree.is_modal_open(MODAL_TAG),
            "Click 2 with tree.clear() must keep modal open"
        );
    }

    // Click 3: Click OUTSIDE on the backdrop scrim (e.g. at (100, 100))
    let outside = Point::new(100.0, 100.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: outside,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: outside,
    });

    // Render after Click 3 (outside)
    {
        let mut scope = UiScope::new(&mut tree, new_root2);
        let outcome = scope.info_modal_tagged(MODAL_TAG, &desc, workspace);
        assert!(
            !outcome.is_active(),
            "Click outside on backdrop scrim must dismiss modal"
        );
        assert!(
            !tree.is_modal_open(MODAL_TAG),
            "Click outside on backdrop scrim must set modal open to false"
        );
    }
}