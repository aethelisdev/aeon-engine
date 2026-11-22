// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for persistent scoped registries, live refresh scheduling, services, and host helpers.

use irisui_dock::{DockState, DockTree};

use super::Workbench;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::lifecycle::LifecycleBranch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FrameTab {
    Inspector,
    Viewport,
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
            )
            .expect("granular frame");
        assert!(matches!(
            out.lifecycle_branch,
            LifecycleBranch::GranularPanels(_)
        ));
    }
    assert_eq!(viewport_draws, 2);
}

#[test]
fn test_scoped_panel_live_refresh_scheduler_throttles_and_resets_on_rebuild() {
    use super::frame::{WorkbenchExternalContext, WorkbenchOwnedFrameParams};
    use crate::descriptors::{
        LiveRefreshInterval, ScopedPanelContext, ScopedPanelRegistry, ScopedWorkbenchPanel,
    };
    use irisui_core::{Color, Size, Style};
    use irisui_widgets::UiScope;

    const LIVE_LABEL_TAG: u64 = 0xAA44;
    struct LiveBorrow<'a> {
        every_frame_ticks: &'a mut usize,
        throttled_ticks: &'a mut usize,
        label_text: &'a str,
    }
    struct LiveScope;
    impl ScopedPanelContext for LiveScope {
        type Context<'frame> = LiveBorrow<'frame>;
    }

    struct EveryPanel;
    impl ScopedWorkbenchPanel<FrameTab, LiveScope> for EveryPanel {
        fn id(&self) -> FrameTab {
            FrameTab::Viewport
        }
        fn title(&self) -> &str {
            "Viewport"
        }
        fn live_refresh_interval(&self) -> LiveRefreshInterval {
            LiveRefreshInterval::EveryFrame
        }
        fn refresh_live<'frame>(
            &mut self,
            scope: &mut UiScope<'_>,
            state: &mut LiveBorrow<'frame>,
        ) {
            *state.every_frame_ticks += 1;
            scope.update_live_text(LIVE_LABEL_TAG, state.label_text);
        }
        fn render<'frame>(&mut self, scope: &mut UiScope<'_>, state: &mut LiveBorrow<'frame>) {
            scope.live_text_fmt::<32, _>(
                "ViewportLiveText",
                LIVE_LABEL_TAG,
                12.0,
                Color::WHITE,
                Style::new(),
                |buf| {
                    let _ = std::fmt::Write::write_str(buf, state.label_text);
                },
            );
        }
    }

    struct ThrottledPanel;
    impl ScopedWorkbenchPanel<FrameTab, LiveScope> for ThrottledPanel {
        fn id(&self) -> FrameTab {
            FrameTab::Inspector
        }
        fn title(&self) -> &str {
            "Inspector"
        }
        fn live_refresh_interval(&self) -> LiveRefreshInterval {
            LiveRefreshInterval::ThrottledMs(5_000)
        }
        fn refresh_live<'frame>(
            &mut self,
            _scope: &mut UiScope<'_>,
            state: &mut LiveBorrow<'frame>,
        ) {
            *state.throttled_ticks += 1;
        }
        fn render<'frame>(&mut self, scope: &mut UiScope<'_>, _state: &mut LiveBorrow<'frame>) {
            scope.label("Static");
        }
    }

    let mut dock_tree = DockTree::new();
    let left = dock_tree.create_leaf(vec![FrameTab::Viewport]);
    dock_tree.set_root(left);
    let _ = dock_tree.split_right(left, FrameTab::Inspector);
    let dock = DockState::new(dock_tree);
    let mut wb: Workbench<FrameTab, (), ()> =
        Workbench::new(WorkbenchDescriptor::new("SchedulerTest", dock));
    wb.set_screen_size(Size::new(1280.0, 720.0));

    let mut registry = ScopedPanelRegistry::<FrameTab, LiveScope>::new();
    registry.register(EveryPanel);
    registry.register(ThrottledPanel);

    let mut every_ticks = 0usize;
    let mut throttled_ticks = 0usize;

    let mut ctx0 = LiveBorrow {
        every_frame_ticks: &mut every_ticks,
        throttled_ticks: &mut throttled_ticks,
        label_text: "Init",
    };
    let out0 = wb
        .update_frame_with_registry(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut registry,
                context: &mut ctx0,
            },
            |i| i,
            |_s, _c, _sz| {},
        )
        .expect("frame 0");
    assert!(out0.is_full_shell_rebuild());
    assert_eq!((every_ticks, throttled_ticks), (0, 0));

    let mut ctx1 = LiveBorrow {
        every_frame_ticks: &mut every_ticks,
        throttled_ticks: &mut throttled_ticks,
        label_text: "Tick1",
    };
    let out1 = wb
        .update_frame_with_registry(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut registry,
                context: &mut ctx1,
            },
            |i| i,
            |_s, _c, _sz| {},
        )
        .expect("frame 1");
    assert_eq!(out1.lifecycle_branch, LifecycleBranch::InPlaceRefresh);
    assert_eq!((every_ticks, throttled_ticks), (1, 1));
    assert_eq!(
        wb.tree
            .node_tagged(LIVE_LABEL_TAG)
            .and_then(|n| n.text.as_deref()),
        Some("Tick1")
    );

    let mut ctx2 = LiveBorrow {
        every_frame_ticks: &mut every_ticks,
        throttled_ticks: &mut throttled_ticks,
        label_text: "Tick2",
    };
    let _ = wb
        .update_frame_with_registry(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut registry,
                context: &mut ctx2,
            },
            |i| i,
            |_s, _c, _sz| {},
        )
        .expect("frame 2");
    assert_eq!((every_ticks, throttled_ticks), (2, 1));

    wb.notifier.tag_redraw(FrameTab::Inspector);
    let mut ctx3 = LiveBorrow {
        every_frame_ticks: &mut every_ticks,
        throttled_ticks: &mut throttled_ticks,
        label_text: "Tick2",
    };
    let _ = wb
        .update_frame_with_registry(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut registry,
                context: &mut ctx3,
            },
            |i| i,
            |_s, _c, _sz| {},
        )
        .expect("frame 3 granular");
    let mut ctx4 = LiveBorrow {
        every_frame_ticks: &mut every_ticks,
        throttled_ticks: &mut throttled_ticks,
        label_text: "Tick3",
    };
    let _ = wb
        .update_frame_with_registry(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            WorkbenchExternalContext {
                registry: &mut registry,
                context: &mut ctx4,
            },
            |i| i,
            |_s, _c, _sz| {},
        )
        .expect("frame 4 post-reset");
    assert_eq!((every_ticks, throttled_ticks), (3, 2));
}

