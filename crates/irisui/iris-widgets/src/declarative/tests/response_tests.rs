// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Declarative Button and Checkbox Interaction Responses (`iris-widgets::declarative::tests::response_tests`)
//!
//! Verifies immediate reactive interaction states ([`WidgetResponse`]) produced by
//! declarative buttons and checkboxes during autonomous event dispatch cycles.

use crate::declarative::{
    UiScope,
    widget::{Button, Checkbox, NumericDrag},
};
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;

#[test]
fn test_declarative_button_click_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    // 1. Initial build: render button in tree without any events
    let btn_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(
            !resp.clicked(),
            "New button must not report clicked on clean tree"
        );
        assert!(
            !resp.hovered(),
            "New button must not report hovered on clean tree"
        );
        assert!(
            !resp.pressed(),
            "New button must not report pressed on clean tree"
        );
        resp.id
    };

    // Assign a valid bounding box to the button for spatial hit-testing
    let btn_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
    if let Some(node) = tree.get_mut(btn_id) {
        node.computed_rect = btn_rect;
    }

    let click_pt = Point::new(60.0, 60.0);

    // 2. MouseMove over button -> Autonomous hover detection
    let move_res = tree.dispatch_event(&UiEvent::MouseMove { point: click_pt });
    assert!(move_res.target == Some(btn_id));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(
            resp.hovered(),
            "Button must report hovered() == true when pointer is over it"
        );
        assert!(
            !resp.pressed(),
            "Button must not report pressed on simple mouse move"
        );
        assert!(
            !resp.clicked(),
            "Button must not report clicked on mouse move"
        );
    }

    // 3. MouseDown on button -> Autonomous active pressed state
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(down_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(resp.hovered(), "Button must remain hovered during press");
        assert!(
            resp.pressed(),
            "Button must report pressed() == true during active mouse press"
        );
        assert!(
            !resp.clicked(),
            "Button must not report clicked before button release"
        );
    }

    // 4. MouseUp on button -> Confirmed click gesture!
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(up_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Kaydet");
        assert!(
            resp.clicked(),
            "Button MUST report clicked() == true upon mouse release confirmation"
        );
        assert!(
            !resp.pressed(),
            "Button must clear pressed state on release"
        );
    }
}

#[test]
fn test_declarative_checkbox_click_and_toggle_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let mut is_active = false;

    // 1. Initial build: unchecked checkbox
    let chk_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Otomatik Kaydet", &mut is_active);
        assert!(!resp.clicked());
        assert!(!resp.changed());
        assert!(!is_active);
        resp.id
    };

    let chk_rect = Rect::new(10.0, 10.0, 140.0, 24.0);
    if let Some(node) = tree.get_mut(chk_id) {
        node.computed_rect = chk_rect;
    }

    let click_pt = Point::new(20.0, 20.0);

    // 2. Perform click sequence on checkbox: MouseDown -> MouseUp
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(up_res.is_consumed());

    // 3. Render scope with checkbox: must toggle boolean in-place and report clicked & changed
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Otomatik Kaydet", &mut is_active);
        assert!(resp.clicked(), "Checkbox must report clicked() == true");
        assert!(resp.changed(), "Checkbox must report changed() == true");
        assert!(
            is_active,
            "Checkbox underlying boolean MUST be toggled to true in-place"
        );
    }

    // 4. Click again: should toggle back to false
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Otomatik Kaydet", &mut is_active);
        assert!(
            resp.clicked(),
            "Checkbox must report clicked() == true on second click"
        );
        assert!(
            resp.changed(),
            "Checkbox must report changed() == true on second click"
        );
        assert!(
            !is_active,
            "Checkbox underlying boolean MUST be toggled back to false"
        );
    }
}

