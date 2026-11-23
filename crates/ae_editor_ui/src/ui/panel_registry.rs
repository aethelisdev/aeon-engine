// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Built-In Editor Panel Registration & WorkbenchHost Assembly
//!
//! Declares [`EditorPanelApp`] binding [`EditorFrameData`], [`EngineUiAction`], and [`PanelId`],
//! registers all 10 built-in editor panels onto [`ScopedPanelRegistry`], and builds the
//! configured [`WorkbenchHost<EditorPanelApp>`] with declarative shell chrome.

use crate::ui::icons::register_editor_icon_atlas;
use crate::ui::iris_bridge::assets::AssetsPanelState;
use crate::ui::iris_bridge::console::ConsolePanelState;
use crate::ui::iris_bridge::editor_dock_style_with_tokens;
use crate::ui::iris_bridge::export_modal::render_export_project_modal;
use crate::ui::iris_bridge::hierarchy::HierarchyPanelState;
use crate::ui::iris_bridge::inspector::InspectorPanelState;
use crate::ui::iris_bridge::material::MaterialPanelState;
use crate::ui::iris_bridge::menubar::{
    MENUBAR_HEIGHT, WINDOW_CONTROL_TAG_CLOSE, WINDOW_CONTROL_TAG_MAXIMIZE,
    WINDOW_CONTROL_TAG_MINIMIZE, build_top_menu_bar,
};
use crate::ui::iris_bridge::preferences::PreferencesDialogState;
use crate::ui::iris_bridge::stats::StatsPanelState;
use crate::ui::iris_bridge::status_bar::{STATUS_BAR_HEIGHT, render_workbench_status_bar};
use crate::ui::iris_bridge::theme::{EditorThemeTokens, ThemePreset};
use crate::ui::iris_bridge::timeline::TimelinePanelState;
use crate::ui::iris_bridge::ui_designer::UiDesignerPanelState;
use crate::ui::iris_bridge::viewport_hud::ViewportHudState;
use crate::ui::panel_layout::{PanelId, create_default_dock_state};
use crate::ui::types::{EditorFrameData, EngineUiAction};
use irisui::IrisSurfaceHost;
use irisui::prelude::{Size, UiScope};
use irisui::workbench::{
    LiveRefreshInterval, MenuBarDescriptor, ScopedPanelRegistry, StatusBarDescriptor, Workbench,
    WorkbenchAppData, WorkbenchAppScope, WorkbenchDescriptor, WorkbenchHost, WorkbenchPanelContext,
};

/// Application contract binding [`EditorFrameData`], [`EngineUiAction`], and [`PanelId`]
/// to Iris UI's generic [`WorkbenchHost`] and [`ScopedPanelRegistry`].
pub struct EditorPanelApp;

impl WorkbenchAppData for EditorPanelApp {
    type Data<'frame> = EditorFrameData<'frame>;
    type Action = EngineUiAction;
    type TabId = PanelId;
}

/// Persistent panel scope alias for [`ScopedPanelRegistry`] driven by [`EditorPanelApp`].
pub type EditorPanelScope = WorkbenchAppScope<EditorPanelApp>;

/// Standardized per-frame panel context supplied to all built-in editor panel and chrome renderers.
pub type EditorPanelContext<'frame> = WorkbenchPanelContext<'frame, EditorPanelApp>;

fn draw_editor_chrome<'frame>(
    scope: &mut UiScope<'_>,
    ctx: &mut EditorPanelContext<'frame>,
    size: Size,
) {
    build_top_menu_bar(
        scope,
        size.width,
        ctx.data.view_panel_mask,
        ctx.data.can_undo,
        ctx.data.can_redo,
        ctx.data.is_editing,
        ctx.action_bus,
    );
    render_export_project_modal(scope, ctx.data.export_modal, ctx.action_bus);
    render_workbench_status_bar(scope, size.width, size.height, ctx.status_spans);
}

