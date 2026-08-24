// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Interactive Quick Asset Inspector Modal Window.
//!
//! Provides a centralized multi-format asset viewer supporting 3D mesh orbit inspection,
//! RGBA channel texture analysis, WGSL shader syntax highlighting, and metadata breakdown.
//!

use super::types::{AssetBrowserState, AssetCategory, PreviewModalState};
use crate::ui::types::EngineUiAction;
use ae_renderer::asset::{AssetStorage, ShaderAsset};
use ae_renderer::render::{ModelAsset, TextureAsset};
use egui::{Color32, Context, CornerRadius, Pos2, RichText, Sense, Stroke, Ui, Vec2};

/// Draws the interactive quick asset inspector modal window if open.
pub fn draw_asset_preview_modal(
    ctx: &Context,
    state: &mut AssetBrowserState,
    models: &AssetStorage<ModelAsset>,
    textures: &AssetStorage<TextureAsset>,
    shaders: &AssetStorage<ShaderAsset>,
    ui_actions: &mut Vec<EngineUiAction>,
) -> Option<egui::Rect> {
    let mut close_modal = false;
    let mut modal_rect = None;

    if let Some(modal) = &mut state.preview_modal {
        let win_resp = egui::Window::new("asset_preview_modal")
            .id(egui::Id::new("asset_preview_modal"))
            .title_bar(false)
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(660.0, 520.0))
            .min_size(Vec2::new(500.0, 380.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(20, 20, 25))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::ZERO)
                    .shadow(egui::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: Color32::from_rgba_premultiplied(0, 0, 0, 180),
                    }),
            )
            .show(ctx, |ui| {
                // 1. Custom Sleek Header Bar
                egui::Frame::new()
                    .fill(Color32::from_rgb(15, 15, 20))
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
                    .corner_radius(CornerRadius {
                        nw: 8,
                        ne: 8,
                        sw: 0,
                        se: 0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(modal.item.category.badge())
                                    .color(modal.item.category.badge_color())
                                    .strong()
                                    .size(11.5),
                            );

                            ui.label(
                                RichText::new(&modal.item.name)
                                    .strong()
                                    .size(13.0)
                                    .color(Color32::WHITE),
                            );

                            ui.separator();

                            ui.label(
                                RichText::new(format!(
                                    "Size: {}",
                                    AssetBrowserState::format_file_size(modal.item.file_size_bytes)
                                ))
                                .size(11.0)
                                .color(Color32::from_gray(160)),
                            );

                            ui.separator();

                            if modal.item.is_loaded_in_memory {
                                ui.label(
                                    RichText::new("● In VRAM")
                                        .color(Color32::from_gray(210))
                                        .size(11.0),
                                );
                            } else {
                                ui.label(
                                    RichText::new("○ On Disk")
                                        .color(Color32::from_gray(130))
                                        .size(11.0),
                                );
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("✖")
                                                    .size(11.0)
                                                    .color(Color32::from_gray(160)),
                                            )
                                            .fill(Color32::TRANSPARENT)
                                            .frame(false),
                                        )
                                        .on_hover_text("Close (Esc)")
                                        .clicked()
                                    {
                                        close_modal = true;
                                    }
                                },
                            );
                        });
                    });

                // 2. Domain-Specific Preview Content
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(14, 12))
                    .show(ui, |ui| match modal.item.category {
                        AssetCategory::Models3D => {
                            draw_model_preview_section(ui, modal, models, ui_actions);
                        }
                        AssetCategory::Textures2D => {
                            draw_texture_preview_section(ui, modal, textures, ui_actions);
                        }
                        AssetCategory::Shaders => {
                            draw_shader_preview_section(ui, modal, shaders);
                        }
                        AssetCategory::Scenes => {
                            draw_scene_preview_section(ui, modal, ui_actions);
                        }
                        _ => {
                            draw_generic_metadata_section(ui, modal);
                        }
                    });
            });

        if let Some(r) = win_resp {
            modal_rect = Some(r.response.rect);
        }

        // Close on Esc key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_modal = true;
        }
    }

    if close_modal {
        state.preview_modal = None;
    }

    modal_rect
}

