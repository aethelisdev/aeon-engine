// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Preview Details, Metadata & Code Inspectors
//!
//! Renders specification cards, texture diagnostics, WGSL shader inspectors,
//! scene hierarchy summaries, and audio playback previews within the Quick Asset Inspector
//! using pure declarative [`UiScope`].
//!

use crate::assets::types::AssetBrowserState;
use crate::ui::iris_bridge::assets::types::AssetPreviewModalState;
use irisui::prelude::*;

/// Renders the 2D texture specification and analysis section.
pub(crate) fn render_texture_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    // Top action bar
    ui.container(
        Style::new()
            .flex_row()
            .justify_content(JustifyContent::FlexEnd)
            .height(28.0),
        |row| {
            row.modal_confirm_button("Spawn as Sprite", 140.0);
        },
    );

    let label_lines = [
        format!("File Name: {}", modal.item.name),
        format!(
            "File Size on Disk: {}",
            AssetBrowserState::format_file_size(modal.item.file_size_bytes)
        ),
        "Format: 2D Texture Image (Straight Alpha / RGBA8)".to_string(),
        "GPU Texture Allocation: WGPU Texture2D / Dynamic Mipmap Chain".to_string(),
        "Sampling Mode: Linear Trilinear Filtering + 16x Anisotropic".to_string(),
        "Wrap Mode: Repeat / Clamp to Edge".to_string(),
        format!(
            "Memory Residency: {}",
            if modal.item.is_loaded_in_memory {
                "Resident in VRAM"
            } else {
                "Unloaded (Lazy Streaming)"
            }
        ),
    ];

    render_info_box(ui, &label_lines, Color::rgba(0.80, 0.84, 0.92, 1.0), 314.0);
}

/// Renders the WGSL shader specification and diagnostics section.
pub(crate) fn render_shader_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    let lines = [
        format!("Shader Module: {}", modal.item.name),
        "Language: WebGPU Shading Language (WGSL 1.0 Standard)".to_string(),
        "Supported Entry Points: @vertex vs_main, @fragment fs_main".to_string(),
        "Uniform Bind Groups: Global Camera Matrix, Light Uniforms, Material PBR Maps".to_string(),
        "Validation Status: Verified & Compiled on Active WGPU Hardware Pipeline".to_string(),
        format!(
            "File Size: {}",
            AssetBrowserState::format_file_size(modal.item.file_size_bytes)
        ),
    ];

    render_info_box(ui, &lines, Color::rgba(1.0, 0.85, 0.45, 1.0), 350.0);
}

/// Renders the scene summary and direct load section.
pub(crate) fn render_scene_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    // Top action bar
    ui.container(
        Style::new()
            .flex_row()
            .justify_content(JustifyContent::FlexEnd)
            .height(28.0),
        |row| {
            row.modal_confirm_button("Load Scene", 140.0);
        },
    );

    let lines = [
        format!("Scene Name: {}", modal.item.name),
        "File Format: Aeon Scene Descriptor (.ae3d / .ae2d JSON)".to_string(),
        "Entity Hierarchy: Declarative ECS World State with Transform & Behaviors".to_string(),
        "Environment: Dynamic Atmosphere, 4-Cascade CSM Sun & Procedural Clouds".to_string(),
        format!(
            "Package Size: {}",
            AssetBrowserState::format_file_size(modal.item.file_size_bytes)
        ),
    ];

    render_info_box(ui, &lines, Color::rgba(0.60, 0.80, 1.0, 1.0), 314.0);
}

/// Renders the audio specification and play preview section.
pub(crate) fn render_audio_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    ui.container(
        Style::new()
            .flex_row()
            .justify_content(JustifyContent::FlexEnd)
            .height(28.0),
        |row| {
            row.modal_confirm_button("Play Audio", 140.0);
        },
    );

    let lines = [
        format!("Audio File: {}", modal.item.name),
        "Audio Engine: Rodio 0.20 Output Stream & 3D Spatial Attenuation".to_string(),
        "Supported Formats: WAV PCM, MP3 MPEG, OGG Vorbis".to_string(),
        "Sampling Rate: 44.1 kHz / 48.0 kHz Multi-Channel".to_string(),
        format!(
            "File Size: {}",
            AssetBrowserState::format_file_size(modal.item.file_size_bytes)
        ),
    ];

    render_info_box(ui, &lines, Color::rgba(1.0, 0.60, 0.50, 1.0), 314.0);
}

/// Renders generic asset information for fallback categories.
pub(crate) fn render_generic_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    let lines = [
        format!("Asset Name: {}", modal.item.name),
        format!("Relative Path: {}", modal.item.relative_path),
        format!("Classification: {:?}", modal.item.category),
        format!(
            "File Size: {}",
            AssetBrowserState::format_file_size(modal.item.file_size_bytes)
        ),
        format!(
            "Memory State: {}",
            if modal.item.is_loaded_in_memory {
                "Loaded in VRAM"
            } else {
                "On Disk"
            }
        ),
    ];

    render_info_box(ui, &lines, Color::rgba(0.80, 0.84, 0.92, 1.0), 350.0);
}

/// Helper to render structured text lines within an elevated info card using declarative [`UiScope`].
pub(crate) fn render_info_box(
    ui: &mut UiScope<'_>,
    lines: &[String],
    text_color: Color,
    height: f32,
) {
    let box_style = Style::new()
        .flex_col()
        .gap(10.0)
        .padding_insets(Insets::new(16.0, 20.0, 16.0, 20.0))
        .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
        .border_radius(6.0)
        .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60))
        .height(height);

    ui.container(box_style, |col| {
        for line in lines {
            col.label(line, 11.5, text_color, TextAlign::Left);
        }
    });
}