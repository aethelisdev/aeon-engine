// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top Application Menu Bar & Floating Dropdown Subsystem
//!
//! Provides the primary desktop application header menu bar (File, Edit, View,
//! Window, Help) and elevated floating dropdown popup cascades using 100% pure
//! declarative [`irisui::workbench::MenuBarDescriptor`] and [`irisui::workbench::ActionBus`].

use crate::ui::EngineUiAction;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::prelude::*;

/// Height of the top menubar panel in physical pixels.
pub const MENUBAR_HEIGHT: f32 = 36.0;

/// Default width of floating dropdown popup menus in physical pixels.
pub const DROPDOWN_WIDTH: f32 = 260.0;

/// Configuration descriptor passed to [`build_top_menu_bar`].
#[derive(Debug, Clone, Copy)]
pub struct MenuBarParams<'a> {
    /// Screen width constraint in physical pixels.
    pub screen_width: f32,
    /// Reference to the active panel layout and visibility manager.
    pub layout_state: &'a PanelLayoutState,
    /// Whether the editor command stack supports undo operations.
    pub can_undo: bool,
    /// Whether the editor command stack supports redo operations.
    pub can_redo: bool,
    /// Whether the engine is currently in play/editing mode.
    pub is_editing: bool,
}

/// Builds the top application menu bar widget tree directly using [`irisui::workbench::MenuBarDescriptor`].
///
/// Dispatches menu actions, dropdown popups, and play/stop toggle buttons autonomously
/// directly into the engine's [`irisui::workbench::ActionBus`].
pub fn build_top_menu_bar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &MenuBarParams<'_>,
    action_bus: &mut irisui::workbench::ActionBus<EngineUiAction>,
) -> Option<u64> {
    let mut scope = UiScope::new(tree, parent_id);
    let descriptor = build_workbench_menubar_descriptor(params);
    let is_editing = params.is_editing;
    descriptor.render_with_action_bus(&mut scope, params.screen_width, action_bus, |tag| {
        map_menubar_tag(tag, is_editing)
    })
}

/// Maps a semantic numeric tag emitted by [`irisui::workbench::MenuBarDescriptor`] to a type-safe [`EngineUiAction`].
#[inline]
pub fn map_menubar_tag(tag: u64, is_editing: bool) -> Option<EngineUiAction> {
    match tag {
        0x1002 => Some(EngineUiAction::OpenLoadSceneDialog),
        0x1003 => Some(EngineUiAction::SaveScene),
        0x1004 => Some(EngineUiAction::OpenSaveSceneDialog),
        0x1005 => Some(EngineUiAction::Exit),
        0x2001 => Some(EngineUiAction::Undo),
        0x2002 => Some(EngineUiAction::Redo),
        0x2003 => Some(EngineUiAction::OpenPreferences),
        0x3001 | 0x4001 => Some(EngineUiAction::TogglePanel(PanelId::Hierarchy)),
        0x3002 | 0x4002 => Some(EngineUiAction::TogglePanel(PanelId::Inspector)),
        0x3003 | 0x4003 => Some(EngineUiAction::TogglePanel(PanelId::Assets)),
        0x3004 | 0x4004 => Some(EngineUiAction::TogglePanel(PanelId::Console)),
        0x3005 | 0x4005 => Some(EngineUiAction::TogglePanel(PanelId::MaterialEditor)),
        0x3006 | 0x4006 => Some(EngineUiAction::TogglePanel(PanelId::AnimationTimeline)),
        0x3007 | 0x4007 => Some(EngineUiAction::TogglePanel(PanelId::Stats)),
        0x3008 | 0x4008 => Some(EngineUiAction::TogglePanel(PanelId::UiDesigner)),
        0x40FF => Some(EngineUiAction::ResetLayout),
        0x5001 => Some(EngineUiAction::OpenAbout),
        0x6001 => Some(EngineUiAction::ChangeMode(
            ae_core::modules::EngineMode::Play,
        )),
        0x6002 => Some(EngineUiAction::ChangeMode(if is_editing {
            ae_core::modules::EngineMode::Play
        } else {
            ae_core::modules::EngineMode::Edit
        })),
        0x7001 => Some(EngineUiAction::TitleBarAction(
            irisui::widgets::TitleBarAction::Minimize,
        )),
        0x7002 => Some(EngineUiAction::TitleBarAction(
            irisui::widgets::TitleBarAction::Maximize,
        )),
        0x7003 => Some(EngineUiAction::TitleBarAction(
            irisui::widgets::TitleBarAction::Close,
        )),
        _ => None,
    }
}

