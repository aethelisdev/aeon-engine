// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor UI Consumer Facade (`EngineUi`)
//!
//! Thin engine-facing UI facade wrapping [`irisui::workbench::WorkbenchHost`] and
//! owning only UI presentation/panel state (asset browser, UI designer canvas, export modal,
//! and cached telemetry readouts).

use crate::ui::iris_bridge::VIEWPORT_TEXTURE_ID;
use crate::ui::iris_bridge::assets::AssetsPanelState;
use crate::ui::iris_bridge::console::ConsolePanelState;
use crate::ui::iris_bridge::editor_dock_style_with_tokens;
use crate::ui::iris_bridge::menubar::VIEW_PANEL_IDS;
use crate::ui::iris_bridge::preferences::PreferencesDialogState;
use crate::ui::iris_bridge::theme::ThemePreset;
use crate::ui::panel_layout::PanelId;
use crate::ui::panel_registry::{EditorPanelApp, build_editor_workbench_host};
use crate::ui::types::{
    EditorFrameData, EditorTelemetrySnapshot, EditorUiRenderParams, EngineUiAction,
    ExportDialogState,
};
use irisui::prelude::{Color, Rect};
use irisui::workbench::{ProgressModalDescriptor, WorkbenchHost, WorkbenchHostFrameParams};
use winit::window::Window;

/// Engine-facing editor UI coordinator wrapping [`WorkbenchHost<EditorPanelApp>`].
pub struct EngineUi {
    pub(crate) telemetry: EditorTelemetrySnapshot,
    pub(crate) gpu_adapter_name: String,
    pub(crate) gpu_backend: String,
    /// Measured engine frames per second displayed in stats and status readouts.
    pub fps: f32,
    /// Last resolved logical 3D viewport rectangle returned to the renderer.
    pub last_viewport_rect: Rect,
    /// Active UI scale factor.
    pub ui_zoom_factor: f32,
    /// Persistent Content / Asset Browser state.
    pub asset_browser: ae_assets::AssetBrowserState,
    /// Persistent 2D Visual UI Designer canvas state.
    pub ui_designer_state: ae_uidesign::UiDesignerState,
    pub(crate) export_modal: ExportDialogState,
    pub(crate) host: WorkbenchHost<EditorPanelApp>,
}

