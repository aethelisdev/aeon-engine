// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::EngineUiAction;
use ae_editor::gizmo::{GizmoMode, GizmoSpace};
use ae_renderer::camera::{Camera, ProjectionMode};
use egui::{Context, Rect};

/// Draws the Viewport Toolbar containing projection options,
/// shading mode selector, and W/E/R translation/rotation/scale gizmo controls.
pub(super) fn draw_viewport_toolbar(
    ctx: &Context,
    available_rect: Rect,
    wireframe_enabled: &mut bool,
    gizmo_mode: &mut GizmoMode,
    gizmo_space: &mut GizmoSpace,
    camera: &Camera,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    egui::Area::new(egui::Id::new("viewport_toolbar_area"))
        .pivot(egui::Align2::RIGHT_TOP)
        .fixed_pos(egui::pos2(
            available_rect.right() - 115.0,
            available_rect.top() + 20.0,
        ))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Viewport Toolbar Frame
                egui::Frame::NONE
                    .fill(egui::Color32::from_black_alpha(200))
                    .corner_radius(egui::CornerRadius::same(6))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(16.0, 0.0);

                            // 1. Camera Mode Dropdown
                            let is_persp = camera.mode == ProjectionMode::Perspective;
                            let is_top = !is_persp && camera.pitch.0 < -1.5;
                            let is_front =
                                !is_persp && camera.pitch.0.abs() < 0.1 && camera.yaw.0 > 1.5;
                            let is_right =
                                !is_persp && camera.pitch.0.abs() < 0.1 && camera.yaw.0.abs() < 0.1;

                            let current_view = if is_persp {
                                "🎥 Perspective"
                            } else if is_top {
                                "📐 Top"
                            } else if is_front {
                                "📐 Front"
                            } else if is_right {
                                "📐 Right"
                            } else {
                                "📐 Orthographic"
                            };

                            ui.menu_button(
                                egui::RichText::new(current_view)
                                    .size(13.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                                |ui| {
                                    ui.set_min_width(140.0);
                                    if ui.button("🎥 Perspective").clicked() {
                                        ui_actions.push(EngineUiAction::SetCameraMode(
                                            ProjectionMode::Perspective,
                                        ));
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("📐 Top").clicked() {
                                        ui_actions.push(EngineUiAction::SetCameraMode(
                                            ProjectionMode::Orthographic,
                                        ));
                                        let d = 10.0;
                                        ui_actions.push(EngineUiAction::SetCameraTransform {
                                            pitch: cgmath::Rad(
                                                -std::f32::consts::FRAC_PI_2 + 0.001,
                                            ),
                                            yaw: cgmath::Rad(0.0),
                                            position: cgmath::Point3::new(
                                                camera.target.x,
                                                camera.target.y + d,
                                                camera.target.z,
                                            ),
                                        });
                                        ui.close();
                                    }
                                    if ui.button("📐 Front").clicked() {
                                        ui_actions.push(EngineUiAction::SetCameraMode(
                                            ProjectionMode::Orthographic,
                                        ));
                                        let d = 10.0;
                                        ui_actions.push(EngineUiAction::SetCameraTransform {
                                            pitch: cgmath::Rad(0.0),
                                            yaw: cgmath::Rad(std::f32::consts::FRAC_PI_2),
                                            position: cgmath::Point3::new(
                                                camera.target.x,
                                                camera.target.y,
                                                camera.target.z - d,
                                            ),
                                        });
                                        ui.close();
                                    }
                                    if ui.button("📐 Right").clicked() {
                                        ui_actions.push(EngineUiAction::SetCameraMode(
                                            ProjectionMode::Orthographic,
                                        ));
                                        let d = 10.0;
                                        ui_actions.push(EngineUiAction::SetCameraTransform {
                                            pitch: cgmath::Rad(0.0),
                                            yaw: cgmath::Rad(0.0),
                                            position: cgmath::Point3::new(
                                                camera.target.x + d,
                                                camera.target.y,
                                                camera.target.z,
                                            ),
                                        });
                                        ui.close();
                                    }
                                },
                            );

                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(1.0, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                egui::Color32::from_white_alpha(40),
                            );

                            // 2. Shading Mode Dropdown
                            let shading_label = if *wireframe_enabled {
                                "🕸 Wireframe"
                            } else {
                                "💡 Lit"
                            };
                            ui.menu_button(
                                egui::RichText::new(shading_label)
                                    .size(13.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                                |ui| {
                                    ui.set_min_width(120.0);
                                    if ui.button("💡 Lit").clicked() {
                                        *wireframe_enabled = false;
                                        ui.close();
                                    }
                                    if ui.button("🕸 Wireframe").clicked() {
                                        *wireframe_enabled = true;
                                        ui.close();
                                    }
                                },
                            );

                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(1.0, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                egui::Color32::from_white_alpha(40),
                            );

                            // 3. Gizmo Controls (W E R)
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                            let modes = [
                                (GizmoMode::Translate, "✛ W", "Translate (W)"),
                                (GizmoMode::Rotate, "⟳ E", "Rotate (E)"),
                                (GizmoMode::Scale, "⤡ R", "Scale (R)"),
                            ];
                            egui::Frame::NONE
                                .fill(egui::Color32::from_black_alpha(100))
                                .corner_radius(egui::CornerRadius::same(6))
                                .inner_margin(egui::Margin::symmetric(4, 4))
                                .show(ui, |ui| {
                                    for (gm, label, tooltip) in modes {
                                        let is_selected = *gizmo_mode == gm;
                                        let text_color = if is_selected {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::from_gray(140)
                                        };
                                        let bg_color = if is_selected {
                                            egui::Color32::from_rgb(70, 130, 220)
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };
                                        let button_text = egui::RichText::new(label)
                                            .size(13.0)
                                            .strong()
                                            .color(text_color);
                                        let res = ui.add(
                                            egui::Button::new(button_text)
                                                .fill(bg_color)
                                                .stroke(egui::Stroke::NONE)
                                                .min_size(egui::vec2(36.0, 22.0)),
                                        );
                                        if res.on_hover_text(tooltip).clicked() {
                                            *gizmo_mode = gm;
                                        }
                                    }
                                });

                            // 3b. Separator between gizmo mode and space toggle
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(1.0, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                egui::Color32::from_white_alpha(40),
                            );

                            // 3c. World/Local Space Toggle
                            let (space_label, space_tooltip) = match *gizmo_space {
                                GizmoSpace::World => ("🌍 World", "Switch to Local space"),
                                GizmoSpace::Local => ("🏠 Local", "Switch to World space"),
                            };
                            let space_text = egui::RichText::new(space_label)
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::WHITE);
                            let space_bg = match *gizmo_space {
                                GizmoSpace::Local => egui::Color32::from_rgb(180, 100, 50),
                                GizmoSpace::World => egui::Color32::from_black_alpha(100),
                            };
                            let space_btn = ui.add(
                                egui::Button::new(space_text)
                                    .fill(space_bg)
                                    .stroke(egui::Stroke::NONE)
                                    .corner_radius(egui::CornerRadius::same(4))
                                    .min_size(egui::vec2(64.0, 22.0)),
                            );
                            if space_btn.on_hover_text(space_tooltip).clicked() {
                                *gizmo_space = gizmo_space.toggle();
                            }
                        });
                    });
            });
        });
}

