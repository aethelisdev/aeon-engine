// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for `iris-text` typography and layout caching engine.

use super::*;
use iris_core::{Color, Rect, TextAlign};

#[test]
fn test_text_measurement() {
    let mut system = TextSystem::new();
    let size = system.measure_text("Hello Iris UI", 16.0, 20.0, None);

    assert!(size.width > 0.0);
    assert!(size.height >= 20.0);
}

#[test]
fn test_text_section_builder() {
    let section = TextSection::new("Button Label", Rect::new(0.0, 0.0, 100.0, 40.0))
        .with_font_size(14.0, 18.0)
        .with_color(Color::RED)
        .with_align(TextAlign::Center);

    assert_eq!(section.text, "Button Label");
    assert_eq!(section.font_size, 14.0);
    assert_eq!(section.color, Color::RED);
    assert_eq!(section.align, TextAlign::Center);
}

#[test]
fn test_text_system_measure_cache_hit() {
    let mut system = TextSystem::new();
    let size1 = system.measure_text("Cached String", 14.0, 18.0, None);
    let size2 = system.measure_text("Cached String", 14.0, 18.0, None);

    assert_eq!(size1.width, size2.width);
    assert_eq!(size1.height, size2.height);
}