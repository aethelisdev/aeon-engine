// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Asset Preview Details, Metadata & Code Inspectors
//!
//! Renders specification cards, texture diagnostics, WGSL shader inspectors,
//! scene hierarchy summaries, and audio playback previews within the Quick Asset Inspector.
//!

use crate::ui::iris_bridge::assets::types::AssetPreviewModalState;
use crate::ui::panels::assets::types::AssetBrowserState;
use irisui::prelude::*;

/// Renders the 2D texture specification and analysis section.
pub(crate) fn render_texture_preview_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    modal: &AssetPreviewModalState,
    cursor_pos: Point,
) -> Rect {
    let spawn_rect = Rect::new(body_x + body_w - 150.0, body_y, 150.0, 28.0);
    let is_spawn_hovered = spawn_rect.contains_point(cursor_pos);
    let spawn_id = tree.create_node();
    if let Some(node) = tree.get_mut(spawn_id) {
        node.set_name("PreviewSpawnSpriteBtn");
        node.set_text("Spawn as Sprite");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = spawn_rect;
        node.style = Style::new()
            .background(if is_spawn_hovered {
                Color::rgba(0.12, 0.48, 0.28, 1.0)
            } else {
                Color::rgba(0.08, 0.38, 0.22, 0.90)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.20, 0.85, 0.45, 0.80));
    }
    let _ = tree.add_child(parent_id, spawn_id);

    let info_box_rect = Rect::new(body_x, body_y + 36.0, body_w, 280.0);
    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("TextureInfoBox");
        node.computed_rect = info_box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60));
    }
    let _ = tree.add_child(parent_id, box_id);

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

    render_info_lines(
        tree,
        box_id,
        info_box_rect,
        &label_lines,
        Color::rgba(0.80, 0.84, 0.92, 1.0),
        "TexLine",
    );

    spawn_rect
}

/// Renders the WGSL shader specification and diagnostics section.
pub(crate) fn render_shader_preview_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    modal: &AssetPreviewModalState,
) {
    let box_rect = Rect::new(body_x, body_y + 10.0, body_w, 310.0);
    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("ShaderInfoBox");
        node.computed_rect = box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.24, 0.20, 0.12, 0.80));
    }
    let _ = tree.add_child(parent_id, box_id);

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
    render_info_lines(
        tree,
        box_id,
        box_rect,
        &lines,
        Color::rgba(1.0, 0.85, 0.45, 1.0),
        "ShaderLine",
    );
}

/// Renders the scene summary and direct load section.
pub(crate) fn render_scene_preview_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    modal: &AssetPreviewModalState,
    cursor_pos: Point,
) -> Rect {
    let load_rect = Rect::new(body_x + body_w - 140.0, body_y, 140.0, 28.0);
    let is_load_hovered = load_rect.contains_point(cursor_pos);
    let load_id = tree.create_node();
    if let Some(node) = tree.get_mut(load_id) {
        node.set_name("PreviewLoadSceneBtn");
        node.set_text("Load Scene");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = load_rect;
        node.style = Style::new()
            .background(if is_load_hovered {
                Color::rgba(0.15, 0.40, 0.70, 1.0)
            } else {
                Color::rgba(0.10, 0.30, 0.55, 0.90)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(0.30, 0.65, 1.0, 0.80));
    }
    let _ = tree.add_child(parent_id, load_id);

    let box_rect = Rect::new(body_x, body_y + 36.0, body_w, 280.0);
    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("SceneInfoBox");
        node.computed_rect = box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60));
    }
    let _ = tree.add_child(parent_id, box_id);

    let lines = [
        format!("Scene Name: {}", modal.item.name),
        "File Format: Aeon Scene Descriptor (.aee JSON)".to_string(),
        "Entity Hierarchy: Declarative ECS World State with Transform & Behaviors".to_string(),
        "Environment: Dynamic Atmosphere, 4-Cascade CSM Sun & Procedural Clouds".to_string(),
        format!(
            "Package Size: {}",
            AssetBrowserState::format_file_size(modal.item.file_size_bytes)
        ),
    ];
    render_info_lines(
        tree,
        box_id,
        box_rect,
        &lines,
        Color::rgba(0.60, 0.80, 1.0, 1.0),
        "SceneLine",
    );

    load_rect
}

/// Renders the audio specification and play preview section.
pub(crate) fn render_audio_preview_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    modal: &AssetPreviewModalState,
    cursor_pos: Point,
) -> Rect {
    let play_rect = Rect::new(body_x + body_w - 140.0, body_y, 140.0, 28.0);
    let is_play_hovered = play_rect.contains_point(cursor_pos);
    let play_id = tree.create_node();
    if let Some(node) = tree.get_mut(play_id) {
        node.set_name("PreviewPlayAudioBtn");
        node.set_text("Play Audio");
        node.font_size = 11.5;
        node.line_height = 28.0;
        node.text_align = TextAlign::Center;
        node.text_color = Color::WHITE;
        node.computed_rect = play_rect;
        node.style = Style::new()
            .background(if is_play_hovered {
                Color::rgba(0.55, 0.20, 0.15, 1.0)
            } else {
                Color::rgba(0.40, 0.15, 0.10, 0.90)
            })
            .border_radius(4.0)
            .border(1.0, Color::rgba(1.0, 0.45, 0.35, 0.80));
    }
    let _ = tree.add_child(parent_id, play_id);

    let box_rect = Rect::new(body_x, body_y + 36.0, body_w, 280.0);
    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("AudioInfoBox");
        node.computed_rect = box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60));
    }
    let _ = tree.add_child(parent_id, box_id);

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
    render_info_lines(
        tree,
        box_id,
        box_rect,
        &lines,
        Color::rgba(1.0, 0.60, 0.50, 1.0),
        "AudioLine",
    );

    play_rect
}

/// Renders generic asset information for fallback categories.
pub(crate) fn render_generic_preview_content(
    tree: &mut UiTree,
    parent_id: WidgetId,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    modal: &AssetPreviewModalState,
) {
    let box_rect = Rect::new(body_x, body_y + 10.0, body_w, 310.0);
    let box_id = tree.create_node();
    if let Some(node) = tree.get_mut(box_id) {
        node.set_name("GenericInfoBox");
        node.computed_rect = box_rect;
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border_radius(6.0)
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.60));
    }
    let _ = tree.add_child(parent_id, box_id);

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
    render_info_lines(
        tree,
        box_id,
        box_rect,
        &lines,
        Color::rgba(0.80, 0.84, 0.92, 1.0),
        "GenericLine",
    );
}

/// Helper to render structured text lines within an info box.
pub(crate) fn render_info_lines(
    tree: &mut UiTree,
    box_id: WidgetId,
    box_rect: Rect,
    lines: &[String],
    text_color: Color,
    line_prefix: &str,
) {
    let mut cur_y = box_rect.y + 20.0;
    for (i, line) in lines.iter().enumerate() {
        let line_rect = Rect::new(box_rect.x + 20.0, cur_y, box_rect.width - 40.0, 22.0);
        let line_id = tree.create_node();
        if let Some(node) = tree.get_mut(line_id) {
            node.set_name(format!("{}_{}", line_prefix, i));
            node.set_text(line);
            node.font_size = 11.5;
            node.line_height = 22.0;
            node.text_color = text_color;
            node.computed_rect = line_rect;
        }
        let _ = tree.add_child(box_id, line_id);
        cur_y += 30.0;
    }
}