#[test]
fn test_declarative_checkbox_val_response() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let is_active = false;

    // 1. Initial build: read-only checkbox
    let chk_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox_val("Read Only Checkbox", is_active);
        assert!(!resp.clicked());
        assert!(!is_active);
        resp.id
    };

    let chk_rect = Rect::new(10.0, 10.0, 140.0, 24.0);
    if let Some(node) = tree.get_mut(chk_id) {
        node.computed_rect = chk_rect;
    }

    let click_pt = Point::new(20.0, 20.0);

    // 2. Perform click sequence on checkbox: MouseDown -> MouseUp
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });
    assert!(up_res.is_consumed());

    // 3. Render scope with checkbox_val: reports clicked without mutating any dummy variable
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox_val("Read Only Checkbox", is_active);
        assert!(resp.clicked(), "Checkbox_val must report clicked() == true");
        assert!(!is_active, "Underlying value must remain unchanged");
    }
}

#[test]
fn test_declarative_numeric_drag_changed_response() {
    use iris_core::WidgetCursor;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let mut scale = 1.0f32;

    // 1. Initial build: render numeric scrubber in tree without interaction
    let drag_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(
            !resp.changed(),
            "Clean drag scrubber must not report changed"
        );
        assert!(
            !resp.dragged(),
            "Clean drag scrubber must not report dragged"
        );
        assert!(
            !resp.hovered(),
            "Clean drag scrubber must not report hovered"
        );
        assert_eq!(scale, 1.0);
        resp.id
    };

    // Assign computed rect for spatial hit-testing
    let drag_rect = Rect::new(20.0, 20.0, 100.0, 30.0);
    if let Some(node) = tree.get_mut(drag_id) {
        node.computed_rect = drag_rect;
    }

    let start_pt = Point::new(30.0, 25.0);

    // 2. MouseMove over scrubber -> Hovered and Default cursor (no resize until drag begins)
    let move_res = tree.dispatch_event(&UiEvent::MouseMove { point: start_pt });
    assert_eq!(move_res.target, Some(drag_id));
    assert_eq!(move_res.cursor, Some(WidgetCursor::Default));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(resp.hovered(), "Scrubber must report hovered == true");
        assert!(!resp.dragged());
        assert!(!resp.changed());
        assert_eq!(scale, 1.0);
    }

    // 3. MouseDown on scrubber -> Initiates pointer capture for  scrubbing
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: start_pt,
    });
    assert!(down_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture,
        Some(drag_id),
        "Numeric scrubber must capture pointer on MouseDown"
    );

    // 4. MouseMove displacing pointer horizontally by +30.0px
    let moved_pt = Point::new(60.0, 25.0);
    let drag_move_res = tree.dispatch_event(&UiEvent::MouseMove { point: moved_pt });
    assert!(drag_move_res.is_consumed());
    assert_eq!(drag_move_res.cursor, Some(WidgetCursor::EwResize));

    // Next frame declarative rendering: scope.numeric_drag must mutate scale and report changed()
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(
            resp.dragged(),
            "Numeric scrubber must report dragged() == true while active"
        );
        assert!(
            resp.changed(),
            "Numeric scrubber must report changed() == true upon pointer displacement"
        );
        // Default speed is 0.1: delta_x = 30.0 * 0.1 = +3.0 -> scale becomes 4.0
        assert!(
            (scale - 4.0).abs() < 1e-4,
            "Scale value should have incremented by delta_x * speed (expected 4.0, got {})",
            scale
        );
    }

    // 5. MouseUp releasing pointer capture and finalizing scrubber interaction
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: moved_pt,
    });
    assert!(up_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture, None,
        "Pointer capture must be released on MouseUp"
    );
    assert_eq!(
        tree.event_state.last_pointer_pos, None,
        "Last pointer position must be reset on MouseUp"
    );

    // Frame after release: no longer dragging or changing
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.numeric_drag("Scale", &mut scale);
        assert!(!resp.dragged(), "Should no longer be dragged after MouseUp");
        assert!(!resp.changed(), "Should not report changed without delta");
        assert_eq!(scale, 4.0, "Scale must retain its updated value");
    }
}

