// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Top Application Menu Bar & Floating Dropdown Subsystem
//!
//! Provides the primary desktop application header menu bar (File, Edit, View,
//! Window, Help) and elevated floating dropdown popup cascades using 100% pure
//! declarative [`irisui::workbench::MenuBarDescriptor`] and [`irisui::workbench::ActionBus`].

use crate::ui::EngineUiAction;
use crate::ui::iris_bridge::export_modal::EXPORT_MODAL_TAG;
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;
use irisui::workbench::{
    MenuBarBrandingDescriptor, MenuBarDescriptor, MenuSectionDescriptor, WorkbenchUiScopeExt,
};

/// Height of the top menubar panel in physical pixels.
pub const MENUBAR_HEIGHT: f32 = 36.0;

/// Semantic tag emitted by the menubar minimize window control button.
pub const WINDOW_CONTROL_TAG_MINIMIZE: UiTag = UiTag::from_str("window.control.minimize");
/// Semantic tag emitted by the menubar maximize/restore window control button.
pub const WINDOW_CONTROL_TAG_MAXIMIZE: UiTag = UiTag::from_str("window.control.maximize");
/// Semantic tag emitted by the menubar close window control button.
pub const WINDOW_CONTROL_TAG_CLOSE: UiTag = UiTag::from_str("window.control.close");
/// Semantic modal tag reserved for the 'About Aeon Engine' informational dialog.
pub const ABOUT_MODAL_TAG: UiTag = UiTag::from_str("modal.about");

/// Default width of floating dropdown popup menus in physical pixels.
pub const DROPDOWN_WIDTH: f32 = 260.0;

/// Ordered panel identifiers matching bit indices in `view_panel_mask`.
pub const VIEW_PANEL_IDS: [PanelId; 8] = [
    PanelId::Hierarchy,
    PanelId::Inspector,
    PanelId::Assets,
    PanelId::Console,
    PanelId::MaterialEditor,
    PanelId::AnimationTimeline,
    PanelId::Stats,
    PanelId::UiDesigner,
];

/// Semantic panel entries paired with their menu labels (no ad-hoc numeric tag tables).
const VIEW_PANELS: [(PanelId, &str); 8] = [
    (PanelId::Hierarchy, "Hierarchy"),
    (PanelId::Inspector, "Inspector"),
    (PanelId::Assets, "Asset Browser"),
    (PanelId::Console, "Console"),
    (PanelId::MaterialEditor, "Material Editor"),
    (PanelId::AnimationTimeline, "Timeline"),
    (PanelId::Stats, "Statistics"),
    (PanelId::UiDesigner, "UI Designer"),
];

/// Declaratively constructs and renders the top application menu bar into the provided [`UiScope`].
///
/// Directly emits strongly-typed [`EngineUiAction`] commands and opens declarative modals without
/// manual numeric tag dispatch tables.
pub fn build_top_menu_bar(
    scope: &mut UiScope<'_>,
    screen_width: f32,
    view_panel_mask: u64,
    can_undo: bool,
    can_redo: bool,
    is_editing: bool,
    action_bus: &mut irisui::workbench::ActionBus<EngineUiAction>,
) -> Option<UiTag> {
    (
        build_workbench_menubar_descriptor(view_panel_mask, can_undo, can_redo, is_editing)
            .render_into(scope, screen_width, action_bus),
        render_about_modal_if_active(scope),
    )
        .0
}

/// Returns the standardized declarative descriptor for the 'About Aeon Engine' informational card dialog.
#[must_use]
pub fn about_engine_descriptor() -> irisui::workbench::InfoCardDialogDescriptor<()> {
    irisui::workbench::InfoCardDialogDescriptor::new("About Aeon Engine")
        .with_icon("ℹ")
        .with_app_name("Aeon Engine")
        .with_app_name_color(Color::rgba(0.0, 0.90, 1.0, 1.0))
        .with_version("v0.9.0")
        .with_description("Modern Data-Oriented 3D Game Engine & Autonomous Iris IDE")
        .with_copyright("Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.")
        .with_link(
            "License, v. 2.0 (MPL-2.0). https://mozilla.org/MPL/2.0/",
            "https://mozilla.org/MPL/2.0/",
        )
        .with_footer_notice_colored(
            "This program comes with ABSOLUTELY NO WARRANTY.",
            Color::rgba(1.0, 0.40, 0.40, 1.0),
        )
        .with_close_label("Close")
        .with_size(480.0, 250.0)
}

