// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Footer Status Bar Builder
//!
//! Renders the bottom object count and selection state telemetry line purely
//! using declarative [`UiScope`].
//!

use super::types::HierarchyPanelParams;
use irisui::prelude::*;

/// Builds the declarative Scene Hierarchy footer status line directly in the active [`UiScope`].
pub fn build_hierarchy_footer(
    scope: &mut UiScope<'_>,
    total_objects: usize,
    params: &HierarchyPanelParams<'_>,
) {
    let sel_str = if params.selected_entity.is_some() {
        " • 1 Selected"
    } else {
        ""
    };
    let footer_text = format!(
        "{} Object{}{}",
        total_objects,
        if total_objects == 1 { "" } else { "s" },
        sel_str
    );

    scope.separator();

    let footer_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .height(20.0)
        .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0));

    scope.container_named("HierarchyFooterBar", footer_style, |f| {
        f.label(
            footer_text,
            10.5,
            Color::rgba(0.55, 0.58, 0.68, 1.0),
            TextAlign::Left,
        );
    });
}