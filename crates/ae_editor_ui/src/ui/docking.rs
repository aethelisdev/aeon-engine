// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Tree Docking Coordinator (`egui_dock` — Multi-zone split and tab dock layout)
/// Implements `egui_dock::TabViewer` for all Aeon Engine panels and the central 3D Viewport.
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::types::ConsoleEntry;
use crate::ui::{EngineUi, EngineUiAction};
use egui::{Color32, CornerRadius, Rect, Stroke};

/// Tab viewer context struct binding engine runtime state to `egui_dock`.
pub struct EditorTabViewer<'a> {
    pub world: &'a hecs::World,
    pub hierarchy_cache: &'a mut crate::ui::panels::hierarchy::HierarchyCache,
    pub hierarchy_search_query: &'a mut String,
    pub selected_entity: &'a mut Option<hecs::Entity>,
    pub last_selected_entity: &'a mut Option<hecs::Entity>,
    pub inspector_euler: &'a mut [f32; 3],
    pub inspector_color_hex: &'a mut String,
    pub saved_swatches: &'a mut Vec<[f32; 4]>,
    pub is_editing: bool,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    pub camera: &'a ae_renderer::camera::Camera,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub console_entries: &'a [ConsoleEntry],
    pub wireframe_enabled: &'a mut bool,
    pub grid_enabled: &'a mut bool,
    pub fps: f32,
    pub profiler_ecs_ms: f32,
    pub profiler_render_ms: f32,
    pub profiler_present_ms: f32,
    pub profiler_ui_ms: f32,
    pub profiler_frame_ms: f32,
    pub memory_models_mb: f32,
    pub memory_textures_mb: f32,
    pub viewport_texture_id: Option<egui::TextureId>,
    pub viewport_rect_out: &'a std::cell::Cell<Rect>,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub gizmo_mode: &'a mut ae_editor::gizmo::GizmoMode,
    pub gizmo_space: &'a mut ae_editor::gizmo::GizmoSpace,
}

impl<'a> egui_dock::TabViewer for EditorTabViewer<'a> {
    type Tab = PanelId;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(format!("tab_{:?}", tab))
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{} {}", tab.icon(), tab.title()).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelId::Viewport => {
                let rect = ui.available_rect_before_wrap();
                if let Some(texture_id) = self.viewport_texture_id {
                    ui.image(egui::load::SizedTexture {
                        id: texture_id,
                        size: rect.size(),
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Rendering viewport...");
                    });
                }
                self.viewport_rect_out.set(rect);