/// Renders the informational 'About Aeon Engine' modal dialogue using [`WorkbenchUiScopeExt::info_modal_tagged_auto`].
pub fn render_about_modal_if_active(scope: &mut UiScope<'_>) {
    scope.info_modal_tagged_auto(ABOUT_MODAL_TAG.raw(), &about_engine_descriptor());
}

/// Constructs a strongly-typed [`MenuBarDescriptor`] representing the engine's application menu bar.
#[must_use]
pub fn build_workbench_menubar_descriptor(
    view_panel_mask: u64,
    can_undo: bool,
    can_redo: bool,
    is_editing: bool,
) -> MenuBarDescriptor<EngineUiAction> {
    attach_right_controls(
        MenuBarDescriptor::new()
            .with_height(MENUBAR_HEIGHT)
            .with_branding(MenuBarBrandingDescriptor::new(
                super::icons::ICON_AEON_LOGO,
                Color::hex("#E4E7EB"),
                28.0,
            ))
            .with_menu(file_menu_descriptor())
            .with_menu(edit_menu_descriptor(can_undo, can_redo))
            .with_menu(view_menu_descriptor(view_panel_mask))
            .with_menu(window_menu_descriptor(view_panel_mask))
            .with_menu(help_menu_descriptor()),
        is_editing,
    )
}

/// Constructs the declarative 'File' menu section descriptor.
fn file_menu_descriptor() -> MenuSectionDescriptor<EngineUiAction> {
    MenuSectionDescriptor::titled("File")
        .with_disabled(Some(UiIcon::Glyph("🗎")), "New Project", Some("Ctrl N"))
        .with_command_full(
            "🗁",
            "Load Scene",
            Some("Ctrl O"),
            EngineUiAction::OpenLoadSceneDialog,
        )
        .with_separator()
        .with_command_full("🖫", "Save Scene", Some("Ctrl S"), EngineUiAction::SaveScene)
        .with_command_full(
            "🖫",
            "Save Scene As",
            Some("Ctrl Shift S"),
            EngineUiAction::OpenSaveSceneDialog,
        )
        .with_separator()
        .with_modal_trigger(
            Some(UiIcon::Glyph("📦")),
            "Export Project...",
            None::<&str>,
            EXPORT_MODAL_TAG,
        )
        .with_separator()
        .with_command_full("⏻", "Exit", Some("Alt F4"), EngineUiAction::Exit)
}

/// Constructs the declarative 'Edit' menu section descriptor.
fn edit_menu_descriptor(can_undo: bool, can_redo: bool) -> MenuSectionDescriptor<EngineUiAction> {
    attach_redo_action(
        attach_undo_action(MenuSectionDescriptor::titled("Edit"), can_undo),
        can_redo,
    )
    .with_separator()
    .with_command_full(
        "⚙",
        "Preferences...",
        Some("Ctrl ,"),
        EngineUiAction::OpenPreferences,
    )
}

/// Conditionally appends active or disabled Undo command item to `menu`.
fn attach_undo_action(
    menu: MenuSectionDescriptor<EngineUiAction>,
    can_undo: bool,
) -> MenuSectionDescriptor<EngineUiAction> {
    if can_undo {
        menu.with_command_full("↩", "Undo", Some("Ctrl Z"), EngineUiAction::Undo)
    } else {
        menu.with_disabled(Some(UiIcon::Glyph("↩")), "Undo", Some("Ctrl Z"))
    }
}

/// Conditionally appends active or disabled Redo command item to `menu`.
fn attach_redo_action(
    menu: MenuSectionDescriptor<EngineUiAction>,
    can_redo: bool,
) -> MenuSectionDescriptor<EngineUiAction> {
    if can_redo {
        menu.with_command_full("↪", "Redo", Some("Ctrl Y"), EngineUiAction::Redo)
    } else {
        menu.with_disabled(Some(UiIcon::Glyph("↪")), "Redo", Some("Ctrl Y"))
    }
}

/// Constructs the declarative 'View' menu section descriptor.
fn view_menu_descriptor(view_panel_mask: u64) -> MenuSectionDescriptor<EngineUiAction> {
    VIEW_PANELS.iter().enumerate().fold(
        MenuSectionDescriptor::titled("View"),
        |menu, (idx, &(pid, label))| {
            menu.with_toggle_check(
                label,
                (view_panel_mask & (1u64 << idx)) != 0,
                EngineUiAction::TogglePanel(pid),
            )
        },
    )
}