/// Draws the viewport HUD displaying real-time camera position (X, Y, Z) and pitch/yaw rotation angles.
pub(super) fn draw_camera_hud(ctx: &Context, available_rect: Rect, camera: &Camera) {
    egui::Area::new(egui::Id::new("camera_pos_hud"))
        .pivot(egui::Align2::RIGHT_TOP)
        .fixed_pos(egui::pos2(
            available_rect.right() - 10.0,
            available_rect.top() + 115.0,
        ))
        .show(ctx, |ui| {
            egui::Frame::window(&ctx.global_style())
                .fill(egui::Color32::from_black_alpha(180))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("📷 Camera Info")
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.separator();
                    let p = camera.position;
                    ui.label(format!("X: {:.2}  Y: {:.2}  Z: {:.2}", p.x, p.y, p.z));

                    let pitch_deg = camera.pitch.0.to_degrees();
                    let yaw_deg = camera.yaw.0.to_degrees();
                    ui.label(format!("Pitch: {:.1}°  Yaw: {:.1}°", pitch_deg, yaw_deg));
                });
        });
}

/// Renders the interactive 3D Scene Navigation Compass (Orientation Gizmo) in the top-right corner of the Viewport.
/// Clicking axis handles (X-Red, Y-Green, Z-Blue) snaps the camera to Top, Front, Right, or Iso views.
pub(super) fn draw_scene_navigation_gizmo(
    ctx: &Context,
    available_rect: Rect,
    camera: &Camera,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    let compass_center = egui::pos2(available_rect.right() - 55.0, available_rect.top() + 55.0);
    let radius = 35.0;

    egui::Area::new(egui::Id::new("scene_nav_gizmo"))
        .pivot(egui::Align2::CENTER_CENTER)
        .fixed_pos(compass_center)
        .show(ctx, |ui| {
            // Draw background compass circle
            ui.painter().circle_filled(
                compass_center,
                radius + 10.0,
                egui::Color32::from_black_alpha(160),
            );
            ui.painter().circle_stroke(
                compass_center,
                radius + 10.0,
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(40)),
            );

            let endpoints = ae_editor::scene_gizmo::SceneNavigationGizmo::compute_axis_endpoints(
                camera.pitch.0,
                camera.yaw.0,
                radius,
            );

            for (dx, dy, label, color_rgb) in endpoints {
                let end_pos = egui::pos2(compass_center.x + dx, compass_center.y + dy);
                let color = egui::Color32::from_rgb(color_rgb[0], color_rgb[1], color_rgb[2]);

                // Axis line
                ui.painter()
                    .line_segment([compass_center, end_pos], egui::Stroke::new(2.5, color));

                // Clickable Knob
                let knob_rect = Rect::from_center_size(end_pos, egui::vec2(18.0, 18.0));
                let resp = ui.allocate_rect(knob_rect, egui::Sense::click());

                let is_hovered = resp.hovered();
                let knob_color = if is_hovered {
                    egui::Color32::WHITE
                } else {
                    color
                };

                ui.painter().circle_filled(end_pos, 8.0, knob_color);
                ui.painter().text(
                    end_pos,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::monospace(10.0),
                    if is_hovered {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::WHITE
                    },
                );

                if resp.clicked() {
                    let snap = match label {
                        "X" => ae_editor::scene_gizmo::SceneViewSnap::Right,
                        "-X" => ae_editor::scene_gizmo::SceneViewSnap::Left,
                        "Y" => ae_editor::scene_gizmo::SceneViewSnap::Top,
                        "-Y" => ae_editor::scene_gizmo::SceneViewSnap::Bottom,
                        "Z" => ae_editor::scene_gizmo::SceneViewSnap::Front,
                        "-Z" => ae_editor::scene_gizmo::SceneViewSnap::Back,
                        _ => ae_editor::scene_gizmo::SceneViewSnap::Perspective,
                    };
                    let (target_pitch, target_yaw, target_pos) =
                        snap.compute_transform(camera.target, 12.0);
                    ui_actions.push(EngineUiAction::SetCameraTransform {
                        pitch: target_pitch,
                        yaw: target_yaw,
                        position: target_pos,
                    });
                }
            }
        });
}

