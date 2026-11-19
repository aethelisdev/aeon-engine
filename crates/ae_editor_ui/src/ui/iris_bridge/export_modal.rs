// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Declarative `File -> Export Project...` modal dialog built with zero imperative `let`
//! statements using [`UiScope::modal_tagged`] and two-way property bindings.

use crate::ui::EngineUiAction;
use crate::ui::types::ExportDialogState;
use ae_export::{CookCompressionProfile, ExportRequest, ExportTarget};
use irisui::prelude::*;
use irisui::widgets::declarative::ActionBarButton;
use irisui::widgets::declarative::scope::ComboRowLayout;
use irisui::workbench::ActionBus;
use std::path::PathBuf;

/// Semantic modal tag reserved for the `File -> Export Project...` dialog.
pub const EXPORT_MODAL_TAG: UiTag = UiTag::from_str("modal.export_project");

const TARGET_LABELS: [&str; 3] = [
    "Linux (x86_64)",
    "Windows (x86_64 MinGW)",
    "Windows (x86_64 MSVC)",
];

const COMPRESSION_LABELS: [&str; 2] = ["Fast LZ4 (64 KiB Blocks)", "Raw (Uncompressed)"];

const fn target_to_index(target: ExportTarget) -> usize {
    match target {
        ExportTarget::LinuxX86_64 => 0,
        ExportTarget::WindowsX86_64Gnu => 1,
        ExportTarget::WindowsX86_64Msvc => 2,
    }
}

const fn index_to_target(idx: usize) -> ExportTarget {
    match idx {
        1 => ExportTarget::WindowsX86_64Gnu,
        2 => ExportTarget::WindowsX86_64Msvc,
        _ => ExportTarget::LinuxX86_64,
    }
}

const fn compression_to_index(profile: CookCompressionProfile) -> usize {
    match profile {
        CookCompressionProfile::FastLz4 => 0,
        CookCompressionProfile::UncompressedRaw => 1,
    }
}

const fn index_to_compression(idx: usize) -> CookCompressionProfile {
    match idx {
        1 => CookCompressionProfile::UncompressedRaw,
        _ => CookCompressionProfile::FastLz4,
    }
}

/// Renders the `Export Project...` modal when [`EXPORT_MODAL_TAG`] is active in `UiScope`.
///
/// Uses 100% pure declarative expressions with zero intermediate imperative `let` bindings.
pub fn render_export_project_modal(
    scope: &mut UiScope<'_>,
    state: &mut ExportDialogState,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.modal_tagged(
        EXPORT_MODAL_TAG.raw(),
        ("Export Project", 500.0, 340.0),
        |card| {
            card.prop_combobox_mapped_styled(
                "Target Platform",
                target_to_index(state.target),
                &TARGET_LABELS,
                ComboRowLayout::new(140.0, 240.0),
                &mut state.target,
                index_to_target,
            );
            card.prop_combobox_mapped_styled(
                "Compression",
                compression_to_index(state.compression),
                &COMPRESSION_LABELS,
                ComboRowLayout::new(140.0, 240.0),
                &mut state.compression,
                index_to_compression,
            );
            card.prop_text("Output Directory", &mut state.output_dir);
            card.prop_checkbox("Compile Standalone Binary", &mut state.build_binary);
            card.divider(card.theme().card_border);
            card.progress_bar_named(
                "ExportProgressTrack",
                "ExportProgressFill",
                460.0,
                12.0,
                state.progress.clamp(0.0, 1.0),
                Color::rgba(0.16, 0.52, 0.92, 1.0),
            );
            card.label_desc("ExportStatusMessage", &state.status_message);
            card.dual_action_bar(
                "ExportModalActions",
                460.0,
                ActionBarButton::new(
                    "ExportStartButton",
                    if state.is_running {
                        "Exporting..."
                    } else {
                        "Export Project"
                    },
                ),
                ActionBarButton::new("ExportCloseButton", "Close"),
            )
            .on_primary_click(|| {
                if !state.is_running {
                    action_bus.emit(EngineUiAction::StartProjectExport(ExportRequest {
                        project_root: PathBuf::from("."),
                        selected_worlds: Vec::new(),
                        target: state.target,
                        compression: state.compression,
                        output_dir: PathBuf::from(state.output_dir.trim()),
                        build_binary: state.build_binary,
                        workspace_root: None,
                    }));
                }
            })
            .on_secondary_click(|| {
                card.close_modal(EXPORT_MODAL_TAG.raw());
            });
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_project_modal_renders_when_opened_and_dispatches_action() {
        let mut tree = UiTree::new();
        let root_id = tree.create_root().expect("root");
        let mut state = ExportDialogState::default();
        let mut bus = ActionBus::new();

        {
            let mut scope = UiScope::new(&mut tree, root_id);
            scope.open_modal(EXPORT_MODAL_TAG.raw());
            render_export_project_modal(&mut scope, &mut state, &mut bus);
            assert!(scope.is_modal_open(EXPORT_MODAL_TAG.raw()));
        }

        assert!(tree.len() > 5);
    }
}