#[test]
fn test_declarative_text_input_autonomous_focus_and_text() {
    use iris_core::event::KeyCode;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let mut search_query = String::new();

    // 1. Initial build: render text input in tree with empty buffer
    let input_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(!resp.focused(), "Clean text input must not report focused");
        assert!(!resp.changed(), "Clean text input must not report changed");
        assert!(!resp.clicked(), "Clean text input must not report clicked");
        resp.id
    };

    // Assign computed rect for spatial hit-testing
    let input_rect = Rect::new(10.0, 10.0, 200.0, 24.0);
    if let Some(node) = tree.get_mut(input_id) {
        node.computed_rect = input_rect;
    }

    let click_pt = Point::new(20.0, 20.0);

    // 2. MouseMove over input -> Hovered state
    tree.dispatch_event(&UiEvent::MouseMove { point: click_pt });
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(resp.hovered(), "Text input must report hovered == true");
        assert!(!resp.focused());
    }

    // 3. Click input: MouseDown + MouseUp on input -> Gains keyboard focus
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: click_pt,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: click_pt,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(resp.clicked(), "Text input must report clicked == true");
        assert!(
            resp.focused(),
            "Text input must report focused == true after click"
        );
        assert!(!resp.changed());
    }

    // 4. Typing character 'A' while focused -> In-place buffer update and changed flag
    let key_res = tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Character('A'),
        text: Some("A".to_string()),
        is_pressed: true,
    });
    assert!(key_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(
            resp.changed(),
            "Text input must report changed == true upon typing"
        );
        assert!(resp.focused());
        assert_eq!(search_query, "A");
    }

    // 5. Typing second character 'e'
    tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Character('e'),
        text: Some("e".to_string()),
        is_pressed: true,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(resp.changed());
        assert_eq!(search_query, "Ae");
    }

    // 5b. Key release event for 'e' -> Must NOT duplicate input (prevent double-typing)
    let release_res = tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Character('e'),
        text: Some("e".to_string()),
        is_pressed: false,
    });
    assert!(release_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(!resp.changed(), "Key release must NOT trigger text change");
        assert_eq!(
            search_query, "Ae",
            "Text must not be duplicated on key release"
        );
    }

    // 6. Backspace key -> Pop last character in-place and report changed
    let backspace_res = tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Backspace,
        text: None,
        is_pressed: true,
    });
    assert!(backspace_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(
            resp.changed(),
            "Text input must report changed == true on Backspace"
        );
        assert_eq!(search_query, "A");
    }

    // 7. Clicking outside input on empty screen area -> Clears focus
    let outside_pt = Point::new(400.0, 400.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: outside_pt,
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(
            !resp.focused(),
            "Text input must clear focus when clicking outside"
        );
        assert!(!resp.changed());
        assert_eq!(search_query, "A");
    }

    // 8. Key press while unfocused -> Must NOT update buffer or report changed
    let unfocused_res = tree.dispatch_event(&UiEvent::KeyboardInput {
        key: KeyCode::Character('z'),
        text: Some("z".to_string()),
        is_pressed: true,
    });
    assert!(
        !unfocused_res.is_consumed(),
        "Unfocused keyboard event must be ignored"
    );

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input("Arama...", &mut search_query);
        assert!(!resp.changed());
        assert_eq!(
            search_query, "A",
            "Buffer must not be mutated when unfocused"
        );
    }
}

