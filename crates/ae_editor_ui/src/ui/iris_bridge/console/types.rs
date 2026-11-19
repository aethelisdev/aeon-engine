// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Console & Logger Iris UI Types
//!
//! Type definitions, filtering modes, zero-allocation search matching, semantic [`UiTag`]
//! constants, and persistent state for the Developer Console panel.

pub use crate::ui::types::ConsoleEntry;
use irisui::prelude::*;

/// Semantic hit-testing tag for the Developer Console root panel container.
pub const CONSOLE_TAG_PANEL_ROOT: u64 = UiTag::from_str("console.panel_root").raw();

/// Semantic hit-testing tag for the "Clear Logs" toolbar button.
pub const CONSOLE_TAG_CLEAR: u64 = UiTag::from_str("console.clear").raw();

/// Semantic hit-testing tag for the "All" severity filter button.
pub const CONSOLE_TAG_FILTER_ALL: u64 = UiTag::from_str("console.filter.all").raw();

/// Semantic hit-testing tag for the "Errors" severity filter button.
pub const CONSOLE_TAG_FILTER_ERROR: u64 = UiTag::from_str("console.filter.error").raw();

/// Semantic hit-testing tag for the "Warnings" severity filter button.
pub const CONSOLE_TAG_FILTER_WARN: u64 = UiTag::from_str("console.filter.warn").raw();

/// Semantic hit-testing tag for the "Info" severity filter button.
pub const CONSOLE_TAG_FILTER_INFO: u64 = UiTag::from_str("console.filter.info").raw();

/// Semantic hit-testing tag for the "Debug" severity filter button.
pub const CONSOLE_TAG_FILTER_DEBUG: u64 = UiTag::from_str("console.filter.debug").raw();

/// Semantic hit-testing tag for the "Auto-Scroll" toggle button.
pub const CONSOLE_TAG_AUTOSCROLL: u64 = UiTag::from_str("console.autoscroll").raw();

/// Semantic hit-testing tag for the search clear button.
pub const CONSOLE_TAG_SEARCH_CLEAR: u64 = UiTag::from_str("console.search.clear").raw();

/// Semantic hit-testing tag for the search text input field.
pub const CONSOLE_TAG_SEARCH_INPUT: u64 = UiTag::from_str("console.search.input").raw();

/// Semantic hit-testing tag for the Developer Console toolbar container.
pub const CONSOLE_TAG_TOOLBAR: u64 = UiTag::from_str("console.toolbar").raw();

/// Semantic hit-testing tag for the Developer Console scrollable logs viewport.
pub const CONSOLE_TAG_VIEWPORT: u64 = UiTag::from_str("console.viewport").raw();

/// Semantic hit-testing tag for individual Developer Console log entry rows.
pub const CONSOLE_TAG_ROW: u64 = UiTag::from_str("console.row").raw();

/// Semantic hit-testing tag for the Developer Console scrollbar track.
pub const CONSOLE_TAG_SCROLLBAR_TRACK: u64 = UiTag::from_str("console.scrollbar.track").raw();

/// Semantic hit-testing tag for the Developer Console scrollbar thumb.
pub const CONSOLE_TAG_SCROLLBAR_THUMB: u64 = UiTag::from_str("console.scrollbar.thumb").raw();

/// Semantic hit-testing tag for the "Logs" sub-tab button.
pub const CONSOLE_TAG_TAB_LOGS: u64 = UiTag::from_str("console.tab.logs").raw();

/// Semantic hit-testing tag for the "Scratchpad" sub-tab button.
pub const CONSOLE_TAG_TAB_SCRATCHPAD: u64 = UiTag::from_str("console.tab.scratchpad").raw();

/// Semantic hit-testing tag for the multi-line code/text editor widget.
pub const CONSOLE_TAG_SCRATCHPAD_EDITOR: u64 = UiTag::from_str("console.scratchpad.editor").raw();

/// Semantic hit-testing tag for the "Clear" button in the Scratchpad toolbar.
pub const CONSOLE_TAG_SCRATCHPAD_CLEAR: u64 = UiTag::from_str("console.scratchpad.clear").raw();

/// Semantic hit-testing tag for the "Reset" button in the Scratchpad toolbar.
pub const CONSOLE_TAG_SCRATCHPAD_RESET: u64 = UiTag::from_str("console.scratchpad.reset").raw();

/// Default sample template pre-loaded into the multi-line scratchpad editor.
pub const DEFAULT_SCRATCHPAD_TEXT: &str = "// Aeon Engine Multi-line Scratchpad\n// Type, edit, use arrow keys, Enter, and Backspace!\n\nfn update_simulation(delta_time: f32) {\n    let speed = 9.81 * delta_time;\n    println!(\"Simulation tick: {}\", speed);\n}\n\n// Gutter line numbers and active line highlighting active.";

