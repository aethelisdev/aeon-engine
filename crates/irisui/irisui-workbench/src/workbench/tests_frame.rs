// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for [`Workbench::update_frame`] and [`Workbench::update_frame_with`].

use irisui_core::Rect;
use irisui_dock::{DockState, DockTree, FloatingWindowClampBounds};

use super::Workbench;
use super::frame::WorkbenchFrameUpdateParams;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::hooks::{OverlayId, VolatileRefreshResult};
use crate::lifecycle::LifecycleBranch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FrameTab {
    Inspector,
    Viewport,
}

#[derive(Default)]
struct FrameAppState {
    panel_builds: u32,
    hook_calls: u32,
    callback_calls: u32,
}

#[test]
fn test_update_frame_with_orchestrates_all_branches_and_clamps_floating() {
    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Inspector, FrameTab::Viewport]);
    dock_tree.set_root(root);
    let mut dock = DockState::new(dock_tree);
    dock.open_or_focus_floating_tab(
        FrameTab::Viewport,
        "Viewport",
        Rect::new(2000.0, 2000.0, 240.0, 180.0),
    );

    let mut wb = Workbench::<FrameTab, (), FrameAppState>::new(WorkbenchDescriptor::new(
        "FrameTest",
        dock.clone(),
    ));
    wb.set_screen_size((800.0, 600.0));
    wb.panel_registry
        .register_domain_panel(FrameTab::Inspector, |scope, state| {
            state.panel_builds += 1;
            scope.label("Inspector");
        });
    wb.panel_registry
        .register_panel(FrameTab::Viewport, |scope, _| {
            scope.label("Viewport");
        });
    wb.register_volatile_fn(OverlayId::new(0xA001), |_tree, app| {
        app.hook_calls += 1;
        VolatileRefreshResult::quads_only()
    });

    let mut app = FrameAppState::default();

    // 1. Initial frame -> FullShellRebuild + floating window clamped into 800x600
    dock.clamp_floating_windows(&FloatingWindowClampBounds::new(800.0, 600.0));
    let out0 = wb
        .update_frame_with(
            &mut app,
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            |inputs| inputs,
            |_tree, s| {
                s.callback_calls += 1;
                VolatileRefreshResult::none()
            },
        )
        .expect("initial update_frame_with succeeds");

    assert_eq!(out0.lifecycle_branch, LifecycleBranch::FullShellRebuild);
    assert!(out0.needs_text_prepare);
    assert!(out0.needs_quad_reupload);
    assert!(wb.notifier.dirty_panels().is_empty());
    assert_eq!(app.callback_calls, 0);
    let clamped_rect = dock.floating_windows[0].rect;
    assert!(clamped_rect.x <= 800.0);
    assert!(clamped_rect.y <= 600.0);
    // Simulate GPU render pass consuming dirty boundaries from FullShellRebuild.
    wb.tree.clear_dirty_boundaries();

    // 2. Idle frame -> Dormant
    let out1 = wb
        .update_frame(
            &mut app,
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            |inputs| inputs,
        )
        .expect("dormant update_frame succeeds");

    assert_eq!(out1.lifecycle_branch, LifecycleBranch::Dormant);
    assert!(!out1.needs_quad_reupload);

    // 3. Volatile updates -> InPlaceRefresh runs both registered hook and callback
    let out2 = wb
        .update_frame_with(
            &mut app,
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            |inputs| inputs.with_volatile_updates(true),
            |_tree, s| {
                s.callback_calls += 1;
                VolatileRefreshResult::text_and_quads()
            },
        )
        .expect("in_place update_frame_with succeeds");

    assert_eq!(out2.lifecycle_branch, LifecycleBranch::InPlaceRefresh);
    assert_eq!(app.hook_calls, 1);
    assert_eq!(app.callback_calls, 1);
    assert!(out2.volatile_refresh.needs_text_prepare);
    assert!(out2.volatile_refresh.needs_quad_reupload);
    assert!(out2.needs_text_prepare);
    assert!(out2.needs_quad_reupload);

    // 4. Selection change -> GranularPanels rebuilds domain panel and clears notifier
    let prev_builds = app.panel_builds;
    let out3 = wb
        .update_frame(
            &mut app,
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: Some(77),
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            |inputs| inputs,
        )
        .expect("granular update_frame succeeds");

    assert!(out3.selection_changed);
    assert_eq!(
        out3.lifecycle_branch,
        LifecycleBranch::GranularPanels(vec![FrameTab::Inspector])
    );
    assert!(app.panel_builds > prev_builds);
    assert!(wb.notifier.dirty_panels().is_empty());
}

