// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use egui::Context;

/// Loads custom TrueType fonts including NotoSans, specialized symbols, mathematical symbols,
/// and monochrome emojis, establishing the fallback priority stack for egui.
pub(crate) fn load_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "noto_sans".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../ae_engine/assets/fonts/NotoSans-Regular.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "noto_sans_symbols".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../ae_engine/assets/fonts/NotoSansSymbols-Regular.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "noto_sans_symbols2".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../ae_engine/assets/fonts/NotoSansSymbols2-Regular.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "noto_sans_math".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../ae_engine/assets/fonts/NotoSansMath-Regular.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "noto_emoji".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../ae_engine/assets/fonts/NotoEmoji-Regular.ttf"
        ))
        .into(),
    );

    if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        let mut new_prop = vec![
            "noto_sans".to_owned(),
            "noto_sans_symbols".to_owned(),
            "noto_sans_symbols2".to_owned(),
            "noto_sans_math".to_owned(),
            "noto_emoji".to_owned(),
        ];
        new_prop.retain(|f| !prop.contains(f));
        prop.splice(0..0, new_prop);
    }
    if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        let mut new_mono = vec![
            "noto_sans".to_owned(),
            "noto_sans_symbols".to_owned(),
            "noto_sans_symbols2".to_owned(),
            "noto_sans_math".to_owned(),
            "noto_emoji".to_owned(),
        ];
        new_mono.retain(|f| !mono.contains(f));
        mono.extend(new_mono);
    }
    ctx.set_fonts(fonts);
}

/// Configures the global style variables, color palette, padding, rounding, and spacing
/// of the egui context to match the Aeon Engine custom professional dark theme.
pub(crate) fn setup_custom_style(ctx: &Context) {
    let mut style: egui::Style = (*ctx.global_style()).clone();

    // 1. Accent & Main Colors
    let accent_color = egui::Color32::from_rgb(0, 229, 255);
    let hover_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15);
    let window_bg = egui::Color32::from_rgb(20, 20, 25);
    let border_color = egui::Color32::from_rgb(45, 48, 60);

    // 2. Visuals & Rounding (egui CornerRadius)
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(6);
    style.visuals.window_shadow.offset = [0, 8];
    style.visuals.window_shadow.blur = 20;
    style.visuals.window_shadow.spread = 0;
    style.visuals.window_shadow.color = egui::Color32::from_black_alpha(180);
    style.visuals.popup_shadow.offset = [0, 6];
    style.visuals.popup_shadow.blur = 16;
    style.visuals.popup_shadow.spread = 0;
    style.visuals.popup_shadow.color = egui::Color32::from_black_alpha(160);

    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(4);

    // 3. Selection & Accent Highlighting
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 60, 80);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, accent_color);

    // 4. Widget background fills
    style.visuals.widgets.hovered.bg_fill = hover_color;
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 60, 80);
    style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(26, 28, 36);

    // 5. Spacing & Padding
    style.spacing.item_spacing = egui::vec2(6.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.menu_margin = egui::Margin::symmetric(6, 6);

    // 6. Window transparency and Outlines
    style.visuals.window_fill = window_bg;
    style.visuals.window_stroke = egui::Stroke::new(1.0, border_color);
    style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 25);

    ctx.set_global_style(style);
}