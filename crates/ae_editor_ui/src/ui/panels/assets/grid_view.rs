// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asset Browser Interactive Card Grid View.
//!
//! Renders thumbnail cards with category badges, hover animations,
//! selection outlines, drag sources, double-click spawn, and context menus.
//!

use super::context_menu::attach_asset_context_menu;
use super::drag_drop::handle_asset_drag_source;
use super::thumbnails::get_or_load_thumbnail;
use super::types::{AssetBrowserState, AssetCategory, AssetItem, PreviewModalState};
use crate::ui::types::EngineUiAction;
use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

/// Draws the asset browser items as a responsive wrapping grid of interactive cards.
pub fn draw_asset_grid_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    items: &[AssetItem],
    ui_actions: &mut Vec<EngineUiAction>,
) {
    let ctx = ui.ctx().clone();

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);

        for item in items {
            let is_selected = state.selected_asset.as_ref() == Some(&item.path);
            let card_size = Vec2::new(115.0, 125.0);

            let (rect, response) = ui.allocate_exact_size(card_size, Sense::click_and_drag());

            let is_hovered = response.hovered();

            // Background & Border Styling
            let bg_color = if is_selected {
                Color32::from_rgb(26, 32, 44)
            } else if is_hovered {
                Color32::from_rgb(28, 30, 40)
            } else {
                Color32::from_rgb(18, 20, 26)
            };

            let stroke = if is_selected {
                Stroke::new(1.5, Color32::from_rgb(0, 229, 255))
            } else if is_hovered {
                Stroke::new(1.0, item.category.badge_color())
            } else {
                Stroke::new(1.0, Color32::from_rgb(38, 42, 54))
            };

            let painter = ui.painter();
            painter.rect(
                rect,
                CornerRadius::same(6),
                bg_color,
                stroke,
                StrokeKind::Inside,
            );

            // 1. Category Badge & Memory Indicator (Top)
            let badge_rect =
                Rect::from_min_size(rect.min + Vec2::new(6.0, 6.0), Vec2::new(36.0, 16.0));
            painter.rect_filled(
                badge_rect,
                CornerRadius::same(3),
                Color32::from_rgba_unmultiplied(
                    item.category.badge_color().r(),
                    item.category.badge_color().g(),
                    item.category.badge_color().b(),
                    40,
                ),
            );
            painter.text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                item.category.badge(),
                egui::FontId::proportional(9.0),
                item.category.badge_color(),
            );

            // Memory Dot (Green/Cyan if loaded in VRAM)
            if item.is_loaded_in_memory {
                let dot_pos = rect.max - Vec2::new(12.0, rect.height() - 14.0);
                painter.circle_filled(dot_pos, 3.5, Color32::from_rgb(0, 229, 255));
            }

            // 2. Thumbnail Preview or Category Icon
            let thumb_opt = get_or_load_thumbnail(&ctx, state, &item.path, item.category);

            if let Some(thumb) = thumb_opt {
                let img_rect = Rect::from_center_size(
                    rect.center() - Vec2::new(0.0, 9.0),
                    Vec2::new(54.0, 54.0),
                );

                // Background preview frame
                painter.rect(
                    img_rect.expand(1.0),
                    CornerRadius::same(4),
                    Color32::from_rgb(12, 14, 18),
                    Stroke::new(1.0, Color32::from_rgb(40, 44, 56)),
                    StrokeKind::Inside,
                );

                painter.image(
                    thumb.id(),
                    img_rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                let icon_text = match item.category {
                    AssetCategory::Models3D => "📦",
                    AssetCategory::Textures2D => "🖼",
                    AssetCategory::Shaders => "⚡",
                    AssetCategory::Scenes => "🎬",
                    AssetCategory::Materials => "🎨",
                    AssetCategory::Audio => "🔊",
                    AssetCategory::All => "📄",
                };
                painter.text(
                    rect.center() - Vec2::new(0.0, 9.0),
                    egui::Align2::CENTER_CENTER,
                    icon_text,
                    egui::FontId::proportional(26.0),
                    Color32::WHITE,
                );
            }

            // 3. Name Label (Truncated)
            let display_name = if item.name.len() > 14 {
                format!("{}...", &item.name[..11])
            } else {
                item.name.clone()
            };
            painter.text(
                rect.center() + Vec2::new(0.0, 24.0),
                egui::Align2::CENTER_CENTER,
                display_name,
                egui::FontId::proportional(11.0),
                if is_selected {
                    Color32::WHITE
                } else {
                    Color32::from_gray(210)
                },
            );

            // 4. Metadata Badge (Bottom, e.g. "24.5k Verts", "15.9 KB")
            painter.text(
                rect.center() + Vec2::new(0.0, 42.0),
                egui::Align2::CENTER_CENTER,
                &item.metadata_badge,
                egui::FontId::proportional(9.0),
                Color32::from_gray(130),
            );

            // Interactions
            if response.clicked() {
                state.selected_asset = Some(item.path.clone());
            }

            // Drag Source
            handle_asset_drag_source(&response, item, state);

            if response.double_clicked() {
                match item.category {
                    AssetCategory::Models3D => {
                        if let Some(handle) = item.model_handle {
                            ui_actions.push(EngineUiAction::SpawnModel(handle));
                        } else {
                            ui_actions.push(EngineUiAction::SpawnModelPathAt(
                                item.path.clone(),
                                [0.0, 0.0, 0.0],
                            ));
                        }
                    }
                    AssetCategory::Textures2D => {
                        if let Some(handle) = item.texture_handle {
                            ui_actions.push(EngineUiAction::SpawnSprite(handle));
                        } else {
                            ui_actions.push(EngineUiAction::SpawnSpritePathAt(
                                item.path.clone(),
                                [0.0, 0.0, 0.0],
                            ));
                        }
                    }
                    AssetCategory::Scenes => {
                        ui_actions.push(EngineUiAction::LoadSceneFromPath(item.path.clone()));
                    }
                    _ => {
                        state.preview_modal = Some(PreviewModalState {
                            item: item.clone(),
                            orbit_yaw: 0.0,
                            orbit_pitch: 0.3,
                            zoom_distance: 1.0,
                            show_wireframe: true,
                            channel_mask: [true, true, true, true],
                            wgsl_source: None,
                        });
                    }
                }
            }

            // Context Menu
            attach_asset_context_menu(&response, item, state, ui_actions);
        }
    });
}