impl EngineUi {
    /// Initializes the editor UI subsystem backed by Iris UI's retained [`WorkbenchHost`].
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        _window: &Window,
    ) -> Self {
        Self {
            host: build_editor_workbench_host(device, output_color_format),
            telemetry: EditorTelemetrySnapshot::default(),
            gpu_adapter_name: String::new(),
            gpu_backend: String::new(),
            fps: 60.0,
            last_viewport_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            ui_zoom_factor: 1.0,
            asset_browser: ae_assets::AssetBrowserState::new(),
            ui_designer_state: ae_uidesign::UiDesignerState::default(),
            export_modal: ExportDialogState::default(),
        }
    }

    /// Executes one full retained UI update and WGPU render pass, returning the resolved 3D viewport rect.
    pub fn render(&mut self, params: EditorUiRenderParams<'_>) -> ae_camera::ViewportRect {
        self.fps = params.fps;
        self.host
            .set_loading_progress(params.editor_state.is_loading_assets, || {
                ProgressModalDescriptor::new("Loading Assets")
                    .with_icon("🚀")
                    .with_progress(0.65)
                    .with_cancellable(false)
                    .with_size(360.0, 140.0)
                    .with_status_message("Processing geometry, materials & textures")
            });
        if ae_assets::scanner::rescan_assets_if_needed(
            &mut self.asset_browser,
            params.models,
            params.textures,
            params.shaders,
        ) {
            self.host.tag_redraw(PanelId::Assets);
        }
        if self.host.is_panel_open(&PanelId::Console)
            && self
                .host
                .panel_state_mut::<ConsolePanelState>(&PanelId::Console)
                .is_some_and(ConsolePanelState::sync_from_logger)
        {
            self.host.tag_redraw(PanelId::Console);
        }
        self.host
            .registry
            .panel_state_mut::<AssetsPanelState>(&PanelId::Assets)
            .into_iter()
            .for_each(|assets| {
                assets.ensure_thumbnails(
                    &mut self.host.surface,
                    params.queue,
                    &self.asset_browser.cached_items,
                );
            });
        self.host.set_external_texture(
            params.device,
            VIEWPORT_TEXTURE_ID,
            params.viewport_texture_view,
        );
        self.render_host_pass(self.host.tab_visibility_mask(&VIEW_PANEL_IDS), params)
    }

    fn render_host_pass(
        &mut self,
        view_panel_mask: u64,
        mut params: EditorUiRenderParams<'_>,
    ) -> ae_camera::ViewportRect {
        drop(
            self.host.render_frame_with(
                WorkbenchHostFrameParams {
                    device: params.device,
                    queue: params.queue,
                    encoder: params.encoder,
                    target_view: params.window_surface_view,
                    window: &mut params.window,
                    physical_size: params.surface_size,
                    selection_id: params
                        .editor_state
                        .primary_selected()
                        .map(|e| e.to_bits().get()),
                    domain_revision: params.editor_state.history_revision,
                    drag_active: self.asset_browser.drag_payload.is_some(),
                    surface_revision: u64::from(params.viewport_texture_view.is_some()),
                },
                |pass| {
                    pass.execute_with(|viewport_rect| EditorFrameData {
                        is_editing: *params.mode == ae_core::modules::EngineMode::Edit,
                        is_2d_mode: params.is_2d_mode,
                        view_panel_mask,
                        can_undo: !params.undo_stack.is_empty(),
                        can_redo: !params.redo_stack.is_empty(),
                        has_viewport_texture: params.viewport_texture_view.is_some(),
                        viewport_rect,
                        camera: params.camera,
                        world: params.world,
                        editor_state: params.editor_state,
                        graphics_settings: params.graphics_settings,
                        snapping_settings: params.snapping,
                        editor_config: &params.editor_state.config,
                        enable_live_updates: params.editor_state.enable_live_editor_updates,
                        enabled_modules: params.enabled_modules,
                        telemetry: &self.telemetry,
                        gpu_adapter_name: &self.gpu_adapter_name,
                        gpu_backend: &self.gpu_backend,
                        asset_browser: &mut self.asset_browser,
                        ui_designer_state: &mut self.ui_designer_state,
                        export_modal: &mut self.export_modal,
                        textures: params.textures,
                        models: params.models,
                    })
                },
            ),
        );
        self.host
            .dispatch_actions(|_, action| params.ui_actions.push(action));
        self.last_viewport_rect = self.host.last_viewport_rect();
        ae_camera::ViewportRect {
            min_x: self.last_viewport_rect.x,
            min_y: self.last_viewport_rect.y,
            max_x: self.last_viewport_rect.right(),
            max_y: self.last_viewport_rect.bottom(),
        }
    }

    /// Routes an input event through [`WorkbenchHost::handle_event`] and applies host outcomes.
    pub fn handle_ui_event(&mut self, window: &Window, ui_event: &irisui::core::UiEvent) -> bool {
        Self::apply_host_event(
            self.host.handle_event(
                &mut { window },
                (window.inner_size().width, window.inner_size().height),
                ui_event,
            ),
            self,
        )
    }

    fn apply_host_event(
        result: irisui::workbench::WorkbenchHostEventResult,
        ui: &mut Self,
    ) -> bool {
        if result.close_requested {
            ui.host.emit_action(EngineUiAction::Exit);
        }
        result
            .zoom_changed
            .into_iter()
            .for_each(|zoom| ui.ui_zoom_factor = zoom);
        result.consumed
    }

    /// Renders the retained UI tree into an offscreen target texture view (e.g. MCP screenshots).
    pub fn render_offscreen_snapshot(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
    ) {
        self.host.render_offscreen_snapshot(
            device,
            queue,
            encoder,
            target_view,
            physical_screen_size,
            zoom_factor,
        );
    }

    /// Applies `preset` to the workbench theme, dock chrome style, and Preferences panel state.
    pub fn apply_theme_preset(&mut self, preset: ThemePreset) {
        self.host
            .panel_state_mut::<PreferencesDialogState>(&PanelId::Preferences)
            .into_iter()
            .for_each(|prefs| prefs.theme_preset = preset);
        self.host.set_theme_and_dock_style(
            preset.tokens().to_iris_theme(),
            editor_dock_style_with_tokens(&preset.tokens()),
        );
    }

    /// Displays a temporary colored status bar notification for 2.5 seconds.
    pub fn set_status_message(&mut self, text: impl Into<String>, color: Color) {
        self.host.set_status_text(text, color);
    }

    /// Active UI scale factor managed by [`WorkbenchHost`].
    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        self.host.scale_factor()
    }

    /// Steps the active UI scale preset up or down and returns the updated factor.
    pub fn step_ui_scale(&mut self, increase: bool) -> f32 {
        self.ui_zoom_factor = self.host.step_zoom(increase);
        self.ui_zoom_factor
    }

    /// Resets the UI scale factor back to `1.0`.
    pub fn reset_ui_scale(&mut self) -> f32 {
        self.ui_zoom_factor = self.host.reset_zoom();
        self.ui_zoom_factor
    }

    /// Returns `true` when any modal or focused text field captures keyboard input.
    #[must_use]
    pub fn wants_keyboard_input(&self) -> bool {
        self.host.wants_keyboard_input()
    }

    /// Returns `true` when pointer events are captured by UI panels or chrome.
    #[must_use]
    pub fn wants_pointer_input(&self) -> bool {
        self.host.wants_pointer_input()
    }

    /// Returns `true` when `pos` lands over an interactive UI surface rather than the 3D viewport.
    #[must_use]
    pub fn wants_pointer_at(&self, pos: [f32; 2]) -> bool {
        self.host.wants_pointer_at(pos)
    }

    /// Synchronizes logical UI bounds when the host OS window resizes.
    pub fn on_window_resized(&mut self, physical_width: u32, physical_height: u32) {
        self.host.resize(physical_width, physical_height);
    }

    /// Returns `true` while an external OS file drag is hovering over the window.
    #[must_use]
    pub const fn is_file_drop_hovered(&self) -> bool {
        self.host.is_file_drop_hovered()
    }

    /// Updates cached CPU/GPU frame telemetry snapshot.
    pub fn update_telemetry(&mut self, snapshot: EditorTelemetrySnapshot) {
        self.telemetry = snapshot;
    }

    /// Returns `true` once physical GPU adapter metadata has been cached.
    #[must_use]
    pub fn has_gpu_adapter_info(&self) -> bool {
        !self.gpu_adapter_name.is_empty()
    }

    /// Caches physical GPU adapter name and rendering backend identifier.
    pub fn set_gpu_adapter_info(&mut self, adapter: impl Into<String>, backend: impl Into<String>) {
        self.gpu_adapter_name = adapter.into();
        self.gpu_backend = backend.into();
    }

    /// Returns mutable access to Export Project modal dialog state.
    pub fn export_modal_mut(&mut self) -> &mut ExportDialogState {
        &mut self.export_modal
    }

    /// Dismisses any open Quick Asset Preview modal inside Assets panel state.
    pub fn clear_asset_preview_modal(&mut self) {
        self.host
            .panel_state_mut::<AssetsPanelState>(&PanelId::Assets)
            .into_iter()
            .for_each(|assets| assets.preview_modal = None);
    }

    /// Marks the full retained UI tree dirty for redraw on the next frame.
    pub fn invalidate_all(&mut self) {
        self.host.invalidate_all();
    }

    /// Tags selection-sensitive panels (`Inspector` and `Hierarchy`) dirty for redraw.
    pub fn refresh_selection_panels(&mut self) {
        self.host.tag_redraw(PanelId::Inspector);
        self.host.tag_redraw(PanelId::Hierarchy);
    }

    /// Activates or brings `panel` into focus within the workbench dock tree.
    pub fn open_or_focus_panel(&mut self, panel: PanelId) {
        self.host.open_or_focus_panel(panel);
    }

    /// Toggles `panel` visibility in the workbench dock tree.
    pub fn toggle_panel(&mut self, panel: PanelId) {
        self.host.toggle_panel(panel);
    }

    /// Resets the workbench dock tree to the default editor layout.
    pub fn reset_dock_layout(&mut self) {
        self.host
            .reset_dock_state(crate::ui::panel_layout::create_default_dock_state());
    }

    /// Marks `panel` dirty for redraw on the next UI pass.
    pub fn tag_panel_redraw(&mut self, panel: PanelId) {
        self.host.tag_redraw(panel);
    }

    /// Clears transient asset/designer/modal interaction states before entering Play mode.
    pub fn prepare_for_play_mode(&mut self) {
        self.open_or_focus_panel(PanelId::Viewport);
        self.asset_browser.selected_asset = None;
        self.clear_asset_preview_modal();
        self.host.close_modal();
        self.asset_browser.drag_payload = None;
        self.ui_designer_state.drag_state = None;
    }

    /// Clears cached Asset Browser items after file drops or filesystem mutations and redraws the panel.
    pub fn refresh_asset_browser(&mut self, clear_selected_matching: Option<&std::path::Path>) {
        if clear_selected_matching.is_some()
            && self.asset_browser.selected_asset.as_deref() == clear_selected_matching
        {
            self.asset_browser.selected_asset = None;
        }
        self.asset_browser.cached_items.clear();
        self.host.tag_redraw(PanelId::Assets);
    }

    /// Returns a concise read-only telemetry summary of the retained UI without exposing `UiTree`.
    #[must_use]
    pub fn ui_inspection_summary(&self) -> irisui::workbench::UiInspectionSummary {
        self.host.ui_inspection_summary()
    }

    /// Captures a filtered node & tag-bounds snapshot of the active UI for automation or diagnostics.
    #[must_use]
    pub fn inspect_ui_nodes(
        &self,
        tag_filter: Option<u64>,
        name_filter: Option<&str>,
        only_popups: bool,
    ) -> irisui::workbench::UiTreeDump {
        self.host
            .inspect_ui_nodes(tag_filter, name_filter, only_popups)
    }

    /// Resolves the bounding rectangle for a semantic widget `tag`, if laid out with non-zero size.
    #[must_use]
    pub fn resolve_tag_rect(&self, tag: u64) -> Option<irisui::core::Rect> {
        self.host.resolve_tag_rect(tag)
    }

    /// Resolves the non-zero bounding rectangle for the `index`-th widget matching `name` or text.
    #[must_use]
    pub fn resolve_widget_rect(&self, name: &str, index: usize) -> Option<irisui::core::Rect> {
        self.host.resolve_widget_rect(name, index)
    }

    /// Returns resolved logical viewport dimensions `[width, height]`, if initialized.
    #[must_use]
    pub fn logical_screen_size(&self) -> Option<[f32; 2]> {
        self.host.logical_screen_size()
    }

    /// Returns an immutable reference to the authoritative dock state.
    #[must_use]
    pub fn dock_state(&self) -> &irisui::dock::DockState<PanelId> {
        self.host.dock_state()
    }
}