// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use chrono::Local;
use log::{Level, Metadata, Record};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_LOGS: usize = 1000;

/// Single log entry stored in the in-memory ring buffer.
/// Contains the log level, module target, formatted message, and timestamp.
/// Displayed by the editor's Console panel via `LOGGER.logs`.
pub struct LogEntry {
    pub level: Level,
    pub target: String,
    pub msg: String,
    pub timestamp: String,
}

/// Dual-output logger: prints colorized logs to the terminal AND stores them
/// in a `Mutex<VecDeque>` ring buffer for the egui Console panel.
/// Capped at `MAX_LOGS` (1000) entries to prevent unbounded memory growth.
/// Filters out verbose wgpu/winit/naga/mio logs to prevent UI lag.
/// Uses `AtomicU64` counter for lock-free change detection by the UI.
pub struct EditorLogger {
    // We use a VecDeque to maintain a capped history of logs (discarding oldest).
    pub logs: Mutex<VecDeque<LogEntry>>,
    /// Monotonic counter to track updates without locking.
    pub log_count: AtomicU64,
}

/// Global static logger instance registered with the `log` crate.
/// Initialized once via `init()`. Safe for multi-threaded access due to
/// `Mutex`-protected log storage and `AtomicU64` counter.
pub static LOGGER: EditorLogger = EditorLogger {
    logs: Mutex::new(VecDeque::new()),
    log_count: AtomicU64::new(0),
};

impl log::Log for EditorLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Here we can filter levels. For now, we allow <= Info (Info, Warn, Error).
        // If we want Debug/Trace, we can adjust here.
        metadata.level() <= Level::Debug
        // ignore verbose wgpu/winit logs to prevent UI lag.
        && !metadata.target().starts_with("wgpu")
        && !metadata.target().starts_with("winit")
        && !metadata.target().starts_with("mio")
        && !metadata.target().starts_with("naga")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let ts = Local::now().format("%H:%M:%S").to_string();
            let msg_str = format!("{}", record.args());

            // 1. Print to the actual terminal just like env_logger does.
            // (We colorize it simply but can keep it basic)
            let color_code = match record.level() {
                Level::Error => "\x1b[31m", // Red
                Level::Warn => "\x1b[33m",  // Yellow
                Level::Info => "\x1b[32m",  // Green
                Level::Debug => "\x1b[34m", // Blue
                Level::Trace => "\x1b[36m", // Cyan
            };
            println!(
                "{}{}\x1b[0m [{}] {}",
                color_code,
                record.level(),
                record.target(),
                msg_str
            );

            // 2. Store safely in memory for Egui
            // If another thread panicked while holding the lock (poisoned),
            // recover by taking the inner data — losing logs is worse than a stale lock.
            if let Ok(mut lock) = self.logs.lock().or_else(|poisoned| {
                eprintln!("[EditorLogger] WARNING: Log mutex was poisoned, recovering.");
                Ok::<_, ()>(poisoned.into_inner())
            }) {
                if lock.len() >= MAX_LOGS {
                    lock.pop_front();
                }
                lock.push_back(LogEntry {
                    level: record.level(),
                    target: record.target().to_string(),
                    msg: msg_str,
                    timestamp: ts,
                });
                self.log_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn flush(&self) {}
}

/// Registers the global `EditorLogger` with the `log` crate at Debug level.
/// Must be called exactly once at engine startup, before any `log::info!()` calls.
/// Returns `Err` if another logger has already been registered.
pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
}