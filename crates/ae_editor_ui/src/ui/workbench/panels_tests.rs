// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Panel Registry Unit Tests
//!
//! Verifies panel registration, metadata descriptors, and tab viewer adapters.

use super::*;
use irisui::dock::TabViewer;

#[test]
fn test_editor_panel_registry_contains_all_panels_and_metadata() {
    let registry = create_editor_panel_registry();

    // 1. Verify all standard editor panels are registered
    for &panel_id in PanelId::all() {
        assert!(
            registry.is_registered(&panel_id),
            "Panel {panel_id:?} must be registered in workbench panel registry"
        );
        assert!(
            registry.panel_metadata(&panel_id).is_some(),
            "Metadata for panel {panel_id:?} must be present"
        );
    }

    // 2. Verify Viewport specific metadata invariants (persistent, transparent background)
    let vp_meta = registry
        .panel_metadata(&PanelId::Viewport)
        .expect("Viewport metadata");
    assert_eq!(vp_meta.title, "Viewport");
    assert_eq!(vp_meta.icon.as_deref(), Some("🎥"));
    assert!(!vp_meta.closeable, "Viewport must not be closeable");
    assert!(
        vp_meta.clear_background,
        "Viewport must have transparent background for WGPU scene pass"
    );
    assert!(!vp_meta.is_domain_bound);

    // 3. Verify Assets specific metadata (custom GPU texture atlas folder icon)
    let assets_meta = registry
        .panel_metadata(&PanelId::Assets)
        .expect("Assets metadata");
    assert_eq!(assets_meta.title, "Assets");
    assert!(
        assets_meta.atlas_icon.is_some(),
        "Assets must provide texture atlas UV coordinates"
    );
    assert!(assets_meta.closeable);

    // 4. Verify Domain-bound panels (subscribed to undo/redo/ECS transactions)
    let hierarchy_meta = registry
        .panel_metadata(&PanelId::Hierarchy)
        .expect("Hierarchy metadata");
    assert!(
        hierarchy_meta.is_domain_bound,
        "Hierarchy must be subscribed to domain mutations"
    );
    assert!(registry.is_domain_panel(&PanelId::Hierarchy));

    let inspector_meta = registry
        .panel_metadata(&PanelId::Inspector)
        .expect("Inspector metadata");
    assert!(
        inspector_meta.is_domain_bound,
        "Inspector must be subscribed to domain mutations"
    );
    assert!(registry.is_domain_panel(&PanelId::Inspector));

    // 5. Verify RegistryTabViewer integration
    let viewer = registry.tab_viewer();
    assert_eq!(viewer.title(&PanelId::Viewport), "🎥 Viewport");
    assert_eq!(viewer.raw_title(&PanelId::Viewport), "Viewport");
    assert_eq!(viewer.title(&PanelId::Hierarchy), "🏗️ Hierarchy");
    assert_eq!(viewer.title(&PanelId::Inspector), "⚙️ Inspector");
    assert_eq!(viewer.title(&PanelId::Console), "📜 Console");
    // Assets uses atlas icon so title has no emoji prefix
    assert_eq!(viewer.title(&PanelId::Assets), "Assets");
    assert_eq!(viewer.title(&PanelId::Stats), "📊 Stats");
    assert_eq!(viewer.title(&PanelId::MaterialEditor), "🌐 Material Editor");
    assert_eq!(viewer.title(&PanelId::AnimationTimeline), "🎬 Timeline");
    assert_eq!(viewer.title(&PanelId::UiDesigner), "📐 UI Designer");
    assert_eq!(viewer.title(&PanelId::Preferences), "⚙️ Preferences");

    assert!(!viewer.closeable(&PanelId::Viewport));
    assert!(viewer.closeable(&PanelId::Hierarchy));
    assert!(viewer.closeable(&PanelId::Preferences));
    assert!(viewer.clear_background(&PanelId::Viewport));
    assert!(!viewer.clear_background(&PanelId::Hierarchy));
    assert!(!viewer.clear_background(&PanelId::Preferences));
}