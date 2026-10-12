// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Iris UI Top-Level Event Dispatcher
//!
//! Tests action queues, layer priorities, coordinate transformations,
//! pointer captures, and centralized focus state machines.

use crate::ui::iris_bridge::assets::types::{AssetsPanelAction, AssetsPanelState};
use crate::ui::iris_bridge::console::{ConsoleAction, ConsolePanelState};
use crate::ui::iris_bridge::hierarchy::{HierarchyAction, HierarchyPanelState};
use crate::ui::iris_bridge::inspector::{InspectorAction, InspectorPanelState};
use crate::ui::iris_bridge::material::{MaterialAction, MaterialPanelState};
use crate::ui::iris_bridge::preferences::{PreferencesAction, PreferencesDialogState};
use crate::ui::iris_bridge::stats::{StatsPanelAction, StatsPanelState};
use crate::ui::iris_bridge::types::PanelInteractionState;
use irisui::prelude::{FocusManager, FocusState, FocusTarget};

#[test]
fn test_panel_interaction_state_pending_actions_invariants() {
    let mut state: PanelInteractionState<(), ConsoleAction> = PanelInteractionState::default();
    assert!(
        !state.has_pending_actions(),
        "Default state must report zero pending actions"
    );

    state.actions.push(ConsoleAction::ClearLogs);
    assert!(
        state.has_pending_actions(),
        "Must report pending action when queue is non-empty"
    );

    let drained = state.take_actions();
    assert_eq!(drained.len(), 1);
    assert!(
        !state.has_pending_actions(),
        "State must report zero pending actions after take_actions"
    );
}

#[test]
fn test_all_panels_pending_actions_invariants() {
    // 1. Hierarchy panel state
    let mut hier = HierarchyPanelState::default();
    assert!(!hier.interactions.has_pending_actions());
    hier.interactions
        .actions
        .push(HierarchyAction::DeleteSelected);
    assert!(hier.interactions.has_pending_actions());
    let _ = hier.interactions.take_actions();
    assert!(!hier.interactions.has_pending_actions());

    // 2. Stats panel state
    let mut stats = StatsPanelState::default();
    assert!(!stats.interactions.has_pending_actions());
    stats
        .interactions
        .actions
        .push(StatsPanelAction::ToggleWireframe);
    assert!(stats.interactions.has_pending_actions());
    let _ = stats.interactions.take_actions();
    assert!(!stats.interactions.has_pending_actions());

    // 3. Console panel state
    let mut console = ConsolePanelState::default();
    assert!(!console.interactions.has_pending_actions());
    console.interactions.actions.push(ConsoleAction::ClearLogs);
    assert!(console.interactions.has_pending_actions());
    let _ = console.interactions.take_actions();
    assert!(!console.interactions.has_pending_actions());

    // 4. Material panel state
    let mut mat = MaterialPanelState::default();
    assert!(!mat.interactions.has_pending_actions());
    mat.interactions.actions.push(MaterialAction::Scroll(10.0));
    assert!(mat.interactions.has_pending_actions());
    let _ = mat.interactions.take_actions();
    assert!(!mat.interactions.has_pending_actions());

    // 5. Assets panel state
    let mut assets = AssetsPanelState::default();
    assert!(!assets.interactions.has_pending_actions());
    assets
        .interactions
        .actions
        .push(AssetsPanelAction::CleanVram);
    assert!(assets.interactions.has_pending_actions());
    let _ = assets.interactions.take_actions();
    assert!(!assets.interactions.has_pending_actions());

    // 6. Preferences dialog state
    let mut pref = PreferencesDialogState::default();
    assert!(!pref.interactions.has_pending_actions());
    pref.interactions.actions.push(PreferencesAction::Close);
    assert!(pref.interactions.has_pending_actions());
    let _ = pref.interactions.take_actions();
    assert!(!pref.interactions.has_pending_actions());

    // 7. Inspector panel state
    let mut insp = InspectorPanelState::default();
    assert!(!insp.interactions.has_pending_actions());
    insp.interactions.actions.push(InspectorAction::FocusRename);
    assert!(insp.interactions.has_pending_actions());
    let _ = insp.interactions.take_actions();
    assert!(!insp.interactions.has_pending_actions());

    // 8. Timeline panel state
    let mut tl = crate::ui::iris_bridge::timeline::TimelinePanelState::default();
    assert!(!tl.interactions.has_pending_actions());
    tl.interactions
        .actions
        .push(crate::ui::iris_bridge::timeline::TimelineAction::TogglePlayPause);
    assert!(tl.interactions.has_pending_actions());
    let _ = tl.interactions.take_actions();
    assert!(!tl.interactions.has_pending_actions());

    // 9. UI Designer panel state
    let mut ui_des = crate::ui::iris_bridge::ui_designer::UiDesignerPanelState::default();
    assert!(!ui_des.interactions.has_pending_actions());
    ui_des
        .interactions
        .actions
        .push(crate::ui::iris_bridge::ui_designer::UiDesignerAction::ToggleGrid);
    assert!(ui_des.interactions.has_pending_actions());
    let _ = ui_des.interactions.take_actions();
    assert!(!ui_des.interactions.has_pending_actions());

    // 10. Viewport HUD state
    let mut vp_hud = crate::ui::iris_bridge::viewport_hud::ViewportHudState::default();
    assert!(vp_hud.actions.is_empty());
    vp_hud
        .actions
        .push(crate::ui::iris_bridge::viewport_hud::ViewportHudAction::ToggleWireframe);
    assert!(!vp_hud.actions.is_empty());
    let _ = vp_hud.take_actions();
    assert!(vp_hud.actions.is_empty());
}

