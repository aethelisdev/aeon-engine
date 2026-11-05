// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::action_bus::ActionBus;
use crate::error::WorkbenchError;
use iris_core::UiTree;
use iris_dock::{DockState, DockTree};
use iris_widgets::UiScope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
enum MockTab {
    Hierarchy,
    Inspector,
    Viewport,
}

struct MockContext {
    rebuild_counter: u32,
    last_tab_drawn: Option<MockTab>,
}

#[test]
fn test_menu_bar_descriptor_hierarchy_and_tags() {
    let mut menu_bar = MenuBarDescriptor::new();

    let file_menu = MenuSectionDescriptor::new("File", 100)
        .with_action_shortcut("New Scene", "Ctrl+N", 101)
        .with_action_shortcut("Save Scene", "Ctrl+S", 102)
        .with_separator()
        .with_submenu(
            "Export",
            103,
            vec![
                MenuItemDescriptor::action("glTF 2.0", 104),
                MenuItemDescriptor::disabled_action("FBX (Unavailable)", 105),
            ],
        );

    menu_bar.add_menu(file_menu);

    assert_eq!(menu_bar.menus.len(), 1);
    let found = menu_bar.find_menu("File").expect("File menu must exist");
    assert_eq!(found.tag, 100);
    assert_eq!(found.items.len(), 4);

    // Item 0: Action with shortcut
    assert_eq!(found.items[0].tag(), Some(101));
    assert!(found.items[0].is_enabled());
    assert!(!found.items[0].is_separator());

    // Item 2: Separator
    assert_eq!(found.items[2].tag(), None);
    assert!(!found.items[2].is_enabled());
    assert!(found.items[2].is_separator());

    // Item 3: Submenu
    if let MenuItemDescriptor::Submenu { items, .. } = &found.items[3] {
        assert_eq!(items.len(), 2);
        assert!(items[0].is_enabled());
        assert!(!items[1].is_enabled()); // disabled action
    } else {
        panic!("Expected submenu variant");
    }
}

#[test]
fn test_status_bar_descriptor_telemetry_pills() {
    let status_bar = StatusBarDescriptor::new()
        .with_left_status("Compiling Shaders (3 remaining)...")
        .with_height(28.0)
        .with_pill(
            StatusPillDescriptor::new("144 FPS", 501)
                .with_style(StatusPillStyle::Success)
                .with_tooltip("Framerate solid at display refresh"),
        )
        .with_pill(
            StatusPillDescriptor::new("2 Warnings", 502).with_style(StatusPillStyle::Warning),
        );

    assert_eq!(status_bar.left_status, "Compiling Shaders (3 remaining)...");
    assert_eq!(status_bar.height, 28.0);
    assert_eq!(status_bar.pills.len(), 2);

    assert_eq!(status_bar.pills[0].label, "144 FPS");
    assert_eq!(status_bar.pills[0].style, StatusPillStyle::Success);
    assert_eq!(
        status_bar.pills[0].tooltip.as_deref(),
        Some("Framerate solid at display refresh")
    );

    assert_eq!(status_bar.pills[1].label, "2 Warnings");
    assert_eq!(status_bar.pills[1].style, StatusPillStyle::Warning);
    assert_eq!(status_bar.pills[1].tooltip, None);
}

