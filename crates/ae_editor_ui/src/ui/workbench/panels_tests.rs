// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Panel Registry Unit Tests
//!
//! Verifies panel registration, metadata descriptors, and tab viewer adapters.

use super::*;
use crate::ui::PanelId;
use irisui::dock::TabViewer;

#[test]
fn test_editor_panel_registry_contains_all_panels_and_metadata() {
    let registry = create_editor_panel_registry();
    let viewer = registry.tab_viewer();

    let expected: &[(PanelId, &str, &str, bool, bool, bool)] = &[
        (
            PanelId::Viewport,
            "Viewport",
            "🎥 Viewport",
            false,
            true,
            false,
        ),
        (
            PanelId::Hierarchy,
            "Hierarchy",
            "🏗️ Hierarchy",
            true,
            false,
            true,
        ),
        (
            PanelId::Inspector,
            "Inspector",
            "⚙️ Inspector",
            true,
            false,
            true,
        ),
        (
            PanelId::Console,
            "Console",
            "📜 Console",
            true,
            false,
            false,
        ),
        (PanelId::Assets, "Assets", "Assets", true, false, false),
        (PanelId::Stats, "Stats", "📊 Stats", true, false, false),
        (
            PanelId::MaterialEditor,
            "Material Editor",
            "🌐 Material Editor",
            true,
            false,
            true,
        ),
        (
            PanelId::AnimationTimeline,
            "Timeline",
            "🎬 Timeline",
            true,
            false,
            true,
        ),
        (
            PanelId::UiDesigner,
            "UI Designer",
            "📐 UI Designer",
            true,
            false,
            false,
        ),
        (
            PanelId::Preferences,
            "Preferences",
            "⚙️ Preferences",
            true,
            false,
            false,
        ),
    ];

    assert_eq!(expected.len(), PanelId::all().len());
    for &(panel_id, raw_title, display_title, closeable, clear_bg, domain_bound) in expected {
        let meta = registry
            .panel_metadata(&panel_id)
            .expect("Panel metadata must be registered");
        assert!(meta.placement.is_some());
        assert_eq!(meta.title, raw_title);
        assert_eq!(meta.closeable, closeable);
        assert_eq!(meta.clear_background, clear_bg);
        assert_eq!(meta.is_domain_bound, domain_bound);
        assert_eq!(registry.is_domain_panel(&panel_id), domain_bound);
        assert_eq!(viewer.raw_title(&panel_id), raw_title);
        assert_eq!(viewer.title(&panel_id), display_title);
        assert_eq!(viewer.closeable(&panel_id), closeable);
        assert_eq!(viewer.clear_background(&panel_id), clear_bg);
    }

    let assets_meta = registry
        .panel_metadata(&PanelId::Assets)
        .expect("Assets metadata");
    assert!(assets_meta.atlas_icon.is_some());
}