/// Constructs and populates a persistent [`ScopedPanelRegistry`] with all 10 built-in panels.
#[must_use]
pub fn create_editor_panel_registry() -> ScopedPanelRegistry<PanelId, EditorPanelScope> {
    ScopedPanelRegistry::<PanelId, EditorPanelScope>::new()
        .with_live_stateful_panel(
            PanelId::Viewport.metadata(LiveRefreshInterval::EveryFrame),
            ViewportHudState::draw_panel,
            ViewportHudState::refresh_panel,
        )
        .with_stateful_panel(
            PanelId::Hierarchy.metadata(LiveRefreshInterval::OnChange),
            HierarchyPanelState::draw_panel,
        )
        .with_stateful_panel(
            PanelId::Inspector.metadata(LiveRefreshInterval::OnChange),
            InspectorPanelState::draw_panel,
        )
        .with_stateful_panel(
            PanelId::Console.metadata(LiveRefreshInterval::OnChange),
            ConsolePanelState::draw_panel,
        )
        .with_stateful_panel(
            PanelId::Assets.metadata(LiveRefreshInterval::OnChange),
            AssetsPanelState::draw_panel,
        )
        .with_live_stateful_panel(
            PanelId::Stats.metadata(LiveRefreshInterval::ThrottledMs(33)),
            StatsPanelState::draw_panel,
            StatsPanelState::refresh_panel,
        )
        .with_stateful_panel(
            PanelId::MaterialEditor.metadata(LiveRefreshInterval::OnChange),
            MaterialPanelState::draw_panel,
        )
        .with_live_stateful_panel(
            PanelId::AnimationTimeline.metadata(LiveRefreshInterval::EveryFrame),
            TimelinePanelState::draw_panel,
            TimelinePanelState::refresh_panel,
        )
        .with_stateful_panel(
            PanelId::UiDesigner.metadata(LiveRefreshInterval::OnChange),
            UiDesignerPanelState::draw_panel,
        )
        .with_stateful_panel(
            PanelId::Preferences.metadata(LiveRefreshInterval::OnChange),
            PreferencesDialogState::draw_panel,
        )
}

/// Constructs the configured [`WorkbenchHost<EditorPanelApp>`] for [`crate::ui::EngineUi`].
#[must_use]
pub(crate) fn build_editor_workbench_host(
    device: &wgpu::Device,
    output_color_format: wgpu::TextureFormat,
) -> WorkbenchHost<EditorPanelApp> {
    with_initial_theme(
        ThemePreset::default().tokens(),
        WorkbenchHost::new(
            Workbench::new(
                WorkbenchDescriptor::new("Aeon Engine Studio", create_default_dock_state())
                    .with_menubar(MenuBarDescriptor::new())
                    .with_statusbar(StatusBarDescriptor::new()),
            )
            .with_menubar_height(MENUBAR_HEIGHT)
            .with_statusbar_height(STATUS_BAR_HEIGHT)
            .with_non_ui_tab(PanelId::Viewport)
            .with_file_drop_action(|paths, pt| EngineUiAction::ImportDroppedFiles {
                paths,
                target: crate::ui::types::DroppedFileTarget::Workspace(pt.map(|p| [p.x, p.y])),
            })
            .with_zoom_action(EngineUiAction::SetUiScale)
            .with_preferences_panel(PanelId::Preferences)
            .with_window_control_tags(
                WINDOW_CONTROL_TAG_MINIMIZE.raw(),
                WINDOW_CONTROL_TAG_MAXIMIZE.raw(),
                WINDOW_CONTROL_TAG_CLOSE.raw(),
            ),
            create_editor_panel_registry(),
            IrisSurfaceHost::new(device, output_color_format),
        )
        .with_surface_init(|surface| register_editor_icon_atlas(surface, device))
        .with_chrome_builder(draw_editor_chrome),
    )
}

fn with_initial_theme(
    tokens: EditorThemeTokens,
    host: WorkbenchHost<EditorPanelApp>,
) -> WorkbenchHost<EditorPanelApp> {
    host.with_theme_and_dock_style(
        tokens.to_iris_theme(),
        editor_dock_style_with_tokens(&tokens),
    )
}