#[test]
fn test_update_frame_live_runs_scope_callback_without_exposing_tree() {
    use irisui_core::{Color, Size, Style};
    use irisui_widgets::LiveTextProps;

    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Inspector]);
    dock_tree.set_root(root);
    let dock = DockState::new(dock_tree);

    let mut wb = Workbench::<FrameTab, (), FrameAppState>::new(WorkbenchDescriptor::new(
        "LiveFrameTest",
        dock.clone(),
    ));
    wb.set_screen_size((800.0, 600.0));
    const LIVE_CANVAS_TAG: u64 = 0xE001;
    const LIVE_TEXT_TAG: u64 = 0xE002;
    wb.panel_registry
        .register_panel(FrameTab::Inspector, |scope, _| {
            scope.live_canvas_tagged("Scope", LIVE_CANVAS_TAG, Size::new(100.0, 30.0), |c| {
                c.fill_rect(c.bounds(), Color::BLACK, 0.0);
            });
            scope.live_text(LiveTextProps::new(
                "Hud",
                LIVE_TEXT_TAG,
                "X: 0.0",
                12.0,
                Color::WHITE,
                Style::default(),
            ));
        });

    let mut app = FrameAppState::default();

    // 1. Initial FullShellRebuild creates live nodes
    let out0 = wb
        .update_frame_live(
            &mut app,
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            |i| i,
            |_scope, _s| {},
        )
        .expect("initial update_frame_live succeeds");
    assert_eq!(out0.lifecycle_branch, LifecycleBranch::FullShellRebuild);
    wb.tree.clear_dirty_boundaries();

    // 2. Subsequent frame automatically triggers InPlaceRefresh because live nodes exist
    let out1 = wb
        .update_frame_live(
            &mut app,
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            |i| i,
            |scope, s| {
                s.callback_calls += 1;
                assert!(scope.update_live_canvas(LIVE_CANVAS_TAG, |c| {
                    c.fill_rect(c.bounds(), Color::GREEN, 0.0);
                }));
                assert!(scope.update_live_text(LIVE_TEXT_TAG, "X: 4.5"));
            },
        )
        .expect("live refresh frame succeeds");

    assert_eq!(out1.lifecycle_branch, LifecycleBranch::InPlaceRefresh);
    assert_eq!(app.callback_calls, 1);
    assert!(out1.needs_text_prepare);
    assert!(out1.needs_quad_reupload);
}

