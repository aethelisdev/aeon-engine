// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Periodic non-blocking editor Auto-Save manager persisting rotating binary recovery
//! snapshots inside `Saved/Autosaves/` without mutating authored `Worlds/` directories.

use ae_core::modules::EngineMode;
use ae_world::{ProjectWorkspaceLayout, WorldDocument, compute_crc32};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Default interval between periodic background editor autosaves in seconds (`5` minutes).
pub const DEFAULT_AUTOSAVE_INTERVAL_SECS: f32 = 300.0;

/// Number of rotating autosave slots maintained inside `Saved/Autosaves/`.
pub const AUTOSAVE_ROTATING_SLOTS: usize = 3;

/// Tracks edit-mode frame elapsed time and dispatches non-blocking background autosaves
/// to rotating `Saved/Autosaves/<world>_autosave_<slot>.aeworld` recovery archives.
///
/// Identical worlds (matching full binary CRC32) skip disk writes to eliminate redundant I/O.
pub struct EditorAutosaveState {
    /// Whether periodic background autosaves are enabled.
    pub enabled: bool,
    /// Seconds of Edit mode accumulation between autosave checks.
    pub interval_secs: f32,
    elapsed_secs: f32,
    next_slot: usize,
    last_crc32: Arc<AtomicU32>,
    in_flight: Arc<AtomicBool>,
}

impl Default for EditorAutosaveState {
    fn default() -> Self {
        Self::new(DEFAULT_AUTOSAVE_INTERVAL_SECS)
    }
}

impl EditorAutosaveState {
    /// Creates an autosave tracker triggering every `interval_secs` seconds in Edit mode.
    #[must_use]
    pub fn new(interval_secs: f32) -> Self {
        Self {
            enabled: true,
            interval_secs: interval_secs.max(10.0),
            elapsed_secs: 0.0,
            next_slot: 0,
            last_crc32: Arc::new(AtomicU32::new(0)),
            in_flight: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Resets the accumulated countdown when a scene is opened or explicitly saved.
    pub fn reset_timer(&mut self) {
        self.elapsed_secs = 0.0;
    }

    /// Advances the Edit-mode countdown and returns the rotating target `.aeworld` path
    /// when a new autosave snapshot should be captured and flushed in the background.
    ///
    /// Returns `None` during Play mode, when disabled, when an empty world is open, when a prior
    /// worker is still writing, or before `interval_secs` has elapsed.
    pub fn poll_trigger(
        &mut self,
        mode: EngineMode,
        dt: f32,
        entity_count: u32,
        autosaves_root: Option<&Path>,
        saved_world_dir: Option<&Path>,
        active_scene_path: Option<&Path>,
    ) -> Option<PathBuf> {
        if !self.enabled || mode != EngineMode::Edit || entity_count == 0 {
            return None;
        }
        if !dt.is_finite() || dt <= 0.0 {
            return None;
        }
        self.elapsed_secs += dt;
        if self.elapsed_secs < self.interval_secs {
            return None;
        }
        if self.in_flight.swap(true, Ordering::AcqRel) {
            return None;
        }

        self.elapsed_secs = 0.0;
        let slot = self.next_slot % AUTOSAVE_ROTATING_SLOTS;
        self.next_slot = (slot + 1) % AUTOSAVE_ROTATING_SLOTS;
        Some(resolve_autosave_target(
            autosaves_root,
            saved_world_dir,
            active_scene_path,
            slot,
        ))
    }

    /// Spawns a background `rayon` worker to compress and atomically write `doc` to `target_path`
    /// only if its encoded CRC32 differs from the previous autosave checksum.
    pub fn dispatch_background_write(&self, doc: WorldDocument, target_path: PathBuf) {
        let last_crc = Arc::clone(&self.last_crc32);
        let in_flight = Arc::clone(&self.in_flight);
        rayon::spawn(move || {
            let result = write_autosave_if_changed(&doc, &target_path, &last_crc);
            in_flight.store(false, Ordering::Release);
            match result {
                Ok(true) => log::info!("Auto-saved recovery snapshot to {:?}", target_path),
                Ok(false) => log::debug!("Skipped auto-save: world snapshot unchanged"),
                Err(err) => {
                    log::warn!("Background auto-save failed for {:?}: {}", target_path, err)
                }
            }
        });
    }
}

/// Computes the deterministic rotating autosave file path inside `autosaves_root`.
#[must_use]
pub fn resolve_autosave_target(
    autosaves_root: Option<&Path>,
    saved_world_dir: Option<&Path>,
    active_scene_path: Option<&Path>,
    slot: usize,
) -> PathBuf {
    let world_stem = saved_world_dir
        .and_then(|d| d.file_name())
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            active_scene_path
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
                .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("world"))
        })
        .unwrap_or("MainWorld");

    let autosave_dir = match autosaves_root {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => ProjectWorkspaceLayout::new(".").autosaves_dir(),
    };
    autosave_dir.join(format!("{world_stem}_autosave_{slot}.aeworld"))
}

