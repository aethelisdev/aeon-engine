// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Disjoint mutable view into [`IrisEditorOverlay`] state for workbench panel builders.
//!
//! Splits engine-level retained resources (`tree`, `layout_engine`, `text_system`, `notifier`)
//! from chrome and panel states via field-level borrows, eliminating per-frame `std::mem::take`
//! heap/database churn on `TextSystem`, `UiTree`, and `LayoutEngine`.

use super::assets::AssetsPanelState;
use super::console::ConsolePanelState;
use super::hierarchy::HierarchyPanelState;
use super::inspector::InspectorPanelState;
use super::material::MaterialPanelState;
use super::preferences::PreferencesDialogState;
use super::stats::StatsPanelState;
use super::timeline::TimelinePanelState;
use super::types::{IrisChromeState, IrisEditorOverlay};
use super::ui_designer::UiDesignerPanelState;
use super::viewport_hud::ViewportHudState;
use crate::ui::EngineUiAction;
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;
use irisui::widgets::controls::{FocusState, FocusTarget};

/// Buffered panel redraw queue used inside workbench panel closures while `Workbench`
/// holds exclusive mutable access to the primary [`irisui::dock::UiNotifier`].
///
/// Requests recorded here are replayed onto [`IrisEditorOverlay::notifier`] immediately
/// after `Workbench::update_frame_live_with_registry` returns, incurring zero heap
/// allocations on frames where no panel emits an invalidation signal.
pub struct DeferredPanelNotifier<'a> {
    pending_panels: &'a mut Vec<PanelId>,
    pending_tag_all: &'a mut bool,
}

impl DeferredPanelNotifier<'_> {
    /// Queues a selective panel redraw for the specified `panel_id` after the current pass.
    #[inline]
    pub fn tag_redraw(&mut self, panel_id: PanelId) {
        if !self.pending_panels.contains(&panel_id) {
            self.pending_panels.push(panel_id);
        }
    }

    /// Queues a full shell invalidation across all workbench panels after the current pass.
    #[inline]
    pub fn tag_all(&mut self) {
        *self.pending_tag_all = true;
    }
}

/// Disjoint mutable borrow of panel, chrome, focus, and modal states inside [`IrisEditorOverlay`].
///
/// Passed to [`crate::ui::workbench::EditorPanelContext`] so workbench panels and chrome
/// callbacks mutate panel state concurrently with `&mut SubtreeEngines<'_>` without cloning
/// or taking ownership of `UiTree`, `LayoutEngine`, or `TextSystem`.
pub struct IrisPanelViews<'a> {
    /// Buffered notifier forwarding panel redraw requests back to the primary notifier.
    pub(crate) notifier: DeferredPanelNotifier<'a>,
    /// Window chrome, cursor cache, modifiers, and invalidation flags.
    pub(crate) chrome: &'a mut IrisChromeState,
    /// Preferences modal dialog interactive state.
    pub(crate) preferences: &'a mut PreferencesDialogState,
    /// Viewport HUD controls and camera overlay caches.
    pub(crate) viewport_hud: &'a mut ViewportHudState,
    /// Performance stats and telemetry panel state.
    pub(crate) stats: &'a mut StatsPanelState,
    /// Scene hierarchy tree panel state.
    pub(crate) hierarchy: &'a mut HierarchyPanelState,
    /// Developer console and log filter state.
    pub(crate) console: &'a mut ConsolePanelState,
    /// Content and asset browser panel state.
    pub(crate) assets: &'a mut AssetsPanelState,
    /// Animation timeline studio panel state.
    pub(crate) timeline: &'a mut TimelinePanelState,
    /// Material and surface studio panel state.
    pub(crate) material: &'a mut MaterialPanelState,
    /// Visual 2D UI designer panel state.
    pub(crate) ui_designer: &'a mut UiDesignerPanelState,
    /// Scene inspector component editor state.
    pub(crate) inspector: &'a mut InspectorPanelState,
    /// Core keyboard focus manager shared across overlays.
    pub(crate) focus_manager: &'a mut FocusManager,
    /// Autonomous focus state synchronized with text inputs.
    pub(crate) focus_state: &'a mut FocusState<FocusTarget>,
    /// Central queue of pending engine UI actions emitted by panels.
    pub(crate) pending_actions: &'a mut Vec<EngineUiAction>,
    /// Export project dialog state.
    pub(crate) export_modal: &'a mut crate::ui::types::ExportDialogState,
    /// Caret visibility sampled from the persistent [`irisui::text::TextSystem`] before splitting.
    pub(crate) blink_caret: bool,
    /// Logical cursor position sampled before splitting.
    pub(crate) cursor_pos: Point,
}

impl IrisEditorOverlay {
    /// Creates a disjoint [`IrisPanelViews`] bundle for active panel and chrome callbacks.
    ///
    /// Uses caller-sampled `blink_caret` and `cursor_pos` from `Workbench` so panel builders read
    /// accurate typography blink and pointer coordinates without borrowing `UiTree` or `TextSystem`.
    pub(crate) fn as_panel_views<'a>(
        &'a mut self,
        blink_caret: bool,
        cursor_pos: Point,
        deferred_panels: &'a mut Vec<PanelId>,
        deferred_tag_all: &'a mut bool,
    ) -> IrisPanelViews<'a> {
        IrisPanelViews {
            notifier: DeferredPanelNotifier {
                pending_panels: deferred_panels,
                pending_tag_all: deferred_tag_all,
            },
            chrome: &mut self.chrome,
            preferences: &mut self.preferences,
            viewport_hud: &mut self.viewport_hud,
            stats: &mut self.stats,
            hierarchy: &mut self.hierarchy,
            console: &mut self.console,
            assets: &mut self.assets,
            timeline: &mut self.timeline,
            material: &mut self.material,
            ui_designer: &mut self.ui_designer,
            inspector: &mut self.inspector,
            focus_manager: &mut self.focus_manager,
            focus_state: &mut self.focus_state,
            pending_actions: &mut self.pending_actions,
            export_modal: &mut self.export_modal,
            blink_caret,
            cursor_pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeferredPanelNotifier;
    use crate::ui::panel_layout::PanelId;

    #[test]
    fn deferred_notifier_deduplicates_panels_and_tracks_tag_all() {
        let mut panels = Vec::new();
        let mut tag_all = false;

        {
            let mut notifier = DeferredPanelNotifier {
                pending_panels: &mut panels,
                pending_tag_all: &mut tag_all,
            };
            notifier.tag_redraw(PanelId::Stats);
            notifier.tag_redraw(PanelId::Console);
            notifier.tag_redraw(PanelId::Stats);
            assert!(!*notifier.pending_tag_all);
            notifier.tag_all();
        }

        assert_eq!(panels, vec![PanelId::Stats, PanelId::Console]);
        assert!(tag_all);
    }
}