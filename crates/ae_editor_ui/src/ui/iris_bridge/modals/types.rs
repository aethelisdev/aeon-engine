// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modal dialog and splash overlay state descriptors for Iris UI.

/// Persistent interactive state for modal dialogs and splash overlays.
///
/// Tracks cross-frame modal visibility flags to detect dialog popup transitions
/// and schedule appropriate layout/shell invalidations in the unified Iris UI tree.
#[derive(Debug, Default, Clone)]
pub struct ModalsOverlayState {
    /// Last recorded modal visibility state for detecting dialog popups and triggering redraws.
    pub last_modal_active: bool,
}