/// Log severity filter levels supported by the Developer Console.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ConsoleFilterLevel {
    /// Display all log messages without severity filtering.
    #[default]
    All,
    /// Display only error messages.
    Error,
    /// Display only warning messages.
    Warn,
    /// Display only informational messages.
    Info,
    /// Display only debug messages.
    Debug,
}

impl ConsoleFilterLevel {
    /// Returns the human-readable display label for the filter button.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Error => "Errors",
            Self::Warn => "Warnings",
            Self::Info => "Info",
            Self::Debug => "Debug",
        }
    }
}

/// Extension methods for `ConsoleFilterLevel` log matching.
pub trait ConsoleFilterExt {
    /// Checks if a log entry matches the current filter level.
    fn matches(&self, level: log::Level) -> bool;
}

impl ConsoleFilterExt for ConsoleFilterLevel {
    fn matches(&self, level: log::Level) -> bool {
        match self {
            Self::All => true,
            Self::Error => level == log::Level::Error,
            Self::Warn => level == log::Level::Warn,
            Self::Info => level == log::Level::Info,
            Self::Debug => level == log::Level::Debug || level == log::Level::Trace,
        }
    }
}

/// Active sub-tab mode within the Developer Console panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ConsoleTab {
    /// Standard game engine event and logger telemetry feed.
    #[default]
    Logs,
    /// Interactive multi-line code and text scratchpad editor.
    Scratchpad,
}

/// Severity level representation for individual console log entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ConsoleLogLevel {
    /// Severe error condition or unrecoverable subsystem failure.
    Error,
    /// Non-fatal warning or unexpected state.
    Warn,
    /// Informational status milestone or progress message.
    #[default]
    Info,
    /// Detailed diagnostic debug trace.
    Debug,
    /// Fine-grained internal execution trace.
    Trace,
}

impl ConsoleLogLevel {
    /// Returns the badge display label for the severity level.
    #[must_use]
    pub const fn badge_text(&self) -> &'static str {
        match self {
            Self::Error => "ERR",
            Self::Warn => "WRN",
            Self::Info => "INF",
            Self::Debug => "DBG",
            Self::Trace => "TRC",
        }
    }

    /// Returns the badge foreground text color.
    #[must_use]
    pub const fn badge_color(&self) -> Color {
        match self {
            Self::Error => Color::rgba(0.98, 0.45, 0.45, 1.0),
            Self::Warn => Color::rgba(0.98, 0.78, 0.20, 1.0),
            Self::Info => Color::rgba(0.25, 0.78, 0.98, 1.0),
            Self::Debug => Color::rgba(0.70, 0.75, 0.85, 1.0),
            Self::Trace => Color::rgba(0.50, 0.55, 0.65, 1.0),
        }
    }

    /// Returns the badge background fill color.
    #[must_use]
    pub const fn badge_bg(&self) -> Color {
        match self {
            Self::Error => Color::rgba(0.38, 0.08, 0.08, 0.90),
            Self::Warn => Color::rgba(0.35, 0.20, 0.02, 0.90),
            Self::Info => Color::rgba(0.05, 0.20, 0.32, 0.90),
            Self::Debug => Color::rgba(0.12, 0.15, 0.22, 0.90),
            Self::Trace => Color::rgba(0.08, 0.10, 0.15, 0.90),
        }
    }

    /// Assembles a standardized [`LogRowBadge`] descriptor for this severity level.
    #[must_use]
    pub const fn badge(&self) -> LogRowBadge<'static> {
        LogRowBadge::new(self.badge_text(), self.badge_color(), self.badge_bg())
    }
}

/// Aggregated log entry counts per severity level for display in toolbar badges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConsoleLogCounts {
    /// Total count of all log entries across all severity levels.
    pub all: usize,
    /// Count of log entries with `Error` severity.
    pub error: usize,
    /// Count of log entries with `Warn` severity.
    pub warn: usize,
    /// Count of log entries with `Info` severity.
    pub info: usize,
    /// Count of log entries with `Debug` or `Trace` severity.
    pub debug: usize,
}

impl ConsoleLogCounts {
    /// Creates a new log count descriptor with all fields explicitly initialized.
    #[inline]
    #[must_use]
    pub const fn new(all: usize, error: usize, warn: usize, info: usize, debug: usize) -> Self {
        Self {
            all,
            error,
            warn,
            info,
            debug,
        }
    }

    /// Aggregates log severity counts across `entries` without heap allocation.
    #[must_use]
    pub fn from_entries(entries: &[ConsoleEntry]) -> Self {
        entries
            .iter()
            .fold(Self::new(entries.len(), 0, 0, 0, 0), |mut counts, entry| {
                match entry.level {
                    log::Level::Error => counts.error += 1,
                    log::Level::Warn => counts.warn += 1,
                    log::Level::Info => counts.info += 1,
                    log::Level::Debug | log::Level::Trace => counts.debug += 1,
                }
                counts
            })
    }