/// Renders the 3D model inspection section with interactive orbit canvas and metadata cards.
fn draw_model_preview_section(
    ui: &mut Ui,
    modal: &mut PreviewModalState,
    models: &AssetStorage<ModelAsset>,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    let model_opt = modal.item.model_handle.and_then(|h| models.get(h));

    ui.horizontal(|ui| {
        // Controls Row
        ui.checkbox(&mut modal.show_wireframe, "Wireframe Projection");
        ui.separator();
        ui.label(
            RichText::new("🖱 Left Drag: Orbit | Scroll: Zoom")
                .size(11.0)
                .color(Color32::from_gray(150)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(h) = modal.item.model_handle
                && ui.button("➕ Spawn into Scene").clicked()
            {
                ui_actions.push(EngineUiAction::SpawnModel(h));
            }
        });
    });

    ui.add_space(6.0);

    // Interactive 3D Wireframe Orbit Canvas
    let canvas_size = Vec2::new(ui.available_width(), 320.0);
    let (rect, response) = ui.allocate_exact_size(canvas_size, Sense::drag());

    if response.dragged() {
        let delta = response.drag_delta();
        modal.orbit_yaw += delta.x * 0.01;
        modal.orbit_pitch = (modal.orbit_pitch + delta.y * 0.01).clamp(-1.5, 1.5);
    }

    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
    if response.hovered() && scroll != 0.0 {
        modal.zoom_distance = (modal.zoom_distance - scroll * 0.002).clamp(0.4, 3.0);
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(6), Color32::from_rgb(14, 16, 22));
    painter.rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0, Color32::from_rgb(35, 38, 48)),
        egui::StrokeKind::Inside,
    );

    // Render 3D Projected Wireframe Box / Mesh
    let center = rect.center();
    let zoom = 100.0 / modal.zoom_distance;

    let (min_pt, max_pt, vert_count, tri_count) = if let Some(m) = model_opt {
        let vc = m.raw_vertices.len();
        let tc = m.num_indices as usize / 3;
        (m.min, m.max, vc, tc)
    } else {
        ([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0], 0, 0)
    };

    draw_projected_cube_wireframe(
        &painter,
        center,
        min_pt,
        max_pt,
        modal.orbit_yaw,
        modal.orbit_pitch,
        zoom,
    );

    ui.add_space(8.0);

    // Model Metrics Card
    egui::Frame::NONE
        .fill(Color32::from_rgb(20, 24, 32))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Vertices: {}", vert_count))
                        .strong()
                        .size(11.5),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!("Triangles: {}", tri_count))
                        .strong()
                        .size(11.5),
                );
                ui.separator();
                let dim_x = max_pt[0] - min_pt[0];
                let dim_y = max_pt[1] - min_pt[1];
                let dim_z = max_pt[2] - min_pt[2];
                ui.label(
                    RichText::new(format!(
                        "Extents: {:.2}m x {:.2}m x {:.2}m",
                        dim_x, dim_y, dim_z
                    ))
                    .size(11.5)
                    .color(Color32::from_gray(180)),
                );
            });
        });
}

/// Projects a 3D bounding box wireframe onto the 2D egui painter canvas.
fn draw_projected_cube_wireframe(
    painter: &egui::Painter,
    center: Pos2,
    min: [f32; 3],
    max: [f32; 3],
    yaw: f32,
    pitch: f32,
    scale: f32,
) {
    let corners = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];

    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let cos_p = pitch.cos();
    let sin_p = pitch.sin();

    let mut projected = [Pos2::ZERO; 8];

    for (i, c) in corners.iter().enumerate() {
        // Rotate around Y (Yaw)
        let x1 = c[0] * cos_y - c[2] * sin_y;
        let z1 = c[0] * sin_y + c[2] * cos_y;

        // Rotate around X (Pitch)
        let y2 = c[1] * cos_p - z1 * sin_p;
        let _z2 = c[1] * sin_p + z1 * cos_p;

        let screen_x = center.x + x1 * scale;
        let screen_y = center.y - y2 * scale;
        projected[i] = Pos2::new(screen_x, screen_y);
    }

    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let stroke = Stroke::new(1.5, Color32::from_rgb(0, 229, 255));
    for (i0, i1) in edges {
        painter.line_segment([projected[i0], projected[i1]], stroke);
    }

    // Draw origin axes
    let origin_screen = center;
    let x_axis_screen = Pos2::new(
        center.x + cos_y * scale * 0.4,
        center.y + sin_y * sin_p * scale * 0.4,
    );
    let y_axis_screen = Pos2::new(center.x, center.y - cos_p * scale * 0.4);
    let z_axis_screen = Pos2::new(
        center.x - sin_y * scale * 0.4,
        center.y + cos_y * sin_p * scale * 0.4,
    );

    painter.line_segment(
        [origin_screen, x_axis_screen],
        Stroke::new(2.0, Color32::from_rgb(255, 60, 60)),
    );
    painter.line_segment(
        [origin_screen, y_axis_screen],
        Stroke::new(2.0, Color32::from_rgb(60, 255, 60)),
    );
    painter.line_segment(
        [origin_screen, z_axis_screen],
        Stroke::new(2.0, Color32::from_rgb(60, 160, 255)),
    );
}

