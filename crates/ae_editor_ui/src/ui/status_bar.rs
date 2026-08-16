// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use crate::ui::EngineUi;
use crate::ui::panel_layout::PanelLayoutState;

impl EngineUi {
    /// Renders a persistent thin bar at the absolute bottom for quick status info.
    pub(super) fn draw_utility_bar(
        _layout_state: &mut PanelLayoutState,
        status_message: &mut Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
        ui: &mut egui::Ui,
    ) -> Option<egui::Rect> {
        let resp = egui::Panel::bottom("utility_bar")
            .exact_size(22.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(15, 15, 20))
                    .inner_margin(egui::Margin::symmetric(10, 3))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 42, 52))),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 1. Status Message or Ready indicator
                    if let Some((spans, _)) = status_message {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (text, color) in spans {
                            ui.label(egui::RichText::new(text.as_str()).color(*color).size(11.0));
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("● Ready")
                                .color(egui::Color32::from_rgb(70, 190, 120))
                                .size(11.0),
                        );
                    }

                    // 2. Right-side Engine info
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Aeon Engine v{}",
                                env!("CARGO_PKG_VERSION")
                            ))
                            .color(egui::Color32::from_gray(90))
                            .size(11.0),
                        );
                    });
                });
            });

        Some(resp.response.rect)
    }
}