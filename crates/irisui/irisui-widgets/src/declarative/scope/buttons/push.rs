// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Standard Push Buttons
//!
//! Provides interactive standard push buttons and compact button variants on [`UiScope`].

use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;

impl<'a> UiScope<'a> {
    /// Emits an interactive push button returning immediate [`WidgetResponse`].
    ///
    /// Generates a deterministic persistent tag from `label` to support cross-frame
    /// interaction tracking, hover highlighting, and hit testing without manual identifier management.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    pub fn button(&mut self, label: impl Into<String>) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.add(crate::declarative::widget::Button::new(visible).tag(tag))
    }

    /// Emits a compact, small interactive push button (height: 20px, font: 9.5px).
    ///
    /// Designed for dense tables, inspector cards, and action rows.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    pub fn small_button(&mut self, label: impl Into<String>) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.add(
            crate::declarative::widget::Button::new(visible)
                .small()
                .tag(tag),
        )
    }
}