    /// Returns the entry count corresponding to `filter`.
    #[inline]
    #[must_use]
    pub const fn count_for(&self, filter: ConsoleFilterLevel) -> usize {
        match filter {
            ConsoleFilterLevel::All => self.all,
            ConsoleFilterLevel::Error => self.error,
            ConsoleFilterLevel::Warn => self.warn,
            ConsoleFilterLevel::Info => self.info,
            ConsoleFilterLevel::Debug => self.debug,
        }
    }
}

/// Zero-allocation case-insensitive substring matcher (`haystack` contains `needle`).
#[must_use]
pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.is_ascii() && needle.is_ascii() {
        let nb = needle.as_bytes();
        return haystack
            .as_bytes()
            .windows(nb.len())
            .any(|w| w.eq_ignore_ascii_case(nb));
    }
    haystack.char_indices().any(|(start, _)| {
        let mut h_chars = haystack[start..].chars().flat_map(char::to_lowercase);
        let mut n_chars = needle.chars().flat_map(char::to_lowercase);
        loop {
            match n_chars.next() {
                None => return true,
                Some(nc) => match h_chars.next() {
                    Some(hc) if hc == nc => continue,
                    _ => return false,
                },
            }
        }
    })
}

/// Persistent interactive state for the Developer Console panel overlay.
#[derive(Debug, Clone)]
pub struct ConsolePanelState {
    /// Search input text state managing log filter buffer, cursor, and selection.
    pub search_input: TextInputState,
    /// Active log level severity filter.
    pub filter: ConsoleFilterLevel,
    /// Whether Developer Console automatically scrolls down when new logs arrive.
    pub auto_scroll: bool,
    /// Active sub-tab mode within the console panel.
    pub active_tab: ConsoleTab,
    /// Interactive multi-line code and text scratchpad editor state.
    pub scratchpad_state: MultilineTextState,
    /// Pre-allocated index scratch buffer storing rows matching `filter` and `search_input`.
    pub filtered_indices: Vec<usize>,
    /// Set by immediate declarative toolbar `.on_click` callbacks to request a panel rebuild.
    pub dirty_rebuild: bool,
}

impl Default for ConsolePanelState {
    fn default() -> Self {
        Self {
            search_input: TextInputState::default(),
            filter: ConsoleFilterLevel::All,
            auto_scroll: true,
            active_tab: ConsoleTab::Logs,
            scratchpad_state: MultilineTextState::new(DEFAULT_SCRATCHPAD_TEXT),
            filtered_indices: Vec::new(),
            dirty_rebuild: false,
        }
    }
}

impl ConsolePanelState {
    /// Updates [`Self::filtered_indices`] in place without allocating per-entry lowercase strings.
    pub fn sync_filtered_indices(&mut self, entries: &[ConsoleEntry]) {
        let query = self.search_input.text();
        let filter = self.filter;
        self.filtered_indices.clear();
        for (idx, entry) in entries.iter().enumerate() {
            if filter.matches(entry.level)
                && (contains_ignore_case(&entry.msg, query)
                    || contains_ignore_case(&entry.target, query))
            {
                self.filtered_indices.push(idx);
            }
        }
    }

    /// Switches the active sub-tab and flags the panel dirty for rebuild.
    #[inline]
    pub fn select_tab(&mut self, tab: ConsoleTab) {
        self.active_tab = tab;
        self.dirty_rebuild = true;
    }

    /// Clears stored engine logs, resets viewport scroll, and marks the panel dirty.
    #[inline]
    pub fn clear_logs(&mut self, scope: &mut UiScope<'_>) {
        ae_editor::editor_logger::LOGGER.clear();
        scope.set_scroll_offset(CONSOLE_TAG_VIEWPORT, 0.0);
        self.dirty_rebuild = true;
    }

    /// Applies the active severity filter and marks the panel dirty.
    #[inline]
    pub fn set_filter(&mut self, level: ConsoleFilterLevel) {
        self.filter = level;
        self.dirty_rebuild = true;
    }

    /// Toggles automatic scroll-to-bottom behavior and marks the panel dirty.
    #[inline]
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        self.dirty_rebuild = true;
    }

    /// Clears search query text, releases focus, and marks the panel dirty.
    #[inline]
    pub fn clear_search(&mut self, scope: &mut UiScope<'_>) {
        self.search_input.clear();
        scope.clear_focus();
        scope.clear_text_edit();
        self.dirty_rebuild = true;
    }

    /// Clears the scratchpad buffer and marks the panel dirty.
    #[inline]
    pub fn clear_scratchpad(&mut self) {
        self.scratchpad_state.clear();
        self.dirty_rebuild = true;
    }

    /// Resets the scratchpad buffer to [`DEFAULT_SCRATCHPAD_TEXT`] and marks the panel dirty.
    #[inline]
    pub fn reset_scratchpad(&mut self) {
        self.scratchpad_state.set_text(DEFAULT_SCRATCHPAD_TEXT);
        self.dirty_rebuild = true;
    }
}

/// Semantic alias for [`ConsolePanelState`].
pub type ConsoleState = ConsolePanelState;