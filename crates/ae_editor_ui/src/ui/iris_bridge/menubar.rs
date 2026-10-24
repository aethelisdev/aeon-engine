// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top Application Menu Bar & Floating Dropdown Subsystem
//!
//! Provides the primary desktop application header menu bar (File, Edit, View,
//! Window, Help) and elevated floating dropdown popup cascades using 100% pure
//! declarative [`UiScope::menu_bar`].
//!

use crate::ui::EngineUiAction;
use crate::ui::iris_bridge::types::DropdownAction;
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

/// Builds the top application menu bar widget tree directly using declarative [`UiScope::menu_bar`].
///
/// Dispatches menu actions, dropdown popups, and play/stop toggle buttons autonomously
/// without leaking internal widget identifiers to the engine layer.
pub fn build_top_menu_bar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &MenuBarParams<'_>,
) -> Option<DropdownAction> {
    let mut scope = UiScope::new(tree, parent_id);
    scope
        .menu_bar(params.screen_width, MENUBAR_HEIGHT)
        .item_padding(6.0)
        .item_gap(4.0)
        .logo_gap(10.0)
        .show(|bar| {
            // 0. Application Branding Logo
            bar.branding_icon(super::icons::ICON_AEON_LOGO, Color::hex("#E4E7EB"), 28.0);

            // 1. File Menu
            bar.menu("File", |m| {
                m.item_disabled("🗎", "New Project", Some("Ctrl N"));
                m.item(
                    "🗁",
                    "Load Scene",
                    Some("Ctrl O"),
                    DropdownAction::UiAction(EngineUiAction::OpenLoadSceneDialog),
                );
                m.separator();
                m.item(
                    "🖫",
                    "Save Scene",
                    Some("Ctrl S"),
                    DropdownAction::UiAction(EngineUiAction::SaveScene),
                );
                m.item(
                    "🖫",
                    "Save Scene As",
                    Some("Ctrl Shift S"),
                    DropdownAction::UiAction(EngineUiAction::OpenSaveSceneDialog),
                );
                m.separator();
                m.item(
                    "⏻",
                    "Exit",
                    Some("Alt F4"),
                    DropdownAction::UiAction(EngineUiAction::Exit),
                );
            });

            // 2. Edit Menu
            bar.menu("Edit", |m| {
                if params.can_undo {
                    m.item(
                        "↩",
                        "Undo",
                        Some("Ctrl Z"),
                        DropdownAction::UiAction(EngineUiAction::Undo),
                    );
                } else {
                    m.item_disabled("↩", "Undo", Some("Ctrl Z"));
                }

                if params.can_redo {
                    m.item(
                        "↪",
                        "Redo",
                        Some("Ctrl Y"),
                        DropdownAction::UiAction(EngineUiAction::Redo),
                    );
                } else {
                    m.item_disabled("↪", "Redo", Some("Ctrl Y"));
                }

                m.separator();
                m.item(
                    "⚙",
                    "Preferences...",
                    Some("Ctrl ,"),
                    DropdownAction::OpenPreferences,
                );
            });

            // 3. View Menu
            bar.menu("View", |m| {
                let panels = [
                    (PanelId::Hierarchy, "Hierarchy"),
                    (PanelId::Inspector, "Inspector"),
                    (PanelId::Assets, "Asset Browser"),
                    (PanelId::Console, "Console"),
                    (PanelId::MaterialEditor, "Material Editor"),
                    (PanelId::AnimationTimeline, "Timeline"),
                    (PanelId::Stats, "Statistics"),
                    (PanelId::UiDesigner, "UI Designer"),
                ];

                for (pid, label) in panels {
                    let is_open = params.layout_state.is_panel_visible(pid);
                    let check = if is_open { "✓" } else { " " };
                    m.item("", label, Some(check), DropdownAction::TogglePanel(pid));
                }
            });

            // 4. Window Menu
            bar.menu("Window", |m| {
                let panels = [
                    (PanelId::Hierarchy, "Hierarchy"),
                    (PanelId::Inspector, "Inspector"),
                    (PanelId::Assets, "Asset Browser"),
                    (PanelId::Console, "Console"),
                    (PanelId::MaterialEditor, "Material Editor"),
                    (PanelId::AnimationTimeline, "Timeline"),
                    (PanelId::Stats, "Statistics"),
                    (PanelId::UiDesigner, "UI Designer"),
                ];

                for (pid, label) in panels {
                    let is_open = params.layout_state.is_panel_visible(pid);
                    let check = if is_open { "✓" } else { " " };
                    m.item("", label, Some(check), DropdownAction::TogglePanel(pid));
                }

                m.separator();
                m.item(
                    "↺",
                    "Reset Layout to Default",
                    None,
                    DropdownAction::ResetLayout,
                );
            });

            // 5. Help Menu
            bar.menu("Help", |m| {
                m.item(
                    "ℹ",
                    "About Aeon Engine",
                    Some("F1"),
                    DropdownAction::OpenAbout,
                );
            });

            // 6. Right Group: Play/Stop Action Button & Window Management Controls
            bar.right(|right| {
                if params.is_editing {
                    right.action_button(
                        "▶ Play",
                        Color::hex("#228b22"),
                        Color::WHITE,
                        DropdownAction::UiAction(EngineUiAction::ChangeMode(
                            ae_core::modules::EngineMode::Play,
                        )),
                    );
                } else {
                    right.action_button(
                        "⏹ Stop",
                        Color::hex("#dc2626"),
                        Color::WHITE,
                        DropdownAction::UiAction(EngineUiAction::ChangeMode(
                            ae_core::modules::EngineMode::Play,
                        )),
                    );
                }

                // Window Management Controls (Minimize, Maximize/Restore, Close)
                right.window_button("—", false, DropdownAction::WindowMinimize);
                right.window_button("□", false, DropdownAction::WindowMaximize);
                right.window_button("✕", true, DropdownAction::WindowClose);
            });
        })
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

        let action = build_top_menu_bar(
            &mut tree,
            root,
            &MenuBarParams {
                screen_width: 1920.0,
                layout_state: &layout_state,
                can_undo: true,
                can_redo: false,
                is_editing: true,
            },
        );

        assert!(action.is_none(), "No action triggered on initial build");
        assert_eq!(tree.active_menu_bar_tag, None, "Menus start closed");
    }

    #[test]
    fn test_menu_bar_click_activates_dropdown() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let layout_state = PanelLayoutState::default();

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
}