/// Encodes `doc` and atomically writes to `target_path` if its CRC32 changed since `last_crc`.
///
/// Returns `Ok(true)` if written, `Ok(false)` if skipped as identical.
///
/// # Errors
/// Returns [`std::io::Error`] if binary serialization or atomic file replacement fails.
pub fn write_autosave_if_changed(
    doc: &WorldDocument,
    target_path: &Path,
    last_crc: &AtomicU32,
) -> std::io::Result<bool> {
    let bytes = doc.encode_full().map_err(std::io::Error::other)?;
    let crc = compute_crc32(&bytes);
    if crc != 0 && last_crc.load(Ordering::Acquire) == crc && target_path.exists() {
        return Ok(false);
    }

    if let Some(parent) = target_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = target_path.with_extension("aeworld.tmp");
    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp_path)?;
        file.write_all(&bytes)?;
        file.flush()?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp_path, target_path)?;
    last_crc.store(crc, Ordering::Release);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::Position;
    use ae_world::{ComponentRecord, EntityRecord, GuidGenerator};

    #[test]
    fn test_autosave_skips_play_mode_rotates_slots_and_skips_unchanged_crc() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("ae_engine_autosave_{nanos}"));
        let autosaves_dir = root.join("Saved/Autosaves");
        let _ = std::fs::remove_dir_all(&root);

        let mut state = EditorAutosaveState::new(30.0);
        // Never triggers in Play mode or on empty worlds
        assert!(
            state
                .poll_trigger(EngineMode::Play, 60.0, 10, Some(&autosaves_dir), None, None)
                .is_none()
        );
        assert!(
            state
                .poll_trigger(EngineMode::Edit, 60.0, 0, Some(&autosaves_dir), None, None)
                .is_none()
        );
        // Before interval expires -> None
        assert!(
            state
                .poll_trigger(EngineMode::Edit, 10.0, 5, Some(&autosaves_dir), None, None)
                .is_none()
        );

        let world_dir = root.join("Worlds/Frontier");
        let slot0 = state
            .poll_trigger(
                EngineMode::Edit,
                25.0,
                5,
                Some(&autosaves_dir),
                Some(&world_dir),
                None,
            )
            .expect("slot 0 triggers");
        assert!(slot0.ends_with("Frontier_autosave_0.aeworld"));
        // Simulate worker finishing so slot 1 can trigger next
        state.in_flight.store(false, Ordering::Release);

        let slot1 = state
            .poll_trigger(
                EngineMode::Edit,
                30.0,
                5,
                Some(&autosaves_dir),
                Some(&world_dir),
                None,
            )
            .expect("slot 1 triggers");
        assert!(slot1.ends_with("Frontier_autosave_1.aeworld"));
        state.in_flight.store(false, Ordering::Release);

        let slot2 = state
            .poll_trigger(
                EngineMode::Edit,
                30.0,
                5,
                Some(&autosaves_dir),
                Some(&world_dir),
                None,
            )
            .expect("slot 2 triggers");
        assert!(slot2.ends_with("Frontier_autosave_2.aeworld"));
        state.in_flight.store(false, Ordering::Release);

        let wrap0 = state
            .poll_trigger(
                EngineMode::Edit,
                30.0,
                5,
                Some(&autosaves_dir),
                Some(&world_dir),
                None,
            )
            .expect("slot wraps to 0");
        assert!(wrap0.ends_with("Frontier_autosave_0.aeworld"));

        // Write doc atomically and verify unchanged CRC32 skips redundant write
        let mut ids = GuidGenerator::from_seed(11);
        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, 128.0);
        let mut rec = EntityRecord::new(ids.next_entity_guid(), None);
        rec.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(1.0, 2.0, 3.0))
                .expect("pos record"),
        );
        doc.insert_entity_at([1.0, 2.0, 3.0], rec);

        let crc = AtomicU32::new(0);
        assert!(write_autosave_if_changed(&doc, &slot0, &crc).expect("first write"));
        assert!(!write_autosave_if_changed(&doc, &slot0, &crc).expect("skip identical"));

        let decoded = WorldDocument::load_from_file(&slot0, None).expect("load autosave");
        assert_eq!(decoded.total_entity_count(), 1);

        let _ = std::fs::remove_dir_all(&root);
    }
}