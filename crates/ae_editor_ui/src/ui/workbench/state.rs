// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::panel_layout::{PanelId, create_default_dock_state};
use crate::ui::types::{ConsoleEntry, EditorTelemetrySnapshot, EngineUiAction};
use irisui::prelude::{Color, Rect};
use winit::window::Window;

/// Action payload sent from async native file dialog threads to the main UI thread.
pub enum SceneDialogAction {
    SaveTo(std::path::PathBuf),
    LoadFrom(std::path::PathBuf),
}

/// The main UI management system for the Aeon Engine.
///
/// Owns the Iris UI overlay pipeline, docking layout, and all persistent
/// editor state (selection, inspector, preferences, console).
pub struct EngineUi {
    pub selected_entity: Option<hecs::Entity>,
    pub status_message: Option<(Vec<(String, Color)>, std::time::Instant)>,
    pub wireframe_enabled: bool,
    pub grid_enabled: bool,
    pub is_loading_assets: bool,
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
    pub should_save_scene: bool,
    pub should_load_scene: bool,
    pub active_scene_path: String,
    pub pending_save_path: Option<std::path::PathBuf>,
    pub pending_load_path: Option<std::path::PathBuf>,
    pub scene_dialog_receivers: Vec<std::sync::mpsc::Receiver<SceneDialogAction>>,
    pub should_exit: bool,
    pub hierarchy_search_query: String,
    /// Snapshot of log entries (updated at most once per frame, only when count changed).
    pub(crate) console_entries: Vec<ConsoleEntry>,
    /// The log count we last snapshotted from – used for change detection.
    pub(crate) console_last_count: u64,
    /// Batched frame performance, memory, and rendering telemetry snapshot updated by the engine.
    pub(crate) telemetry: EditorTelemetrySnapshot,
    /// Physical GPU adapter display name cached after initialization.
    pub(crate) gpu_adapter_name: String,
    /// Active rendering backend identifier cached after initialization.
    pub(crate) gpu_backend: String,
    /// Real-time frames per second (FPS) calculation updated per-frame directly from engine timing.
    pub fps: f32,
    /// Last recorded 3D viewport screen rectangle in logical coordinates.
    pub last_viewport_rect: Rect,
    /// Active UI Zoom / Scaling factor (e.g. 1.0 = 100%, 0.8 = 80%, 1.25 = 125%).
    pub ui_zoom_factor: f32,
    /// Persistent Content / Asset Browser state (directory path, search query, active category filter).
    pub asset_browser: crate::assets::AssetBrowserState,
    /// Persistent 2D UI Designer canvas state (aspect ratio, zoom, pan, grid snap).
    pub ui_designer_state: ae_uidesign::UiDesignerState,
    /// Pending UI actions queued from window event dispatchers.
    pub(crate) pending_actions: Vec<EngineUiAction>,
    /// Autonomous workbench orchestrator coordinating dock hierarchy, retained UI tree, layout, and modal dialogs.
    pub(crate) workbench: irisui::workbench::Workbench<PanelId, EngineUiAction, ()>,
    /// Persistent scoped panel registry retaining all 10 editor panel drawers across frames with zero allocation.
    pub(crate) panel_registry:
        irisui::workbench::ScopedPanelRegistry<PanelId, crate::ui::workbench::EditorPanelScope>,
    /// Reusable deferred invalidation queue replayed onto `workbench.notifier` after each frame pass.
    pub(crate) deferred_notifier: irisui::dock::DeferredNotifier<PanelId>,
    /// Framework-owned GPU surface host coordinating SDF quads, layered typography, and WGPU render passes.
    pub(crate) surface: irisui::IrisSurfaceHost,
    /// Editor overlay panel/modal state bag (transitional until panel views move into direct panel structs).
    pub(crate) iris_overlay: IrisEditorOverlay,
}