/// Constructs the declarative 'Window' menu section descriptor.
fn window_menu_descriptor(view_panel_mask: u64) -> MenuSectionDescriptor<EngineUiAction> {
    VIEW_PANELS
        .iter()
        .enumerate()
        .fold(
            MenuSectionDescriptor::titled("Window"),
            |menu, (idx, &(pid, label))| {
                menu.with_toggle_check(
                    label,
                    (view_panel_mask & (1u64 << idx)) != 0,
                    EngineUiAction::TogglePanel(pid),
                )
            },
        )
        .with_separator()
        .with_command_with_icon("↺", "Reset Layout to Default", EngineUiAction::ResetLayout)
}

/// Constructs the declarative 'Help' menu section descriptor.
fn help_menu_descriptor() -> MenuSectionDescriptor<EngineUiAction> {
    MenuSectionDescriptor::titled("Help").with_modal_trigger(
        Some(UiIcon::Glyph("ℹ")),
        "About Aeon Engine",
        Some("F1"),
        ABOUT_MODAL_TAG,
    )
}

/// Attaches play/stop mode toggles and window management controls to the menubar.
fn attach_right_controls(
    descriptor: MenuBarDescriptor<EngineUiAction>,
    is_editing: bool,
) -> MenuBarDescriptor<EngineUiAction> {
    (if is_editing {
        descriptor.with_command_button(
            "▶ Play",
            Color::hex("#228b22"),
            Color::WHITE,
            EngineUiAction::ChangeMode(ae_core::modules::EngineMode::Play),
        )
    } else {
        descriptor.with_command_button(
            "⏹ Stop",
            Color::hex("#dc2626"),
            Color::WHITE,
            EngineUiAction::ChangeMode(ae_core::modules::EngineMode::Edit),
        )
    })
    .with_window_control("—", false, WINDOW_CONTROL_TAG_MINIMIZE)
    .with_window_control("□", false, WINDOW_CONTROL_TAG_MAXIMIZE)
    .with_window_control("✕", true, WINDOW_CONTROL_TAG_CLOSE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_menu_bar_declarative_build_and_roles() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let dock_state = crate::ui::panel_layout::create_default_dock_state();
        let view_panel_mask = dock_state.tab_visibility_mask(&VIEW_PANEL_IDS);
        let mut action_bus = irisui::workbench::ActionBus::new();
        let mut scope = UiScope::new(&mut tree, root);

        let clicked = build_top_menu_bar(
            &mut scope,
            1920.0,
            view_panel_mask,
            true,
            false,
            true,
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
        let dock_state = crate::ui::panel_layout::create_default_dock_state();
        let view_panel_mask = dock_state.tab_visibility_mask(&VIEW_PANEL_IDS);
        let mut action_bus = irisui::workbench::ActionBus::new();

        let file_tag = UiTag::from_str("File").raw();
        tree.event_state.frame_clicked_tag = Some(file_tag);
        let mut scope = UiScope::new(&mut tree, root);

        let _ = build_top_menu_bar(
            &mut scope,
            1920.0,
            view_panel_mask,
            true,
            false,
            true,
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
        let dock_state = crate::ui::panel_layout::create_default_dock_state();
        let view_panel_mask = dock_state.tab_visibility_mask(&VIEW_PANEL_IDS);
        let mut action_bus = irisui::workbench::ActionBus::new();

        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = build_top_menu_bar(
                &mut scope,
                1920.0,
                view_panel_mask,
                true,
                false,
                true,
                &mut action_bus,
            );
        }

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
        let dock_state = crate::ui::panel_layout::create_default_dock_state();
        let view_panel_mask = dock_state.tab_visibility_mask(&VIEW_PANEL_IDS);
        let mut action_bus = irisui::workbench::ActionBus::new();

        // Simulate active File menu dropdown and clicking Exit using hierarchical UiTag
        let file_tag = UiTag::from_str("File");
        let exit_tag = file_tag.child("Exit");
        tree.active_menu_bar_tag = Some(file_tag.raw());
        tree.event_state.frame_clicked_tag = Some(exit_tag.raw());
        let mut scope = UiScope::new(&mut tree, root);

        let clicked = build_top_menu_bar(
            &mut scope,
            1920.0,
            view_panel_mask,
            true,
            false,
            true,
            &mut action_bus,
        );

        assert_eq!(clicked, Some(exit_tag), "Exit tag must be captured");
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

    #[test]
    fn test_menu_bar_export_trigger_opens_export_modal_directly() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let dock_state = crate::ui::panel_layout::create_default_dock_state();
        let view_panel_mask = dock_state.tab_visibility_mask(&VIEW_PANEL_IDS);
        let mut action_bus = irisui::workbench::ActionBus::new();

        let file_tag = UiTag::from_str("File");
        let export_item_tag = file_tag.child("Export Project...");
        tree.active_menu_bar_tag = Some(file_tag.raw());
        tree.event_state.frame_clicked_tag = Some(export_item_tag.raw());

        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = build_top_menu_bar(
                &mut scope,
                1920.0,
                view_panel_mask,
                true,
                false,
                true,
                &mut action_bus,
            );
        }

        assert!(
            tree.is_modal_open(EXPORT_MODAL_TAG.raw()),
            "Clicking Export Project... must open EXPORT_MODAL_TAG directly"
        );
        assert!(action_bus.is_empty());
    }

    #[test]
    fn test_top_menu_bar_about_modal_opens_and_renders() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node creation must succeed");
        let dock_state = crate::ui::panel_layout::create_default_dock_state();
        let view_panel_mask = dock_state.tab_visibility_mask(&VIEW_PANEL_IDS);
        let mut action_bus = irisui::workbench::ActionBus::new();

        // 1. Initially About modal is closed
        assert!(!tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        // 2. Open modal directly or via tag
        tree.open_modal(ABOUT_MODAL_TAG.raw());
        assert!(tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        // 3. Render menubar; about modal must be present in UiLayer::Modal
        {
            let mut scope = UiScope::new(&mut tree, root);
            let _ = build_top_menu_bar(
                &mut scope,
                1920.0,
                view_panel_mask,
                true,
                false,
                true,
                &mut action_bus,
            );
        }

        let mut has_scrim = false;
        let mut has_about_card = false;
        let mut card_rect = Rect::ZERO;
        for (_id, node) in tree.iter() {
            if node.layer == UiLayer::Modal {
                if node.name.as_deref() == Some("ModalScrim") {
                    has_scrim = true;
                    assert!(
                        node.computed_rect.width >= 1920.0,
                        "ModalScrim must span full screen width, got {:?}",
                        node.computed_rect
                    );
                } else if node.name.as_deref() == Some("ModalDialogCard") {
                    has_about_card = true;
                    card_rect = node.computed_rect;
                }
            }
        }

        assert!(has_scrim, "Modal scrim must be rendered on UiLayer::Modal");
        assert!(has_about_card, "Modal dialog card must be rendered");
        assert!(
            card_rect.x > 0.0 && card_rect.y > 0.0,
            "Modal dialog card must be centered in positive viewport space, found: {:?}",
            card_rect
        );
        let expected_x = (1920.0 - 480.0) / 2.0;
        assert!(
            (card_rect.x - expected_x).abs() < 1.0,
            "Expected x to be centered near {}, found {}",
            expected_x,
            card_rect.x
        );
    }

    #[test]
    fn test_modal_second_open_click_inside_card_does_not_dismiss_root_cause() {
        use irisui::core::event::{MouseButton, UiEvent};
        use irisui::core::geometry::Point;

        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();

        tree.open_modal(ABOUT_MODAL_TAG.raw());
        assert!(tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        {
            let mut scope = UiScope::new(&mut tree, root);
            render_about_modal_if_active(&mut scope);
        }
        assert!(tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        tree.close_modal(ABOUT_MODAL_TAG.raw());
        assert!(!tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        tree.open_modal(ABOUT_MODAL_TAG.raw());
        assert!(tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        tree.clear();
        let root = tree.create_root().unwrap();
        {
            let mut scope = UiScope::new(&mut tree, root);
            render_about_modal_if_active(&mut scope);
        }
        assert!(tree.is_modal_open(ABOUT_MODAL_TAG.raw()));

        tree.dispatch_event(&UiEvent::MouseDown {
            point: Point::new(960.0, 540.0),
            button: MouseButton::Left,
        });
        tree.dispatch_event(&UiEvent::MouseUp {
            point: Point::new(960.0, 540.0),
            button: MouseButton::Left,
        });

        {
            let mut scope = UiScope::new(&mut tree, root);
            render_about_modal_if_active(&mut scope);
        }
        assert!(
            tree.is_modal_open(ABOUT_MODAL_TAG.raw()),
            "Clicking inside the modal card on the 2nd open must NOT dismiss the modal"
        );

        tree.dispatch_event(&UiEvent::MouseDown {
            point: Point::new(50.0, 50.0),
            button: MouseButton::Left,
        });
        tree.dispatch_event(&UiEvent::MouseUp {
            point: Point::new(50.0, 50.0),
            button: MouseButton::Left,
        });

        {
            let mut scope = UiScope::new(&mut tree, root);
            render_about_modal_if_active(&mut scope);
        }
        assert!(
            !tree.is_modal_open(ABOUT_MODAL_TAG.raw()),
            "Clicking outside on the backdrop scrim MUST dismiss the modal"
        );
    }
}