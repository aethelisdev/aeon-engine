// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Event routing logic for the top Menubar and floating dropdown menus.

use super::super::types::{IrisEditorOverlay, IrisOverlayEventResult};
use winit::event::WindowEvent;

impl IrisEditorOverlay {
    /// Handles cursor hovering and mouse clicks over the top Menubar and open dropdown popups.
    pub(crate) fn handle_menubar_event(
        &mut self,
        _event: &WindowEvent,
    ) -> Option<IrisOverlayEventResult> {
        None
    }
}