impl EngineUi {
    /// Initializes the native Iris UI overlay manager and editor subsystems.
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        _window: &Window,
    ) -> Self {
        let iris_overlay = IrisEditorOverlay::new();
        let surface = irisui::IrisSurfaceHost::new(device, output_color_format);

        let workbench_desc = irisui::workbench::WorkbenchDescriptor::new(
            "Aeon Engine Studio",
            create_default_dock_state(),
        )
        .with_menubar(irisui::workbench::MenuBarDescriptor::new())
        .with_statusbar(irisui::workbench::StatusBarDescriptor::new());

        let mut workbench: irisui::workbench::Workbench<PanelId, EngineUiAction, ()> =
            irisui::workbench::Workbench::new(workbench_desc)
                .with_menubar_height(crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT)
                .with_statusbar_height(crate::ui::iris_bridge::status_bar::STATUS_BAR_HEIGHT)
                .with_non_ui_tab(PanelId::Viewport)
                .with_file_drop_action(|paths, pt| EngineUiAction::ImportDroppedFiles {
                    paths,
                    screen_pos: Some([pt.x, pt.y]),
                })
                .with_zoom_action(EngineUiAction::SetUiScale)
                .with_preferences_panel(PanelId::Preferences)
                .with_window_control_tags(
                    crate::ui::iris_bridge::menubar::WINDOW_CONTROL_TAG_MINIMIZE.raw(),
                    crate::ui::iris_bridge::menubar::WINDOW_CONTROL_TAG_MAXIMIZE.raw(),
                    crate::ui::iris_bridge::menubar::WINDOW_CONTROL_TAG_CLOSE.raw(),
                );

        let panel_registry = crate::ui::workbench::create_editor_panel_registry();
        workbench.sync_domain_panels_from_registry(&panel_registry);
        let init_tokens = iris_overlay.preferences.theme_preset.tokens();
        workbench.set_theme(init_tokens.to_iris_theme());
        workbench.set_dock_style(crate::ui::iris_bridge::editor_dock_style_with_tokens(
            &init_tokens,
        ));
        workbench.tree.set_theme(init_tokens.to_iris_theme());

        Self {
            iris_overlay,
            surface,
            workbench,
            panel_registry,
            deferred_notifier: irisui::dock::DeferredNotifier::new(),
            pending_actions: Vec::new(),
            selected_entity: None,
            status_message: None,
            wireframe_enabled: false,
            grid_enabled: true,
            is_loading_assets: false,
            gizmo_mode: ae_editor::gizmo::GizmoMode::Select,
            gizmo_space: ae_editor::gizmo::GizmoSpace::Local,
            should_save_scene: false,
            should_load_scene: false,
            active_scene_path: "scene.aeworld".to_string(),
            pending_save_path: None,
            pending_load_path: None,
            scene_dialog_receivers: Vec::new(),
            should_exit: false,
            hierarchy_search_query: String::new(),
            console_entries: Vec::new(),
            console_last_count: 0,
            telemetry: EditorTelemetrySnapshot::default(),
            gpu_adapter_name: String::new(),
            gpu_backend: String::new(),
            fps: 60.0,
            last_viewport_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            ui_zoom_factor: 1.0,
            asset_browser: crate::assets::AssetBrowserState::new(),
            ui_designer_state: ae_uidesign::UiDesignerState::default(),
        }
    }

    /// Active UI scaling factor.
    pub fn scale_factor(&self) -> f32 {
        self.workbench.scale_factor()
    }

    /// Steps the active UI scale up or down across predefined scale presets.
    pub fn step_ui_scale(&mut self, increase: bool) -> f32 {
        self.ui_zoom_factor = self.workbench.step_zoom(increase);
        self.ui_zoom_factor
    }

    /// Resets the UI scale back to default 100% (1.0).
    pub fn reset_ui_scale(&mut self) -> f32 {
        self.ui_zoom_factor = self.workbench.reset_zoom();
        self.ui_zoom_factor
    }

    /// Sets a temporary status message displayed at the bottom status bar.
    ///
    /// Triggers an immediate reactive overlay redraw so that the status message
    /// is instantly reflected in the bottom status bar.
    pub fn set_status_message(&mut self, text: impl Into<String>, color: Color) {
        self.status_message = Some((vec![(text.into(), color)], std::time::Instant::now()));
        self.workbench.notifier.tag_all();
    }

    /// Checks status message expiration and clears it if 2.5 seconds have elapsed.
    ///
    /// If expired, clears the message and flags the overlay notifier dirty to
    /// revert the status bar back to "● Ready".
    pub fn expire_status_message(&mut self) {
        if let Some((_, timestamp)) = self.status_message
            && is_status_message_expired(timestamp)
        {
            self.status_message = None;
            self.workbench.notifier.tag_all();
        }
    }

    /// Returns whether any modal dialog, search input, or inspector field currently captures keyboard events.
    pub fn wants_keyboard_input(&self) -> bool {
        self.workbench.wants_keyboard_input()
    }

    /// Polls asynchronous native file dialog receivers and applies their actions.
    pub fn poll_dialog_receivers(&mut self) {
        let mut completed = Vec::new();
        let mut actions = Vec::new();
        for (idx, rx) in self.scene_dialog_receivers.iter().enumerate() {
            if let Ok(action) = rx.try_recv() {
                actions.push(action);
                completed.push(idx);
            }
        }
        for &idx in completed.iter().rev() {
            self.scene_dialog_receivers.swap_remove(idx);
        }
        for action in actions {
            match action {
                SceneDialogAction::SaveTo(path) => {
                    self.active_scene_path = path.to_string_lossy().to_string();
                    self.pending_save_path = Some(path);
                    self.should_save_scene = true;
                }
                SceneDialogAction::LoadFrom(path) => {
                    self.active_scene_path = path.to_string_lossy().to_string();
                    self.pending_load_path = Some(path);
                    self.should_load_scene = true;
                    self.is_loading_assets = true;
                }
            }
        }
    }

    /// Returns the authoritative docking hierarchy and floating windows owned by [`Self::workbench`].
    #[inline]
    #[must_use]
    pub fn dock_state(&self) -> &irisui::dock::DockState<PanelId> {
        self.workbench.dock_state()
    }

    /// Returns a mutable reference to the authoritative docking hierarchy owned by [`Self::workbench`].
    #[inline]
    pub fn dock_state_mut(&mut self) -> &mut irisui::dock::DockState<PanelId> {
        self.workbench.dock_state_mut()
    }

    /// Called once per frame before rendering to snapshot the global log buffer.
    pub fn sync_console(&mut self) {
        if !self.workbench.is_panel_open(&PanelId::Console) {
            return;
        }

        let current_total = ae_editor::editor_logger::LOGGER
            .log_count
            .load(std::sync::atomic::Ordering::Relaxed);

        if current_total != self.console_last_count {
            if let Ok(lock) = ae_editor::editor_logger::LOGGER.logs.try_lock() {
                self.console_entries = lock
                    .iter()
                    .map(|e| ConsoleEntry {
                        level: e.level,
                        target: e.target.clone(),
                        msg: e.msg.clone(),
                        timestamp: e.timestamp.clone(),
                    })
                    .collect();
                self.console_last_count = current_total;
            }
        } else if current_total == 0 && !self.console_entries.is_empty() {
            self.console_entries.clear();
            self.console_last_count = 0;
        }
    }

    /// Returns whether the UI currently captures the pointer or if the pointer is positioned over
    /// any active workbench UI element (menubar, statusbar, panels, floating windows, popups).
    ///
    /// Delegates directly to [`irisui::workbench::Workbench::wants_pointer_input`].
    #[must_use]
    #[inline]
    pub fn wants_pointer_input(&self) -> bool {
        self.workbench.wants_pointer_self()
    }

    /// Evaluates whether the specified screen-space coordinate lands over active workbench UI elements
    /// rather than the interactive 3D viewport canvas.
    ///
    /// Delegates directly to [`irisui::workbench::Workbench::wants_pointer_at_self`].
    #[must_use]
    #[inline]
    pub fn wants_pointer_at(&self, pos: [f32; 2]) -> bool {
        self.workbench.wants_pointer_at_self(pos)
    }

    /// Routes a UI event via `Workbench::handle_event_with_surface` and applies host state flags.
    pub fn handle_ui_event(&mut self, window: &Window, ui_event: &irisui::core::UiEvent) -> bool {
        let mut window_ref = window;
        let win_size = window.inner_size();
        let outcome = self.workbench.handle_event_with_surface(
            &mut self.surface,
            &mut window_ref,
            (win_size.width, win_size.height),
            ui_event,
        );
        if !outcome.actions.is_empty() {
            self.pending_actions.extend(outcome.actions);
        }
        if outcome.close_requested {
            self.should_exit = true;
        }
        if let Some(zoom) = outcome.zoom_changed {
            self.ui_zoom_factor = zoom;
        }
        outcome.consumed
    }

    /// Synchronizes logical workbench dimensions from physical window size.
    #[inline]
    pub fn on_window_resized(&mut self, physical_width: u32, physical_height: u32) {
        self.workbench
            .sync_physical_window_size(physical_width, physical_height);
    }

    /// Returns `true` when an external OS file drag is hovering over the window.
    #[inline]
    #[must_use]
    pub const fn is_file_drop_hovered(&self) -> bool {
        self.workbench.is_file_drop_hovered()
    }

    /// Updates cached frame performance and rendering telemetry snapshot.
    #[inline]
    pub fn update_telemetry(&mut self, snapshot: EditorTelemetrySnapshot) {
        self.telemetry = snapshot;
    }

    /// Returns `true` once physical GPU adapter metadata is cached.
    #[inline]
    #[must_use]
    pub fn has_gpu_adapter_info(&self) -> bool {
        !self.gpu_adapter_name.is_empty()
    }

    /// Caches physical GPU adapter name and rendering backend identifier.
    pub fn set_gpu_adapter_info(&mut self, adapter: impl Into<String>, backend: impl Into<String>) {
        self.gpu_adapter_name = adapter.into();
        self.gpu_backend = backend.into();
    }

    /// Returns mutable access to Export Project modal dialog state for progress reporting.
    #[inline]
    pub fn export_modal_mut(&mut self) -> &mut crate::ui::types::ExportDialogState {
        &mut self.iris_overlay.export_modal
    }

    /// Returns read-only access to the retained UI tree for external inspection tooling.
    #[inline]
    #[must_use]
    pub const fn ui_tree(&self) -> &irisui::core::UiTree {
        self.workbench.tree()
    }

    /// Returns logical workbench dimensions `[width, height]`, if synchronized.
    #[inline]
    #[must_use]
    pub fn logical_screen_size(&self) -> Option<[f32; 2]> {
        self.workbench.screen_size().map(|s| [s.width, s.height])
    }

    /// Marks the entire UI tree dirty for redraw.
    #[inline]
    pub fn invalidate_all(&mut self) {
        self.workbench.notifier.tag_all();
    }

    /// Updates entity selection and tags Inspector and Hierarchy panels dirty.
    pub fn select_entity_and_refresh(&mut self, entity: Option<hecs::Entity>) {
        self.selected_entity = entity;
        self.workbench.notifier.tag_redraw(PanelId::Inspector);
        self.workbench.notifier.tag_redraw(PanelId::Hierarchy);
    }
}

/// Returns whether a status message has expired based on the standard 2.5s duration.
pub fn is_status_message_expired(timestamp: std::time::Instant) -> bool {
    const STATUS_MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2500);
    timestamp.elapsed() >= STATUS_MESSAGE_TIMEOUT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_message_expiration_timing() {
        let now = std::time::Instant::now();
        assert!(!is_status_message_expired(now));

        let expired = now - std::time::Duration::from_millis(2600);
        assert!(is_status_message_expired(expired));
    }
}