#[test]
fn test_panel_registry_registration_and_drawing() {
    let mut registry = PanelRegistry::<MockTab, MockContext>::new();
    assert!(registry.is_empty());

    // 1. Register Hierarchy panel
    registry.register_panel(MockTab::Hierarchy, |_scope, ctx| {
        ctx.rebuild_counter += 1;
        ctx.last_tab_drawn = Some(MockTab::Hierarchy);
    });

    assert_eq!(registry.len(), 1);
    assert!(registry.is_registered(&MockTab::Hierarchy));
    assert!(!registry.is_registered(&MockTab::Inspector));

    // 2. Draw registered panel
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);
    let mut context = MockContext {
        rebuild_counter: 0,
        last_tab_drawn: None,
    };

    let result = registry.draw(&MockTab::Hierarchy, &mut scope, &mut context);
    assert!(result.is_ok());
    assert_eq!(context.rebuild_counter, 1);
    assert_eq!(context.last_tab_drawn, Some(MockTab::Hierarchy));

    // 3. Draw unregistered panel returns error
    let missing_result = registry.draw(&MockTab::Inspector, &mut scope, &mut context);
    assert!(matches!(
        missing_result,
        Err(WorkbenchError::PanelNotFound(_))
    ));

    // 4. Unregister panel
    assert!(registry.unregister_panel(&MockTab::Hierarchy));
    assert!(!registry.is_registered(&MockTab::Hierarchy));
    assert_eq!(registry.len(), 0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestCopyTab {
    Hierarchy,
    Inspector,
    Console,
}

#[test]
fn test_panel_registry_domain_panel_subscription_and_notifier_sync() {
    let mut registry = WorkbenchPanelRegistry::<TestCopyTab, MockContext>::new();

    registry.register_domain_panel(TestCopyTab::Inspector, |_scope, _ctx| {});
    registry.register_panel(TestCopyTab::Console, |_scope, _ctx| {});
    registry.mark_domain_panel(TestCopyTab::Hierarchy);

    assert!(registry.is_domain_panel(&TestCopyTab::Inspector));
    assert!(registry.is_domain_panel(&TestCopyTab::Hierarchy));
    assert!(!registry.is_domain_panel(&TestCopyTab::Console));
    assert_eq!(registry.domain_panels().len(), 2);

    let mut notifier = iris_dock::UiNotifier::<TestCopyTab>::clean();
    registry.sync_domain_panels_to_notifier(&mut notifier);

    assert!(notifier.is_domain_panel(&TestCopyTab::Inspector));
    assert!(notifier.is_domain_panel(&TestCopyTab::Hierarchy));
    assert!(!notifier.is_domain_panel(&TestCopyTab::Console));

    // Verify unmarking domain panel
    assert!(registry.unmark_domain_panel(&TestCopyTab::Hierarchy));
    assert!(!registry.is_domain_panel(&TestCopyTab::Hierarchy));

    // Verify unregistering panel removes from domain set
    assert!(registry.unregister_panel(&TestCopyTab::Inspector));
    assert!(!registry.is_domain_panel(&TestCopyTab::Inspector));
}

#[test]
fn test_workbench_descriptor_shell_composition() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec![MockTab::Viewport, MockTab::Hierarchy]);
    tree.set_root(leaf);
    let dock_state = DockState::new(tree);

    let workbench = WorkbenchDescriptor::new("Aeon Engine Studio", dock_state)
        .with_menubar(MenuBarDescriptor::new().with_menu(MenuSectionDescriptor::new("Edit", 200)))
        .with_statusbar(StatusBarDescriptor::new().with_left_status("Ready"));

    assert_eq!(workbench.title(), "Aeon Engine Studio");
    assert!(workbench.menubar().is_some());
    assert_eq!(workbench.menubar().unwrap().menus.len(), 1);
    assert_eq!(workbench.statusbar().unwrap().left_status, "Ready");
    assert!(workbench.dock_state().tree.root().is_some());
}

#[test]
fn test_menubar_descriptor_render_and_divider() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");
    let mut scope = UiScope::new(&mut tree, root);

    let menubar = MenuBarDescriptor::new()
        .with_height(36.0)
        .with_branding(MenuBarBrandingDescriptor::new(
            [0.0, 0.0, 1.0, 1.0],
            iris_core::Color::WHITE,
            28.0,
        ))
        .with_menu(
            MenuSectionDescriptor::new("File", 100)
                .with_action_full("🗎", "New Scene", Some("Ctrl+N"), 101)
                .with_separator()
                .with_action("Save Scene", 102),
        )
        .with_action_button(
            "▶ Play",
            iris_core::Color::hex("#228b22"),
            iris_core::Color::WHITE,
            301,
        )
        .with_window_control("✕", true, 401);

    let action = menubar.render(&mut scope, 1920.0);
    assert_eq!(
        action, None,
        "No action should be triggered without interaction"
    );

    // Verify baseline divider exists and has proper name & styling
    let divider_node = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("IrisTopMenuBarDivider"));
    assert!(
        divider_node.is_some(),
        "Top menubar divider 'IrisTopMenuBarDivider' must be emitted"
    );
    let (_, div) = divider_node.unwrap();
    assert_eq!(div.style.height, Some(1.0));
    assert_eq!(div.style.inset_top, Some(35.0)); // height 36 - 1.0
}

