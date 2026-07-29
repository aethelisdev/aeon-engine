// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use egui::{Color32, Context, Rect};

/// Renders the premium 'About Aeon Engine' dialogue window containing
/// project copyright, license terms under MPL 2.0, warranty disclaimer, and license hyperlinks.
pub(crate) fn draw_about_dialog(ctx: &Context, show_about: &mut bool, ui_rects: &mut Vec<Rect>) {
    if *show_about {
        let dialog_resp = egui::Window::new(
            egui::RichText::new("About Aeon Engine")
                .strong()
                .size(16.0)
                .color(Color32::from_rgb(77, 163, 255)),
        )
        .collapsible(false)
        .resizable(false)
        .fixed_size([460.0, 320.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                // Title & Version
                ui.label(
                    egui::RichText::new("Aeon Engine")
                        .size(24.0)
                        .strong()
                        .color(egui::Color32::from_rgb(77, 163, 255)),
                );
                ui.label(
                    egui::RichText::new("Copyright (C) 2026 AethelisDEV")
                        .size(13.0)
                        .color(egui::Color32::from_gray(200)),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(10.0);

                // MPL 2.0 License Intro
                ui.label(
                    egui::RichText::new(
                        "This Source Code Form is subject to the terms of the Mozilla Public\n\
                        License, v. 2.0. If a copy of the MPL was not distributed with this\n\
                        file, You can obtain one at https://mozilla.org/MPL/2.0/.",
                    )
                    .size(13.0)
                    .line_height(Some(18.0)),
                );

                ui.add_space(12.0);

                // Warranty Disclaimer (highlighted in red/orange warning color)
                ui.label(
                    egui::RichText::new("This program comes with ABSOLUTELY NO WARRANTY.")
                        .strong()
                        .size(13.0)
                        .color(egui::Color32::from_rgb(255, 100, 100)),
                );

                ui.add_space(12.0);

                // License Hyperlink
                ui.horizontal(|ui| {
                    ui.label("For full license terms, see the LICENSE file or visit: ");
                });
                ui.hyperlink_to(
                    egui::RichText::new("https://mozilla.org/MPL/2.0/")
                        .color(egui::Color32::from_rgb(77, 163, 255))
                        .underline(),
                    "https://mozilla.org/MPL/2.0/",
                );

                ui.add_space(16.0);
                if ui
                    .button(egui::RichText::new("  Close  ").strong())
                    .clicked()
                {
                    *show_about = false;
                }

                ui.add_space(4.0);
            });
        });
        if let Some(rect) = dialog_resp.map(|r| r.response.rect) {
            ui_rects.push(rect);
        }
    }
}