// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use image::GenericImageView;
use winit::window::Icon;

/// Aeon Engine Icon Loader Module
/// This module provides cross-platform functionality to load and convert
/// image assets into winit-compatible Window Icons.
/// It supports various file formats (PNG, JPG, etc.) through the `image` crate.

/// Loads a window icon from the specified file path.
/// # Arguments
/// * `path` - A string slice that holds the relative or absolute path to the image file.
/// # Returns
/// * `Some(Icon)` - If the image was successfully loaded and converted to RGBA pixels.
/// * `None` - If the file was not found, couldn't be decoded, or conversion failed.
/// # Implementation Details
/// 1. Opens the image file.
/// 2. Converts it to RGBA8 format (required by winit).
/// 3. Extracts dimensions and raw pixel data.
/// 4. Creates a `winit::window::Icon`.
pub fn load_window_icon(path: &str) -> Option<Icon> {
    // 1. Open the image file using `image` crate
    let img = image::open(path).ok()?;

    // 2. Get dimensions
    let (width, height) = img.dimensions();

    // 3. Convert to RGBA8 (Standard for winit Icon)
    let rgba = img.to_rgba8().into_raw();

    // 4. Transform into winit::window::Icon
    Icon::from_rgba(rgba, width, height).ok()
}

/// Loads a window icon from raw image data (bytes).
/// This is preferred for cross-platform stability as it allows
/// embedding the icon in the binary using `include_bytes!`.
/// # Arguments
/// * `bytes` - The raw image data (e.g., contents of a .png file).
pub fn load_icon_from_memory(bytes: &[u8]) -> Option<Icon> {
    // 1. Load image from memory
    let img = image::load_from_memory(bytes).ok()?;

    // 2. Get dimensions
    let (width, height) = img.dimensions();

    // 3. Convert to RGBA8
    let rgba = img.to_rgba8().into_raw();

    // 4. Transform into winit::window::Icon
    Icon::from_rgba(rgba, width, height).ok()
}