#[test]
fn test_statusbar_descriptor_render_and_divider() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");
    let mut scope = UiScope::new(&mut tree, root);

    let statusbar = StatusBarDescriptor::new()
        .with_height(22.0)
        .with_left_status("Ready")
        .with_right_text("Aeon Engine v0.9.0")
        .with_pill(StatusPillDescriptor::new("144 FPS", 501).with_style(StatusPillStyle::Success));

    statusbar.render(&mut scope, 1920.0, 1080.0);

    // Verify statusbar divider exists and has proper name & styling
    let divider_node = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("IrisBottomStatusBarDivider"));
    assert!(
        divider_node.is_some(),
        "Bottom statusbar divider 'IrisBottomStatusBarDivider' must be emitted"
    );
    let (_, div) = divider_node.unwrap();
    assert_eq!(div.style.height, Some(1.0));
    assert_eq!(div.style.inset_top, Some(1058.0)); // 1080 - 22.0

    // Verify statusbar container exists with semantic tag
    let bar_id = tree.find_node_by_tag(StatusBarDescriptor::STATUS_BAR_TAG);
    assert!(
        bar_id.is_some(),
        "Status bar container must exist with STATUS_BAR_TAG"
    );
}

#[test]
fn test_workbench_descriptor_render_chrome_integration() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec![MockTab::Viewport]);
    tree.set_root(leaf);
    let dock_state = DockState::new(tree);

    let workbench = WorkbenchDescriptor::new("Aeon Engine Studio", dock_state)
        .with_menubar(
            MenuBarDescriptor::new()
                .with_height(36.0)
                .with_menu(MenuSectionDescriptor::new("File", 100)),
        )
        .with_statusbar(
            StatusBarDescriptor::new()
                .with_height(22.0)
                .with_left_status("Ready"),
        );

    let mut ui_tree = UiTree::new();
    let root = ui_tree.create_root().expect("Root node must exist");
    let mut scope = UiScope::new(&mut ui_tree, root);

    let action = workbench.render_chrome(&mut scope, 1920.0, 1080.0);
    assert_eq!(action, None);

    // Verify both dividers exist in the tree
    assert!(
        ui_tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("IrisTopMenuBarDivider")),
        "IrisTopMenuBarDivider must exist"
    );
    assert!(
        ui_tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("IrisBottomStatusBarDivider")),
        "IrisBottomStatusBarDivider must exist"
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MockMenuAction {
    NewFile,
    SaveFile,
    RequestExit,
    ToggleFullscreen,
}

#[test]
fn test_menu_bar_descriptor_render_with_action_bus_emission() {
    let mut bus = ActionBus::<MockMenuAction>::new();

    let menu_bar = MenuBarDescriptor::new()
        .with_height(36.0)
        .with_menu(
            MenuSectionDescriptor::new("File", 1000)
                .with_action_shortcut("New File", "Ctrl+N", 1001)
                .with_action_shortcut("Save File", "Ctrl+S", 1002)
                .with_action_shortcut("Exit", "Alt+F4", 1003),
        )
        .with_window_control("□", false, 7002);

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    // 1. Initial render without any click event
    let mut scope = UiScope::new(&mut tree, root);
    let action_tag =
        menu_bar.render_with_action_bus(&mut scope, 1920.0, &mut bus, |tag| match tag {
            1001 => Some(MockMenuAction::NewFile),
            1002 => Some(MockMenuAction::SaveFile),
            1003 => Some(MockMenuAction::RequestExit),
            7002 => Some(MockMenuAction::ToggleFullscreen),
            _ => None,
        });

    assert_eq!(action_tag, None);
    assert!(
        bus.is_empty(),
        "No action should be emitted when not clicked"
    );

    // 2. Simulate opening "File" menu and then clicking "Exit" (tag 1003)
    let file_header_tag = iris_widgets::declarative::hash_label("File");
    tree.active_menu_bar_tag = Some(file_header_tag);
    tree.event_state.frame_clicked_tag = Some(1003);
    let mut scope2 = UiScope::new(&mut tree, root);
    let clicked_tag =
        menu_bar.render_with_action_bus(&mut scope2, 1920.0, &mut bus, |tag| match tag {
            1001 => Some(MockMenuAction::NewFile),
            1002 => Some(MockMenuAction::SaveFile),
            1003 => Some(MockMenuAction::RequestExit),
            7002 => Some(MockMenuAction::ToggleFullscreen),
            _ => None,
        });

    assert_eq!(clicked_tag, Some(1003));
    assert_eq!(bus.len(), 1, "Clicked action must be pushed to ActionBus");
    let drained: Vec<_> = bus.drain().collect();
    assert_eq!(drained, vec![MockMenuAction::RequestExit]);
    assert!(bus.is_empty(), "ActionBus drained successfully");

    // 3. Simulate clicking window control button (tag 7002)
    tree.event_state.frame_clicked_tag = Some(7002);
    let mut scope3 = UiScope::new(&mut tree, root);
    let win_btn_tag =
        menu_bar.render_with_action_bus(&mut scope3, 1920.0, &mut bus, |tag| match tag {
            1001 => Some(MockMenuAction::NewFile),
            1002 => Some(MockMenuAction::SaveFile),
            1003 => Some(MockMenuAction::RequestExit),
            7002 => Some(MockMenuAction::ToggleFullscreen),
            _ => None,
        });

    assert_eq!(win_btn_tag, Some(7002));
    assert_eq!(bus.len(), 1);
    let drained_win: Vec<_> = bus.drain().collect();
    assert_eq!(drained_win, vec![MockMenuAction::ToggleFullscreen]);
}

