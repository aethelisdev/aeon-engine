// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Workbench Theme and Screen Size Synchronization
//!
//! Verifies [`Workbench::set_theme`], [`Workbench::with_theme`], [`Workbench::set_dock_style`],
//! [`Workbench::set_screen_size`], [`Workbench::with_screen_size`], idempotence invariants,
//! non-finite/negative geometry clamping, and `UiTree` theme propagation during `update_shell`
//! and `render_shell`.

use super::runner::Workbench;
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::layout_sync::dual_layout::SubtreeEngines;
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};
use irisui_core::color::Color;
use irisui_core::geometry::Size;
use irisui_core::theme::Theme;
use irisui_core::tree::UiTree;
use irisui_dock::UiNotifier;
use irisui_dock::state::DockState;
use irisui_dock::style::DockChromeStyle;
use irisui_dock::tree::DockTree;
use irisui_layout::LayoutEngine;
use irisui_text::TextSystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ThemeTestTab {
    Scene,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ThemeTestAction {}

fn create_theme_test_workbench() -> Workbench<ThemeTestTab, ThemeTestAction, ()> {
    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![ThemeTestTab::Scene]);
    dock_tree.set_root(root);

    let descriptor = WorkbenchDescriptor::new("Theme Screen Test", DockState::new(dock_tree));
    let mut wb = Workbench::new(descriptor);
    let mut registry = WorkbenchPanelRegistry::<ThemeTestTab, ()>::new();
    registry.register_panel(ThemeTestTab::Scene, |_scope, _ctx| {});
    wb.panel_registry = registry;
    wb
}

fn custom_theme(accent: Color, panel_bg: Color) -> Theme {
    Theme {
        accent,
        panel_bg,
        ..Theme::default()
    }
}

#[test]
fn test_workbench_set_theme_and_dock_style_idempotence_and_revision() {
    let custom = custom_theme(Color::rgb(0.9, 0.3, 0.2), Color::rgb(0.12, 0.14, 0.18));
    let mut wb = create_theme_test_workbench().with_theme(custom);

    assert_eq!(wb.theme(), &custom);
    assert_eq!(wb.theme_revision(), 0);

    // Clear initial cold-start shell rebuild flag via one lifecycle evaluate
    let inputs = LifecycleInputs::new((1280.0, 720.0), 1.0, 1);
    assert_eq!(
        wb.lifecycle.evaluate(inputs),
        LifecycleBranch::FullShellRebuild
    );
    assert!(!wb.lifecycle.is_forced_shell_rebuild());

    // Calling set_theme with identical theme is a no-op
    wb.set_theme(custom);
    assert_eq!(wb.theme_revision(), 0);
    assert!(!wb.lifecycle.is_forced_shell_rebuild());

    // Switching to another custom theme increments revision and dirties shell
    let second = custom_theme(Color::rgb(0.1, 0.8, 0.5), Color::rgb(0.2, 0.22, 0.26));
    wb.set_theme(second);
    assert_eq!(wb.theme(), &second);
    assert_eq!(wb.theme_revision(), 1);
    assert!(wb.lifecycle.is_forced_shell_rebuild());

    // Consuming evaluation clears forced rebuild flag; dock_style switch dirties again
    assert_eq!(
        wb.lifecycle
            .evaluate(inputs.with_theme_revision(wb.theme_revision())),
        LifecycleBranch::FullShellRebuild
    );
    assert!(!wb.lifecycle.is_forced_shell_rebuild());

    let dock_style = DockChromeStyle {
        tab_active_line_color: Color::rgb(0.95, 0.5, 0.1),
        ..DockChromeStyle::default()
    };
    wb.set_dock_style(dock_style);
    assert_eq!(wb.dock_style(), &dock_style);
    assert_eq!(wb.theme_revision(), 2);
    assert!(wb.lifecycle.is_forced_shell_rebuild());

    // Re-applying identical dock_style is a no-op
    let _ = wb
        .lifecycle
        .evaluate(inputs.with_theme_revision(wb.theme_revision()));
    wb.set_dock_style(dock_style);
    assert_eq!(wb.theme_revision(), 2);
    assert!(!wb.lifecycle.is_forced_shell_rebuild());
}