#[test]
fn test_update_frame_with_services_borrows_action_bus_and_dialog_host() {
    use super::frame::WorkbenchOwnedFrameParams;
    use super::frame_services::WorkbenchFrameServices;
    use crate::action_bus::ActionBus;
    use crate::descriptors::{ScopedPanelContext, ScopedPanelRegistry, ScopedWorkbenchPanel};
    use crate::dialogs::DialogHost;
    use irisui_core::Size;
    use irisui_widgets::UiScope;

    struct ServiceCtx<'a> {
        action_bus: &'a mut ActionBus<u32>,
        dialog_host: &'a mut DialogHost<u32>,
        trigger_dialog: bool,
    }

    struct ServiceScope;
    impl ScopedPanelContext for ServiceScope {
        type Context<'frame> = ServiceCtx<'frame>;
    }

    struct SelfOwnedPanel {
        renders: usize,
    }

    impl ScopedWorkbenchPanel<FrameTab, ServiceScope> for SelfOwnedPanel {
        fn id(&self) -> FrameTab {
            FrameTab::Viewport
        }

        fn title(&self) -> &str {
            "Viewport"
        }

        fn render<'frame>(&mut self, scope: &mut UiScope<'_>, ctx: &mut ServiceCtx<'frame>) {
            self.renders += 1;
            ctx.action_bus.emit(self.renders as u32);
            if ctx.trigger_dialog {
                ctx.dialog_host
                    .open_confirm_danger("Confirm", "Proceed?", 99);
            }
            scope.label("Viewport");
        }
    }

    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Viewport]);
    dock_tree.set_root(root);
    let mut wb: Workbench<FrameTab, u32, ()> = Workbench::new(WorkbenchDescriptor::new(
        "Services",
        DockState::new(dock_tree),
    ));
    wb.set_screen_size(Size::new(1024.0, 768.0));

    let mut registry = ScopedPanelRegistry::<FrameTab, ServiceScope>::new();
    registry.register(SelfOwnedPanel { renders: 0 });

    let has_live = wb.has_active_live_for(&registry);
    let out1 = wb
        .update_frame_with_services(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            has_live,
            |i| i,
            |runner, services: WorkbenchFrameServices<'_, u32, FrameTab>| {
                let mut ctx = ServiceCtx {
                    action_bus: services.action_bus,
                    dialog_host: services.dialog_host,
                    trigger_dialog: false,
                };
                runner.execute(&mut registry, &mut ctx, |_scope, ctx, _size| {
                    ctx.action_bus.emit(500);
                })
            },
        )
        .expect("services full rebuild");
    assert!(out1.is_full_shell_rebuild());
    assert_eq!(
        wb.action_bus.drain().collect::<Vec<_>>(),
        vec![1, 500],
        "Panel and chrome actions must land directly on wb.action_bus"
    );
    assert!(!wb.dialog_host.is_active());

    wb.notifier.tag_redraw(FrameTab::Viewport);
    let has_live2 = wb.has_active_live_for(&registry);
    let out2 = wb
        .update_frame_with_services(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            has_live2,
            |i| i,
            |runner, services| {
                let mut ctx = ServiceCtx {
                    action_bus: services.action_bus,
                    dialog_host: services.dialog_host,
                    trigger_dialog: true,
                };
                runner.execute(&mut registry, &mut ctx, |_scope, _ctx, _size| {})
            },
        )
        .expect("services granular rebuild");
    assert!(matches!(
        out2.lifecycle_branch,
        LifecycleBranch::GranularPanels(_)
    ));
    assert!(
        wb.dialog_host.is_active(),
        "Dialog staged during granular pass must persist on wb.dialog_host"
    );
    assert_eq!(
        registry
            .panel::<SelfOwnedPanel>(&FrameTab::Viewport)
            .expect("panel lookup")
            .renders,
        2
    );

    let out3 = wb
        .update_frame_with_services(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            false,
            |i| i,
            |runner, services| {
                services.notifier.tag_redraw(FrameTab::Viewport);
                let mut ctx = ServiceCtx {
                    action_bus: services.action_bus,
                    dialog_host: services.dialog_host,
                    trigger_dialog: false,
                };
                runner.execute(&mut registry, &mut ctx, |_scope, _ctx, _size| {})
            },
        )
        .expect("services deferred tag frame");
    assert!(out3.is_full_shell_rebuild());
    assert!(
        wb.notifier.is_dirty(FrameTab::Viewport),
        "Deferred notifier queued during frame must survive into next frame's dirty set"
    );

    let out4 = wb
        .update_frame_with_services(
            WorkbenchOwnedFrameParams {
                scale_factor: 1.0,
                selection_id: None,
                domain_revision: 0,
                is_cursor_occluded: false,
            },
            false,
            |i| i,
            |runner, services| {
                let mut ctx = ServiceCtx {
                    action_bus: services.action_bus,
                    dialog_host: services.dialog_host,
                    trigger_dialog: false,
                };
                runner.execute(&mut registry, &mut ctx, |_scope, _ctx, _size| {})
            },
        )
        .expect("services deferred granular wakeup");
    assert!(matches!(
        out4.lifecycle_branch,
        LifecycleBranch::GranularPanels(_)
    ));
}