/// Renders 3D Viewport Editor Billboard Icons for Light (`💡`), Audio (`🔊`), Ear (`👂`), Camera (`🎥`), and Empty (`📦`) entities.
pub(super) fn draw_billboard_icons(
    ctx: &Context,
    available_rect: Rect,
    world: &hecs::World,
    camera: &Camera,
    selected_entity: Option<hecs::Entity>,
    ui_actions: &mut Vec<EngineUiAction>,
) {
    let vp_matrix = camera.build_view_projection_matrix();
    let width = available_rect.width();
    let height = available_rect.height();

    for (ent, pos) in world
        .query::<(hecs::Entity, &ae_core::ecs::Position)>()
        .iter()
    {
        let is_light = world.get::<&ae_core::ecs::Light>(ent).is_ok();
        let is_audio_source = world.get::<&ae_audio::AudioSource>(ent).is_ok();
        let is_audio_listener = world.get::<&ae_audio::AudioListener>(ent).is_ok();

        if !is_light && !is_audio_source && !is_audio_listener {
            continue;
        }

        let icon_glyph = if is_light {
            "💡"
        } else if is_audio_source {
            "🔊"
        } else if is_audio_listener {
            "👂"
        } else {
            "📦"
        };

        // Project 3D position to Clip Space
        let pos_v4 = cgmath::Vector4::new(pos.x, pos.y, pos.z, 1.0);
        let clip_v4 = vp_matrix * pos_v4;

        if clip_v4.w <= 0.001 {
            continue; // Behind near plane
        }

        let ndc_x = clip_v4.x / clip_v4.w;
        let ndc_y = clip_v4.y / clip_v4.w;

        if ndc_x < -1.2 || ndc_x > 1.2 || ndc_y < -1.2 || ndc_y > 1.2 {
            continue; // Outside viewport bounds
        }

        let screen_x = available_rect.left() + (ndc_x + 1.0) * 0.5 * width;
        let screen_y = available_rect.top() + (1.0 - ndc_y) * 0.5 * height;

        let icon_pos = egui::pos2(screen_x, screen_y);
        let is_selected = selected_entity == Some(ent);

        egui::Area::new(egui::Id::new(("billboard_icon", ent)))
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(icon_pos)
            .show(ctx, |ui| {
                let text_color = if is_selected {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::WHITE
                };
                let bg_color = if is_selected {
                    egui::Color32::from_rgb(180, 120, 20)
                } else {
                    egui::Color32::from_black_alpha(180)
                };

                let btn = ui.add(
                    egui::Button::new(egui::RichText::new(icon_glyph).size(14.0).color(text_color))
                        .fill(bg_color)
                        .stroke(egui::Stroke::new(
                            1.0,
                            if is_selected {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::from_white_alpha(40)
                            },
                        ))
                        .corner_radius(egui::CornerRadius::same(12))
                        .min_size(egui::vec2(24.0, 24.0)),
                );

                if btn.clicked() {
                    ui_actions.push(EngineUiAction::SelectEntity(Some(ent)));
                }
            });
    }
}