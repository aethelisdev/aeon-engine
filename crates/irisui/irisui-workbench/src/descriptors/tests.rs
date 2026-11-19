// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::action_bus::ActionBus;
use crate::error::WorkbenchError;
use irisui_core::UiTree;
use irisui_dock::{DockState, DockTree};
use irisui_widgets::UiScope;

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
    let mut menu_bar = MenuBarDescriptor::<MockMenuAction>::new();

    let file_tag = irisui_core::UiTag::from_str("File");
    let new_scene_tag = file_tag.child("New Scene");
    let file_menu = MenuSectionDescriptor::new("File", file_tag)
        .with_command_shortcut("New Scene", "Ctrl+N", MockMenuAction::NewFile)
        .with_command_shortcut("Save Scene", "Ctrl+S", MockMenuAction::SaveFile)
        .with_separator()
        .with_submenu(
            "Export",
            file_tag.child("Export"),
            vec![
                MenuItemDescriptor::command("glTF 2.0", MockMenuAction::SaveFile),
                MenuItemDescriptor::disabled(None, "FBX (Unavailable)", None::<&str>),
            ],
        );

    menu_bar.add_menu(file_menu);

    assert_eq!(menu_bar.menus.len(), 1);
    let found = menu_bar.find_menu("File").expect("File menu must exist");
    assert_eq!(found.tag, file_tag);
    assert_eq!(found.items.len(), 4);

    // Item 0: Command with shortcut and auto-resolved hierarchical tag
    assert_eq!(found.items[0].resolved_tag(found.tag), Some(new_scene_tag));
    assert!(found.items[0].is_enabled());
    assert!(!found.items[0].is_separator());

    // Item 2: Separator
    assert_eq!(found.items[2].resolved_tag(found.tag), None);
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

    let mut notifier = irisui_dock::UiNotifier::<TestCopyTab>::clean();
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
        .with_menubar(MenuBarDescriptor::new().with_menu(MenuSectionDescriptor::titled("Edit")))
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

    let menubar = MenuBarDescriptor::<()>::new()
        .with_height(36.0)
        .with_branding(MenuBarBrandingDescriptor::new(
            [0.0, 0.0, 1.0, 1.0],
            irisui_core::Color::WHITE,
            28.0,
        ))
        .with_menu(
            MenuSectionDescriptor::titled("File")
                .with_command_full("🗎", "New Scene", Some("Ctrl+N"), ())
                .with_separator()
                .with_command("Save Scene", ()),
        )
        .with_command_button(
            "▶ Play",
            irisui_core::Color::hex("#228b22"),
            irisui_core::Color::WHITE,
            (),
        )
        .with_window_control("✕", true, irisui_core::UiTag::from_str("window.close"));

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
                .with_menu(MenuSectionDescriptor::titled("File")),
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
fn test_menu_bar_descriptor_render_into_direct_action_emission() {
    let mut bus = ActionBus::<MockMenuAction>::new();
    let about_modal_tag = irisui_core::UiTag::from_str("modal.about");

    let file_section = MenuSectionDescriptor::titled("File")
        .with_command_shortcut("New File", "Ctrl+N", MockMenuAction::NewFile)
        .with_command_shortcut("Save File", "Ctrl+S", MockMenuAction::SaveFile)
        .with_command_shortcut("Exit", "Alt+F4", MockMenuAction::RequestExit)
        .with_modal_trigger(None, "About", None::<&str>, about_modal_tag);

    let exit_tag = file_section.effective_tag().child("Exit");
    let about_item_tag = file_section.effective_tag().child("About");
    let play_tag = irisui_core::UiTag::from_str("btn.toggle_play");

    let menu_bar = MenuBarDescriptor::new()
        .with_height(36.0)
        .with_menu(file_section)
        .with_action_button(
            "□",
            irisui_core::Color::TRANSPARENT,
            irisui_core::Color::WHITE,
            play_tag,
            Some(MockMenuAction::ToggleFullscreen),
        );

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    // 1. Initial render without any click event
    let mut scope = UiScope::new(&mut tree, root);
    let clicked = menu_bar.render_into(&mut scope, 1920.0, &mut bus);
    assert_eq!(clicked, None);
    assert!(bus.is_empty());

    // 2. Simulate opening "File" menu and clicking "Exit" directly without tag map closure
    let file_header_tag = irisui_core::UiTag::from_str("File").raw();
    tree.active_menu_bar_tag = Some(file_header_tag);
    tree.event_state.frame_clicked_tag = Some(exit_tag.raw());
    let mut scope2 = UiScope::new(&mut tree, root);
    let clicked_exit = menu_bar.render_into(&mut scope2, 1920.0, &mut bus);

    assert_eq!(clicked_exit, Some(exit_tag));
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockMenuAction::RequestExit]
    );

    // 3. Simulate clicking "About" ModalTrigger directly opens modal_tag
    tree.active_menu_bar_tag = Some(file_header_tag);
    tree.event_state.frame_clicked_tag = Some(about_item_tag.raw());
    let mut scope3 = UiScope::new(&mut tree, root);
    let _ = menu_bar.render_into(&mut scope3, 1920.0, &mut bus);
    assert!(tree.is_modal_open(about_modal_tag.raw()));

    // 4. Simulate clicking typed right-aligned button
    tree.event_state.frame_clicked_tag = Some(play_tag.raw());
    let mut scope4 = UiScope::new(&mut tree, root);
    let clicked_play = menu_bar.render_into(&mut scope4, 1920.0, &mut bus);
    assert_eq!(clicked_play, Some(play_tag));
    assert_eq!(
        bus.drain().collect::<Vec<_>>(),
        vec![MockMenuAction::ToggleFullscreen]
    );
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

    fn icon(&self) -> Option<irisui_core::UiIcon> {
        Some(irisui_core::UiIcon::Glyph("viewport_icon"))
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

    fn icon(&self) -> Option<irisui_core::UiIcon> {
        Some(irisui_core::UiIcon::Glyph("inspector_icon"))
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

    fn icon(&self) -> Option<irisui_core::UiIcon> {
        Some(irisui_core::UiIcon::Glyph("hierarchy_icon"))
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
    use irisui_dock::TabViewer;

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
    assert_eq!(
        vp_meta.icon,
        Some(irisui_core::UiIcon::Glyph("viewport_icon"))
    );
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
    assert_eq!(
        viewer.title(&MockTab::Viewport),
        "viewport_icon 3D Viewport"
    );
    assert_eq!(viewer.raw_title(&MockTab::Viewport), "3D Viewport");
    assert_eq!(
        viewer.title(&MockTab::Inspector),
        "inspector_icon Entity Inspector"
    );
    assert_eq!(viewer.raw_title(&MockTab::Inspector), "Entity Inspector");
    assert_eq!(
        viewer.title(&MockTab::Hierarchy),
        "hierarchy_icon Scene Hierarchy"
    );
    assert_eq!(viewer.raw_title(&MockTab::Hierarchy), "Scene Hierarchy");
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

#[test]
fn test_panel_registry_draw_leaf_seals_boundary_and_marks_dirty_paint() {
    use irisui_core::DirtyFlags;

    let mut registry = WorkbenchPanelRegistry::<MockTab, MockPanelAppState>::new();
    registry.register(MockInspectorPanel);

    let mut state = MockPanelAppState::default();
    let mut tree = UiTree::new();
    let leaf = tree.create_node();

    // Reset initial node dirty state and verify preconditions
    if let Some(node) = tree.get_mut(leaf) {
        node.dirty = DirtyFlags::empty();
    }
    assert!(!tree.is_boundary(leaf));
    assert!(!tree.get(leaf).unwrap().dirty.contains(DirtyFlags::PAINT));

    // 1. Successful draw_leaf invocation
    let res = registry.draw_leaf(&MockTab::Inspector, &mut tree, leaf, &mut state);
    assert!(res.is_ok());
    assert_eq!(state.inspector_renders, 1);

    // Verify postconditions: node is sealed as an Invalidation Boundary and marked PAINT dirty
    assert!(
        tree.is_boundary(leaf),
        "draw_leaf must set is_boundary to true"
    );
    assert!(
        tree.get(leaf).unwrap().dirty.contains(DirtyFlags::PAINT),
        "draw_leaf must mark DirtyFlags::PAINT on leaf container"
    );

    // 2. Unregistered panel returns Err and does not affect another clean node
    let unreg_leaf = tree.create_node();
    if let Some(node) = tree.get_mut(unreg_leaf) {
        node.dirty = DirtyFlags::empty();
    }
    let missing_res = registry.draw_leaf(&MockTab::Viewport, &mut tree, unreg_leaf, &mut state);
    assert!(matches!(missing_res, Err(WorkbenchError::PanelNotFound(_))));
    assert!(
        !tree.is_boundary(unreg_leaf),
        "Failed draw_leaf must not seal boundary"
    );
}

#[test]
fn test_panel_registry_render_dirty_tabs_delegates_to_draw_leaf() {
    use irisui_core::DirtyFlags;

    let mut registry = WorkbenchPanelRegistry::<MockTab, MockPanelAppState>::new();
    registry.register(MockInspectorPanel);
    registry.register(MockHierarchyPanel);

    let mut state = MockPanelAppState::default();
    let mut tree = UiTree::new();
    let leaf_insp = tree.create_node();
    let leaf_hier = tree.create_node();

    for &leaf in &[leaf_insp, leaf_hier] {
        if let Some(node) = tree.get_mut(leaf) {
            node.dirty = DirtyFlags::empty();
        }
        assert!(!tree.is_boundary(leaf));
    }

    let dirty_tabs = [MockTab::Inspector, MockTab::Hierarchy];
    let rendered = registry.render_dirty_tabs(
        &dirty_tabs,
        &mut tree,
        |tab| match tab {
            MockTab::Inspector => Some(leaf_insp),
            MockTab::Hierarchy => Some(leaf_hier),
            _ => None,
        },
        &mut state,
    );

    assert_eq!(rendered, 2);
    assert!(tree.is_boundary(leaf_insp));
    assert!(tree.is_boundary(leaf_hier));
    assert!(
        tree.get(leaf_insp)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );
    assert!(
        tree.get(leaf_hier)
            .unwrap()
            .dirty
            .contains(DirtyFlags::PAINT)
    );
}

#[test]
fn test_registry_active_tab_viewer_ui_seals_boundary_and_marks_dirty_paint() {
    use irisui_core::DirtyFlags;
    use irisui_dock::TabViewer;

    let mut registry = WorkbenchPanelRegistry::<MockTab, MockPanelAppState>::new();
    registry.register(MockInspectorPanel);

    let mut state = MockPanelAppState::default();
    let mut tree = UiTree::new();
    let leaf = tree.create_node();

    if let Some(node) = tree.get_mut(leaf) {
        node.dirty = DirtyFlags::empty();
    }
    assert!(!tree.is_boundary(leaf));

    {
        let mut viewer = registry.active_tab_viewer(&mut state);
        let mut scope = UiScope::new(&mut tree, leaf);
        viewer.ui(&mut scope, &MockTab::Inspector);
    }

    assert_eq!(state.inspector_renders, 1);
    assert!(
        tree.is_boundary(leaf),
        "ActiveTabViewer::ui must set is_boundary to true on scope root parent"
    );
    assert!(
        tree.get(leaf).unwrap().dirty.contains(DirtyFlags::PAINT),
        "ActiveTabViewer::ui must mark DirtyFlags::PAINT on leaf"
    );
}