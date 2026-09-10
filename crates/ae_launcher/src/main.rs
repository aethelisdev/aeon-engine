// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae_launcher` binary entry point.
//!
//! Launches the lightweight Aeon Engine Hub for project management and mode selection.
//!

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let mut mode_arg: Option<String> = None;
    let mut project_arg: Option<std::path::PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" if i + 1 < args.len() => {
                mode_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--project" if i + 1 < args.len() => {
                project_arg = Some(std::path::PathBuf::from(&args[i + 1]));
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    if let Some(mode) = mode_arg {
        let project_path = project_arg.unwrap_or_else(|| {
            let registry = ae_launcher::project::ProjectRegistry::load_from_disk();
            if let Some(first) = registry.recent_projects.first() {
                std::path::PathBuf::from(&first.path)
            } else {
                std::path::PathBuf::from("/tmp/aeon_project")
            }
        });
        log::info!(
            "CLI mode override detected: mode={}, project={:?}. Spawning engine directly...",
            mode,
            project_path
        );
        return ae_launcher::spawner::launch_engine_and_exit(&project_path, &mode)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
    }

    log::info!("Starting Aeon Engine Hub...");
    ae_launcher::app::LauncherApp::run()
}