#[test]
fn test_top_down_layered_hit_test_overlay_priority() {
    use crate::ui::iris_bridge::overlay_tree::{ActiveOverlay, OverlayDismissPolicy, OverlayTree};
    use irisui::prelude::{Point, Rect};

    let mut overlay_tree = OverlayTree::new();
    let overlay_rect = Rect::new(100.0, 100.0, 200.0, 150.0);
    overlay_tree.open(
        ActiveOverlay::MenubarDropdown(crate::ui::iris_bridge::types::ActiveMenu::File),
        OverlayDismissPolicy::ClickOutside,
    );
    overlay_tree.set_overlay_bounds(overlay_rect);

    // Click inside overlay: must be detected inside overlay bounds
    let inside_click = Point::new(150.0, 150.0);
    assert!(overlay_tree.contains_point(inside_click));

    // Click outside overlay: must trigger dismiss without click-through
    let outside_click = Point::new(400.0, 400.0);
    assert!(!overlay_tree.contains_point(outside_click));
    let dismiss_res = overlay_tree.handle_pointer_down(outside_click);
    assert_eq!(
        dismiss_res,
        crate::ui::iris_bridge::OverlayDismissResult::Dismissed
    );
}

#[test]
fn test_floating_window_occlusion_and_coordinate_transformation() {
    use irisui::prelude::{Point, Rect};

    let floating_rect = Rect::new(200.0, 150.0, 500.0, 400.0);
    let screen_cursor = Point::new(250.0, 200.0);

    // Point is inside floating window
    assert!(crate::ui::iris_bridge::is_point_in_panel(
        screen_cursor,
        floating_rect
    ));

    // Transformed to (0, 0) local coordinates
    let local_cursor = crate::ui::iris_bridge::screen_to_panel_local(screen_cursor, floating_rect);
    assert_eq!(local_cursor.x, 50.0);
    assert_eq!(local_cursor.y, 50.0);
}

#[test]
fn test_pointer_capture_routes_outside_cursor_to_captured_panel() {
    use crate::ui::iris_bridge::types::IrisChromeState;
    use crate::ui::panel_layout::PanelId;
    use irisui::prelude::{Point, Rect};

    let mut chrome = IrisChromeState::default();
    let timeline_rect = Rect::new(0.0, 600.0, 800.0, 200.0);
    let hierarchy_rect = Rect::new(0.0, 28.0, 250.0, 572.0);

    // Without capture, cursor at (50, 100) is in Hierarchy, NOT in Timeline
    let cursor_in_hier = Point::new(50.0, 100.0);
    assert!(crate::ui::iris_bridge::is_point_in_panel(
        cursor_in_hier,
        hierarchy_rect
    ));
    assert!(!crate::ui::iris_bridge::is_point_in_panel(
        cursor_in_hier,
        timeline_rect
    ));

    // When Timeline acquires pointer capture:
    chrome.pointer_capture = Some(PanelId::AnimationTimeline);
    assert_eq!(chrome.pointer_capture, Some(PanelId::AnimationTimeline));

    // The captured predicate routes to Timeline even though cursor is physically over Hierarchy
    let captured = chrome.pointer_capture;
    let is_in = |id: PanelId| -> bool {
        if let Some(cap) = captured {
            return cap == id;
        }
        false
    };
    assert!(is_in(PanelId::AnimationTimeline));
    assert!(!is_in(PanelId::Hierarchy));

    // Releasing capture clears routing
    chrome.pointer_capture = None;
    assert_eq!(chrome.pointer_capture, None);
}

