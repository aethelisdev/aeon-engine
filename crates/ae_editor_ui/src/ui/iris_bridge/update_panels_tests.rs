// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests verifying panel rendering dispatch, lifecycle synchronization, and FPS cadence.
//!

use irisui::prelude::{DockPanel, PanelRegistry, Rect, UiNotifier, UiTree, WidgetId};

#[test]
fn test_render_custom_panel_dispatch() {
    struct DynamicPluginPanel {
        rendered: bool,
    }

    impl DockPanel for DynamicPluginPanel {
        fn id(&self) -> &str {
            "dynamic_plugin"
        }
        fn title(&self) -> &str {
            "Plugin"
        }
        fn render(&mut self, _tree: &mut UiTree, _parent: WidgetId, _bounds: Rect) {
            self.rendered = true;
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    let mut registry = PanelRegistry::default();
    registry.register(DynamicPluginPanel { rendered: false });

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root widget");

    let panel = registry.get_downcast_mut::<DynamicPluginPanel>("dynamic_plugin");
    assert!(panel.is_some());
    let p = panel.unwrap();
    p.render(&mut tree, root, Rect::new(0.0, 0.0, 200.0, 200.0));
    assert!(p.rendered);
}

#[test]
fn test_stats_visual_sampling_cadence() {
    let mut last_fps_refresh = std::time::Instant::now();
    let mut stats_displayed_fps = 60.0;
    let mut stats_frame_counter: u32 = 0;

    // Simulate 30 frames within an interval before 250ms
    for _ in 0..30 {
        stats_frame_counter += 1;
    }

    // Before 250ms elapses, visual snapshot remains stable
    assert!(last_fps_refresh.elapsed().as_secs_f32() < 0.25);
    assert_eq!(stats_displayed_fps, 60.0);

    // Advance visual sampling manually past 250ms (e.g. 300ms) to verify threshold invariant
    let fake_past = std::time::Instant::now()
        .checked_sub(std::time::Duration::from_millis(300))
        .unwrap_or(last_fps_refresh);
    last_fps_refresh = fake_past;

    let elapsed = std::time::Instant::now()
        .duration_since(last_fps_refresh)
        .as_secs_f32();
    assert!(elapsed >= 0.25);

    if elapsed >= 0.25 {
        stats_displayed_fps = stats_frame_counter as f32 / elapsed;
        stats_frame_counter = 0;
    }
    // 30 frames over ~0.30s = ~100 FPS
    assert!(stats_displayed_fps > 90.0 && stats_displayed_fps < 110.0);
    assert_eq!(stats_frame_counter, 0);
}

#[test]
fn test_ui_notifier_panel_tag_redraw() {
    use crate::ui::panel_layout::PanelId;

    let mut notifier: UiNotifier<PanelId> = UiNotifier::clean();
    assert!(!notifier.is_any_dirty());

    notifier.tag_redraw(PanelId::Console);
    assert!(notifier.is_any_dirty());
    assert!(notifier.is_dirty(PanelId::Console));
    assert!(!notifier.is_dirty(PanelId::Hierarchy));

    notifier.clear_all();
    assert!(!notifier.is_any_dirty());
}