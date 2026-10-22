// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor panel subtree invalidation and partial layout bridge.
//!
//! Delegates isolated subtree clearing, intrinsic text measurement, and boundary layout
//! computation directly to the framework [`irisui::SubtreeSession`], retaining only editor
//! dispatch routing and change notification flags in the host overlay layer.
//!

use super::types::{IrisEditorOverlay, OverlayUpdateParams};
use crate::ui::panel_layout::PanelId;
use irisui::layout::LayoutError;
use irisui::prelude::{SubtreeSession, UiScope, WidgetId};

impl IrisEditorOverlay {
    /// Atomically rebuilds the content of a single docked or floating panel leaf in place.
    ///
    /// Preserves the rest of the UI tree, top menubar, status bar, and sibling panels untouched,
    /// delegating internal layout and arena lifecycle management to [`irisui::SubtreeSession`].
    ///
    /// # Lifecycle:
    /// 1. Begins an isolated subtree session on `leaf_id` via [`SubtreeSession::begin`],
    ///    untracking layout nodes and purging arena descendants.
    /// 2. Dispatches panel declarative widgets via [`Self::render_panel_by_id`].
    /// 3. Finalizes the session via [`SubtreeSession::finish`], measuring typography and solving boundary layout.
    /// 4. Clears the panel's dirty flag in the change notifier.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_panel_content(
        &mut self,
        panel_id: PanelId,
        leaf_id: WidgetId,
        params: &OverlayUpdateParams<'_>,
    ) -> Result<(), LayoutError> {
        let session = SubtreeSession::begin(&mut self.tree, &mut self.layout_engine, leaf_id);

        let mut tree = std::mem::take(&mut self.tree);
        {
            let mut scope = UiScope::new(&mut tree, leaf_id);
            self.render_panel_by_id(panel_id, &mut scope, params);
        }
        self.tree = tree;

        session.finish(
            &mut self.tree,
            &mut self.layout_engine,
            &mut self.text_system,
        )?;
        self.notifier.clear_panel(panel_id);

        Ok(())
    }
}