#[test]
fn test_declarative_tree_row_autonomous_interaction() {
    use crate::declarative::scope::{TreeRowProps, TreeViewConfig};
    use crate::tree_view::TreeState;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");
    let config = TreeViewConfig::default();

    // -------------------------------------------------------------
    // Part A: tree_row_toggle in-place boolean expansion & click isolation
    // -------------------------------------------------------------
    let mut is_expanded = false;

    // 1. Initial build: clean state
    let (row_id, foldout_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let props = TreeRowProps::new(0, true, is_expanded, false, 100, 101);
        let resp = scope.tree_row_toggle("RowToggle", props, &mut is_expanded, &config, |_| {});
        assert!(!resp.clicked(), "Clean tree row must not report clicked");
        assert!(
            !resp.foldout_clicked(),
            "Clean foldout chevron must not report foldout_clicked"
        );
        assert!(!resp.is_expanded);
        assert!(!resp.is_selected);
        (
            resp.id(),
            resp.foldout_id()
                .expect("Foldout chevron must be present for parent row"),
        )
    };

    // Assign computed rects for spatial hit-testing
    // Row bounds: [0, 0, 200, 24], Foldout bounds: [0, 0, 20, 24]
    if let Some(node) = tree.get_mut(row_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }
    if let Some(node) = tree.get_mut(foldout_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 20.0, 24.0);
    }

    // 2. Click on foldout chevron (x = 10.0, y = 12.0)
    let chevron_pt = Point::new(10.0, 12.0);
    let down_chevron = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: chevron_pt,
    });
    assert!(down_chevron.is_consumed());
    assert_eq!(down_chevron.target, Some(foldout_id));

    let up_chevron = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: chevron_pt,
    });
    assert!(up_chevron.is_consumed());

    // Next frame: verify foldout_clicked toggled is_expanded in-place without triggering row.clicked
    {
        let mut scope = UiScope::new(&mut tree, root);
        let props = TreeRowProps::new(0, true, is_expanded, false, 100, 101);
        let resp = scope.tree_row_toggle("RowToggle", props, &mut is_expanded, &config, |_| {});
        assert!(
            resp.foldout_clicked(),
            "Foldout click must report foldout_clicked() == true"
        );
        assert!(
            !resp.clicked(),
            "Chevron click must NOT trigger body clicked() == true"
        );
        assert!(
            is_expanded,
            "tree_row_toggle MUST autonomously toggle is_expanded to true"
        );
    }

    // 3. Click on row body outside foldout (x = 80.0, y = 12.0)
    let body_pt = Point::new(80.0, 12.0);
    let down_body = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: body_pt,
    });
    assert!(down_body.is_consumed());
    assert_eq!(down_body.target, Some(row_id));

    let up_body = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: body_pt,
    });
    assert!(up_body.is_consumed());

    // Next frame: verify row body clicked without mutating chevron expansion
    {
        let mut scope = UiScope::new(&mut tree, root);
        let props = TreeRowProps::new(0, true, is_expanded, false, 100, 101);
        let resp = scope.tree_row_toggle("RowToggle", props, &mut is_expanded, &config, |_| {});
        assert!(
            resp.clicked(),
            "Body click must report row clicked() == true"
        );
        assert!(
            !resp.foldout_clicked(),
            "Body click must NOT report foldout_clicked() == true"
        );
        assert!(is_expanded, "Body click must NOT toggle expansion state");
    }

    // -------------------------------------------------------------
    // Part B: tree_node high-level autonomous TreeState synchronization
    // -------------------------------------------------------------
    use crate::declarative::scope::TreeNodeProps;
    let mut state = TreeState::<u64>::new();
    let node_id = 42u64;

    assert!(!state.is_expanded(&node_id));
    assert!(!state.is_selected(&node_id));

    let node_props = TreeNodeProps::new("Entity42", 0, true, &config);

    // 1. Initial build: tree_node emits row and foldout
    let (node_row_id, node_foldout_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.tree_node(&mut state, &node_id, node_props, |_| {});
        assert!(!resp.clicked());
        assert!(!resp.foldout_clicked());
        (
            resp.id(),
            resp.foldout_id()
                .expect("Foldout chevron must be present for parent node"),
        )
    };

    // Assign computed rects for Entity42
    if let Some(node) = tree.get_mut(node_row_id) {
        node.computed_rect = Rect::new(0.0, 30.0, 200.0, 24.0);
    }
    if let Some(node) = tree.get_mut(node_foldout_id) {
        node.computed_rect = Rect::new(0.0, 30.0, 20.0, 24.0);
    }

    // 2. Click chevron for Entity42 (x = 10.0, y = 42.0)
    let chevron_node_pt = Point::new(10.0, 42.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: chevron_node_pt,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: chevron_node_pt,
    });

    // Next frame: verify state.toggle_expanded was called autonomously
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.tree_node(&mut state, &node_id, node_props, |_| {});
        assert!(resp.foldout_clicked());
        assert!(!resp.clicked());
        assert!(
            state.is_expanded(&node_id),
            "TreeState MUST reflect expansion toggle autonomously"
        );
        assert!(
            !state.is_selected(&node_id),
            "Chevron click MUST NOT select the node in TreeState"
        );
    }

    // 3. Click body for Entity42 (x = 70.0, y = 42.0)
    let body_node_pt = Point::new(70.0, 42.0);
    tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: body_node_pt,
    });
    tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: body_node_pt,
    });

    // Next frame: verify state.select_single was called autonomously
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.tree_node(&mut state, &node_id, node_props, |_| {});
        assert!(resp.clicked());
        assert!(!resp.foldout_clicked());
        assert!(
            state.is_selected(&node_id),
            "TreeState MUST autonomously mark item as selected on body click"
        );
        assert!(
            state.is_expanded(&node_id),
            "Body click MUST preserve existing expansion state"
        );
    }
}