#[test]
fn test_workbench_set_screen_size_variants_clamping_and_nan_guard() {
    let mut wb = create_theme_test_workbench().with_screen_size((1280.0, 720.0));
    assert_eq!(wb.screen_size(), Some(Size::new(1280.0, 720.0)));
    assert_eq!(wb.window_size(), Some(Size::new(1280.0, 720.0)));

    let inputs = LifecycleInputs::new((1280.0, 720.0), 1.0, 1);
    let _ = wb.lifecycle.evaluate(inputs);
    assert!(!wb.lifecycle.is_forced_shell_rebuild());

    // Identical size via Size struct is idempotent
    wb.set_screen_size(Size::new(1280.0, 720.0));
    assert!(!wb.lifecycle.is_forced_shell_rebuild());

    // Tuple and array inputs update stored size and dirty shell
    wb.set_screen_size([1600.0, 900.0]);
    assert_eq!(wb.screen_size(), Some(Size::new(1600.0, 900.0)));
    assert!(wb.lifecycle.is_forced_shell_rebuild());

    // Non-finite inputs must be ignored without corrupting valid dimensions
    wb.set_screen_size(Size::new(f32::NAN, 800.0));
    wb.set_screen_size((1920.0, f32::INFINITY));
    assert_eq!(wb.screen_size(), Some(Size::new(1600.0, 900.0)));

    // Negative finite dimensions clamp cleanly to zero
    wb.set_screen_size((-24.0, -10.0));
    assert_eq!(wb.screen_size(), Some(Size::ZERO));
}

#[test]
fn test_workbench_update_shell_propagates_theme_and_screen_size_to_tree() {
    let mut wb = create_theme_test_workbench();
    let mut tree = UiTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut text_system = TextSystem::new();
    let mut notifier = UiNotifier::<ThemeTestTab>::new();
    let mut context = ();

    let initial_theme = custom_theme(Color::rgb(0.0, 0.83, 0.7), Color::rgb(0.07, 0.08, 0.1));
    wb.set_theme(initial_theme);
    let inputs = LifecycleInputs::new((1440.0, 900.0), 1.0, 1);

    // Turn 1: initial build applies initial_theme and records rendered screen size
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = wb
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Initial update_shell must succeed");
        assert_eq!(branch, LifecycleBranch::FullShellRebuild);
    }
    assert_eq!(tree.theme(), initial_theme);
    assert_eq!(wb.screen_size(), Some(Size::new(1440.0, 900.0)));

    // Turn 2: steady state remains Dormant
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = wb
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Dormant update_shell must succeed");
        assert_eq!(branch, LifecycleBranch::Dormant);
    }

    // Turn 3: calling set_theme alone automatically forces shell rebuild and updates UiTree theme
    let updated_theme = custom_theme(Color::rgb(0.95, 0.6, 0.1), Color::rgb(0.18, 0.19, 0.22));
    wb.set_theme(updated_theme);
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = wb
            .update_shell(&mut engines, &mut notifier, &mut context, &inputs, false)
            .expect("Theme change update_shell must succeed");
        assert_eq!(branch, LifecycleBranch::FullShellRebuild);
    }
    assert_eq!(tree.theme(), updated_theme);

    // Turn 4: calling set_screen_size alone forces shell rebuild even before LifecycleInputs dims change
    wb.set_screen_size((1024.0, 768.0));
    let resized_inputs = LifecycleInputs::new((1024.0, 768.0), 1.0, 1);
    {
        let mut engines = SubtreeEngines::new(&mut tree, &mut layout_engine, &mut text_system);
        let branch = wb
            .update_shell(
                &mut engines,
                &mut notifier,
                &mut context,
                &resized_inputs,
                false,
            )
            .expect("Resize update_shell must succeed");
        assert_eq!(branch, LifecycleBranch::FullShellRebuild);
    }
    assert_eq!(wb.screen_size(), Some(Size::new(1024.0, 768.0)));
    assert_eq!(tree.theme(), updated_theme);
}