#[test]
fn test_workbench_descriptor_render_chrome_with_action_bus() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec![MockTab::Viewport]);
    tree.set_root(leaf);
    let dock_state = DockState::new(tree);

    let workbench = WorkbenchDescriptor::new("Aeon Studio", dock_state)
        .with_menubar(MenuBarDescriptor::new().with_height(36.0).with_menu(
            MenuSectionDescriptor::new("File", 2000).with_action_shortcut(
                "Save Scene",
                "Ctrl+S",
                2001,
            ),
        ))
        .with_statusbar(StatusBarDescriptor::new().with_height(22.0));

    let mut bus = ActionBus::<MockMenuAction>::new();
    let mut ui_tree = UiTree::new();
    let root = ui_tree.create_root().expect("Root node must exist");

    // Simulate File dropdown opened and Save Scene (tag 2001) clicked
    let file_tag = iris_widgets::declarative::hash_label("File");
    ui_tree.active_menu_bar_tag = Some(file_tag);
    ui_tree.event_state.frame_clicked_tag = Some(2001);
    let mut scope = UiScope::new(&mut ui_tree, root);

    let tag = workbench.render_chrome_with_action_bus(&mut scope, 1920.0, 1080.0, &mut bus, |t| {
        if t == 2001 {
            Some(MockMenuAction::SaveFile)
        } else {
            None
        }
    });

    assert_eq!(tag, Some(2001));
    assert_eq!(bus.len(), 1);
    let events: Vec<_> = bus.drain().collect();
    assert_eq!(events, vec![MockMenuAction::SaveFile]);
}

// =========================================================================
// Faz 6.1.1: WorkbenchPanel Trait & Granular Partial Rendering Tests
// =========================================================================

#[derive(Default)]
struct MockPanelAppState {
    viewport_renders: usize,
    inspector_renders: usize,
    hierarchy_renders: usize,
}

struct MockViewportPanel;

impl WorkbenchPanel<MockTab, MockPanelAppState> for MockViewportPanel {
    fn id(&self) -> MockTab {
        MockTab::Viewport
    }

    fn title(&self) -> &str {
        "3D Viewport"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("viewport_icon")
    }

    fn closeable(&self) -> bool {
        false
    }

    fn is_domain_bound(&self) -> bool {
        false
    }

    fn render(&mut self, _scope: &mut UiScope<'_>, state: &mut MockPanelAppState) {
        state.viewport_renders += 1;
    }
}

struct MockInspectorPanel;

impl WorkbenchPanel<MockTab, MockPanelAppState> for MockInspectorPanel {
    fn id(&self) -> MockTab {
        MockTab::Inspector
    }

    fn title(&self) -> &str {
        "Entity Inspector"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("inspector_icon")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn is_domain_bound(&self) -> bool {
        true
    }

    fn render(&mut self, _scope: &mut UiScope<'_>, state: &mut MockPanelAppState) {
        state.inspector_renders += 1;
    }
}

struct MockHierarchyPanel;

impl WorkbenchPanel<MockTab, MockPanelAppState> for MockHierarchyPanel {
    fn id(&self) -> MockTab {
        MockTab::Hierarchy
    }