#[test]
fn test_declarative_instant_effective_state_on_click_frame() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let tag_pill = 0x9001;
    let tag_toggle = 0x9002;
    let tag_check = 0x9003;

    // 1. Initial build: render elements with inactive state
    let (_pill_id, _toggle_id, check_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let pill_resp = scope.filter_pill_tagged(
            "Errors (0)",
            false,
            iris_core::Color::rgba(0.95, 0.3, 0.3, 1.0),
            70.0,
            tag_pill,
        );
        let toggle_resp = scope.add(Button::new("Auto-Scroll").small().tag(tag_toggle));
        let check_resp = scope.add(Checkbox::new("Wireframe", false).tag(tag_check));
        (pill_resp.id, toggle_resp.id, check_resp.id)
    };

    // Verify initial inactive checkbox indicator and label
    let chk_node = tree.get(check_id).unwrap();
    assert_eq!(chk_node.children.len(), 2);
    let box_node = tree.get(chk_node.children[0]).unwrap();
    assert_eq!(box_node.text.as_deref(), Some(""));
    let lbl_node = tree.get(chk_node.children[1]).unwrap();
    assert_eq!(lbl_node.text.as_deref(), Some("Wireframe"));

    // 2. Simulate click on pill, toggle, and checkbox via frame_clicked_tag
    tree.event_state.frame_clicked_tag = Some(tag_check);

    // Rebuild frame with check clicked: verify checkmark appears on the SAME frame!
    {
        let mut scope = UiScope::new(&mut tree, root);
        let check_resp = scope.add(Checkbox::new("Wireframe", false).tag(tag_check));
        assert!(
            check_resp.clicked(),
            "Checkbox must report clicked() == true"
        );
        let chk_id = check_resp.id;
        let chk_node = scope.tree().get(chk_id).unwrap();
        assert_eq!(chk_node.children.len(), 2);
        let box_node = scope.tree().get(chk_node.children[0]).unwrap();
        assert_eq!(
            box_node.text.as_deref(),
            Some("✓"),
            "Checkbox indicator MUST reflect checkmark on click frame even if passed checked=false"
        );
        assert_eq!(
            box_node.style.background_color,
            scope.tree().theme().button_active_bg,
            "Checkbox indicator MUST use active accent background color on click frame"
        );
        let lbl_node = scope.tree().get(chk_node.children[1]).unwrap();
        assert_eq!(lbl_node.text.as_deref(), Some("Wireframe"));
    }

    // 3. Simulate click on filter pill: verify active white text on the SAME frame!
    tree.event_state.frame_clicked_tag = Some(tag_pill);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let pill_resp = scope.filter_pill_tagged(
            "Errors (0)",
            false,
            iris_core::Color::rgba(0.95, 0.3, 0.3, 1.0),
            70.0,
            tag_pill,
        );
        assert!(
            pill_resp.clicked(),
            "Filter pill must report clicked() == true"
        );
        let pill_id = pill_resp.id;
        let node = scope.tree().get(pill_id).unwrap();
        assert_eq!(
            node.text_color,
            iris_core::Color::WHITE,
            "Filter pill MUST use active white text on click frame even if passed is_active=false"
        );
    }
}