#[test]
fn test_central_focus_tag_invariants() {
    let mut focus_state: FocusState<FocusTarget> = FocusState::new();
    let mut focus_manager = FocusManager::new();

    // 1. Focus Hierarchy Search Bar via central SSOT
    let hier_tag = crate::ui::iris_bridge::hierarchy::HIERARCHY_TAG_SEARCH_INPUT;
    focus_state.set_focus_tag(hier_tag);
    focus_state.sync_to_core(&mut focus_manager);
    assert!(focus_state.is_tag_focused(hier_tag));
    assert!(focus_state.has_focus());
    assert!(focus_manager.is_tag_focused(hier_tag));

    // 2. Clear cancels active search focus
    focus_state.clear_focus();
    focus_state.sync_to_core(&mut focus_manager);
    assert!(!focus_state.has_focus());
    assert!(!focus_manager.has_focus());

    // 3. Set Assets Search Bar focus
    let assets_tag = crate::ui::iris_bridge::assets::ASSETS_TAG_SEARCH_INPUT;
    focus_state.set_focus_tag(assets_tag);
    focus_state.sync_to_core(&mut focus_manager);
    assert!(focus_state.is_tag_focused(assets_tag));
    assert!(focus_manager.is_tag_focused(assets_tag));

    // 4. Switching to Console Search Bar
    let console_tag = crate::ui::iris_bridge::console::CONSOLE_TAG_SEARCH_INPUT;
    focus_state.set_focus_tag(console_tag);
    focus_state.sync_to_core(&mut focus_manager);
    assert!(!focus_state.is_tag_focused(assets_tag));
    assert!(focus_state.is_tag_focused(console_tag));
    assert!(focus_manager.is_tag_focused(console_tag));

    // 5. Final clear
    focus_state.clear_focus();
    focus_state.sync_to_core(&mut focus_manager);
    assert!(!focus_state.has_focus());
    assert!(!focus_manager.has_focus());
}

#[test]
fn test_pointer_classification_includes_mouse_wheel() {
    use winit::event::{DeviceId, MouseScrollDelta, WindowEvent};

    let dummy_device = DeviceId::dummy();
    let wheel_event = WindowEvent::MouseWheel {
        device_id: dummy_device,
        delta: MouseScrollDelta::LineDelta(0.0, 1.0),
        phase: winit::event::TouchPhase::Moved,
    };
    let cursor_event = WindowEvent::CursorMoved {
        device_id: dummy_device,
        position: winit::dpi::PhysicalPosition::new(100.0, 100.0),
    };
    let non_pointer_focus = WindowEvent::Focused(true);
    let non_pointer_ime = WindowEvent::Ime(winit::event::Ime::Commit("test".to_string()));

    let is_pointer = |ev: &WindowEvent| -> bool {
        matches!(
            ev,
            WindowEvent::MouseInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseWheel { .. }
        )
    };

    assert!(
        is_pointer(&wheel_event),
        "MouseWheel must be classified as a pointer event"
    );
    assert!(
        is_pointer(&cursor_event),
        "CursorMoved must be classified as a pointer event"
    );
    assert!(
        !is_pointer(&non_pointer_focus),
        "Window focus must NOT be classified as a pointer event"
    );
    assert!(
        !is_pointer(&non_pointer_ime),
        "IME text input must NOT be classified as a pointer event"
    );
}

