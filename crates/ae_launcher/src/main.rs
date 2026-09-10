// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae_launcher` binary entry point.
//!
//! Launches the lightweight Aeon Engine Hub for project management and mode selection.
//!

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::info!("Starting Aeon Engine Hub...");
    ae_launcher::app::LauncherApp::run()
}