#[test]
fn test_declarative_numeric_drag_pointer_capture_across_rebuilds_and_boundaries() {
    use crate::declarative::widget::NumericDrag;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut radius = 10.0f32;
    let drag_tag = 0xABCD_1234_5678_u64;

    // 1. Initial build
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Radius", &mut radius)
                .tag(drag_tag)
                .range(0.0..=100.0)
                .speed(1.0),
        );
        assert!(!resp.dragged());
    }

    // Assign computed rect for spatial hit testing
    let pill_id = tree.find_node_by_tag(drag_tag).expect("Pill must exist");
    if let Some(node) = tree.get_mut(pill_id) {
        node.computed_rect = Rect::new(50.0, 50.0, 60.0, 22.0);
    }

    // 2. MouseDown within pill boundaries (x=60, y=60) -> initiates pointer capture
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(60.0, 60.0),
    });
    assert!(down_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture_tag,
        Some(drag_tag),
        "Numeric scrubber must initiate persistent pointer capture tag"
    );

    // 3. Mouse moves way outside widget boundaries (x=1000, y=1000)
    let move_res = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(100.0, 60.0), // +40px delta
    });
    assert!(move_res.is_consumed());

    // 4. Rebuild scope (simulating next frame re-render)
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Radius", &mut radius)
                .tag(drag_tag)
                .range(0.0..=100.0)
                .speed(1.0),
        );
        assert!(resp.dragged(), "Scrubber must report dragged() == true");
        assert!(resp.changed(), "Scrubber must report changed() == true");
        assert_eq!(radius, 50.0, "Radius must increase by delta (10 + 40)");
    }

    // 5. MouseUp releases pointer capture
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(100.0, 60.0),
    });
    assert!(up_res.is_consumed());
    assert_eq!(tree.event_state.pointer_capture, None);
    assert_eq!(tree.event_state.pointer_capture_tag, None);

    // 6. Next frame: clean state
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            NumericDrag::new("Radius", &mut radius)
                .tag(drag_tag)
                .range(0.0..=100.0)
                .speed(1.0),
        );
        assert!(!resp.dragged());
        assert!(!resp.changed());
        assert_eq!(radius, 50.0);
    }
}

#[test]
fn test_numeric_input_child_node_click_bubbles_capture() {
    use iris_core::WidgetRole;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    // Parent NumericInput node with tag
    let num_id = tree.create_node();
    let _ = tree.add_child(root, num_id);
    let num_tag = 0x9999_8888_7777_u64;
    if let Some(node) = tree.get_mut(num_id) {
        node.set_name("NumericParent");
        node.role = WidgetRole::NumericInput;
        node.tag = num_tag;
        node.interactive = true;
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 30.0);
    }

    // Child text label inside the NumericInput (tag 0, not a button)
    let child_id = tree.create_node();
    let _ = tree.add_child(num_id, child_id);
    if let Some(child) = tree.get_mut(child_id) {
        child.name = Some("InnerNumericLabel".to_string());
        child.role = WidgetRole::Default;
        child.interactive = false;
        child.computed_rect = Rect::new(20.0, 15.0, 50.0, 20.0);
    }

    // Click on the child label
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(25.0, 20.0),
    });
    assert!(down_res.is_consumed());
    assert_eq!(
        tree.event_state.pointer_capture,
        Some(num_id),
        "Click on child inside NumericInput must bubble capture to parent NumericInput"
    );
    assert_eq!(
        tree.event_state.pointer_capture_tag,
        Some(num_tag),
        "Pointer capture tag must reflect the parent NumericInput's semantic tag"
    );
}

