// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone editor-free desktop executable entrypoint (`ae_runtime`).

use ae_runtime::{RuntimeLaunchArgs, run_desktop_player};

#[cfg(all(target_os = "linux", not(target_os = "android")))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = RuntimeLaunchArgs::from_env();
    if let Err(err) = run_desktop_player(args, None) {
        log::error!("Aeon Runtime exited with error: {err}");
        std::process::exit(1);
    }
}