    fn title(&self) -> &str {
        "Scene Hierarchy"
    }

    fn icon(&self) -> Option<&'static str> {
        Some("hierarchy_icon")
    }

    fn closeable(&self) -> bool {
        true
    }

    fn is_domain_bound(&self) -> bool {
        false
    }

    fn render(&mut self, _scope: &mut UiScope<'_>, state: &mut MockPanelAppState) {
        state.hierarchy_renders += 1;
    }
}

#[test]
fn test_workbench_panel_trait_registration_and_metadata() {
    use iris_dock::TabViewer;

    let mut registry = WorkbenchPanelRegistry::<MockTab, MockPanelAppState>::new();
    registry.register(MockViewportPanel);
    registry.register(MockInspectorPanel);
    registry.register(MockHierarchyPanel);

    assert_eq!(registry.len(), 3);
    assert!(registry.is_registered(&MockTab::Viewport));
    assert!(registry.is_registered(&MockTab::Inspector));
    assert!(registry.is_registered(&MockTab::Hierarchy));

    // Verify metadata propagation
    let vp_meta = registry
        .panel_metadata(&MockTab::Viewport)
        .expect("Viewport metadata");
    assert_eq!(vp_meta.title, "3D Viewport");
    assert_eq!(vp_meta.icon.as_deref(), Some("viewport_icon"));
    assert!(!vp_meta.closeable);
    assert!(!vp_meta.is_domain_bound);

    let insp_meta = registry
        .panel_metadata(&MockTab::Inspector)
        .expect("Inspector metadata");
    assert_eq!(insp_meta.title, "Entity Inspector");
    assert!(insp_meta.closeable);
    assert!(insp_meta.is_domain_bound);

    // Verify domain subscriber registration
    assert!(registry.is_domain_panel(&MockTab::Inspector));
    assert!(!registry.is_domain_panel(&MockTab::Viewport));
    assert!(!registry.is_domain_panel(&MockTab::Hierarchy));

    // Verify RegistryTabViewer integration
    let viewer = registry.tab_viewer();
    assert_eq!(viewer.title(&MockTab::Viewport), "3D Viewport");
    assert_eq!(viewer.title(&MockTab::Inspector), "Entity Inspector");
    assert_eq!(viewer.title(&MockTab::Hierarchy), "Scene Hierarchy");
    assert!(!viewer.closeable(&MockTab::Viewport));
    assert!(viewer.closeable(&MockTab::Inspector));
    assert!(viewer.closeable(&MockTab::Hierarchy));
}

#[test]
fn test_workbench_panel_granular_partial_rendering_only_dirty_panel_called() {
    let mut registry = WorkbenchPanelRegistry::<MockTab, MockPanelAppState>::new();
    registry.register(MockViewportPanel);
    registry.register(MockInspectorPanel);
    registry.register(MockHierarchyPanel);

    let mut state = MockPanelAppState::default();
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    // 1. Initial full render: all 3 panels are drawn
    let all_tabs = [MockTab::Viewport, MockTab::Inspector, MockTab::Hierarchy];
    let rendered_all =
        registry.render_dirty_tabs(&all_tabs, &mut tree, |_tab| Some(root), &mut state);
    assert_eq!(rendered_all, 3);
    assert_eq!(state.viewport_renders, 1);
    assert_eq!(state.inspector_renders, 1);
    assert_eq!(state.hierarchy_renders, 1);

    // 2. Granular partial invalidation: ONLY MockTab::Inspector is marked dirty!
    let dirty_tabs = [MockTab::Inspector];
    let rendered_dirty =
        registry.render_dirty_tabs(&dirty_tabs, &mut tree, |_tab| Some(root), &mut state);

    // Verify that strictly 1 panel was rendered
    assert_eq!(rendered_dirty, 1);
    // Viewport and Hierarchy MUST NOT have received render invocations!
    assert_eq!(
        state.viewport_renders, 1,
        "Viewport must not re-render when clean"
    );
    assert_eq!(
        state.hierarchy_renders, 1,
        "Hierarchy must not re-render when clean"
    );
    // Inspector alone was rendered
    assert_eq!(state.inspector_renders, 2, "Dirty Inspector must re-render");
}