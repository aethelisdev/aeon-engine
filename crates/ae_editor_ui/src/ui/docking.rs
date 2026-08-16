// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::panel_layout::{PanelId, PanelLayoutState, PanelZone, TabDragState};
use crate::ui::types::ConsoleEntry;
use crate::ui::{EngineUi, EngineUiAction, tab_bar};
use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};

impl EngineUi {
    /// Polymorphic renderer for any docking panel content based on its `PanelId`.
    /// Enables any panel to be rendered in the Left, Right, or Bottom docking zones interchangeably.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_docked_panel_content(
        panel_id: PanelId,
        ui: &mut egui::Ui,
        world: &hecs::World,
        hierarchy_cache: &mut crate::ui::hierarchy::HierarchyCache,
        hierarchy_search_query: &mut String,
        selected_entity: &mut Option<hecs::Entity>,
        last_selected_entity: &mut Option<hecs::Entity>,
        inspector_euler: &mut [f32; 3],
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        console_entries: &[ConsoleEntry],
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
        layout_state: &mut PanelLayoutState,
    ) {
        match panel_id {
            PanelId::Hierarchy => {
                Self::draw_hierarchy_content(
                    ui,
                    world,
                    hierarchy_cache,
                    hierarchy_search_query,
                    selected_entity,
                    is_editing,
                    ui_actions,
                );
            }
            PanelId::Stats => {
                Self::draw_stats_content(
                    ui,
                    wireframe_enabled,
                    grid_enabled,
                    fps,
                    profiler_ecs_ms,
                    profiler_render_ms,
                    profiler_present_ms,
                    profiler_ui_ms,
                    profiler_frame_ms,
                    memory_models_mb,
                    memory_textures_mb,
                );
            }
            PanelId::Inspector => {
                Self::draw_inspector_content(
                    ui,
                    world,
                    selected_entity,
                    last_selected_entity,
                    inspector_euler,
                    inspector_color_hex,
                    saved_swatches,
                    is_editing,
                    ui_actions,
                    editor_state,
                    camera,
                    models,
                    layout_state,
                );
            }
            PanelId::MaterialEditor => {
                Self::draw_material_editor_content(
                    ui,
                    world,
                    *selected_entity,
                    textures,
                    models,
                    ui_actions,
                );
            }
            PanelId::Assets => {
                Self::draw_assets_content(ui, models, textures, ui_actions);
            }
            PanelId::Console => {
                Self::draw_console_content(ui, console_entries, ui_actions);
            }
            PanelId::AnimationTimeline => {
                Self::draw_timeline_content(ui, world, *selected_entity, ui_actions);
            }
        }
    }

    /// Renders the left side docking panel container and active tab content.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_left_dock(
        ui: &mut egui::Ui,
        layout_state: &mut PanelLayoutState,
        tab_drag_state: &mut Option<TabDragState>,
        world: &hecs::World,
        hierarchy_cache: &mut crate::ui::hierarchy::HierarchyCache,
        hierarchy_search_query: &mut String,
        selected_entity: &mut Option<hecs::Entity>,
        last_selected_entity: &mut Option<hecs::Entity>,
        inspector_euler: &mut [f32; 3],
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        console_entries: &[ConsoleEntry],
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        smoothed_fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
        ui_rects_collector: &std::cell::RefCell<Vec<egui::Rect>>,
    ) {
        if !layout_state.show_left_panel || layout_state.left_tabs.is_empty() {
            return;
        }

        let left_resp = egui::Panel::left("left_docked_panel")
            .default_size(260.0)
            .min_size(150.0)
            .max_size(600.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(20, 20, 25))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60))),
            )
            .show(ui, |ui| {
                // 1. Draggable Tab Bar + Close Button
                ui.horizontal(|ui| {
                    tab_bar::draw_draggable_tab_bar(
                        ui,
                        PanelZone::Left,
                        layout_state,
                        tab_drag_state,
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✖")
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(160)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false),
                            )
                            .on_hover_text("Close Left Panel")
                            .clicked()
                        {
                            layout_state.show_left_panel = false;
                        }
                    });
                });
                ui.add_space(4.0);

                // 2. Panel Content
                if let Some(active_panel) = layout_state.active_left_tab {
                    Self::draw_docked_panel_content(
                        active_panel,
                        ui,
                        world,
                        hierarchy_cache,
                        hierarchy_search_query,
                        selected_entity,
                        last_selected_entity,
                        inspector_euler,
                        inspector_color_hex,
                        saved_swatches,
                        is_editing,
                        ui_actions,
                        editor_state,
                        camera,
                        models,
                        textures,
                        console_entries,
                        wireframe_enabled,
                        grid_enabled,
                        smoothed_fps,
                        profiler_ecs_ms,
                        profiler_render_ms,
                        profiler_present_ms,
                        profiler_ui_ms,
                        profiler_frame_ms,
                        memory_models_mb,
                        memory_textures_mb,
                        layout_state,
                    );
                }
            });
        ui_rects_collector
            .borrow_mut()
            .push(left_resp.response.rect);
    }

    /// Renders the right side docking panel container and active tab content.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_right_dock(
        ui: &mut egui::Ui,
        layout_state: &mut PanelLayoutState,
        tab_drag_state: &mut Option<TabDragState>,
        world: &hecs::World,
        hierarchy_cache: &mut crate::ui::hierarchy::HierarchyCache,
        hierarchy_search_query: &mut String,
        selected_entity: &mut Option<hecs::Entity>,
        last_selected_entity: &mut Option<hecs::Entity>,
        inspector_euler: &mut [f32; 3],
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        console_entries: &[ConsoleEntry],
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        smoothed_fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
        ui_rects_collector: &std::cell::RefCell<Vec<egui::Rect>>,
    ) {
        if !layout_state.show_right_panel || layout_state.right_tabs.is_empty() {
            return;
        }

        let right_resp = egui::Panel::right("right_docked_panel")
            .default_size(320.0)
            .min_size(200.0)
            .max_size(700.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(20, 20, 25))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60))),
            )
            .show(ui, |ui| {
                // 1. Draggable Tab Bar + Close Button
                ui.horizontal(|ui| {
                    tab_bar::draw_draggable_tab_bar(
                        ui,
                        PanelZone::Right,
                        layout_state,
                        tab_drag_state,
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✖")
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(160)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false),
                            )
                            .on_hover_text("Close Right Panel")
                            .clicked()
                        {
                            layout_state.show_right_panel = false;
                        }
                    });
                });
                ui.add_space(4.0);

                // 2. Panel Content
                if let Some(active_panel) = layout_state.active_right_tab {
                    Self::draw_docked_panel_content(
                        active_panel,
                        ui,
                        world,
                        hierarchy_cache,
                        hierarchy_search_query,
                        selected_entity,
                        last_selected_entity,
                        inspector_euler,
                        inspector_color_hex,
                        saved_swatches,
                        is_editing,
                        ui_actions,
                        editor_state,
                        camera,
                        models,
                        textures,
                        console_entries,
                        wireframe_enabled,
                        grid_enabled,
                        smoothed_fps,
                        profiler_ecs_ms,
                        profiler_render_ms,
                        profiler_present_ms,
                        profiler_ui_ms,
                        profiler_frame_ms,
                        memory_models_mb,
                        memory_textures_mb,
                        layout_state,
                    );
                }
            });
        ui_rects_collector
            .borrow_mut()
            .push(right_resp.response.rect);
    }

    /// Renders the bottom side docking panel container within the viewport work area.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_bottom_dock(
        ui: &mut egui::Ui,
        layout_state: &mut PanelLayoutState,
        tab_drag_state: &mut Option<TabDragState>,
        world: &hecs::World,
        hierarchy_cache: &mut crate::ui::hierarchy::HierarchyCache,
        hierarchy_search_query: &mut String,
        selected_entity: &mut Option<hecs::Entity>,
        last_selected_entity: &mut Option<hecs::Entity>,
        inspector_euler: &mut [f32; 3],
        inspector_color_hex: &mut String,
        saved_swatches: &mut Vec<[f32; 4]>,
        is_editing: bool,
        ui_actions: &mut Vec<EngineUiAction>,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        console_entries: &[ConsoleEntry],
        wireframe_enabled: &mut bool,
        grid_enabled: &mut bool,
        smoothed_fps: f32,
        profiler_ecs_ms: f32,
        profiler_render_ms: f32,
        profiler_present_ms: f32,
        profiler_ui_ms: f32,
        profiler_frame_ms: f32,
        memory_models_mb: f32,
        memory_textures_mb: f32,
        ui_rects_collector: &std::cell::RefCell<Vec<egui::Rect>>,
    ) {
        if !layout_state.show_bottom_panel || layout_state.bottom_tabs.is_empty() {
            return;
        }

        let bottom_resp = egui::Panel::bottom("bottom_docked_panel")
            .default_size(240.0)
            .min_size(100.0)
            .max_size(500.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(20, 20, 25))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 48, 60))),
            )
            .show(ui, |ui| {
                // 1. Draggable Tab Bar + Close Button
                ui.horizontal(|ui| {
                    tab_bar::draw_draggable_tab_bar(
                        ui,
                        PanelZone::Bottom,
                        layout_state,
                        tab_drag_state,
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✖")
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(160)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false),
                            )
                            .on_hover_text("Close Bottom Panel")
                            .clicked()
                        {
                            layout_state.show_bottom_panel = false;
                        }
                    });
                });
                ui.add_space(4.0);

                // 2. Panel Content
                if let Some(active_panel) = layout_state.active_bottom_tab {
                    Self::draw_docked_panel_content(
                        active_panel,
                        ui,
                        world,
                        hierarchy_cache,
                        hierarchy_search_query,
                        selected_entity,
                        last_selected_entity,
                        inspector_euler,
                        inspector_color_hex,
                        saved_swatches,
                        is_editing,
                        ui_actions,
                        editor_state,
                        camera,
                        models,
                        textures,
                        console_entries,
                        wireframe_enabled,
                        grid_enabled,
                        smoothed_fps,
                        profiler_ecs_ms,
                        profiler_render_ms,
                        profiler_present_ms,
                        profiler_ui_ms,
                        profiler_frame_ms,
                        memory_models_mb,
                        memory_textures_mb,
                        layout_state,
                    );
                }
            });
        ui_rects_collector
            .borrow_mut()
            .push(bottom_resp.response.rect);
    }

    /// Finalizes tab drag-and-drop actions on pointer release and renders floating detached badges.
    pub(super) fn finalize_tab_drag_interaction(
        layout_state: &mut PanelLayoutState,
        tab_drag_state: &mut Option<TabDragState>,
        ui: &mut egui::Ui,
    ) {
        let mouse_pos = ui.input(|i| i.pointer.hover_pos());

        // 1. If released, commit the tab move
        if ui.input(|i| i.pointer.any_released())
            && let Some(drag) = tab_drag_state.take()
            && let Some(target_zone) = drag.hovered_zone
        {
            layout_state.move_tab(drag.panel_id, target_zone, drag.hovered_index);
        }

        // 2. If detached and still dragging, render floating preview badge over the top tooltip layer
        if let (Some(drag), Some(pos)) = (tab_drag_state.as_ref(), mouse_pos)
            && drag.is_detached
        {
            let layer_id =
                egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tab_drag_badge"));
            let drag_painter = ui.ctx().layer_painter(layer_id);

            let preview_text = format!("{} {}", drag.panel_id.icon(), drag.panel_id.title());
            let badge_font = egui::TextStyle::Button.resolve(ui.style());
            let text_layout = drag_painter.layout_no_wrap(
                preview_text.clone(),
                badge_font.clone(),
                Color32::WHITE,
            );
            let badge_size = Vec2::new(text_layout.size().x + 22.0, 24.0);
            let badge_rect = Rect::from_min_size(Pos2::new(pos.x + 12.0, pos.y + 12.0), badge_size);

            // Semi-transparent glowing dark background with cyan border
            drag_painter.rect_filled(
                badge_rect,
                CornerRadius::same(6),
                Color32::from_rgba_premultiplied(20, 24, 35, 230),
            );
            drag_painter.rect_stroke(
                badge_rect,
                CornerRadius::same(6),
                Stroke::new(1.5, Color32::from_rgb(0, 220, 255)),
                egui::StrokeKind::Outside,
            );

            let badge_text_pos = Pos2::new(
                badge_rect.min.x + 11.0,
                badge_rect.min.y + (badge_size.y - text_layout.size().y) * 0.5,
            );
            drag_painter.text(
                badge_text_pos,
                egui::Align2::LEFT_TOP,
                preview_text,
                badge_font,
                Color32::from_rgb(255, 255, 255),
            );
        }
    }
}