#[test]
fn test_workbench_set_theme_and_dock_style_and_loading_progress_are_idempotent() {
    let mut dock_tree = DockTree::new();
    let root = dock_tree.create_leaf(vec![FrameTab::Viewport]);
    dock_tree.set_root(root);
    let mut wb: Workbench<FrameTab, (), ()> = Workbench::new(WorkbenchDescriptor::new(
        "HostHelpers",
        DockState::new(dock_tree),
    ));

    let custom_theme = irisui_core::Theme::obsidian();
    let custom_dock = irisui_dock::style::DockChromeStyle::default();
    let prev_rev = wb.theme_revision();
    wb.set_theme_and_dock_style(custom_theme, custom_dock);
    assert_eq!(wb.theme(), &custom_theme);
    assert_eq!(wb.tree.theme(), custom_theme);
    assert!(wb.theme_revision() > prev_rev);

    assert!(!wb.dialog_host.has_progress());
    wb.set_loading_progress(true, || {
        crate::dialogs::ProgressModalDescriptor::new("Loading")
    });
    assert!(wb.dialog_host.has_progress());
    let mut calls = 0usize;
    wb.set_loading_progress(true, || {
        calls += 1;
        crate::dialogs::ProgressModalDescriptor::new("ShouldNotRun")
    });
    assert_eq!(
        calls, 0,
        "Active progress modal must not re-run lazy builder"
    );
    wb.set_loading_progress(false, || {
        crate::dialogs::ProgressModalDescriptor::new("Unused")
    });
    assert!(!wb.dialog_host.has_progress());
}