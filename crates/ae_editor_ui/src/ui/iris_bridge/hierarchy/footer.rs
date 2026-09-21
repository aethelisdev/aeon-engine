// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Footer Status Bar Builder
//!
//! Renders the bottom object count and selection state telemetry line.

use super::types::HierarchyPanelParams;
use irisui::prelude::*;

/// Builds the static Scene Hierarchy footer status line in the `UiTree`.
pub fn build_hierarchy_footer(
    tree: &mut UiTree,
    parent_id: WidgetId,
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

    let mut scope = UiScope::new(tree, parent_id);
    scope.separator();
    scope.text_colored(footer_text, Color::rgba(0.55, 0.58, 0.68, 1.0));
}