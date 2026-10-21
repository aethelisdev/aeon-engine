// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Panel Dirty Evaluation Subsystem
//!
//! Evaluates localized dirty states across all editor subsystems (Viewport, Material Editor,
//! Timeline, UI Designer, Inspector, Hierarchy, Console, Assets, Stats) and tags invalidated
//! panels in the central [`UiNotifier`].
//!

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;

impl IrisEditorOverlay {
    /// Evaluates localized dirty states from each specialized panel state structure
    /// and tags invalidated panels in [`UiNotifier`].
    ///
    /// When a panel reports state changes (e.g. selection updates, live text edits, filter toggles),
    /// its corresponding [`PanelId`] is marked dirty in the notifier so that only invalidated
    /// boundaries and layout trees are rebuilt during the frame update pass.
    pub(crate) fn poll_panel_dirty(&mut self, params: &OverlayUpdateParams<'_>) {
        if self.viewport_hud.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::Viewport);
        }
        if self.material.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::MaterialEditor);
        }
        if self.timeline.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::AnimationTimeline);
        }
        if self.ui_designer.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::UiDesigner);
        }
        if self.inspector.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::Inspector);
        }
        if self.hierarchy.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::Hierarchy);
        }
        if self.console.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::Console);
        }
        if self.assets.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::Assets);
        }
        if self.stats.check_and_sync_dirty(params) {
            self.notifier.tag_redraw(PanelId::Stats);
        }
    }
}