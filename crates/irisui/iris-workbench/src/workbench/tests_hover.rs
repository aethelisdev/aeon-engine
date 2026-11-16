// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Layered Pointer Hover Resolution
//!
//! Validates `iris-workbench::pointer::resolve_hover_targets`, overlay occlusion
//! (`UiLayer::Floating` / `UiLayer::Modal` / `UiLayer::Popup`), non-UI tab filtering,
//! semantic ancestor tag inheritance, and [`Workbench`] transition tracking into
//! [`WorkbenchOutcome`].

use super::pointer::{HoverTargetResolution, resolve_hover_targets};
use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use iris_core::event::UiEvent;
use iris_core::geometry::{Point, Rect, Size};
use iris_core::node::{UiLayer, WidgetRole};
use iris_core::tree::UiTree;
use iris_dock::chrome::DockChromeFrame;
use iris_dock::state::DockState;
use iris_dock::tree::DockTree;
use iris_winit::CursorIcon;
use iris_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HoverTestTab {
    Viewport,
    Inspector,
    Console,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HoverTestAction {}

#[derive(Debug, Default)]
struct HoverMockWindow {
    logical_size: Size,
}

impl WindowInterface for HoverMockWindow {
    fn set_cursor(&mut self, _icon: CursorIcon) {}
    fn drag_resize_window(&mut self, _direction: ResizeDirection) -> Result<(), WindowOpError> {
        Ok(())
    }
    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Ok(())
    }
    fn toggle_maximize(&mut self) {}
    fn is_maximized(&self) -> bool {
        false
    }
    fn minimize(&mut self) {}
    fn close(&mut self) {}
    fn logical_size(&self) -> Size {
        self.logical_size
    }
    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn setup_tree_with_layers() -> (UiTree, DockChromeFrame<HoverTestTab>) {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root creation");
    if let Some(n) = tree.get_mut(root) {
        n.computed_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);
    }

    // Content button inside Inspector area (tag = 0x101)
    let inspector_btn = tree.create_node();
    tree.add_child(root, inspector_btn)
        .expect("add inspector_btn");
    if let Some(n) = tree.get_mut(inspector_btn) {
        n.role = WidgetRole::Button;
        n.interactive = true;
        n.tag = 0x101;
        n.layer = UiLayer::Content;
        n.computed_rect = Rect::new(900.0, 100.0, 120.0, 32.0);
    }

    // Untagged child inside inspector_btn verifying ancestor tag resolution
    let inspector_btn_label = tree.create_node();
    tree.add_child(inspector_btn, inspector_btn_label)
        .expect("add inspector_btn_label");
    if let Some(n) = tree.get_mut(inspector_btn_label) {
        n.interactive = true;
        n.tag = 0;
        n.computed_rect = Rect::new(910.0, 104.0, 80.0, 20.0);
    }

    // Elevated popup card overlapping a region across Inspector and Console
    let popup_card = tree.create_node();
    tree.add_child(root, popup_card).expect("add popup_card");
    if let Some(n) = tree.get_mut(popup_card) {
        n.interactive = true;
        n.tag = 0x202;
        n.layer = UiLayer::Popup;
        n.computed_rect = Rect::new(400.0, 200.0, 600.0, 240.0);
    }

    let mut frame = DockChromeFrame::default();
    frame
        .panel_rects
        .push((HoverTestTab::Viewport, Rect::new(0.0, 30.0, 800.0, 450.0)));
    frame.panel_rects.push((
        HoverTestTab::Inspector,
        Rect::new(800.0, 30.0, 480.0, 450.0),
    ));
    frame
        .panel_rects
        .push((HoverTestTab::Console, Rect::new(0.0, 480.0, 1280.0, 216.0)));

    (tree, frame)
}