#[test]
fn test_update_frame_live_with_registry_orchestrates_external_context() {
    use super::frame::WorkbenchExternalContext;
    use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
    use irisui_core::{Color, Size, Style};
    use irisui_widgets::LiveTextProps;

    struct ExternalCtx<'a> {
        label_prefix: &'a str,
        panel_builds: u32,
        chrome_builds: u32,
        live_ticks: u32,
    }

    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Inspector]);
    dock_tree.set_root(root);
    let dock = DockState::new(dock_tree);

    let mut wb = Workbench::<FrameTab, (), ()>::new(WorkbenchDescriptor::new(
        "ExternalRegistryTest",
        dock.clone(),
    ));
    wb.set_screen_size((800.0, 600.0));
    wb.panel_registry
        .register_metadata(crate::descriptors::PanelMetadata::new(
            FrameTab::Inspector,
            "Inspector",
        ));

    const EXT_CANVAS: u64 = 0xF001;
    const EXT_TEXT: u64 = 0xF002;
    let mut ext_registry = WorkbenchPanelRegistry::<FrameTab, ExternalCtx<'_>>::new();
    ext_registry.register_domain_panel(FrameTab::Inspector, |scope, ctx| {
        ctx.panel_builds += 1;
        scope.live_canvas_tagged("ExtCanvas", EXT_CANVAS, Size::new(80.0, 20.0), |c| {
            c.fill_rect(c.bounds(), Color::BLACK, 0.0);
        });
        scope.live_text(LiveTextProps::new(
            "ExtHud",
            EXT_TEXT,
            ctx.label_prefix,
            12.0,
            Color::WHITE,
            Style::default(),
        ));
    });

    let mut ctx = ExternalCtx {
        label_prefix: "Init",
        panel_builds: 0,
        chrome_builds: 0,
        live_ticks: 0,
    };

    // 1. FullShellRebuild invokes on_shell_chrome and external registry drawer
    let out0 = wb
        .update_frame_live_with_registry(
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut ext_registry,
                context: &mut ctx,
            },
            |i| i,
            |_scope, c, _size| {
                c.chrome_builds += 1;
            },
            |_scope, _c| {},
        )
        .expect("initial update_frame_live_with_registry succeeds");
    assert_eq!(out0.lifecycle_branch, LifecycleBranch::FullShellRebuild);
    assert_eq!(ctx.chrome_builds, 1);
    assert_eq!(ctx.panel_builds, 1);
    wb.tree.clear_dirty_boundaries();

    // 2. InPlaceRefresh updates live canvas/text via UiScope without touching layout
    let out1 = wb
        .update_frame_live_with_registry(
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut ext_registry,
                context: &mut ctx,
            },
            |i| i,
            |_scope, _c, _size| {},
            |scope, c| {
                c.live_ticks += 1;
                assert!(scope.update_live_canvas(EXT_CANVAS, |canvas| {
                    canvas.fill_rect(canvas.bounds(), Color::RED, 0.0);
                }));
                assert!(scope.update_live_text(EXT_TEXT, "Updated"));
            },
        )
        .expect("in_place update_frame_live_with_registry succeeds");
    assert_eq!(out1.lifecycle_branch, LifecycleBranch::InPlaceRefresh);
    assert_eq!(ctx.live_ticks, 1);
    assert!(out1.needs_text_prepare);
    assert!(out1.needs_quad_reupload);
}