/// Verifies that micro-motion (<=3.0px) during a press on [`NumericDrag`] confirms a click gesture
/// rather than triggering premature scrubbing, and that drag threshold (>3.0px) activates dragging.
#[test]
fn test_numeric_input_click_vs_drag_threshold_isolation() {
    use crate::declarative::widget::numeric_drag::NumericDrag;
    use iris_core::WidgetCursor;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");
    let mut val = 42.0_f32;
    let tag = 0xbeef_1234_5678_u64;

    // 1. Initial build
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
    }
    let pill_id = tree.find_node_by_tag(tag).expect("Pill must exist");
    if let Some(node) = tree.get_mut(pill_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 80.0, 24.0);
    }

    // 2. MouseDown within pill
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    });
    assert!(down_res.is_consumed());
    assert!(!tree.event_state.is_pointer_drag_active);

    // 3. Micro-move (1.0px delta) -> should NOT trigger drag or EwResize cursor
    let micro_move_res = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(21.0, 20.0),
    });
    assert!(micro_move_res.is_consumed());
    assert!(!tree.event_state.is_pointer_drag_active);
    assert_ne!(
        micro_move_res.cursor,
        Some(WidgetCursor::EwResize),
        "Micro-motion must not display EwResize cursor"
    );
    assert!(tree.event_state.frame_drag_delta.is_none());

    // 4. MouseUp within threshold -> MUST confirm click and start active_numeric_edit
    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(21.0, 20.0),
    });
    assert!(up_res.is_consumed());
    assert!(
        up_res.response.is_some_and(|r| r.clicked()),
        "Release within threshold must register as a CLICK"
    );
    assert_eq!(tree.event_state.frame_clicked_tag, Some(tag));

    // 5. Subsequent scope build enters editing mode
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.clicked());
        assert!(tree.event_state.active_numeric_edit.is_some());
    }
}

#[test]
fn test_numeric_drag_typing_buffering_and_enter_commit() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let tag = 4242u64;
    let mut val = 100.0f32;

    // 1. Initial click enters active edit session
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.clicked());
        assert!(tree.event_state.active_numeric_edit.is_some());
    }

    // 2. Typing '5' replaces buffer (all selected), val MUST NOT CHANGE yet
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = Some("5".to_string());
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(!resp.changed(), "Typing must NOT apply value immediately");
        assert_eq!(val, 100.0, "Original value must be preserved while typing");
    }

    // 3. Typing '-10' into the buffer, val MUST STILL NOT CHANGE
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = Some("-10".to_string());
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(!resp.changed(), "Typing must NOT apply value immediately");
        assert_eq!(val, 100.0, "Original value must be preserved while typing");
    }

    // Verify buffer holds "5-10"
    assert_eq!(
        tree.event_state
            .active_numeric_edit
            .as_ref()
            .map(|(_, b, ..)| b.as_str()),
        Some("5-10")
    );

    // 4. Press Enter -> MUST calculate 5-10 = -5.0, set changed=true, and close edit session
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = None;
    tree.event_state.frame_key = Some(iris_core::event::KeyCode::Enter);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.changed(), "Enter MUST commit the numeric edit");
        assert_eq!(val, -5.0, "Math expression 5-10 must evaluate to -5.0");
        assert!(
            tree.event_state.active_numeric_edit.is_none(),
            "Enter must close editing"
        );
    }
}

#[test]
fn test_numeric_drag_blur_commit() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let tag = 5555u64;
    let mut val = 0.0f32;

    // 1. Initial click enters active edit session
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
    }

    // 2. Type "20+15"
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_key_pressed = true;
    tree.event_state.frame_text_input = Some("20+15".to_string());
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
    }
    assert_eq!(val, 0.0);

    // 3. Click outside on untagged container
    tree.event_state.frame_key_pressed = false;
    tree.event_state.frame_text_input = None;
    tree.event_state.frame_clicked_tag = None;
    tree.event_state.frame_clicked_id = Some(root);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(NumericDrag::new("TestVal", &mut val).tag(tag));
        assert!(resp.changed(), "Blur MUST commit the numeric edit");
        assert_eq!(val, 35.0, "Math expression 20+15 must evaluate to 35.0");
        assert!(
            tree.event_state.active_numeric_edit.is_none(),
            "Blur must close editing"
        );
    }
}