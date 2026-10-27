// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
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