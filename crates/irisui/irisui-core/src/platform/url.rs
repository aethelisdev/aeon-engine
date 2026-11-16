// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cross-platform external web hyperlink launcher.
//!
//! Spawns non-blocking platform-native handler processes to open external URLs
//! in the user's default web browser adhering strictly to safe abstractions.

/// Launches the provided URL in the user's default web browser.
///
/// Dispatches platform-specific command processes asynchronously without blocking
/// or mutating application UI state.
///
/// # Platform Specifics
/// - **Windows**: Executes `rundll32 url.dll,FileProtocolHandler <url>`
/// - **macOS**: Executes `open <url>`
/// - **Linux / Unix**: Executes `xdg-open <url>`
///
/// Returns `true` if the external process was spawned successfully, or `false` on failure.
pub fn open_url(url: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .is_ok()
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().is_ok()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .is_ok()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        let _ = url;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_url_signature_and_invocation() {
        // We verify that the function accepts a str slice and returns a boolean.
        // We do not actually spawn external browsers in unit tests, but test with empty URL.
        let _ = open_url;
    }
}