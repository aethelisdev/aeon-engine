// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Iris UI Hybrid Bridge for the Aeon Engine Editor.
//!
//! Manages the retained-mode `UiTree`, `IrisRenderer`, typography, interactive hover/click
//! event routing, and `MenuBarBuilder`/`DropdownMenuBuilder` rendering directly on top of the editor frame.

pub mod about;
pub mod menubar;
pub mod modals;
pub mod status_bar;
pub mod types;

pub use about::{AboutDialogTargets, build_about_dialog};
pub use modals::*;
pub use types::{ActiveMenu, DropdownAction, IrisOverlayEventResult};

use crate::ui::panel_layout::PanelLayoutState;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSection, TextSystem};
use std::path::Path;
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

/// Central state manager governing Iris UI editor overlays, menu bar, modal dialogs, and status bar rendering.
pub struct IrisEditorOverlay {
    /// Generational UI tree storing active overlay widget nodes.
    pub tree: UiTree,
    /// Taffy-powered flexbox layout computation engine.
    pub layout_engine: LayoutEngine,
    /// GPU SDF quad and geometry renderer.
    pub renderer: IrisRenderer,
    /// Typography layout and shaping engine.
    pub text_system: TextSystem,
    /// GPU text atlas and glyphon text renderer.
    pub text_renderer: Option<TextRenderer>,
    /// Active frame drawing command stream.
    pub command_list: DrawCommandList,
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Currently open dropdown menu category.
    pub active_menu: Option<ActiveMenu>,
    /// Interactive dropdown item hit-testing targets.
    pub dropdown_items: Vec<(Rect, DropdownAction)>,
    /// Cached bounding box of the active floating dropdown.
    pub dropdown_rect: Option<Rect>,
    /// Cached bounding box and close button hit targets of the active About dialog.
    pub about_targets: Option<AboutDialogTargets>,
    /// Cached bounding box and button targets of the active Delete Confirmation modal.
    pub delete_targets: Option<DeleteModalTargets>,
    /// Cached bounding box and input targets of the active New Folder modal.
    pub new_folder_targets: Option<NewFolderModalTargets>,
    /// Cached bounding box and input targets of the active Rename modal.
    pub rename_targets: Option<RenameModalTargets>,
    /// Cached bounding box targets of the active Asset Loading splash screen.
    pub loading_targets: Option<LoadingOverlayTargets>,
    /// Live typing input buffer for the new folder modal.
    pub new_folder_buffer: String,
    /// Live typing input buffer for the rename modal.
    pub rename_buffer: String,
    /// Last measured screen width.
    pub screen_width: f32,
    /// Last measured screen height.
    pub screen_height: f32,
    /// Whether the editor overlays are visible.
    pub is_visible: bool,
    /// Target surface texture format.
    pub target_format: wgpu::TextureFormat,
    /// Creation instant used for smooth sub-second continuous UI animations.
    pub start_time: std::time::Instant,
}

/// Parameters required for reconstructing and resolving all Iris UI editor overlays.
pub struct OverlayUpdateParams<'a> {
    /// Screen dimensions (width, height) in physical pixels.
    pub dimensions: (f32, f32),
    /// Whether the editor is currently in Edit mode.
    pub is_editing: bool,
    /// Active panel layout state reference.
    pub layout_state: &'a PanelLayoutState,
    /// Whether undo is available.
    pub can_undo: bool,
    /// Whether redo is available.
    pub can_redo: bool,
    /// Whether the About Aeon Engine modal dialogue is currently visible.
    pub show_about: bool,
    /// Optional target path pending delete confirmation.
    pub delete_target: Option<&'a Path>,
    /// Optional new folder parent path.
    pub new_folder_parent: Option<&'a Path>,
    /// Optional rename target path and is_folder flag.
    pub rename_target: Option<(&'a Path, bool)>,
    /// Whether background assets are currently being loaded.
    pub is_loading_assets: bool,
    /// Optional status notification message spans with text color.
    pub status_spans: Option<&'a [(String, Color)]>,
}