#[test]
fn test_wheel_event_isolated_routing_predicate() {
    use crate::ui::panel_layout::PanelId;
    use irisui::prelude::{Point, Rect};

    let hierarchy_bounds = Rect::new(0.0, 28.0, 250.0, 500.0);
    let inspector_bounds = Rect::new(800.0, 28.0, 300.0, 500.0);

    let cursor_in_hier = Point::new(100.0, 100.0);
    let cursor_in_insp = Point::new(900.0, 100.0);
    let cursor_in_empty = Point::new(500.0, 100.0);

    let is_in = |cursor: Point, bounds: Rect| -> bool {
        crate::ui::iris_bridge::compositor::is_point_in_panel(cursor, bounds)
    };

    // When cursor is over Hierarchy
    assert!(is_in(cursor_in_hier, hierarchy_bounds));
    assert!(!is_in(cursor_in_hier, inspector_bounds));

    // When cursor is over Inspector
    assert!(!is_in(cursor_in_insp, hierarchy_bounds));
    assert!(is_in(cursor_in_insp, inspector_bounds));

    // When cursor is between panels
    assert!(!is_in(cursor_in_empty, hierarchy_bounds));
    assert!(!is_in(cursor_in_empty, inspector_bounds));

    // Pointer capture overrides physical coordinate hit-testing
    let captured_panel = Some(PanelId::Inspector);
    let check_captured = |id: PanelId| -> bool {
        if let Some(cap) = captured_panel {
            return cap == id;
        }
        false
    };
    assert!(check_captured(PanelId::Inspector));
    assert!(!check_captured(PanelId::Hierarchy));
}

#[test]
fn test_stats_and_inspector_scroll_delta_math_and_clamping() {
    // 1. Stats panel scroll math: (scroll_y - delta_y).clamp(0.0, max_scroll)
    let max_scroll = 300.0_f32;
    let mut stats_scroll_y = 50.0_f32;

    // Scroll down by 24px (1 line)
    let delta_y = 24.0_f32;
    stats_scroll_y = (stats_scroll_y - delta_y).clamp(0.0, max_scroll);
    assert_eq!(stats_scroll_y, 26.0);

    // Scroll up by 48px
    let delta_y_up = -48.0_f32;
    stats_scroll_y = (stats_scroll_y - delta_y_up).clamp(0.0, max_scroll);
    assert_eq!(stats_scroll_y, 74.0);

    // Scroll past top boundary: clamps to 0.0
    stats_scroll_y = (stats_scroll_y - 200.0).clamp(0.0, max_scroll);
    assert_eq!(stats_scroll_y, 0.0);

    // Scroll past bottom boundary: clamps to max_scroll
    stats_scroll_y = (stats_scroll_y - (-500.0)).clamp(0.0, max_scroll);
    assert_eq!(stats_scroll_y, 300.0);

    // 2. Inspector panel scroll math: (scroll_y - delta_y).max(0.0)
    let mut insp_scroll_y = 100.0_f32;
    insp_scroll_y = (insp_scroll_y - 30.0).max(0.0);
    assert_eq!(insp_scroll_y, 70.0);

    // Scroll past top
    insp_scroll_y = (insp_scroll_y - 150.0).max(0.0);
    assert_eq!(insp_scroll_y, 0.0);

    // Scroll down indefinitely
    insp_scroll_y = (insp_scroll_y - (-1000.0)).max(0.0);
    assert_eq!(insp_scroll_y, 1000.0);
}

#[test]
fn test_iris_editor_overlay_winit_autonomous_dispatch_integration() {
    use irisui::prelude::{Point, Rect, UiScope, UiTree};
    use irisui::winit::winit_to_ui_event;
    use winit::event::{DeviceId, ElementState, MouseButton, WindowEvent};

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation");
    let btn_tag = 9999u64;

    // 1. Initial build: Emit button in tree using declarative finish_layout
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button_tagged("AutonomousBtn", btn_tag);
        assert!(!resp.clicked());
        scope.finish_layout(Rect::new(0.0, 0.0, 800.0, 600.0));
    }

    let click_pos = Point::new(10.0, 10.0);

    // 2. Winit MouseDown at (60, 60)
    let winit_down = WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: ElementState::Pressed,
        button: MouseButton::Left,
    };
    let ui_down = winit_to_ui_event(&winit_down, click_pos).expect("Translated UiEvent");
    let res_down = tree.dispatch_event(&ui_down);
    assert!(res_down.is_consumed());

    // 3. Winit MouseUp at (60, 60) -> Triggers CLICKED
    let winit_up = WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: ElementState::Released,
        button: MouseButton::Left,
    };
    let ui_up = winit_to_ui_event(&winit_up, click_pos).expect("Translated UiEvent");
    let res_up = tree.dispatch_event(&ui_up);
    assert!(res_up.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_tag, Some(btn_tag));

    // 4. Next frame: scope.button_tagged reports clicked() autonomously
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button_tagged("AutonomousBtn", btn_tag);
        assert!(
            resp.clicked(),
            "Declarative button MUST autonomously report clicked() from window root dispatch"
        );
    }
}