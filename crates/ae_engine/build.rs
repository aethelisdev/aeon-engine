// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// Build script for the Aeon Engine executable.
/// Embeds Windows resource files (such as the application icon and manifest)
/// into the executable when compiling for Windows targets.
fn main() {
    // Only embed the resource on Windows
    #[cfg(windows)]
    {
        // Compile the resource script into the binary (cosmetic icon resource)
        let _ = embed_resource::compile("assets/icon/icon.rc", embed_resource::NONE)
            .manifest_optional();
    }
}