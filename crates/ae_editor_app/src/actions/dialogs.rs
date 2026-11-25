// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Native desktop OS file dialog dispatchers gated cleanly behind the `editor-dialogs` feature.

use std::sync::mpsc::Receiver;

/// Spawns a background thread to run `pick`, pushing its [`Receiver`] onto `queue`.
///
/// No-ops gracefully when the `editor-dialogs` feature is disabled or on Android targets.
pub fn spawn_file_dialog<T, F>(queue: &mut Vec<Receiver<T>>, pick: F)
where
    T: Send + 'static,
    F: FnOnce() -> Option<T> + Send + 'static,
{
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        queue.push(rx);
        std::thread::spawn(move || {
            if let Some(val) = pick() {
                let _ = tx.send(val);
            }
        });
    }
    #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
    let _ = (queue, pick);
}

/// Opens a synchronous OS file picker filtered to supported texture image formats.
#[must_use]
pub fn pick_texture_file_sync() -> Option<String> {
    #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
    {
        rfd::FileDialog::new()
            .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
            .pick_file()
            .map(|p| p.to_string_lossy().into_owned())
    }
    #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_file_dialog_queue_receives_empty_without_blocking() {
        let mut queue: Vec<Receiver<()>> = Vec::new();
        spawn_file_dialog(&mut queue, || None);

        #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
        assert_eq!(queue.len(), 1);
        #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
        assert!(queue.is_empty());
    }
}