/// Constructs a declarative [`irisui::workbench::MenuBarDescriptor`] representing the engine's application menu bar.
pub fn build_workbench_menubar_descriptor(
    params: &MenuBarParams<'_>,
) -> irisui::workbench::MenuBarDescriptor {
    let mut descriptor = irisui::workbench::MenuBarDescriptor::new()
        .with_height(MENUBAR_HEIGHT)
        .with_branding(irisui::workbench::MenuBarBrandingDescriptor::new(
            super::icons::ICON_AEON_LOGO,
            Color::hex("#E4E7EB"),
            28.0,
        ));

    // File Menu
    let mut file_menu = irisui::workbench::MenuSectionDescriptor::new("File", 0x1000);
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::disabled_action_full(
        Some("🗎"),
        "New Project",
        Some("Ctrl N"),
        0x1001,
    ));
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
        "🗁",
        "Load Scene",
        Some("Ctrl O"),
        0x1002,
    ));
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::separator());
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
        "🖫",
        "Save Scene",
        Some("Ctrl S"),
        0x1003,
    ));
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
        "🖫",
        "Save Scene As",
        Some("Ctrl Shift S"),
        0x1004,
    ));
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::separator());
    file_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
        "⏻",
        "Exit",
        Some("Alt F4"),
        0x1005,
    ));
    descriptor.add_menu(file_menu);

    // Edit Menu
    let mut edit_menu = irisui::workbench::MenuSectionDescriptor::new("Edit", 0x2000);
    if params.can_undo {
        edit_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
            "↩",
            "Undo",
            Some("Ctrl Z"),
            0x2001,
        ));
    } else {
        edit_menu.add_item(irisui::workbench::MenuItemDescriptor::disabled_action_full(
            Some("↩"),
            "Undo",
            Some("Ctrl Z"),
            0x2001,
        ));
    }

    if params.can_redo {
        edit_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
            "↪",
            "Redo",
            Some("Ctrl Y"),
            0x2002,
        ));
    } else {
        edit_menu.add_item(irisui::workbench::MenuItemDescriptor::disabled_action_full(
            Some("↪"),
            "Redo",
            Some("Ctrl Y"),
            0x2002,
        ));
    }

    edit_menu.add_item(irisui::workbench::MenuItemDescriptor::separator());
    edit_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
        "⚙",
        "Preferences...",
        Some("Ctrl ,"),
        0x2003,
    ));
    descriptor.add_menu(edit_menu);

    // View Menu
    let mut view_menu = irisui::workbench::MenuSectionDescriptor::new("View", 0x3000);
    let panels = [
        (PanelId::Hierarchy, "Hierarchy", 0x3001),
        (PanelId::Inspector, "Inspector", 0x3002),
        (PanelId::Assets, "Asset Browser", 0x3003),
        (PanelId::Console, "Console", 0x3004),
        (PanelId::MaterialEditor, "Material Editor", 0x3005),
        (PanelId::AnimationTimeline, "Timeline", 0x3006),
        (PanelId::Stats, "Statistics", 0x3007),
        (PanelId::UiDesigner, "UI Designer", 0x3008),
    ];
    for (pid, label, tag) in panels {
        let is_open = params.layout_state.is_panel_visible(pid);
        let check = if is_open { "✓" } else { " " };
        view_menu.add_item(irisui::workbench::MenuItemDescriptor::action_with_icon(
            check, label, tag,
        ));
    }
    descriptor.add_menu(view_menu);

    // Window Menu
    let mut window_menu = irisui::workbench::MenuSectionDescriptor::new("Window", 0x4000);
    for (pid, label, tag) in panels {
        let is_open = params.layout_state.is_panel_visible(pid);
        let check = if is_open { "✓" } else { " " };
        window_menu.add_item(irisui::workbench::MenuItemDescriptor::action_with_icon(
            check,
            label,
            tag + 0x1000,
        ));
    }
    window_menu.add_item(irisui::workbench::MenuItemDescriptor::separator());
    window_menu.add_item(irisui::workbench::MenuItemDescriptor::action_with_icon(
        "↺",
        "Reset Layout to Default",
        0x40FF,
    ));
    descriptor.add_menu(window_menu);

    // Help Menu
    let mut help_menu = irisui::workbench::MenuSectionDescriptor::new("Help", 0x5000);
    help_menu.add_item(irisui::workbench::MenuItemDescriptor::action_full(
        "ℹ",
        "About Aeon Engine",
        Some("F1"),
        0x5001,
    ));
    descriptor.add_menu(help_menu);

    // Right action buttons & window controls
    if params.is_editing {
        descriptor =
            descriptor.with_action_button("▶ Play", Color::hex("#228b22"), Color::WHITE, 0x6001);
    } else {
        descriptor =
            descriptor.with_action_button("⏹ Stop", Color::hex("#dc2626"), Color::WHITE, 0x6002);
    }

    descriptor
        .with_window_control("—", false, 0x7001)
        .with_window_control("□", false, 0x7002)
        .with_window_control("✕", true, 0x7003)
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui::widgets::declarative::hash_label_with_seed;

    #[test]
    fn test_top_menu_bar_declarative_build_and_roles() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let layout_state = PanelLayoutState::default();
        let mut action_bus = irisui::workbench::ActionBus::new();

        let clicked = build_top_menu_bar(
            &mut tree,
            root,
            &MenuBarParams {
                screen_width: 1920.0,
                layout_state: &layout_state,
                can_undo: true,
                can_redo: false,
                is_editing: true,
            },
            &mut action_bus,
        );

        assert!(clicked.is_none(), "No action triggered on initial build");
        assert!(action_bus.is_empty(), "Action bus must be empty initially");
        assert_eq!(tree.active_menu_bar_tag, None, "Menus start closed");
    }

    #[test]
    fn test_menu_bar_click_activates_dropdown() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let layout_state = PanelLayoutState::default();
        let mut action_bus = irisui::workbench::ActionBus::new();

        let file_tag = hash_label_with_seed(0, "File");
        tree.event_state.frame_clicked_tag = Some(file_tag);

        let _ = build_top_menu_bar(
            &mut tree,
            root,
            &MenuBarParams {
                screen_width: 1920.0,
                layout_state: &layout_state,
                can_undo: true,
                can_redo: false,
                is_editing: true,
            },
            &mut action_bus,
        );

        assert_eq!(
            tree.active_menu_bar_tag,
            Some(file_tag),
            "Clicking File header must activate dropdown"
        );
    }

    #[test]
    fn test_top_menu_bar_button_widths_and_gaps() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let layout_state = PanelLayoutState::default();
        let mut action_bus = irisui::workbench::ActionBus::new();

        let _ = build_top_menu_bar(
            &mut tree,
            root,
            &MenuBarParams {
                screen_width: 1920.0,
                layout_state: &layout_state,
                can_undo: true,
                can_redo: false,
                is_editing: true,
            },
            &mut action_bus,
        );

        let mut text_system = irisui::text::TextSystem::new();
        text_system.measure_subtree_text(&mut tree, root);
        let mut layout_engine = LayoutEngine::new();
        let _ = layout_engine.compute_layout(&mut tree, Size::new(1920.0, 1080.0));

        let mut menu_items: Vec<(String, Rect)> = Vec::new();
        let mut logo_rect = None;

        for (_id, node) in tree.iter() {
            if node.name.as_deref() == Some("MenuBarBrandingLogo") {
                logo_rect = Some(node.computed_rect);
            } else if node.role == WidgetRole::MenuBarItem
                && let Some(ref text) = node.text
            {
                menu_items.push((text.clone(), node.computed_rect));
            }
        }

        assert!(logo_rect.is_some(), "Logo must be present");
        let logo = logo_rect.unwrap();
        assert_eq!(
            menu_items.len(),
            5,
            "5 menu items: File, Edit, View, Window, Help"
        );

        // Gap between logo and File
        let logo_to_file = menu_items[0].1.x - logo.right();
        assert!(
            (logo_to_file - 10.0).abs() < 0.1,
            "Logo to File gap must be 10.0, found {:.2}",
            logo_to_file
        );

        // Gap between adjacent menu buttons
        for i in 0..menu_items.len() - 1 {
            let gap = menu_items[i + 1].1.x - menu_items[i].1.right();
            assert!(
                (gap - 4.0).abs() < 0.1,
                "Button gap between {} and {} must be 4.0, found {:.2}",
                menu_items[i].0,
                menu_items[i + 1].0,
                gap
            );
        }

        // Each button must have width = text_advance + (item_padding * 2.0)
        for (name, rect) in &menu_items {
            let text_advance = irisui::text::measure_text_advance(name, 12.0);
            let expected_min_w = text_advance + 12.0; // 6.0 * 2.0
            assert!(
                (rect.width - expected_min_w).abs() < 1.0,
                "Button {} width must be close to {:.1}, found {:.1}",
                name,
                expected_min_w,
                rect.width
            );
        }
    }

    #[test]
    fn test_menu_bar_action_emission_to_action_bus() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let layout_state = PanelLayoutState::default();
        let mut action_bus = irisui::workbench::ActionBus::new();

        // Simulate active File menu dropdown and clicking Exit (0x1005)
        let file_tag = hash_label_with_seed(0, "File");
        tree.active_menu_bar_tag = Some(file_tag);
        tree.event_state.frame_clicked_tag = Some(0x1005);

        let clicked = build_top_menu_bar(
            &mut tree,
            root,
            &MenuBarParams {
                screen_width: 1920.0,
                layout_state: &layout_state,
                can_undo: true,
                can_redo: false,
                is_editing: true,
            },
            &mut action_bus,
        );

        assert_eq!(clicked, Some(0x1005), "Exit tag must be captured");
        assert_eq!(
            action_bus.len(),
            1,
            "Exit action must be emitted to ActionBus"
        );
        let emitted = action_bus.drain().next();
        assert!(
            matches!(emitted, Some(EngineUiAction::Exit)),
            "Emitted action must match EngineUiAction::Exit, found {:?}",
            emitted
        );
    }
}