#[test]
fn test_empty_descriptor_chrome_does_not_occlude_external_menubar() {
    use super::frame::WorkbenchExternalContext;
    use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
    use irisui_core::{Color, Point};

    const PLAY_TAG: u64 = 0xDEAD_BEEF_9901;
    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Viewport, FrameTab::Inspector]);
    dock_tree.set_root(root);
    let dock = DockState::new(dock_tree);
    let desc = WorkbenchDescriptor::new("MenuTest", dock.clone())
        .with_menubar(crate::descriptors::MenuBarDescriptor::new())
        .with_statusbar(crate::descriptors::StatusBarDescriptor::new());
    let mut wb = Workbench::<FrameTab, (), ()>::new(desc)
        .with_menubar_height(36.0)
        .with_statusbar_height(22.0)
        .with_screen_size((1280.0, 720.0));
    wb.panel_registry
        .register_metadata(crate::descriptors::PanelMetadata::new(
            FrameTab::Inspector,
            "Inspector",
        ));

    let mut registry = WorkbenchPanelRegistry::<FrameTab, ()>::new();
    let mut ctx = ();

    wb.update_frame_live_with_registry(
        WorkbenchFrameUpdateParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
            is_cursor_occluded: false,
        },
        WorkbenchExternalContext {
            registry: &mut registry,
            context: &mut ctx,
        },
        |i| i,
        |scope, _c, size| {
            crate::descriptors::MenuBarDescriptor::new()
                .with_action_button("Play", Color::GREEN, Color::WHITE, PLAY_TAG)
                .render(scope, size.width);
            crate::descriptors::StatusBarDescriptor::new().render(scope, size.width, size.height);
        },
        |_scope, _c| {},
    )
    .expect("update_frame_live_with_registry succeeds");

    let play_rect = wb
        .tree
        .node_rect_tagged(PLAY_TAG, Point::ZERO)
        .expect("external menubar Play button must exist in UiTree");
    assert!(
        play_rect.y >= 0.0 && play_rect.y + play_rect.height <= 36.0,
        "Play button must be positioned inside the top 0..36px menubar, got {:?}",
        play_rect
    );

    let tab_strip_rect = wb
        .tree
        .node_rect("IrisDockTabStrip")
        .expect("IrisDockTabStrip must exist in UiTree");
    assert!(
        (tab_strip_rect.y - 36.0).abs() < 1.0,
        "Dock tab strip must start right below 36px menubar, got {:?}",
        tab_strip_rect
    );

    let close_rects: Vec<Rect> = wb
        .tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("IrisDockTabCloseBtn"))
        .map(|(_, n)| n.computed_rect)
        .collect();
    assert_eq!(close_rects.len(), 2, "Both tabs must have close buttons");
    assert!(
        close_rects[1].x > close_rects[0].x + 60.0 && close_rects[1].y >= 36.0,
        "Second tab close button must sit on the right side of the second tab, got {:?}",
        close_rects
    );

    let status_rect = wb
        .tree
        .node_rect_tagged(
            crate::descriptors::StatusBarDescriptor::STATUS_BAR_TAG,
            Point::ZERO,
        )
        .expect("StatusBar must exist in UiTree");
    assert!(
        (status_rect.y - (720.0 - 22.0)).abs() < 1.0,
        "StatusBar must be pinned at bottom 720-22=698, got {:?}",
        status_rect
    );
}

