// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Tree Docking Coordinator (`iris-dock` — Multi-zone split and tab dock layout).
//!
//! Implements `irisui::dock::TabViewer` for all Aeon Engine panels and the central 3D Viewport.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::{EngineUi, EngineUiAction};
use egui::{Color32, Pos2, Rect};

/// Tab viewer context struct binding engine runtime state to `iris-dock`.
pub struct EditorTabViewer<'a> {
    pub world: &'a hecs::World,
    pub is_editing: bool,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub camera: &'a ae_renderer::camera::Camera,
    pub asset_browser: &'a mut crate::ui::panels::assets::AssetBrowserState,
    pub viewport_texture_id: Option<egui::TextureId>,

    pub viewport_rect_out: &'a std::cell::Cell<Rect>,
    pub stats_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub hierarchy_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub inspector_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub material_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub console_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub assets_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub timeline_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub ui_designer_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
}

impl<'a> EditorTabViewer<'a> {
    /// Renders panel content into the allocated rectangle or registers its bounding box for Iris UI SDF overlay rendering.
    pub fn render_content(&mut self, ui: &mut egui::Ui, panel: PanelId, content_rect: Rect) {
        match panel {
            PanelId::Viewport => {
                let rect = content_rect;
                if let Some(texture_id) = self.viewport_texture_id {
                    ui.painter().image(
                        texture_id,
                        rect,
                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Rendering viewport...",
                        egui::FontId::proportional(14.0),
                        Color32::GRAY,
                    );
                }
                self.viewport_rect_out.set(rect);

                // Viewport Toolbar & HUD Overlays are now 100% rendered via Iris UI GPU SDF pipeline in IrisEditorOverlay
                let is_render_active = self
                    .enabled_modules
                    .contains(&ae_core::modules::EngineModule::Render);
                if self.is_editing && is_render_active {
                    // Content Browser Drag-and-Drop Viewport Raycast Spawning
                    if self.asset_browser.drag_payload.is_some()
                        && let Some(mouse_pos) = ui.ctx().pointer_latest_pos()
                        && rect.contains(mouse_pos)
                        && let Some(hit_pos) =
                            crate::ui::panels::assets::drag_drop::compute_ground_intersection(
                                mouse_pos,
                                rect,
                                self.camera,
                            )
                    {
                        crate::ui::panels::assets::drag_drop::draw_viewport_drop_indicator(
                            ui.ctx(),
                            rect,
                            hit_pos,
                            self.camera,
                        );

                        if ui.input(|i| i.pointer.any_released()) {
                            crate::ui::panels::assets::drag_drop::handle_viewport_drop(
                                self.asset_browser,
                                hit_pos,
                                self.ui_actions,
                            );
                        }
                    }
                } else if !self.is_editing && is_render_active {
                    // Play Mode HUD is rendered via Iris UI GPU SDF pipeline in IrisEditorOverlay

                    // Resolve & render backend-agnostic in-game UI draw commands
                    let mouse_pos = ui
                        .input(|i| i.pointer.hover_pos())
                        .map(|p| [p.x - rect.left(), p.y - rect.top()]);
                    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());

                    let commands = ae_core::ui::UiLayoutResolver::resolve_draw_commands(
                        self.world,
                        rect.width(),
                        rect.height(),
                        mouse_pos,
                        mouse_clicked,
                    );
                    let painter = ui.painter();
                    crate::ui::viewport_hud::render_ui_draw_commands(painter, rect, &commands);
                }
            }
            PanelId::Hierarchy => {
                self.hierarchy_rect_out.set(Some(content_rect));
            }
            PanelId::Stats => {
                self.stats_rect_out.set(Some(content_rect));
            }
            PanelId::Inspector => {
                self.inspector_rect_out.set(Some(content_rect));
            }
            PanelId::MaterialEditor => {
                self.material_rect_out.set(Some(content_rect));
            }
            PanelId::Assets => {
                self.assets_rect_out.set(Some(content_rect));
            }
            PanelId::Console => {
                self.console_rect_out.set(Some(content_rect));
            }
            PanelId::AnimationTimeline => {
                self.timeline_rect_out.set(Some(content_rect));
            }
            PanelId::UiDesigner => {
                self.ui_designer_rect_out.set(Some(content_rect));
            }
        }
    }
}

impl<'a> irisui::dock::TabViewer<PanelId> for EditorTabViewer<'a> {
    fn title(&self, tab: &PanelId) -> String {
        format!("{} {}", tab.icon(), tab.title())
    }

    fn closeable(&self, tab: &PanelId) -> bool {
        *tab != PanelId::Viewport
    }

    fn on_close(&mut self, tab: &mut PanelId) -> bool {
        *tab != PanelId::Viewport
    }

    fn clear_background(&self, tab: &PanelId) -> bool {
        // Viewport clears its own RTT background
        *tab != PanelId::Viewport
    }
}

impl EngineUi {
    /// Renders the complete tree docking system using native `iris-dock`.
    pub(super) fn draw_docking_system(
        ui: &mut egui::Ui,
        layout_state: &mut PanelLayoutState,
        tab_viewer: &mut EditorTabViewer<'_>,
    ) {
        crate::ui::docking_render::render_iris_dock(ui, layout_state, tab_viewer);
    }
}