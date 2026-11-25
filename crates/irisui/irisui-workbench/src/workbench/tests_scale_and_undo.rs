// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Regression tests covering non-100% UI scale splitter/layout alignment and selection/domain
//! history isolation for [`super::Workbench`].

use super::runner::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use irisui_core::event::{MouseButton, UiEvent};
use irisui_core::geometry::{Point, Size};
use irisui_dock::state::DockState;
use irisui_dock::tree::DockTree;
use irisui_winit::CursorIcon;
use irisui_winit::frameless::{ResizeDirection, WindowInterface, WindowOpError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestTab {
    Inspector,
    Console,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    Zoom(u32),
}

#[derive(Debug, Default)]
struct MockWindow {
    cursor: Option<CursorIcon>,
    maximized: bool,
    minimized: bool,
    closed: bool,
    logical_size: Size,
}

impl WindowInterface for MockWindow {
    fn set_cursor(&mut self, icon: CursorIcon) {
        self.cursor = Some(icon);
    }

    fn drag_resize_window(&mut self, _direction: ResizeDirection) -> Result<(), WindowOpError> {
        Ok(())
    }

    fn drag_window(&mut self) -> Result<(), WindowOpError> {
        Ok(())
    }

    fn toggle_maximize(&mut self) {
        self.maximized = !self.maximized;
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }

    fn minimize(&mut self) {
        self.minimized = true;
    }

    fn close(&mut self) {
        self.closed = true;
    }

    fn logical_size(&self) -> Size {
        self.logical_size
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn create_test_workbench() -> Workbench<TestTab, TestAction, ()> {
    let mut dock_tree = DockTree::new();
    let r = dock_tree.create_leaf(vec![TestTab::Inspector, TestTab::Console]);
    dock_tree.set_root(r);

    let mut descriptor = WorkbenchDescriptor::new("Scale & Undo Test", DockState::new(dock_tree));
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    Workbench::new(descriptor).with_zoom_action(|z| TestAction::Zoom((z * 100.0).round() as u32))
}

#[test]
fn test_splitter_drag_works_at_non_100_percent_ui_scale() {
    use irisui_dock::SplitDirection;
    use irisui_dock::tree::DockNode;

    let mut dock_tree = DockTree::new();
    let left = dock_tree.create_leaf(vec![TestTab::Inspector]);
    dock_tree.set_root(left);
    let _ = dock_tree.split(
        left,
        SplitDirection::Horizontal,
        0.5,
        vec![TestTab::Console],
    );
    let root = dock_tree.root().expect("split root");

    let mut descriptor =
        WorkbenchDescriptor::new("Scaled Splitter Test", DockState::new(dock_tree));
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    let mut wb = Workbench::<TestTab, TestAction, ()>::new(descriptor)
        .with_menubar_height(36.0)
        .with_statusbar_height(24.0);

    // Step zoom to 125% (1.0 -> 1.10 -> 1.25) and sync physical 1920x1080 -> zoomed 1536x864
    let _ = wb.step_zoom(true);
    let zoom = wb.step_zoom(true);
    assert!((zoom - 1.25).abs() < 1e-4);
    wb.sync_physical_window_size(1920, 1080);
    let screen = wb.screen_size().expect("zoomed logical size");
    assert_eq!(screen, Size::new(1536.0, 864.0));

    let mut win = MockWindow {
        logical_size: Size::new(1920.0, 1080.0),
        ..Default::default()
    };

    let workspace = wb.compute_workspace_rect(screen.width, screen.height);
    let tree_root = wb.tree.create_root().expect("root");
    let _ = wb.tree.set_root(tree_root);
    let viewer = wb.panel_registry.tab_viewer();
    let dock_style = wb.dock_style;
    let frame = irisui_dock::build_dock_chrome(
        &mut wb.tree,
        tree_root,
        &irisui_dock::DockChromeParams {
            dock_tree: &wb.descriptor.dock_state.tree,
            workspace_rect: workspace,
            cursor_pos: Point::ZERO,
            is_cursor_occluded: false,
            is_dragging_splitter: false,
            active_splitter_node: None,
            is_dragging_tab: false,
            viewer: &viewer,
            style: &dock_style,
        },
    );
    let splitter_rect = frame
        .splitter_targets
        .first()
        .expect("rendered splitter")
        .rect;
    let start_pt = Point::new(
        splitter_rect.x + splitter_rect.width * 0.5,
        splitter_rect.y + splitter_rect.height * 0.5,
    );
    assert!((start_pt.x - 768.0).abs() < 4.0);

    let down = wb.handle_event(
        &mut win,
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            point: start_pt,
        },
    );
    assert!(
        down.consumed,
        "MouseDown on splitter at zoomed 125% coordinate {:?} must be consumed",
        start_pt
    );

    let drag_end = Point::new(start_pt.x + 150.0, start_pt.y);
    let _ = wb.handle_event(&mut win, &UiEvent::MouseMove { point: drag_end });
    let _ = wb.handle_event(
        &mut win,
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            point: drag_end,
        },
    );

    let ratio = match wb.descriptor.dock_state.tree.get(root) {
        Some(DockNode::Split { ratio, .. }) => *ratio,
        other => panic!("Expected split root, got {:?}", other),
    };
    assert!(
        ratio > 0.55,
        "Dragging splitter right at 125% zoom must update ratio > 0.55, got {ratio}"
    );
}

#[test]
fn test_set_scale_factor_clamping_and_75_percent_splitter_drag() {
    use irisui_dock::SplitDirection;
    use irisui_dock::tree::DockNode;

    let mut dock_tree = DockTree::new();
    let left = dock_tree.create_leaf(vec![TestTab::Inspector]);
    dock_tree.set_root(left);
    let _ = dock_tree.split(
        left,
        SplitDirection::Horizontal,
        0.5,
        vec![TestTab::Console],
    );
    let root = dock_tree.root().expect("split root");

    let mut descriptor = WorkbenchDescriptor::new("Scaled 75% Test", DockState::new(dock_tree));
    descriptor.menubar = Some(crate::descriptors::menu::MenuBarDescriptor::new());
    descriptor.statusbar = Some(crate::descriptors::status::StatusBarDescriptor::new());
    let mut wb = Workbench::<TestTab, TestAction, ()>::new(descriptor)
        .with_menubar_height(36.0)
        .with_statusbar_height(24.0);

    // Negative / out-of-range boundary sanitization
    assert_eq!(wb.set_scale_factor(f32::NAN), 1.0);
    assert_eq!(wb.set_scale_factor(-2.0), 1.0);
    assert_eq!(wb.set_scale_factor(10.0), 2.0);

    // Set 75% scale cleanly via set_scale_factor
    assert!((wb.set_scale_factor(0.75) - 0.75).abs() < 1e-4);
    assert!(wb.notifier.is_shell_dirty());
    wb.sync_physical_window_size(1200, 900);
    assert_eq!(wb.screen_size(), Some(Size::new(1600.0, 1200.0)));

    let mut win = MockWindow {
        logical_size: Size::new(1200.0, 900.0),
        ..Default::default()
    };
    let start_pt = Point::new(800.0, 600.0);
    let down = wb.handle_event(
        &mut win,
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            point: start_pt,
        },
    );
    assert!(
        down.consumed,
        "MouseDown on 75%-scaled splitter at x=800 must be consumed"
    );
    let end_pt = Point::new(600.0, 600.0);
    let _ = wb.handle_event(&mut win, &UiEvent::MouseMove { point: end_pt });
    let _ = wb.handle_event(
        &mut win,
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            point: end_pt,
        },
    );

    let ratio = match wb.descriptor.dock_state.tree.get(root) {
        Some(DockNode::Split { ratio, .. }) => *ratio,
        other => panic!("Expected split root, got {:?}", other),
    };
    assert!(
        ratio < 0.42,
        "Dragging splitter left at 75% scale must shrink left ratio < 0.42, got {ratio}"
    );
}