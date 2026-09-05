// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Tree Docking Coordinator (`egui_dock` — Multi-zone split and tab dock layout)
/// Implements `egui_dock::TabViewer` for all Aeon Engine panels and the central 3D Viewport.
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::{EngineUi, EngineUiAction};
use egui::{Color32, CornerRadius, Rect, Stroke};

/// Tab viewer context struct binding engine runtime state to `egui_dock`.
pub struct EditorTabViewer<'a> {
    pub world: &'a hecs::World,
    pub selected_entity: &'a mut Option<hecs::Entity>,
    pub is_editing: bool,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
    pub camera: &'a ae_renderer::camera::Camera,
    pub asset_browser: &'a mut crate::ui::panels::assets::AssetBrowserState,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub viewport_texture_id: Option<egui::TextureId>,

    pub viewport_rect_out: &'a std::cell::Cell<Rect>,
    pub stats_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub hierarchy_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub inspector_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub console_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub assets_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub timeline_rect_out: &'a std::cell::Cell<Option<Rect>>,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub ui_designer_state: &'a mut crate::ui::panels::UiDesignerState,
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
                let rect = ui.available_rect_before_wrap();
                self.hierarchy_rect_out.set(Some(rect));
                ui.allocate_space(rect.size());
            }
            PanelId::Stats => {
                let rect = ui.available_rect_before_wrap();
                self.stats_rect_out.set(Some(rect));
                ui.allocate_space(rect.size());
            }
            PanelId::Inspector => {
                let rect = ui.available_rect_before_wrap();
                self.inspector_rect_out.set(Some(rect));
                ui.allocate_space(rect.size());
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
                let rect = ui.available_rect_before_wrap();
                self.assets_rect_out.set(Some(rect));
                ui.allocate_space(rect.size());
            }
            PanelId::Console => {
                let rect = ui.available_rect_before_wrap();
                self.console_rect_out.set(Some(rect));
                ui.allocate_space(rect.size());
            }
            PanelId::AnimationTimeline => {
                let rect = ui.available_rect_before_wrap();
                self.timeline_rect_out.set(Some(rect));
                ui.allocate_space(rect.size());
            }
            PanelId::UiDesigner => {
                crate::ui::panels::draw_ui_designer_panel(
                    ui,
                    &mut crate::ui::panels::UiDesignerContext {
                        world: self.world,
                        selected_entity: *self.selected_entity,
                        ui_actions: self.ui_actions,
                        state: self.ui_designer_state,
                    },
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