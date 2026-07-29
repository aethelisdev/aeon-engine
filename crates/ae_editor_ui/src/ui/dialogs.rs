// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use egui::{Context, Rect};

/// Renders the asset loading progress indicator overlay, FBX converter automatic downloader prompt,
/// and Python silent installation window. Collects interactive viewport boundaries to block underlying 3D clicks.
pub(super) fn draw_dialogs(ctx: &Context, is_loading_assets: bool, ui_rects: &mut Vec<Rect>) {
    // 1. Loading Overlay
    if is_loading_assets {
        let loading_resp = egui::Area::new(egui::Id::new("loading_overlay"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(30.0));
                        ui.label(egui::RichText::new("🚀 Loading...").strong().size(16.0));
                    });
                });
            });
        ui_rects.push(loading_resp.response.rect);
    }
}