#[test]
fn test_right_dock_panel_popup_coordinates_and_dedup() {
    use super::frame::WorkbenchExternalContext;
    use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
    use irisui_core::{Color, Style};
    use irisui_dock::SplitDirection;

    const ANCHOR_TAG: u64 = 0xCAFE_BABE_0001;
    const POPUP_NAME: &str = "RightPanelTestPopup";

    let mut dock_tree = DockTree::new();
    let left_leaf = dock_tree.create_leaf(vec![FrameTab::Viewport]);
    dock_tree.set_root(left_leaf);
    let _ = dock_tree.split(
        left_leaf,
        SplitDirection::Horizontal,
        0.75,
        vec![FrameTab::Inspector],
    );
    let dock = DockState::new(dock_tree);

    let mut wb = Workbench::<FrameTab, (), ()>::new(WorkbenchDescriptor::new(
        "PopupCoordTest",
        dock.clone(),
    ))
    .with_menubar_height(36.0)
    .with_statusbar_height(22.0)
    .with_screen_size((1280.0, 720.0));

    wb.panel_registry
        .register_metadata(crate::descriptors::PanelMetadata::new(
            FrameTab::Viewport,
            "Viewport",
        ));
    wb.panel_registry
        .register_metadata(crate::descriptors::PanelMetadata::new(
            FrameTab::Inspector,
            "Inspector",
        ));

    let mut registry = WorkbenchPanelRegistry::<FrameTab, bool>::new();
    registry.register_domain_panel(FrameTab::Viewport, |scope, _| {
        scope.label("Viewport");
    });
    registry.register_domain_panel(FrameTab::Inspector, |scope, open| {
        let panel_rect = scope.parent_rect();
        scope
            .panel("InspectorPanelRoot")
            .bounds(panel_rect)
            .show(|p| {
                p.scroll_area("InspectorCardsContainer", 0xCAFE_1000, None, |cards| {
                    let mut idx = 0usize;
                    cards.prop_combobox("Type", &mut idx, &["Dynamic", "Static"]);
                });
                p.row_with().show(|r| {
                    r.label_styled_passive(
                        "Lbl",
                        "Type",
                        11.0,
                        Color::WHITE,
                        irisui_core::TextAlign::Left,
                        Style::new().width(100.0).height(22.0),
                    );
                    r.container_tagged_popup_toggle(
                        "Btn",
                        Style::new().width(90.0).height(22.0),
                        irisui_core::WidgetRole::Button,
                        ANCHOR_TAG,
                        ANCHOR_TAG,
                        |b, _| {
                            b.label("Static");
                        },
                    );
                });
                if *open {
                    p.scoped_popup_anchored_to_tag(
                        POPUP_NAME,
                        ANCHOR_TAG,
                        Rect::ZERO,
                        120.0,
                        irisui_widgets::declarative::PopupPlacement::Below,
                        |m| {
                            m.label("Dynamic");
                        },
                    );
                }
            });
    });

    let mut is_open = false;
    wb.update_frame_live_with_registry(
        WorkbenchFrameUpdateParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
            is_cursor_occluded: false,
        },
        WorkbenchExternalContext {
            registry: &mut registry,
            context: &mut is_open,
        },
        |i| i,
        |_s, _c, _sz| {},
        |_s, _c| {},
    )
    .expect("frame 0");

    let anchor_rect = wb
        .tree
        .resolve_anchor_rect(ANCHOR_TAG)
        .expect("anchor rect after frame 0");
    assert!(
        anchor_rect.x > 900.0,
        "Inspector anchor must be on the right panel (>900), got {:?}",
        anchor_rect
    );

    is_open = true;
    wb.tree.open_popup(ANCHOR_TAG);
    let combo_tag = wb
        .tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
        .map(|(_, n)| n.tag)
        .expect("ComboPill must exist");
    wb.tree.active_combobox_tag = Some(combo_tag);
    wb.notifier.tag_redraw(FrameTab::Inspector);
    wb.update_frame_live_with_registry(
        WorkbenchFrameUpdateParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
            is_cursor_occluded: false,
        },
        WorkbenchExternalContext {
            registry: &mut registry,
            context: &mut is_open,
        },
        |i| i.with_popup_active(true),
        |_s, _c, _sz| {},
        |_s, _c| {},
    )
    .expect("frame 1");

    let combo_popup_rect = wb
        .tree
        .node_rect("ComboBoxPopup")
        .expect("ComboBoxPopup must exist in tree");
    assert!(
        combo_popup_rect.x > 900.0,
        "ComboBoxPopup in right panel must open at x > 900, got {:?}",
        combo_popup_rect
    );

    let popup_rect = wb
        .tree
        .node_rect(POPUP_NAME)
        .expect("popup must exist in tree");
    assert!(
        popup_rect.x > 900.0,
        "Popup in right panel must open at x > 900, got {:?}",
        popup_rect
    );

    // Rebuild Inspector again (simulates hover/click inside panel): popup count must stay 1
    wb.notifier.tag_redraw(FrameTab::Inspector);
    wb.update_frame_live_with_registry(
        WorkbenchFrameUpdateParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
            is_cursor_occluded: false,
        },
        WorkbenchExternalContext {
            registry: &mut registry,
            context: &mut is_open,
        },
        |i| i,
        |_s, _c, _sz| {},
        |_s, _c| {},
    )
    .expect("frame 2");

    let popup_count = wb
        .tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some(POPUP_NAME))
        .count();
    assert_eq!(popup_count, 1, "Popup must not duplicate on panel rebuilds");
}

