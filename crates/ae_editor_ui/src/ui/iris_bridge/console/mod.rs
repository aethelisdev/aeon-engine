// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Developer Console & Logger Iris UI Module
//!
//! Provides a hardware-accelerated, high-performance GPU SDF Developer Console
//! and logging telemetry viewer for the Aeon Engine Editor.
//!

pub mod events;
pub mod panel;
pub mod rows;
pub mod types;

pub use events::{handle_console_click, handle_console_scroll};
pub use panel::{CONSOLE_TOOLBAR_HEIGHT, build_console_panel};
pub use rows::{CONSOLE_ROW_HEIGHT, build_console_rows};
pub use types::{ConsoleAction, ConsoleFilterLevel, ConsolePanelParams, ConsolePanelTargets};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::types::ConsoleEntry;
    use irisui::prelude::{Point, Rect};

    #[test]
    fn test_console_filter_level_matching() {
        assert!(ConsoleFilterLevel::All.matches(log::Level::Error));
        assert!(ConsoleFilterLevel::All.matches(log::Level::Info));

        assert!(ConsoleFilterLevel::Error.matches(log::Level::Error));
        assert!(!ConsoleFilterLevel::Error.matches(log::Level::Warn));

        assert!(ConsoleFilterLevel::Warn.matches(log::Level::Warn));
        assert!(!ConsoleFilterLevel::Warn.matches(log::Level::Info));

        assert!(ConsoleFilterLevel::Info.matches(log::Level::Info));
        assert!(!ConsoleFilterLevel::Info.matches(log::Level::Debug));

        assert!(ConsoleFilterLevel::Debug.matches(log::Level::Debug));
        assert!(ConsoleFilterLevel::Debug.matches(log::Level::Trace));
        assert!(!ConsoleFilterLevel::Debug.matches(log::Level::Info));
    }

    #[test]
    fn test_console_click_target_routing() {
        let targets = ConsolePanelTargets {
            panel_rect: Rect::new(0.0, 0.0, 500.0, 300.0),
            clear_btn_rect: Rect::new(10.0, 5.0, 80.0, 24.0),
            filter_error_rect: Rect::new(160.0, 5.0, 60.0, 24.0),
            autoscroll_toggle_rect: Rect::new(400.0, 5.0, 90.0, 24.0),
            search_clear_btn_rect: Some(Rect::new(350.0, 5.0, 16.0, 16.0)),
            ..Default::default()
        };

        // Click outside panel
        assert_eq!(
            handle_console_click(&targets, Point::new(600.0, 10.0)),
            None
        );

        // Click Clear
        assert_eq!(
            handle_console_click(&targets, Point::new(20.0, 10.0)),
            Some(ConsoleAction::ClearLogs)
        );

        // Click Errors filter
        assert_eq!(
            handle_console_click(&targets, Point::new(170.0, 10.0)),
            Some(ConsoleAction::SetFilter(ConsoleFilterLevel::Error))
        );

        // Click AutoScroll toggle
        assert_eq!(
            handle_console_click(&targets, Point::new(420.0, 10.0)),
            Some(ConsoleAction::ToggleAutoScroll)
        );

        // Click Search Clear "✖"
        assert_eq!(
            handle_console_click(&targets, Point::new(355.0, 10.0)),
            Some(ConsoleAction::ClearSearch)
        );
    }

    #[test]
    fn test_console_scroll_clamping_and_autoscroll() {
        let targets = ConsolePanelTargets {
            panel_rect: Rect::new(0.0, 0.0, 500.0, 300.0),
            max_scroll_y: 200.0,
            ..Default::default()
        };

        let mut scroll_y = 200.0;
        let mut auto_scroll = true;

        // User scrolls up (delta > 0)
        let handled = handle_console_scroll(
            &targets,
            Point::new(100.0, 100.0),
            2.0, // scroll up 2 ticks = 48px
            &mut scroll_y,
            &mut auto_scroll,
        );

        assert!(handled);
        assert_eq!(scroll_y, 152.0);
        assert!(!auto_scroll); // auto-scroll disabled when scrolled up

        // User scrolls back down to bottom
        let handled = handle_console_scroll(
            &targets,
            Point::new(100.0, 100.0),
            -5.0, // scroll down 5 ticks = 120px
            &mut scroll_y,
            &mut auto_scroll,
        );

        assert!(handled);
        assert_eq!(scroll_y, 200.0); // clamped to max_scroll_y
        assert!(auto_scroll); // auto-scroll re-enabled at bottom
    }

    #[test]
    fn test_console_virtualized_empty_and_matching() {
        let mut tree = irisui::prelude::UiTree::new();
        let root = tree.create_root().unwrap();

        let entries = vec![
            ConsoleEntry {
                level: log::Level::Info,
                target: "ae_engine".to_string(),
                msg: "Engine started".to_string(),
                timestamp: "12:00:00".to_string(),
            },
            ConsoleEntry {
                level: log::Level::Error,
                target: "ae_audio".to_string(),
                msg: "Device disconnected".to_string(),
                timestamp: "12:00:01".to_string(),
            },
        ];

        let params = ConsolePanelParams {
            panel_rect: Rect::new(0.0, 0.0, 400.0, 200.0),
            entries: &entries,
            scroll_y: 0.0,
            filter: ConsoleFilterLevel::Error,
            search_query: "",
            is_search_focused: false,
            auto_scroll: true,
            cursor_pos: Point::new(50.0, 50.0),
            blink_caret: false,
        };

        let vp_rect = Rect::new(0.0, 34.0, 400.0, 166.0);
        let (content_h, max_scroll) = build_console_rows(&mut tree, root, &params, vp_rect);

        // Only 1 log matches Error filter
        assert_eq!(content_h, 166.0); // clamped to viewport height min
        assert_eq!(max_scroll, 0.0);
    }

    #[test]
    fn test_console_panel_clipping_and_clear_action() {
        let mut tree = irisui::prelude::UiTree::new();
        let parent_id = tree.create_root().unwrap();

        let entries = vec![];
        let params = ConsolePanelParams {
            panel_rect: Rect::new(0.0, 0.0, 500.0, 300.0),
            entries: &entries,
            scroll_y: 0.0,
            filter: ConsoleFilterLevel::All,
            search_query: "",
            is_search_focused: false,
            auto_scroll: true,
            cursor_pos: Point::new(15.0, 15.0),
            blink_caret: false,
        };

        let mut targets = ConsolePanelTargets::default();
        build_console_panel(&mut tree, parent_id, &params, &mut targets);

        // Find ConsolePanelRoot and ConsoleViewport and verify clip_children is true
        let mut found_root = false;
        let mut found_vp = false;

        tree.traverse_depth_first(parent_id, &mut |_id, node| {
            if node.name.as_deref() == Some("ConsolePanelRoot") {
                assert!(node.style.clip_children);
                found_root = true;
            } else if node.name.as_deref() == Some("ConsoleViewport") {
                assert!(node.style.clip_children);
                found_vp = true;
            }
        });

        assert!(found_root, "ConsolePanelRoot node must exist");
        assert!(found_vp, "ConsoleViewport node must exist");

        // Verify that clicking inside clear_btn_rect triggers ConsoleAction::ClearLogs
        let click_action = handle_console_click(
            &targets,
            Point::new(
                targets.clear_btn_rect.x + 2.0,
                targets.clear_btn_rect.y + 2.0,
            ),
        );
        assert_eq!(click_action, Some(ConsoleAction::ClearLogs));
    }
}