impl IrisEditorOverlay {
    /// Height of the top menubar panel in physical pixels (matching egui geometry).
    pub const MENUBAR_HEIGHT: f32 = menubar::MENUBAR_HEIGHT;

    /// Height of the bottom status bar in physical pixels.
    pub const STATUS_BAR_HEIGHT: f32 = status_bar::STATUS_BAR_HEIGHT;

    /// Initializes a new Iris UI editor overlay pipeline for the specified surface format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self {
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            renderer: IrisRenderer::new(device, target_format),
            text_system: TextSystem::new(),
            text_renderer: None,
            command_list: DrawCommandList::new(),
            cursor_pos: Point::new(-1000.0, -1000.0),
            active_menu: None,
            dropdown_items: Vec::new(),
            dropdown_rect: None,
            about_targets: None,
            delete_targets: None,
            new_folder_targets: None,
            rename_targets: None,
            loading_targets: None,
            new_folder_buffer: String::new(),
            rename_buffer: String::new(),
            screen_width: 1920.0,
            screen_height: 1080.0,
            is_visible: true,
            target_format,
            start_time: std::time::Instant::now(),
        }
    }

    /// Reconstructs and resolves layout for the top menu bar, active dropdown, modals, and bottom status bar.
    pub fn update_overlays(&mut self, params: OverlayUpdateParams<'_>) {
        let (screen_width, screen_height) = params.dimensions;
        self.screen_width = screen_width;
        self.screen_height = screen_height;

        if !self.is_visible {
            self.command_list.clear();
            return;
        }

        self.tree.clear();
        self.command_list.clear();
        self.dropdown_items.clear();
        self.dropdown_rect = None;
        self.about_targets = None;
        self.delete_targets = None;
        self.new_folder_targets = None;
        self.rename_targets = None;
        self.loading_targets = None;

        let Ok(root) = self.tree.create_root() else {
            return;
        };

        if let Some(root_node) = self.tree.get_mut(root) {
            root_node.set_name("IrisRoot");
            root_node.set_style(
                Style::new()
                    .flex_col()
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(screen_width)
                    .height(screen_height),
            );
        }

        // 1. Top MenuBar
        let menu_bar_id = menubar::build_top_menu_bar(
            &mut self.tree,
            screen_width,
            self.cursor_pos,
            self.active_menu,
            params.is_editing,
        );
        let _ = self.tree.add_child(root, menu_bar_id);

        // 2. Bottom Diagnostics & Status Bar
        let status_bar_id = status_bar::build_bottom_status_bar(
            &mut self.tree,
            status_bar::StatusBarParams {
                screen_width,
                screen_height,
                status_spans: params.status_spans,
            },
        );
        let _ = self.tree.add_child(root, status_bar_id);

        // Pre-measure all text nodes to populate intrinsic content_size
        self.measure_tree_text(root);

        // Compute Taffy layout for top menu bar
        let _ = self
            .layout_engine
            .compute_layout(&mut self.tree, Size::new(screen_width, screen_height));

        // 3. If a dropdown menu is open, build and position its floating popup
        if let Some(active) = self.active_menu {
            let anchor_x = match active {
                ActiveMenu::File => 6.0,
                ActiveMenu::Edit => 44.0,
                ActiveMenu::View => 84.0,
                ActiveMenu::Window => 126.0,
                ActiveMenu::Help => 186.0,
            };

            let (dropdown_id, items, dd_rect) = menubar::build_floating_dropdown(
                &mut self.tree,
                active,
                anchor_x,
                self.cursor_pos,
                params.layout_state,
                params.can_undo,
                params.can_redo,
            );

            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, dropdown_id);
            }
            self.dropdown_items = items;
            self.dropdown_rect = Some(dd_rect);
        }

        // 4. If About Aeon Engine modal dialogue is active, build its centered card
        if params.show_about {
            let (about_id, targets) =
                build_about_dialog(&mut self.tree, screen_width, screen_height, self.cursor_pos);
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, about_id);
            }
            self.about_targets = Some(targets);
        }

        // 5. If Delete Confirmation modal is active, build its card
        if let Some(target_path) = params.delete_target {
            let (del_id, targets) = build_delete_modal(
                &mut self.tree,
                target_path,
                screen_width,
                screen_height,
                self.cursor_pos,
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, del_id);
            }
            self.delete_targets = Some(targets);
        }

        let elapsed_secs = self.start_time.elapsed().as_secs_f32();
        let cursor_blink_visible = (self.start_time.elapsed().as_millis() / 530).is_multiple_of(2);

        // 6. If New Folder modal is active, build its card
        if let Some(parent_path) = params.new_folder_parent {
            let input_name = self.new_folder_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let (folder_id, targets) = build_new_folder_modal(
                &mut self.tree,
                modals::FolderModalParams {
                    parent_path,
                    input_text: input_name,
                    text_width,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: self.cursor_pos,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, folder_id);
            }
            self.new_folder_targets = Some(targets);
        }

        // 7. If Rename modal is active, build its card
        if let Some((target_path, is_folder)) = params.rename_target {
            let input_name = self.rename_buffer.as_str();
            let text_width = self
                .text_system
                .measure_text(input_name, 12.0, 28.0, None)
                .width;
            let (rename_id, targets) = build_rename_modal(
                &mut self.tree,
                modals::RenameModalParams {
                    target_path,
                    input_text: input_name,
                    text_width,
                    is_folder,
                    cursor_blink_visible,
                    screen_width,
                    screen_height,
                    cursor_pos: self.cursor_pos,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, rename_id);
            }
            self.rename_targets = Some(targets);
        }

        // 8. If Asset Loading overlay is active, build its splash screen
        if params.is_loading_assets {
            let (loading_id, targets) = build_loading_overlay(
                &mut self.tree,
                modals::LoadingOverlayParams {
                    screen_width,
                    screen_height,
                    time_secs: elapsed_secs,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, loading_id);
            }
            self.loading_targets = Some(targets);
        }

        // Populate DrawCommandList from resolved layout nodes
        self.populate_draw_commands(root);
    }

    /// Returns true if the given coordinate is over the menubar, status bar, or active dropdown/modal.
    pub fn is_point_over_overlay(&self, point: Point) -> bool {
        if self.about_targets.is_some()
            || self.delete_targets.is_some()
            || self.new_folder_targets.is_some()
            || self.rename_targets.is_some()
            || self.loading_targets.is_some()
        {
            return true;
        }
        if point.y <= Self::MENUBAR_HEIGHT {
            return true;
        }
        if self.screen_height > Self::STATUS_BAR_HEIGHT
            && point.y >= (self.screen_height - Self::STATUS_BAR_HEIGHT)
        {
            return true;
        }
        if let Some(dd_rect) = self.dropdown_rect
            && dd_rect.contains_point(point)
        {
            return true;
        }
        false
    }

    /// Intercepts and processes window mouse input and cursor movement events.
    pub fn handle_event(&mut self, event: &WindowEvent) -> IrisOverlayEventResult {
        let mut result = IrisOverlayEventResult::default();

        // 0a. If Loading splash is active, consume all interaction to block underlying clicks
        if self.loading_targets.is_some() {
            result.consumed = true;
            return result;
        }

        // 0b. If About modal is active, intercept clicks and escape key with highest priority
        if let Some(ref targets) = self.about_targets {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if *key == winit::keyboard::KeyCode::Escape {
                        result.close_about = true;
                        result.consumed = true;
                        return result;
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.bottom_close_rect.contains_point(click_point)
                    {
                        result.close_about = true;
                        result.consumed = true;
                        return result;
                    }
                    if targets.link_rect.contains_point(click_point) {
                        about::open_url("https://mozilla.org/MPL/2.0/");
                        result.consumed = true;
                        return result;
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.close_about = true;
                        result.consumed = true;
                        return result;
                    }
                    result.consumed = true;
                    return result;
                }
                _ => {}
            }
        }

        // 0c. If Delete Confirmation modal is active
        if let Some(ref targets) = self.delete_targets {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match *key {
                    winit::keyboard::KeyCode::Escape => {
                        result.cancel_delete = true;
                        result.consumed = true;
                        return result;
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        result.confirm_delete = true;
                        result.consumed = true;
                        return result;
                    }
                    _ => {}
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.cancel_btn_rect.contains_point(click_point)
                    {
                        result.cancel_delete = true;
                        result.consumed = true;
                        return result;
                    }
                    if targets.confirm_btn_rect.contains_point(click_point) {
                        result.confirm_delete = true;
                        result.consumed = true;
                        return result;
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.cancel_delete = true;
                        result.consumed = true;
                        return result;
                    }
                    result.consumed = true;
                    return result;
                }
                _ => {}
            }
        }

        // 0d. If New Folder modal is active
        if let Some(ref targets) = self.new_folder_targets {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    self.new_folder_buffer.push_str(text);
                    result.consumed = true;
                    return result;
                }
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            text,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match *key {
                    winit::keyboard::KeyCode::Escape => {
                        result.cancel_new_folder = true;
                        result.consumed = true;
                        return result;
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if !self.new_folder_buffer.trim().is_empty() {
                            result.create_folder = Some(self.new_folder_buffer.clone());
                        }
                        result.consumed = true;
                        return result;
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        self.new_folder_buffer.pop();
                        result.consumed = true;
                        return result;
                    }
                    _ => {
                        if let Some(t) = text
                            && !t.chars().any(|c| c.is_control())
                        {
                            self.new_folder_buffer.push_str(t);
                            result.consumed = true;
                            return result;
                        }
                    }
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.cancel_btn_rect.contains_point(click_point)
                    {
                        result.cancel_new_folder = true;
                        result.consumed = true;
                        return result;
                    }
                    if targets.confirm_btn_rect.contains_point(click_point) {
                        if !self.new_folder_buffer.trim().is_empty() {
                            result.create_folder = Some(self.new_folder_buffer.clone());
                        }
                        result.consumed = true;
                        return result;
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.cancel_new_folder = true;
                        result.consumed = true;
                        return result;
                    }
                    result.consumed = true;
                    return result;
                }
                _ => {}
            }
        }

        // 0e. If Rename modal is active
        if let Some(ref targets) = self.rename_targets {
            match event {
                WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                    self.rename_buffer.push_str(text);
                    result.consumed = true;
                    return result;
                }
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            text,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match *key {
                    winit::keyboard::KeyCode::Escape => {
                        result.cancel_rename = true;
                        result.consumed = true;
                        return result;
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if !self.rename_buffer.trim().is_empty() {
                            result.apply_rename = Some(self.rename_buffer.clone());
                        }
                        result.consumed = true;
                        return result;
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        self.rename_buffer.pop();
                        result.consumed = true;
                        return result;
                    }
                    _ => {
                        if let Some(t) = text
                            && !t.chars().any(|c| c.is_control())
                        {
                            self.rename_buffer.push_str(t);
                            result.consumed = true;
                            return result;
                        }
                    }
                },
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    let click_point = self.cursor_pos;
                    if targets.header_close_rect.contains_point(click_point)
                        || targets.cancel_btn_rect.contains_point(click_point)
                    {
                        result.cancel_rename = true;
                        result.consumed = true;
                        return result;
                    }
                    if targets.confirm_btn_rect.contains_point(click_point) {
                        if !self.rename_buffer.trim().is_empty() {
                            result.apply_rename = Some(self.rename_buffer.clone());
                        }
                        result.consumed = true;
                        return result;
                    }
                    if !targets.dialog_rect.contains_point(click_point) {
                        result.cancel_rename = true;
                        result.consumed = true;
                        return result;
                    }
                    result.consumed = true;
                    return result;
                }
                _ => {}
            }
        }

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Point::new(position.x as f32, position.y as f32);

                // Desktop-standard behavior: when any dropdown menu is active,
                // hovering over other menu headers automatically switches the open menu.
                if self.active_menu.is_some() && self.cursor_pos.y <= Self::MENUBAR_HEIGHT {
                    if self.cursor_pos.x >= 6.0 && self.cursor_pos.x < 44.0 {
                        self.active_menu = Some(ActiveMenu::File);
                    } else if self.cursor_pos.x >= 44.0 && self.cursor_pos.x < 84.0 {
                        self.active_menu = Some(ActiveMenu::Edit);
                    } else if self.cursor_pos.x >= 84.0 && self.cursor_pos.x < 126.0 {
                        self.active_menu = Some(ActiveMenu::View);
                    } else if self.cursor_pos.x >= 126.0 && self.cursor_pos.x < 186.0 {
                        self.active_menu = Some(ActiveMenu::Window);
                    } else if self.cursor_pos.x >= 186.0 && self.cursor_pos.x < 226.0 {
                        self.active_menu = Some(ActiveMenu::Help);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                let click_point = self.cursor_pos;

                // 1. Check if clicking on menubar buttons
                if click_point.y <= Self::MENUBAR_HEIGHT {
                    result.consumed = true;

                    if click_point.x >= 6.0 && click_point.x < 44.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::File) {
                            None
                        } else {
                            Some(ActiveMenu::File)
                        };
                        return result;
                    }

                    if click_point.x >= 44.0 && click_point.x < 84.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::Edit) {
                            None
                        } else {
                            Some(ActiveMenu::Edit)
                        };
                        return result;
                    }

                    if click_point.x >= 84.0 && click_point.x < 126.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::View) {
                            None
                        } else {
                            Some(ActiveMenu::View)
                        };
                        return result;
                    }

                    if click_point.x >= 126.0 && click_point.x < 186.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::Window) {
                            None
                        } else {
                            Some(ActiveMenu::Window)
                        };
                        return result;
                    }

                    if click_point.x >= 186.0 && click_point.x < 226.0 {
                        self.active_menu = if self.active_menu == Some(ActiveMenu::Help) {
                            None
                        } else {
                            Some(ActiveMenu::Help)
                        };
                        return result;
                    }

                    if click_point.x >= (self.screen_width - 90.0) {
                        self.active_menu = None;
                        result.ui_action = Some(crate::ui::EngineUiAction::ChangeMode(
                            ae_core::modules::EngineMode::Play,
                        ));
                        return result;
                    }

                    // Clicked menubar empty area -> close dropdown
                    self.active_menu = None;
                    return result;
                }

                // 2. Check if clicking on an item inside the active dropdown popup
                if self.active_menu.is_some() {
                    let mut clicked_item = None;

                    for (item_rect, action) in &self.dropdown_items {
                        if item_rect.contains_point(click_point) {
                            clicked_item = Some(action.clone());
                            break;
                        }
                    }

                    if let Some(action) = clicked_item {
                        match action {
                            DropdownAction::UiAction(act) => result.ui_action = Some(act),
                            DropdownAction::TogglePanel(p) => result.toggle_panel = Some(p),
                            DropdownAction::ResetLayout => result.reset_layout = true,
                            DropdownAction::OpenPreferences => result.open_preferences = true,
                            DropdownAction::OpenAbout => result.open_about = true,
                        }
                        self.active_menu = None;
                        result.consumed = true;
                        return result;
                    }

                    // Clicked outside dropdown -> dismiss popup
                    self.active_menu = None;
                    result.consumed = true;
                    return result;
                }
            }
            _ => {}
        }

        result
    }

    /// Measures intrinsic text dimensions for all nodes with text content in the subtree.
    fn measure_tree_text(&mut self, current: WidgetId) {
        let (font_size, line_height, text_content, children) = {
            let Some(node) = self.tree.get(current) else {
                return;
            };
            (
                node.font_size,
                node.line_height,
                node.text.clone(),
                node.children.clone(),
            )
        };

        if let Some(text) = text_content {
            let measured = self
                .text_system
                .measure_text(&text, font_size, line_height, None);
            if let Some(node) = self.tree.get_mut(current) {
                node.content_size = measured;
            }
        }

        for child in children {
            self.measure_tree_text(child);
        }
    }

    /// Recursively converts computed node bounds and styles into `DrawCommandList` instances.
    fn populate_draw_commands(&mut self, current: WidgetId) {
        let (children, quad) = {
            let Some(node) = self.tree.get(current) else {
                return;
            };
            if !node.visible {
                return;
            }
            let has_border = (node.style.border.width.top > 0.0
                || node.style.border.width.bottom > 0.0
                || node.style.border.width.left > 0.0
                || node.style.border.width.right > 0.0)
                && node.style.border.color.a > 0.0;

            let quad = if node.computed_rect.width > 0.0
                && node.computed_rect.height > 0.0
                && (node.style.background_color.a > 0.0
                    || has_border
                    || node.style.box_shadow.is_some())
            {
                Some(QuadInstance::from_style(
                    node.computed_rect,
                    &node.style,
                    None,
                ))
            } else {
                None
            };

            (node.children.clone(), quad)
        };

        if let Some(q) = quad {
            self.command_list.push_quad(q);
        }

        for child in children {
            self.populate_draw_commands(child);
        }
    }

    /// Collects text rendering sections from all visible layout nodes in the tree.
    pub fn collect_text_sections_from_tree(tree: &UiTree) -> Vec<TextSection<'_>> {
        let mut sections = Vec::new();
        if let Some(root) = tree.root() {
            Self::collect_node_text_from_tree(tree, root, &mut sections);
        }
        sections
    }

    /// Recursive helper extracting text sections from a node subtree.
    fn collect_node_text_from_tree<'a>(
        tree: &'a UiTree,
        current: WidgetId,
        sections: &mut Vec<TextSection<'a>>,
    ) {
        let Some(node) = tree.get(current) else {
            return;
        };
        if !node.visible {
            return;
        }

        if let Some(text) = &node.text
            && !text.is_empty()
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            sections.push(
                TextSection::new(text.clone(), node.computed_rect)
                    .with_font_size(node.font_size, node.line_height)
                    .with_color(node.text_color)
                    .with_align(node.text_align),
            );
        }

        for &child in &node.children {
            Self::collect_node_text_from_tree(tree, child, sections);
        }
    }

    /// Renders the Iris UI overlay into the target surface framebuffer.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        screen_size: (u32, u32),
    ) {
        if !self.is_visible || (self.command_list.quads.is_empty() && self.tree.root().is_none()) {
            return;
        }

        if self.text_renderer.is_none() {
            self.text_renderer = Some(TextRenderer::new(device, queue, self.target_format));
        }

        let sections = Self::collect_text_sections_from_tree(&self.tree);
        if let Some(txt_renderer) = &mut self.text_renderer {
            txt_renderer.prepare(
                device,
                queue,
                &mut self.text_system,
                [screen_size.0 as f32, screen_size.1 as f32],
                &sections,
            );
        }

        ae_renderer::render::iris_render_pass(ae_renderer::render::IrisRenderPassParams {
            device,
            queue,
            encoder,
            target_view,
            renderer: &mut self.renderer,
            command_list: &self.command_list,
            text_renderer: self.text_renderer.as_ref(),
            screen_size,
        });
    }
}