                // Viewport Toolbar & HUD Overlays
                let is_render_active = self
                    .enabled_modules
                    .contains(&ae_core::modules::EngineModule::Render);
                if self.is_editing && is_render_active {
                    crate::ui::viewport_hud::draw_viewport_toolbar(
                        ui.ctx(),
                        rect,
                        self.wireframe_enabled,
                        self.gizmo_mode,
                        self.gizmo_space,
                        self.camera,
                        self.ui_actions,
                    );
                    crate::ui::viewport_hud::draw_camera_hud(ui.ctx(), rect, self.camera);
                    crate::ui::viewport_hud::draw_scene_navigation_gizmo(
                        ui.ctx(),
                        rect,
                        self.camera,
                        self.ui_actions,
                    );
                    crate::ui::viewport_hud::draw_billboard_icons(
                        ui.ctx(),
                        rect,
                        self.world,
                        self.camera,
                        *self.selected_entity,
                        self.ui_actions,
                    );
                }
            }
            PanelId::Hierarchy => {
                EngineUi::draw_hierarchy_content(
                    ui,
                    self.world,
                    self.hierarchy_cache,
                    self.hierarchy_search_query,
                    self.selected_entity,
                    self.is_editing,
                    self.ui_actions,
                );
            }
            PanelId::Stats => {
                EngineUi::draw_stats_content(
                    ui,
                    self.wireframe_enabled,
                    self.grid_enabled,
                    self.fps,
                    self.profiler_ecs_ms,
                    self.profiler_render_ms,
                    self.profiler_present_ms,
                    self.profiler_ui_ms,
                    self.profiler_frame_ms,
                    self.memory_models_mb,
                    self.memory_textures_mb,
                );
            }
            PanelId::Inspector => {
                EngineUi::draw_inspector_content(
                    ui,
                    self.world,
                    self.selected_entity,
                    self.last_selected_entity,
                    self.inspector_euler,
                    self.inspector_color_hex,
                    self.saved_swatches,
                    self.is_editing,
                    self.ui_actions,
                    self.editor_state,
                    self.camera,
                    self.models,
                    self.textures,
                );
            }
            PanelId::MaterialEditor => {
                EngineUi::draw_material_editor_content(
                    ui,
                    self.world,
                    *self.selected_entity,
                    self.textures,
                    self.models,
                    self.ui_actions,
                );
            }
            PanelId::Assets => {
                EngineUi::draw_assets_content(ui, self.models, self.textures, self.ui_actions);
            }
            PanelId::Console => {
                EngineUi::draw_console_content(ui, self.console_entries, self.ui_actions);
            }
            PanelId::AnimationTimeline => {
                EngineUi::draw_timeline_content(
                    ui,
                    self.world,
                    *self.selected_entity,
                    self.ui_actions,
                );
            }
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        *tab != PanelId::Viewport
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        // Viewport clears its own RTT background
        *tab != PanelId::Viewport
    }
}

/// Configures the visual style of the `egui_dock` tree to match the Aeon Dark Cyan theme.
pub fn create_aeon_dock_style(ctx: &egui::Context) -> egui_dock::Style {
    let mut style = egui_dock::Style::from_egui(ctx.global_style().as_ref());

    // 1. Tab Bar & Main Surface
    style.tab_bar.bg_fill = Color32::from_rgb(15, 15, 20);
    style.tab_bar.height = 24.0;
    style.main_surface_border_stroke = Stroke::new(1.0, Color32::from_rgb(45, 48, 60));

    // 2. Resizable Split Dividers
    style.separator.color_idle = Color32::from_rgb(45, 48, 60);
    style.separator.color_hovered = Color32::from_rgb(0, 229, 255);
    style.separator.color_dragged = Color32::from_rgb(0, 229, 255);
    style.separator.width = 2.0;
    style.separator.extra = 4.0;

    // 3. Tab Styling & Aeon Cyan Line Highlight
    style.tab.tab_body.bg_fill = Color32::from_rgb(20, 20, 25);
    style.tab.tab_body.corner_radius = CornerRadius {
        nw: 4,
        ne: 4,
        sw: 0,
        se: 0,
    };
    style.tab.active.text_color = Color32::from_rgb(0, 229, 255);
    style.tab.active.bg_fill = Color32::from_rgb(20, 20, 25);
    style.tab.focused.text_color = Color32::from_rgb(0, 229, 255);
    style.tab.focused.bg_fill = Color32::from_rgb(20, 20, 25);
    style.tab.inactive.text_color = Color32::from_rgb(160, 160, 175);
    style.tab.inactive.bg_fill = Color32::from_rgb(18, 19, 24);
    style.tab.hovered.text_color = Color32::WHITE;
    style.tab.hovered.bg_fill = Color32::from_rgb(28, 30, 38);

    style
}

impl EngineUi {
    /// Renders the complete tree docking system using `egui_dock`.
    pub(super) fn draw_docking_system(
        ui: &mut egui::Ui,
        layout_state: &mut PanelLayoutState,
        tab_viewer: &mut EditorTabViewer<'_>,
    ) {
        let style = create_aeon_dock_style(ui.ctx());
        egui_dock::DockArea::new(&mut layout_state.dock_state)
            .style(style)
            .show_inside(ui, tab_viewer);
    }
}