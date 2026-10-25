// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Subtree invalidation, partial layout recomputation, and leaf panel reconstruction.
//!
//! Provides granular, per-panel rebuilding pipelines for the retained Iris UI editor overlay,
//! eliminating full generational tree clears (`UiTree::clear()`) and global layout recalculations
//! when only an individual docking leaf panel's state or contents become dirty.
//!

use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
use irisui::core::WidgetId;
use irisui::layout::LayoutError;
use irisui::widgets::declarative::{SubtreeSession, UiScope};

impl IrisEditorOverlay {
    /// Rebuilds the declarative UI content of an individual active dock leaf panel in-place.
    ///
    /// Executes an isolated, generational subtree reconstruction cycle:
    /// 1. Untracks previous child layout nodes from Taffy and clears descendant nodes from [`irisui::core::UiTree`]
    ///    under `leaf_id` via [`SubtreeSession::begin`].
    /// 2. Declaratively renders fresh widgets under `leaf_id` using [`Self::render_panel_by_id`].
    /// 3. Shapes typography and recomputes boundary layout geometry exclusively within the subtree
    ///    in $O(M)$ time via [`SubtreeSession::finish`].
    /// 4. Marks the panel clean in [`irisui::core::UiNotifier`] and flags typography preparation for the active frame.
    ///
    /// This eliminates full tree reconstruction (`UiTree::clear()`) and global layout recalculation
    /// for isolated panel invalidation.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_panel_content(
        &mut self,
        panel_id: PanelId,
        leaf_id: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) -> Result<(), LayoutError> {
        let mut tree = std::mem::take(&mut self.tree);
        let session = SubtreeSession::begin(&mut tree, &mut self.layout_engine, leaf_id);
        {
            let mut scope = UiScope::new(&mut tree, leaf_id);
            self.render_panel_by_id(panel_id, &mut scope, params);
        }
        let finish_res = session.finish(&mut tree, &mut self.layout_engine, &mut self.text_system);
        self.tree = tree;

        finish_res?;

        self.notifier.clear_panel(panel_id);
        self.chrome.needs_text_prepare = true;
        Ok(())
    }
}