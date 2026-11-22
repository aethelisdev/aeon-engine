// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Footer Status Bar Builder
//!
//! Renders the bottom object count and selection state telemetry line using
//! stack-formatted zero-allocation [`UiScope::text_colored_fmt`].

use irisui::prelude::*;
use std::fmt::Write as _;

/// Builds the declarative Scene Hierarchy footer status line directly in the active [`UiScope`].
pub fn build_hierarchy_footer(total_objects: usize, has_selection: bool, scope: &mut UiScope<'_>) {
    scope.separator();
    scope.container_named(
        "HierarchyFooterBar",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(20.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0)),
        |f| {
            f.text_colored_fmt::<64, _>(Color::rgba(0.55, 0.58, 0.68, 1.0), |buf| {
                _ = write!(
                    buf,
                    "{} Object{}{}",
                    total_objects,
                    if total_objects == 1 { "" } else { "s" },
                    if has_selection { " • 1 Selected" } else { "" }
                );
            });
        },
    );
}