/// Renders the 2D texture inspection section with RGBA channel filters and dimensions.
fn draw_texture_preview_section(
    ui: &mut Ui,
    modal: &mut PreviewModalState,
    textures: &AssetStorage<TextureAsset>,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    let tex_opt = modal.item.texture_handle.and_then(|h| textures.get(h));

    ui.horizontal(|ui| {
        ui.label(RichText::new("Channels:").strong().size(11.5));
        ui.checkbox(&mut modal.channel_mask[0], "Red");
        ui.checkbox(&mut modal.channel_mask[1], "Green");
        ui.checkbox(&mut modal.channel_mask[2], "Blue");
        ui.checkbox(&mut modal.channel_mask[3], "Alpha");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(h) = modal.item.texture_handle
                && ui.button("🖼 Spawn as Sprite").clicked()
            {
                ui_actions.push(EngineUiAction::SpawnSprite(h));
            }
        });
    });

    ui.add_space(8.0);

    let (width, height) = if let Some(t) = tex_opt {
        (t.width, t.height)
    } else {
        (0, 0)
    };

    egui::Frame::NONE
        .fill(Color32::from_rgb(18, 20, 26))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(24, 20))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🖼").size(48.0));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("Resolution: {} x {} px", width, height))
                        .strong()
                        .size(13.0)
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(format!("File Path: {}", modal.item.path.display()))
                        .size(11.0)
                        .color(Color32::from_gray(150)),
                );
            });
        });
}

/// Renders the WGSL shader inspection section with syntax highlighting.
fn draw_shader_preview_section(
    ui: &mut Ui,
    modal: &mut PreviewModalState,
    shaders: &AssetStorage<ShaderAsset>,
) {
    if modal.wgsl_source.is_none() {
        if let Some(h) = modal.item.shader_handle {
            if let Some(s) = shaders.get(h) {
                modal.wgsl_source = Some(s.source_code.clone());
            }
        } else if let Ok(code) = std::fs::read_to_string(&modal.item.path) {
            modal.wgsl_source = Some(code);
        }
    }

    egui::ScrollArea::vertical()
        .max_height(380.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(source) = &modal.wgsl_source {
                for (line_idx, line) in source.lines().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{:4} | ", line_idx + 1))
                                .monospace()
                                .size(11.0)
                                .color(Color32::from_gray(100)),
                        );

                        let color = if line.trim_start().starts_with("//") {
                            Color32::from_rgb(100, 160, 100) // Comments
                        } else if line.contains("@vertex")
                            || line.contains("@fragment")
                            || line.contains("@group")
                            || line.contains("@binding")
                        {
                            Color32::from_rgb(255, 190, 60) // Attributes / Decorators
                        } else if line.contains("fn ")
                            || line.contains("struct ")
                            || line.contains("var ")
                        {
                            Color32::from_rgb(0, 229, 255) // Keywords
                        } else {
                            Color32::from_gray(220)
                        };

                        ui.label(RichText::new(line).monospace().size(11.0).color(color));
                    });
                }
            } else {
                ui.label("Failed to load shader source code.");
            }
        });
}

/// Renders the scene inspection section with a direct load button.
fn draw_scene_preview_section(
    ui: &mut Ui,
    modal: &mut PreviewModalState,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.label(RichText::new("🎬").size(48.0));
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Scene File: {}", modal.item.name))
                .strong()
                .size(13.0)
                .color(Color32::WHITE),
        );
        ui.label(
            RichText::new(modal.item.path.display().to_string())
                .size(11.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(16.0);
        if ui.button("🎬 Load Scene into World").clicked() {
            ui_actions.push(EngineUiAction::LoadSceneFromPath(modal.item.path.clone()));
        }
    });
}

/// Renders a generic key-value metadata breakdown for unsupported preview types.
fn draw_generic_metadata_section(ui: &mut Ui, modal: &PreviewModalState) {
    ui.vertical(|ui| {
        ui.label(RichText::new(format!("Name: {}", modal.item.name)).strong());
        ui.label(RichText::new(format!(
            "Path: {}",
            modal.item.path.display()
        )));
        ui.label(RichText::new(format!(
            "Category: {:?}",
            modal.item.category
        )));
        ui.label(RichText::new(format!(
            "Size: {} bytes",
            modal.item.file_size_bytes
        )));
    });
}