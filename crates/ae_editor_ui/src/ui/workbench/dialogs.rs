// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Declarative dialog presentation and lifecycle management for the editor workbench.

use crate::ui::EngineUi;
use irisui::core::Color;
use irisui::workbench::InfoCardDialogDescriptor;

impl EngineUi {
    /// Opens the informational 'About Aeon Engine' modal dialogue using Iris UI's autonomous [`DialogHost`].
    pub fn open_about_dialog(&mut self) {
        self.workbench.dialog_host.open_info(
            InfoCardDialogDescriptor::new("About Aeon Engine")
                .with_icon("ℹ")
                .with_app_name("Aeon Engine")
                .with_app_name_color(Color::rgba(0.0, 0.90, 1.0, 1.0))
                .with_copyright("Copyright (C) 2026 AethelisDEV / Aeon Engine")
                .with_description(
                    "This Source Code Form is subject to the terms of the Mozilla Public",
                )
                .with_link(
                    "License, v. 2.0 (MPL-2.0). https://mozilla.org/MPL/2.0/",
                    "https://mozilla.org/MPL/2.0/",
                )
                .with_footer_notice_colored(
                    "This program comes with ABSOLUTELY NO WARRANTY.",
                    Color::rgba(1.0, 0.40, 0.40, 1.0),
                )
                .with_close_label("Close")
                .with_size(480.0, 250.0),
        );
    }
}