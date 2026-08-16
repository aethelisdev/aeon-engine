// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use egui::{Color32, Context, CornerRadius, Margin, Rect, Stroke};

/// Renders the premium 'About Aeon Engine' dialogue window containing
/// project copyright, license terms under MPL 2.0, warranty disclaimer, and license hyperlinks.
pub(crate) fn draw_about_dialog(ctx: &Context, show_about: &mut bool, ui_rects: &mut Vec<Rect>) {
    if *show_about {
        let dialog_resp = egui::Window::new("About Aeon Engine")
            .id(egui::Id::new("about_dialog"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([460.0, 330.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(20, 20, 25))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::ZERO)
                    .shadow(egui::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: Color32::from_rgba_premultiplied(0, 0, 0, 180),
                    }),
            )
            .show(ctx, |ui| {
                // ── 1. CUSTOM SLEEK HEADER BAR ──
                egui::Frame::new()
                    .fill(Color32::from_rgb(15, 15, 20))
                    .inner_margin(Margin::symmetric(14, 8))
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
                                egui::RichText::new("ℹ  About Aeon Engine")
                                    .strong()
                                    .size(13.0)
                                    .color(Color32::from_gray(225)),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("✖")
                                                    .size(11.0)
                                                    .color(Color32::from_gray(160)),
                                            )
                                            .fill(Color32::TRANSPARENT)
                                            .frame(false),
                                        )
                                        .on_hover_text("Close")
                                        .clicked()
                                    {
                                        *show_about = false;
                                    }
                                },
                            );
                        });
                    });

                // ── 2. BODY CONTENT ──
                egui::Frame::new()
                    .inner_margin(Margin::symmetric(18, 14))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(4.0);

                            // Title & Version
                            ui.label(
                                egui::RichText::new("Aeon Engine")
                                    .size(22.0)
                                    .strong()
                                    .color(Color32::from_rgb(0, 229, 255)),
                            );
                            ui.label(
                                egui::RichText::new("Copyright (C) 2026 AethelisDEV")
                                    .size(12.0)
                                    .color(Color32::from_gray(180)),
                            );

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            // MPL 2.0 License Intro
                            ui.label(
                                egui::RichText::new(
                                    "This Source Code Form is subject to the terms of the Mozilla Public\n\
                                    License, v. 2.0. If a copy of the MPL was not distributed with this\n\
                                    file, You can obtain one at https://mozilla.org/MPL/2.0/.",
                                )
                                .size(12.0)
                                .color(Color32::from_gray(190))
                                .line_height(Some(17.0)),
                            );

                            ui.add_space(10.0);

                            // Warranty Disclaimer (highlighted in warning color)
                            ui.label(
                                egui::RichText::new("This program comes with ABSOLUTELY NO WARRANTY.")
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );

                            ui.add_space(10.0);

                            // License Hyperlink
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("For full license terms, see the LICENSE file or visit: ")
                                        .size(12.0)
                                        .color(Color32::from_gray(170)),
                                );
                            });
                            ui.hyperlink_to(
                                egui::RichText::new("https://mozilla.org/MPL/2.0/")
                                    .color(Color32::from_rgb(0, 229, 255))
                                    .underline(),
                                "https://mozilla.org/MPL/2.0/",
                            );

                            ui.add_space(14.0);
                            let close_btn = egui::Button::new(
                                egui::RichText::new("  Close  ").strong().color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(30, 34, 46))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(55, 60, 75)))
                            .corner_radius(CornerRadius::same(4));

                            if ui.add(close_btn).clicked() {
                                *show_about = false;
                            }
                        });
                    });
            });
        if let Some(rect) = dialog_resp.map(|r| r.response.rect) {
            ui_rects.push(rect);
        }
    }
}