#[test]
fn test_pointer_resolve_hover_targets_layered_occlusion_and_non_ui_exclusion() {
    let (tree, frame) = setup_tree_with_layers();
    let non_ui = Some(HoverTestTab::Viewport);

    // 1. Inside interactive Viewport (non_ui_tab excluded, no tagged widget hit)
    let vp_res = resolve_hover_targets(&tree, Some(&frame), non_ui, Point::new(200.0, 100.0));
    assert_eq!(vp_res.hovered_panel, None);
    assert_eq!(vp_res.hovered_tag, None);
    assert!(!vp_res.is_over_overlay);

    // 2. Inside untagged child of Inspector button -> inherits parent tag 0x101 + Inspector panel
    let btn_res = resolve_hover_targets(&tree, Some(&frame), non_ui, Point::new(920.0, 110.0));
    assert_eq!(btn_res.hovered_tag, Some(0x101));
    assert_eq!(btn_res.hovered_panel, Some(HoverTestTab::Inspector));
    assert!(!btn_res.is_over_overlay);

    // 3. Inside Console panel empty area -> panel detected, no semantic tag
    let console_res = resolve_hover_targets(&tree, Some(&frame), non_ui, Point::new(300.0, 560.0));
    assert_eq!(console_res.hovered_panel, Some(HoverTestTab::Console));
    assert_eq!(console_res.hovered_tag, None);
    assert!(!console_res.is_over_overlay);

    // 4. Inside elevated Popup overlapping Inspector -> overlay wins, hovered_panel suppressed
    let popup_res = resolve_hover_targets(&tree, Some(&frame), non_ui, Point::new(850.0, 260.0));
    assert_eq!(popup_res.hovered_tag, Some(0x202));
    assert_eq!(popup_res.hovered_panel, None);
    assert!(popup_res.is_over_overlay);

    // 5. Active Modal blocks underlying Content hover targets outside the modal bounds
    let mut modal_tree = tree;
    let modal = modal_tree.create_node();
    let modal_root = modal_tree.root().expect("root exists");
    modal_tree.add_child(modal_root, modal).expect("add modal");
    if let Some(n) = modal_tree.get_mut(modal) {
        n.interactive = true;
        n.tag = 0x303;
        n.layer = UiLayer::Modal;
        n.computed_rect = Rect::new(100.0, 100.0, 300.0, 200.0);
    }
    let blocked =
        resolve_hover_targets(&modal_tree, Some(&frame), non_ui, Point::new(920.0, 110.0));
    assert_eq!(blocked.hovered_tag, None);
    let inside_modal =
        resolve_hover_targets(&modal_tree, Some(&frame), non_ui, Point::new(150.0, 150.0));
    assert_eq!(inside_modal.hovered_tag, Some(0x303));
    assert!(inside_modal.is_over_overlay);
    assert_eq!(inside_modal.hovered_panel, None);
}

#[test]
fn test_workbench_resolve_hover_targets_tracks_transitions_in_outcome() {
    let mut dock_tree = DockTree::new();
    let leaf = dock_tree.create_leaf(vec![HoverTestTab::Inspector]);
    dock_tree.set_root(leaf);

    let descriptor = WorkbenchDescriptor::new("Hover Test", DockState::new(dock_tree));
    let (mut tree, frame) = setup_tree_with_layers();
    let mut wb = Workbench::<HoverTestTab, HoverTestAction, ()>::new(descriptor)
        .with_non_ui_tab(HoverTestTab::Viewport);
    wb.dock_frame = Some(frame);

    let mut win = HoverMockWindow {
        logical_size: Size::new(1280.0, 720.0),
    };
    let mut dock_state = wb.descriptor.dock_state.clone();

    // 1. Move onto Inspector button tag 0x101 -> transition = true
    let out1 = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove {
            point: Point::new(920.0, 110.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(out1.hovered_tag(), Some(0x101));
    assert_eq!(out1.hovered_panel(), Some(&HoverTestTab::Inspector));
    assert!(out1.is_hover_target_changed());
    assert_eq!(wb.hovered_tag(), Some(0x101));
    assert_eq!(wb.hovered_panel(), Some(HoverTestTab::Inspector));

    // 2. Move within the exact same button -> tag unchanged, transition = false
    let out2 = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove {
            point: Point::new(930.0, 112.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(out2.hovered_tag(), Some(0x101));
    assert!(!out2.is_hover_target_changed());

    // 3. Move onto Console panel empty space -> leaves tag (transition = true), switches panel
    let out3 = wb.handle_event_full(
        &mut win,
        &UiEvent::MouseMove {
            point: Point::new(300.0, 560.0),
        },
        &mut dock_state,
        Some(&mut tree),
    );
    assert_eq!(out3.hovered_tag(), None);
    assert_eq!(out3.hovered_panel(), Some(&HoverTestTab::Console));
    assert!(out3.is_hover_target_changed());

    // 4. Direct resolve_hover_targets(None) when cursor_pos is None clears hover state
    wb.set_cursor_pos(None);
    let cleared: HoverTargetResolution<HoverTestTab> = wb.resolve_hover_targets(&tree, None);
    assert_eq!(cleared.hovered_tag, None);
    assert_eq!(cleared.hovered_panel, None);
}