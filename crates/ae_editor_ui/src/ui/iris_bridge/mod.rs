// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Iris UI Hybrid Bridge for the Aeon Engine Editor.
//!
//! Manages the retained-mode `UiTree`, `IrisRenderer`, typography, interactive hover/click
//! event routing, and `MenuBarBuilder`/`DropdownMenuBuilder` rendering directly on top of the editor frame.

pub mod about;
pub mod menubar;
pub mod modals;
pub mod preferences;
pub mod stats;
pub mod status_bar;
pub mod types;
pub mod viewport_hud;

pub use about::{AboutDialogTargets, build_about_dialog};
pub use modals::*;
pub use preferences::{
    PreferencesAction, PreferencesDropdownId, PreferencesParams, PreferencesSliderId,
    PreferencesTargets, PreferencesToggleId, build_preferences_dialog,
};
pub use stats::{
    StatsPanelAction, StatsPanelNodes, StatsPanelParams, StatsPanelTargets, build_stats_panel,
};
pub use types::{ActiveMenu, DropdownAction, IrisOverlayEventResult};
pub use viewport_hud::{
    ViewportHudAction, ViewportHudDropdownId, ViewportHudParams, ViewportHudTargets,
    build_viewport_hud,
};

use crate::ui::panel_layout::PanelLayoutState;
use ae_core::modules::EngineModule;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSection, TextSystem};
use std::collections::HashSet;
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
    /// Cached bounding box and interactive widget targets of the active Preferences dialog.
    pub preferences_targets: Option<PreferencesTargets>,
    /// Cached interaction targets for 3D Viewport HUD.
    pub viewport_hud_targets: Option<ViewportHudTargets>,
    /// Currently open dropdown menu in Viewport HUD.
    pub viewport_hud_dropdown: Option<ViewportHudDropdownId>,
    /// Dispatched action queue for Viewport HUD interactions.
    pub viewport_hud_actions: Vec<ViewportHudAction>,
    /// Cached interaction targets for Performance Stats & Telemetry panel.
    pub stats_targets: Option<StatsPanelTargets>,
    /// Persistent node handles for the Stats & Profiler panel in retained mode.
    pub stats_nodes: Option<StatsPanelNodes>,
    /// Last bounding rectangle allocated for the Stats & Profiler panel.
    pub last_stats_rect: Option<Rect>,
    /// Last recorded screen dimensions.
    pub last_dimensions: (f32, f32),
    /// Last recorded UI Zoom factor.
    pub last_zoom_factor: f32,
    /// Explicit flag requesting full layout reconstruction on invalidation.
    pub needs_layout_rebuild: bool,
    /// Content area vertical scroll offset for Stats & Telemetry panel.
    pub stats_scroll_y: f32,
    /// Dispatched action queue for Stats & Telemetry panel interactions.
    pub stats_actions: Vec<StatsPanelAction>,
    /// Custom floating position coordinates for the Preferences panel.
    pub preferences_pos: Option<Point>,
    /// Active drag offset from window top-left when dragging the title bar.
    pub preferences_drag_offset: Option<Point>,
    /// Currently selected tab index in the Preferences dialog (0..=9).
    pub preferences_tab: u8,
    /// Content area vertical scroll offset for Preferences dialog.
    pub preferences_scroll_y: f32,
    /// Currently open dropdown ComboBox in the Preferences dialog.
    pub preferences_dropdown: Option<PreferencesDropdownId>,
    /// Currently active slider drag descriptor: `(slider_id, track_rect, min_val, max_val)`.
    pub active_slider_drag: Option<(PreferencesSliderId, Rect, f32, f32)>,
    /// Live typing input buffer for the new folder modal.
    pub new_folder_buffer: String,
    /// Live typing input buffer for the rename modal.
    pub rename_buffer: String,
    /// Set of currently collapsed card/section identifiers in the Preferences dialog.
    pub collapsed_sections: HashSet<&'static str>,
    /// Currently active inline number input editing state in Preferences: `(slider_id, typed_buffer)`.
    pub active_number_input: Option<(PreferencesSliderId, String)>,
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
    /// Whether the Preferences modal dialogue is currently visible.
    pub show_preferences: bool,
    /// Reference to graphics settings for Preferences rendering.
    pub graphics_settings: &'a GraphicsSettings,
    /// Reference to snapping settings for Preferences rendering.
    pub snapping_settings: &'a SnapSettings,
    /// Reference to editor configuration for Preferences rendering.
    pub editor_config: &'a EditorConfig,
    /// Whether live hot-reload editor updates are active.
    pub enable_live_updates: bool,
    /// Set of enabled engine core modules for Preferences rendering.
    pub enabled_modules: &'a HashSet<EngineModule>,
    /// Current display/UI zoom factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
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
    /// Screen rectangle bounding the 3D viewport canvas.
    pub viewport_rect: Rect,
    /// Reference to the active 3D camera.
    pub camera: &'a ae_renderer::camera::Camera,
    /// Whether wireframe rendering is currently enabled.
    pub wireframe_enabled: bool,
    /// Currently active gizmo manipulation mode (Translate, Rotate, Scale).
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    /// Currently active gizmo coordinate space (World, Local).
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
    /// Currently selected entity in the editor, if any.
    pub selected_entity: Option<hecs::Entity>,
    /// Active ECS world reference for billboard entity query.
    pub world: &'a hecs::World,
    /// Bounding rectangle allocated for the Performance Stats panel, if active.
    pub stats_panel_rect: Option<Rect>,
    /// Whether the viewport coordinate grid is enabled.
    pub grid_enabled: bool,
    /// Smoothed FPS rate.
    pub fps: f32,
    /// Historical frame pacing ring buffer.
    pub frame_pacing: &'a ae_core::telemetry::FrameRingBuffer<240>,
    /// Calculated frametime variance, 1% low, and 0.1% low stats.
    pub frame_pacing_stats: &'a ae_core::telemetry::FramePacingStats,
    /// CPU thread synchronization timings breakdown.
    pub cpu_timings: &'a ae_core::telemetry::CpuSyncTimings,
    /// GPU render pass execution durations.
    pub gpu_pass_timings: &'a ae_core::telemetry::GpuPassTimings,
    /// Granular draw call metrics and batch counts.
    pub draw_call_stats: &'a ae_core::telemetry::DrawCallBreakdown,
    /// Categorized VRAM memory consumption.
    pub vram_stats: &'a ae_core::telemetry::VramStats,
    /// Total rendered triangles in current frame.
    pub render_triangles: u64,
    /// Total rendered vertices in current frame.
    pub render_vertices: u64,
    /// Hardware GPU adapter device name.
    pub gpu_adapter_name: &'a str,
    /// Active graphics API backend (e.g. Vulkan, Metal, DX12).
    pub gpu_backend: &'a str,
    /// Count of active entities in the ECS world.
    pub active_entities_count: usize,
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
            preferences_targets: None,
            viewport_hud_targets: None,
            viewport_hud_dropdown: None,
            viewport_hud_actions: Vec::new(),
            stats_targets: None,
            stats_nodes: None,
            last_stats_rect: None,
            last_dimensions: (0.0, 0.0),
            last_zoom_factor: 1.0,
            needs_layout_rebuild: false,
            stats_scroll_y: 0.0,
            stats_actions: Vec::new(),
            preferences_pos: None,
            preferences_drag_offset: None,
            preferences_tab: 0,
            preferences_scroll_y: 0.0,
            preferences_dropdown: None,
            active_slider_drag: None,
            new_folder_buffer: String::new(),
            rename_buffer: String::new(),
            collapsed_sections: HashSet::new(),
            active_number_input: None,
            screen_width: 1920.0,
            screen_height: 1080.0,
            is_visible: true,
            target_format,
            start_time: std::time::Instant::now(),
        }
    }

    /// Consumes and returns all queued Viewport HUD actions.
    pub fn take_viewport_hud_actions(&mut self) -> Vec<ViewportHudAction> {
        std::mem::take(&mut self.viewport_hud_actions)
    }

    /// Consumes and returns all queued Stats & Profiler panel actions.
    pub fn take_stats_actions(&mut self) -> Vec<StatsPanelAction> {
        std::mem::take(&mut self.stats_actions)
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
        self.layout_engine.clear();
        self.command_list.clear();
        self.dropdown_items.clear();
        self.dropdown_rect = None;
        self.about_targets = None;
        self.delete_targets = None;
        self.new_folder_targets = None;
        self.rename_targets = None;
        self.loading_targets = None;
        self.preferences_targets = None;
        self.viewport_hud_targets = None;
        self.stats_targets = None;

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

        // 9. If Preferences dialogue is active, build its floating card
        if params.show_preferences {
            let blink_caret = (self.start_time.elapsed().as_millis() % 1060) < 530;
            let (pref_id, targets) = build_preferences_dialog(
                &mut self.tree,
                preferences::PreferencesParams {
                    screen_width,
                    screen_height,
                    window_pos: self.preferences_pos,
                    active_tab: self.preferences_tab,
                    scroll_offset_y: self.preferences_scroll_y,
                    active_dropdown: self.preferences_dropdown,
                    collapsed_sections: &self.collapsed_sections,
                    active_number_input: self
                        .active_number_input
                        .as_ref()
                        .map(|(id, s)| (*id, s.as_str())),
                    blink_caret,
                    cursor_pos: self.cursor_pos,
                    zoom_factor: params.zoom_factor,
                    graphics_settings: params.graphics_settings,
                    snapping_settings: params.snapping_settings,
                    editor_config: params.editor_config,
                    enable_live_updates: params.enable_live_updates,
                    enabled_modules: params.enabled_modules,
                },
            );
            if let Some(root_id) = self.tree.root() {
                let _ = self.tree.add_child(root_id, pref_id);
            }
            self.preferences_targets = Some(targets);
        }

        // 10. If Viewport canvas is valid, build Viewport HUD (Toolbar, Compass, Camera HUD, Billboards, Play Mode)
        if params.viewport_rect.width > 20.0 && params.viewport_rect.height > 20.0 {
            let mut hud_targets = ViewportHudTargets::default();
            viewport_hud::build_viewport_hud(
                &mut self.tree,
                root,
                &ViewportHudParams {
                    viewport_rect: params.viewport_rect,
                    camera: params.camera,
                    wireframe_enabled: params.wireframe_enabled,
                    gizmo_mode: params.gizmo_mode,
                    gizmo_space: params.gizmo_space,
                    snapping: params.snapping_settings,
                    cursor_pos: self.cursor_pos,
                    active_dropdown: self.viewport_hud_dropdown,
                    selected_entity: params.selected_entity,
                    world: params.world,
                    is_editing: params.is_editing,
                },
                &mut hud_targets,
            );
            self.viewport_hud_targets = Some(hud_targets);
        }

        // 11. If Stats & Profiler panel is active, manage its retained lifecycle and dynamic updates
        if let Some(stats_rect) = params.stats_panel_rect
            && stats_rect.width > 20.0
            && stats_rect.height > 20.0
        {
            let stats_params = StatsPanelParams {
                panel_rect: stats_rect,
                scroll_y: self.stats_scroll_y,
                cursor_pos: self.cursor_pos,
                wireframe_enabled: params.wireframe_enabled,
                grid_enabled: params.grid_enabled,
                fps: params.fps,
                frame_pacing: params.frame_pacing,
                frame_pacing_stats: params.frame_pacing_stats,
                cpu_timings: params.cpu_timings,
                gpu_pass_timings: params.gpu_pass_timings,
                draw_call_stats: params.draw_call_stats,
                vram_stats: params.vram_stats,
                render_triangles: params.render_triangles,
                render_vertices: params.render_vertices,
                gpu_adapter_name: params.gpu_adapter_name,
                gpu_backend: params.gpu_backend,
                active_entities_count: params.active_entities_count,
                selected_entity: params.selected_entity,
            };

            let mut stats_targets = StatsPanelTargets::default();
            let nodes =
                stats::build_stats_panel(&mut self.tree, root, &stats_params, &mut stats_targets);
            stats::update_stats_panel_values(&mut self.tree, &nodes, &stats_params, &stats_targets);
            self.stats_targets = Some(stats_targets);
            self.stats_nodes = Some(nodes);
            self.last_stats_rect = Some(stats_rect);
            self.last_zoom_factor = params.zoom_factor;
        } else {
            self.stats_nodes = None;
            self.stats_targets = None;
            self.last_stats_rect = None;
        }

        // Populate DrawCommandList from resolved layout nodes
        self.populate_draw_commands(root, None);

        // Zero-Tree GPU Oscilloscope: directly push 60 curve quads into DrawCommandList
        if let Some(ref nodes) = self.stats_nodes {
            stats::append_oscilloscope_quads(
                &mut self.command_list,
                nodes.canvas_rect,
                params.frame_pacing,
            );
        }

        self.needs_layout_rebuild = false;
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
        if let Some(ref targets) = self.preferences_targets
            && (targets.card_rect.contains_point(point)
                || targets
                    .active_dropdown_popup_rect
                    .is_some_and(|r| r.contains_point(point)))
        {
            return true;
        }
        if let Some(ref hud) = self.viewport_hud_targets {
            if let Some(dd_rect) = hud.active_dropdown_popup_rect
                && dd_rect.contains_point(point)
            {
                return true;
            }
            if hud.buttons.iter().any(|(_, r)| r.contains_point(point))
                || hud
                    .dropdown_triggers
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
                || hud
                    .compass_knobs
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
                || hud
                    .billboard_icons
                    .iter()
                    .any(|(_, r)| r.contains_point(point))
            {
                return true;
            }
        }
        if let Some(ref targets) = self.stats_targets
            && targets.panel_rect.contains_point(point)
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

        // 0b. If Preferences panel is active, intercept clicks, drags, number inputs, and escape key
        if let Some(ref targets) = self.preferences_targets {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key),
                            text,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if let Some((slider_id, ref mut buffer)) = self.active_number_input {
                        match *key {
                            winit::keyboard::KeyCode::Escape => {
                                self.active_number_input = None;
                                result.consumed = true;
                                return result;
                            }
                            winit::keyboard::KeyCode::Enter
                            | winit::keyboard::KeyCode::NumpadEnter => {
                                if let Some(&(_, _, min_val, max_val, _)) = targets
                                    .number_inputs
                                    .iter()
                                    .find(|(id, _, _, _, _)| *id == slider_id)
                                    && let Ok(mut val) = buffer.trim().parse::<f32>()
                                {
                                    val = val.clamp(min_val, max_val);
                                    if slider_id == PreferencesSliderId::PhysicsFrequency {
                                        val = preferences::PHYSICS_HZ_PRESETS
                                            .iter()
                                            .copied()
                                            .min_by(|a, b| {
                                                (a - val).abs().total_cmp(&(b - val).abs())
                                            })
                                            .unwrap_or(val);
                                    }
                                    result.preferences_action =
                                        Some(PreferencesAction::SetSliderValue(slider_id, val));
                                }
                                self.active_number_input = None;
                                result.consumed = true;
                                return result;
                            }
                            winit::keyboard::KeyCode::Backspace => {
                                buffer.pop();
                                result.consumed = true;
                                return result;
                            }
                            _ => {
                                if let Some(t) = text {
                                    for c in t.chars() {
                                        if c.is_ascii_digit()
                                            || (c == '.' && !buffer.contains('.'))
                                            || (c == '-' && buffer.is_empty())
                                        {
                                            buffer.push(c);
                                        }
                                    }
                                }
                                result.consumed = true;
                                return result;
                            }
                        }
                    }

                    if *key == winit::keyboard::KeyCode::Escape {
                        if self.preferences_dropdown.is_some() {
                            self.preferences_dropdown = None;
                        } else {
                            result.close_preferences = true;
                            self.preferences_drag_offset = None;
                            self.active_slider_drag = None;
                            self.active_number_input = None;
                        }
                        result.consumed = true;
                        return result;
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if targets.content_rect.contains_point(self.cursor_pos) {
                        let scroll_y = match delta {
                            winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 28.0,
                            winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };
                        let max_scroll =
                            (targets.total_content_height - targets.content_rect.height + 32.0)
                                .max(0.0);
                        self.preferences_scroll_y =
                            (self.preferences_scroll_y - scroll_y).clamp(0.0, max_scroll);
                        result.consumed = true;
                        return result;
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.cursor_pos = Point::new(position.x as f32, position.y as f32);
                    if let Some(drag_offset) = self.preferences_drag_offset {
                        let max_x = (self.screen_width - preferences::PREF_CARD_WIDTH).max(0.0);
                        let max_y = (self.screen_height - preferences::PREF_CARD_HEIGHT).max(28.0);
                        let new_x = (self.cursor_pos.x - drag_offset.x).clamp(0.0, max_x);
                        let new_y = (self.cursor_pos.y - drag_offset.y).clamp(28.0, max_y);
                        self.preferences_pos = Some(Point::new(new_x, new_y));
                        result.consumed = true;
                        return result;
                    }
                    if let Some((slider_id, track_rect, min_val, max_val)) = self.active_slider_drag
                    {
                        let norm =
                            ((self.cursor_pos.x - track_rect.x) / track_rect.width).clamp(0.0, 1.0);
                        let mut val = min_val + norm * (max_val - min_val);
                        if slider_id == PreferencesSliderId::PhysicsFrequency {
                            val = preferences::PHYSICS_HZ_PRESETS
                                .iter()
                                .copied()
                                .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                                .unwrap_or(val);
                        }
                        result.preferences_action =
                            Some(PreferencesAction::SetSliderValue(slider_id, val));
                        result.consumed = true;
                        return result;
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: WinitMouseButton::Left,
                    ..
                } => {
                    if self.preferences_drag_offset.is_some() {
                        self.preferences_drag_offset = None;
                        result.consumed = true;
                        return result;
                    }
                    if self.active_slider_drag.is_some() {
                        self.active_slider_drag = None;
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

                    // 1. If an active dropdown popup is open
                    if let Some(popup_rect) = targets.active_dropdown_popup_rect {
                        if popup_rect.contains_point(click_point) {
                            if let Some(&(idx, _, _)) = targets
                                .active_dropdown_items
                                .iter()
                                .find(|(_, r, _)| r.contains_point(click_point))
                                && let Some(dd_id) = self.preferences_dropdown
                            {
                                result.preferences_action =
                                    Some(PreferencesAction::SelectDropdownItem(dd_id, idx));
                                self.preferences_dropdown = None;
                                result.consumed = true;
                                return result;
                            }
                        } else {
                            self.preferences_dropdown = None;
                        }
                    }

                    // 2. Direct numeric input box clicks
                    for &(slider_id, box_rect, _, _, cur_val) in &targets.number_inputs {
                        if box_rect.contains_point(click_point) {
                            let initial_str = match slider_id {
                                PreferencesSliderId::PhysicsFrequency
                                | PreferencesSliderId::UndoHistoryLimit
                                | PreferencesSliderId::CloudAltitude
                                | PreferencesSliderId::FogDistance => format!("{:.0}", cur_val),
                                PreferencesSliderId::ShadowBias => format!("{:.4}", cur_val),
                                _ => format!("{:.2}", cur_val),
                            };
                            self.active_number_input = Some((slider_id, initial_str));
                            self.active_slider_drag = None;
                            self.preferences_dropdown = None;
                            result.consumed = true;
                            return result;
                        }
                    }

                    // If clicked outside active number box, commit and close it
                    if let Some((slider_id, buffer)) = self.active_number_input.take()
                        && let Some(&(_, _, min_val, max_val, _)) = targets
                            .number_inputs
                            .iter()
                            .find(|(id, _, _, _, _)| *id == slider_id)
                        && let Ok(mut val) = buffer.trim().parse::<f32>()
                    {
                        val = val.clamp(min_val, max_val);
                        if slider_id == PreferencesSliderId::PhysicsFrequency {
                            val = preferences::PHYSICS_HZ_PRESETS
                                .iter()
                                .copied()
                                .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                                .unwrap_or(val);
                        }
                        result.preferences_action =
                            Some(PreferencesAction::SetSliderValue(slider_id, val));
                    }

                    // 3. Close button
                    if targets.close_button.contains_point(click_point) {
                        result.close_preferences = true;
                        self.preferences_drag_offset = None;
                        self.active_slider_drag = None;
                        self.preferences_dropdown = None;
                        self.active_number_input = None;
                        result.consumed = true;
                        return result;
                    }

                    // 4. Titlebar dragging
                    if targets.title_bar_rect.contains_point(click_point) {
                        let card_x = targets.card_rect.x;
                        let card_y = targets.card_rect.y;
                        self.preferences_drag_offset =
                            Some(Point::new(click_point.x - card_x, click_point.y - card_y));
                        result.consumed = true;
                        return result;
                    }

                    // 5. Tab clicks
                    for &(tab_idx, tab_rect) in &targets.tabs {
                        if tab_rect.contains_point(click_point) {
                            self.preferences_tab = tab_idx;
                            self.preferences_dropdown = None;
                            self.active_number_input = None;
                            self.preferences_scroll_y = 0.0;
                            result.preferences_action = Some(PreferencesAction::SelectTab(tab_idx));
                            result.consumed = true;
                            return result;
                        }
                    }

                    // 6. Content Area Interactive Elements (Dropdowns, Toggles, Sliders, Section Toggles)
                    if targets.content_rect.contains_point(click_point) {
                        // Section Header Collapsible Accordion Toggles
                        for &(sec_id, sec_rect) in &targets.section_toggles {
                            if sec_rect.contains_point(click_point) {
                                if self.collapsed_sections.contains(sec_id) {
                                    self.collapsed_sections.remove(sec_id);
                                } else {
                                    self.collapsed_sections.insert(sec_id);
                                }
                                result.preferences_action =
                                    Some(PreferencesAction::ToggleSection(sec_id));
                                result.consumed = true;
                                return result;
                            }
                        }

                        for &(dd_id, dd_rect) in &targets.dropdowns {
                            if dd_rect.contains_point(click_point) {
                                if self.preferences_dropdown == Some(dd_id) {
                                    self.preferences_dropdown = None;
                                } else {
                                    self.preferences_dropdown = Some(dd_id);
                                }
                                result.consumed = true;
                                return result;
                            }
                        }

                        for &(toggle_id, toggle_rect) in &targets.toggles {
                            if toggle_rect.contains_point(click_point) {
                                result.preferences_action =
                                    Some(PreferencesAction::Toggle(toggle_id));
                                result.consumed = true;
                                return result;
                            }
                        }

                        for &(slider_id, track_rect, min_val, max_val, _) in &targets.sliders {
                            if track_rect.contains_point(click_point) {
                                self.active_slider_drag =
                                    Some((slider_id, track_rect, min_val, max_val));
                                let norm = ((click_point.x - track_rect.x) / track_rect.width)
                                    .clamp(0.0, 1.0);
                                let mut val = min_val + norm * (max_val - min_val);
                                if slider_id == PreferencesSliderId::PhysicsFrequency {
                                    val = preferences::PHYSICS_HZ_PRESETS
                                        .iter()
                                        .copied()
                                        .min_by(|a, b| (a - val).abs().total_cmp(&(b - val).abs()))
                                        .unwrap_or(val);
                                }
                                result.preferences_action =
                                    Some(PreferencesAction::SetSliderValue(slider_id, val));
                                result.consumed = true;
                                return result;
                            }
                        }
                    }

                    // 7. If click is inside card, consume it so it doesn't click through to underlying canvas
                    if targets.card_rect.contains_point(click_point) {
                        result.consumed = true;
                        return result;
                    }
                }
                _ => {}
            }
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

        // 0f. If Viewport HUD is active, intercept clicks and dropdown interactions
        if let Some(ref hud_targets) = self.viewport_hud_targets
            && let WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } = event
        {
            let click_point = self.cursor_pos;

            // 1. If an active dropdown popup is open
            if self.viewport_hud_dropdown.is_some() {
                // Check if clicked inside dropdown items
                for (action, rect, _) in &hud_targets.active_dropdown_items {
                    if rect.contains_point(click_point) {
                        self.viewport_hud_actions.push(action.clone());
                        self.viewport_hud_dropdown = None;
                        result.consumed = true;
                        return result;
                    }
                }

                // If clicked outside popup, close it
                if let Some(popup_rect) = hud_targets.active_dropdown_popup_rect
                    && !popup_rect.contains_point(click_point)
                {
                    self.viewport_hud_dropdown = None;
                }
            }

            // 2. Check dropdown triggers
            for (dd_id, rect) in &hud_targets.dropdown_triggers {
                if rect.contains_point(click_point) {
                    self.viewport_hud_dropdown = if self.viewport_hud_dropdown == Some(*dd_id) {
                        None
                    } else {
                        Some(*dd_id)
                    };
                    result.consumed = true;
                    return result;
                }
            }

            // 3. Check toolbar buttons
            for (action, rect) in &hud_targets.buttons {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions.push(action.clone());
                    result.consumed = true;
                    return result;
                }
            }

            // 4. Check compass knobs
            for (action, rect) in &hud_targets.compass_knobs {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions.push(action.clone());
                    result.consumed = true;
                    return result;
                }
            }

            // 5. Check billboard icons
            for (ent, rect) in &hud_targets.billboard_icons {
                if rect.contains_point(click_point) {
                    self.viewport_hud_actions
                        .push(ViewportHudAction::SelectEntity(*ent));
                    result.consumed = true;
                    return result;
                }
            }
        }

        // 0g. If Stats panel is active, intercept checkbox clicks
        if let Some(ref stats_targets) = self.stats_targets
            && let WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } = event
        {
            let click_point = self.cursor_pos;
            if let Some(wire_rect) = stats_targets.wireframe_checkbox_rect
                && wire_rect.contains_point(click_point)
            {
                self.stats_actions.push(StatsPanelAction::ToggleWireframe);
                result.consumed = true;
                return result;
            }
            if let Some(grid_rect) = stats_targets.grid_checkbox_rect
                && grid_rect.contains_point(click_point)
            {
                self.stats_actions.push(StatsPanelAction::ToggleGrid);
                result.consumed = true;
                return result;
            }
            if stats_targets.panel_rect.contains_point(click_point) {
                result.consumed = true;
                return result;
            }
        }

        // 0h. Mouse Wheel scrolling for Stats panel and Preferences dialog
        if let WindowEvent::MouseWheel { delta, .. } = event {
            let delta_y = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y * 24.0,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            };
            if let Some(ref targets) = self.stats_targets
                && targets.panel_rect.contains_point(self.cursor_pos)
            {
                self.stats_scroll_y = (self.stats_scroll_y - delta_y).max(0.0);
                result.consumed = true;
                return result;
            }
            if let Some(ref targets) = self.preferences_targets
                && targets.card_rect.contains_point(self.cursor_pos)
            {
                self.preferences_scroll_y = (self.preferences_scroll_y - delta_y).max(0.0);
                result.consumed = true;
                return result;
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
        let (font_size, line_height, child_count) = {
            let Some(node) = self.tree.get(current) else {
                return;
            };
            (node.font_size, node.line_height, node.children.len())
        };

        if let Some(node) = self.tree.get(current)
            && let Some(ref text) = node.text
        {
            let measured = self
                .text_system
                .measure_text(text, font_size, line_height, None);
            if let Some(node_mut) = self.tree.get_mut(current) {
                node_mut.content_size = measured;
            }
        }

        for i in 0..child_count {
            if let Some(child) = self
                .tree
                .get(current)
                .and_then(|n| n.children.get(i).copied())
            {
                self.measure_tree_text(child);
            }
        }
    }

    /// Recursively converts computed node bounds and styles into `DrawCommandList` instances.
    fn populate_draw_commands(&mut self, current: WidgetId, clip_rect: Option<Rect>) {
        let (child_count, quad, next_clip) = {
            let Some(node) = self.tree.get(current) else {
                return;
            };
            if !node.visible {
                return;
            }

            let child_clip = if node.style.clip_children {
                match clip_rect {
                    Some(existing) => Some(existing.intersect(node.computed_rect)),
                    None => Some(node.computed_rect),
                }
            } else {
                clip_rect
            };

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
                    clip_rect,
                ))
            } else {
                None
            };

            (node.children.len(), quad, child_clip)
        };

        if let Some(q) = quad {
            self.command_list.push_quad(q);
        }

        for i in 0..child_count {
            if let Some(child) = self
                .tree
                .get(current)
                .and_then(|n| n.children.get(i).copied())
            {
                self.populate_draw_commands(child, next_clip);
            }
        }
    }

    /// Collects text rendering sections from all visible layout nodes in the tree.
    pub fn collect_text_sections_from_tree(
        tree: &UiTree,
        active_popup_rect: Option<Rect>,
    ) -> Vec<TextSection<'_>> {
        let mut sections = Vec::new();
        if let Some(root) = tree.root() {
            Self::collect_node_text_from_tree(
                tree,
                root,
                None,
                active_popup_rect,
                false,
                &mut sections,
            );
        }
        sections
    }

    /// Recursive helper extracting text sections from a node subtree.
    fn collect_node_text_from_tree<'a>(
        tree: &'a UiTree,
        current: WidgetId,
        clip_rect: Option<Rect>,
        active_popup_rect: Option<Rect>,
        is_inside_popup: bool,
        sections: &mut Vec<TextSection<'a>>,
    ) {
        let Some(node) = tree.get(current) else {
            return;
        };
        if !node.visible {
            return;
        }

        let child_is_inside_popup = is_inside_popup
            || node
                .name
                .as_deref()
                .map(|n| n.contains("Popup"))
                .unwrap_or(false);

        let child_clip = if node.style.clip_children {
            match clip_rect {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            clip_rect
        };

        if let Some(text) = &node.text
            && !text.is_empty()
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            let is_visible_in_clip = match clip_rect {
                Some(clip) => {
                    node.computed_rect.right() > clip.x
                        && node.computed_rect.x < clip.right()
                        && node.computed_rect.bottom() > clip.y
                        && node.computed_rect.y < clip.bottom()
                }
                None => true,
            };

            let is_occluded_by_popup = if !child_is_inside_popup {
                if let Some(popup) = active_popup_rect {
                    node.computed_rect.right() > popup.x
                        && node.computed_rect.x < popup.right()
                        && node.computed_rect.bottom() > popup.y
                        && node.computed_rect.y < popup.bottom()
                } else {
                    false
                }
            } else {
                false
            };

            if is_visible_in_clip && !is_occluded_by_popup {
                sections.push(
                    TextSection::new(text.clone(), node.computed_rect)
                        .with_font_size(node.font_size, node.line_height)
                        .with_color(node.text_color)
                        .with_align(node.text_align)
                        .with_clip(clip_rect),
                );
            }
        }

        for &child in &node.children {
            Self::collect_node_text_from_tree(
                tree,
                child,
                child_clip,
                active_popup_rect,
                child_is_inside_popup,
                sections,
            );
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

        let active_popup_rect = self
            .preferences_targets
            .as_ref()
            .and_then(|t| t.active_dropdown_popup_rect);
        let sections = Self::collect_text_sections_from_tree(&self.tree, active_popup_rect);
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