#[test]
fn test_update_frame_with_persistent_scoped_registry() {
    use super::frame::{WorkbenchExternalContext, WorkbenchOwnedFrameParams};
    use crate::descriptors::{ScopedPanelContext, ScopedPanelRegistry};
    use irisui_core::Size;

    struct FrameBorrow<'a> {
        title_text: &'a str,
        inspector_draws: &'a mut usize,
        viewport_draws: &'a mut usize,
    }

    struct PersistentScope;
    impl ScopedPanelContext for PersistentScope {
        type Context<'frame> = FrameBorrow<'frame>;
    }

    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Viewport, FrameTab::Inspector]);
    dock_tree.set_root(root);
    let dock = DockState::new(dock_tree);
    let mut wb: Workbench<FrameTab, (), ()> =
        Workbench::new(WorkbenchDescriptor::new("Test", dock));
    wb.set_screen_size(Size::new(1280.0, 720.0));

    let mut registry = ScopedPanelRegistry::<FrameTab, PersistentScope>::new();
    registry.register_panel(FrameTab::Viewport, |scope, ctx| {
        *ctx.viewport_draws += 1;
        scope.label(ctx.title_text);
    });
    registry.register_domain_panel(FrameTab::Inspector, |scope, ctx| {
        *ctx.inspector_draws += 1;
        scope.label(ctx.title_text);
    });
    wb.panel_registry
        .catalog_mut()
        .import_metadata_from(registry.catalog());
    registry.sync_domain_panels_to_notifier(&mut wb.notifier);

    let mut inspector_draws = 0usize;
    let mut viewport_draws = 0usize;

    {
        let label = String::from("Boot");
        let mut ctx = FrameBorrow {
            title_text: &label,
            inspector_draws: &mut inspector_draws,
            viewport_draws: &mut viewport_draws,
        };
        let out = wb
            .update_frame_with_registry(
                WorkbenchOwnedFrameParams {
                    scale_factor: 1.0,
                    selection_id: None,
                    domain_revision: 0,
                    is_cursor_occluded: false,
                },
                WorkbenchExternalContext {
                    registry: &mut registry,
                    context: &mut ctx,
                },
                |i| i,
                |_s, _c, _sz| {},
                |_s, _c| {},
            )
            .expect("cold shell frame");
        assert!(out.is_full_shell_rebuild());
    }
    assert_eq!(viewport_draws, 1);
    wb.tree.clear_dirty_boundaries();

    {
        let label = String::from("Dormant");
        let mut ctx = FrameBorrow {
            title_text: &label,
            inspector_draws: &mut inspector_draws,
            viewport_draws: &mut viewport_draws,
        };
        let out = wb
            .update_frame_with_registry(
                WorkbenchOwnedFrameParams {
                    scale_factor: 1.0,
                    selection_id: None,
                    domain_revision: 0,
                    is_cursor_occluded: false,
                },
                WorkbenchExternalContext {
                    registry: &mut registry,
                    context: &mut ctx,
                },
                |i| i,
                |_s, _c, _sz| {},
                |_s, _c| {},
            )
            .expect("dormant frame");
        assert_eq!(out.lifecycle_branch, LifecycleBranch::Dormant);
    }
    assert_eq!(viewport_draws, 1);

    wb.notifier.tag_redraw(FrameTab::Viewport);
    {
        let label = String::from("Granular");
        let mut ctx = FrameBorrow {
            title_text: &label,
            inspector_draws: &mut inspector_draws,
            viewport_draws: &mut viewport_draws,
        };
        let out = wb
            .update_frame_with_registry(
                WorkbenchOwnedFrameParams {
                    scale_factor: 1.0,
                    selection_id: None,
                    domain_revision: 0,
                    is_cursor_occluded: false,
                },
                WorkbenchExternalContext {
                    registry: &mut registry,
                    context: &mut ctx,
                },
                |i| i,
                |_s, _c, _sz| {},
                |_s, _c| {},
            )
            .expect("granular frame");
        assert!(matches!(
            out.lifecycle_branch,
            LifecycleBranch::GranularPanels(_)
        ));
    }
    assert_eq!(viewport_draws, 2);
}