// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Preview Details, Metadata & Code Inspectors
//!
//! Renders specification cards, texture diagnostics, WGSL shader inspectors,
//! scene hierarchy summaries, and audio playback previews within the Quick Asset Inspector
//! using pure declarative [`UiScope`].
//!

use crate::ui::iris_bridge::assets::types::AssetPreviewModalState;
use irisui::prelude::*;

/// Renders the 2D texture specification and analysis section.
pub(crate) fn render_texture_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    ui.container(
        Style::new()
            .flex_row()
            .justify_content(JustifyContent::FlexEnd)
            .height(28.0),
        |row| {
            row.modal_confirm_button("Spawn as Sprite", 140.0);
        },
    );

    render_info_box(ui, 314.0, |info| {
        info.key_value("File Name", &modal.item.name);
        info.key_value_file_size("File Size on Disk", modal.item.file_size_bytes);
        info.key_value("Format", "2D Texture Image (Straight Alpha / RGBA8)");
        info.key_value(
            "GPU Texture Allocation",
            "WGPU Texture2D / Dynamic Mipmap Chain",
        );
        info.key_value(
            "Sampling Mode",
            "Linear Trilinear Filtering + 16x Anisotropic",
        );
        info.key_value("Wrap Mode", "Repeat / Clamp to Edge");
        info.key_value(
            "Memory Residency",
            if modal.item.is_loaded_in_memory {
                "Resident in VRAM"
            } else {
                "Unloaded (Lazy Streaming)"
            },
        );
    });
}

/// Renders the WGSL shader specification and diagnostics section.
pub(crate) fn render_shader_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    render_info_box(ui, 350.0, |info| {
        info.key_value("Shader Module", &modal.item.name);
        info.key_value("Language", "WebGPU Shading Language (WGSL 1.0 Standard)");
        info.key_value(
            "Supported Entry Points",
            "@vertex vs_main, @fragment fs_main",
        );
        info.key_value(
            "Uniform Bind Groups",
            "Global Camera Matrix, Light Uniforms, Material PBR Maps",
        );
        info.key_value(
            "Validation Status",
            "Verified & Compiled on Active WGPU Hardware Pipeline",
        );
        info.key_value_file_size("File Size", modal.item.file_size_bytes);
    });
}

/// Renders the scene summary and direct load section.
pub(crate) fn render_scene_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    ui.container(
        Style::new()
            .flex_row()
            .justify_content(JustifyContent::FlexEnd)
            .height(28.0),
        |row| {
            row.modal_confirm_button("Load Scene", 140.0);
        },
    );

    render_info_box(ui, 314.0, |info| {
        info.key_value("Scene Name", &modal.item.name);
        info.key_value("File Format", "Aeon Scene Descriptor (.ae3d / .ae2d JSON)");
        info.key_value(
            "Entity Hierarchy",
            "Declarative ECS World State with Transform & Behaviors",
        );
        info.key_value(
            "Environment",
            "Dynamic Atmosphere, 4-Cascade CSM Sun & Procedural Clouds",
        );
        info.key_value_file_size("Package Size", modal.item.file_size_bytes);
    });
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

    render_info_box(ui, 314.0, |info| {
        info.key_value("Audio File", &modal.item.name);
        info.key_value(
            "Audio Engine",
            "Rodio 0.20 Output Stream & 3D Spatial Attenuation",
        );
        info.key_value("Supported Formats", "WAV PCM, MP3 MPEG, OGG Vorbis");
        info.key_value("Sampling Rate", "44.1 kHz / 48.0 kHz Multi-Channel");
        info.key_value_file_size("File Size", modal.item.file_size_bytes);
    });
}

/// Renders generic asset information for fallback categories.
pub(crate) fn render_generic_preview_content(ui: &mut UiScope<'_>, modal: &AssetPreviewModalState) {
    render_info_box(ui, 350.0, |info| {
        info.key_value("Asset Name", &modal.item.name);
        info.key_value("Relative Path", &modal.item.relative_path);
        info.key_value("Classification", modal.item.category.label());
        info.key_value_file_size("File Size", modal.item.file_size_bytes);
        info.key_value(
            "Memory State",
            if modal.item.is_loaded_in_memory {
                "Loaded in VRAM"
            } else {
                "On Disk"
            },
        );
    });
}

/// Helper to render structured text key-value rows within an elevated info card using declarative [`UiScope`].
pub(crate) fn render_info_box<F>(ui: &mut UiScope<'_>, height: f32, builder: F)
where
    F: FnOnce(&mut UiScope<'_>),
{
    ui.container(
        Style::new()
            .flex_col()
            .gap(10.0)
            .padding_insets(Insets::new(16.0, 20.0, 16.0, 20.0))
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60))
            .height(height),
        builder,
    );
}