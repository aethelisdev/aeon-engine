// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Timed Workbench Status Messages
//!
//! Framework-managed temporary status notifications displayed in the workbench
//! status bar and automatically expired before subsequent frame passes.

use std::fmt::Debug;
use std::hash::Hash;
use std::time::{Duration, Instant};

use irisui_core::Color;

use super::Workbench;

/// Default display lifetime applied by [`Workbench::set_status_text`].
pub const DEFAULT_STATUS_MESSAGE_DURATION: Duration = Duration::from_millis(2500);

/// Transient color-coded status spans scheduled for automatic expiration.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedStatusMessage {
    spans: Vec<(String, Color)>,
    expires_at: Instant,
}

impl TimedStatusMessage {
    /// Returns the color-coded spans contained in this status notification.
    #[inline]
    #[must_use]
    pub fn spans(&self) -> &[(String, Color)] {
        &self.spans
    }

    /// Returns `true` when `now` has reached or passed the expiration deadline.
    #[inline]
    #[must_use]
    pub fn is_expired_at(&self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Displays temporary colored status spans for `duration` and invalidates the shell chrome.
    pub fn set_status_message(&mut self, spans: Vec<(String, Color)>, duration: Duration) {
        self.status_message = Some(TimedStatusMessage {
            spans,
            expires_at: Instant::now() + duration,
        });
        self.notifier.tag_all();
    }

    /// Convenience helper displaying a single colored status string for [`DEFAULT_STATUS_MESSAGE_DURATION`].
    pub fn set_status_text(&mut self, text: impl Into<String>, color: Color) {
        self.set_status_message(vec![(text.into(), color)], DEFAULT_STATUS_MESSAGE_DURATION);
    }

    /// Clears any active timed status message immediately and invalidates the shell chrome.
    pub fn clear_status_message(&mut self) {
        if self.status_message.take().is_some() {
            self.notifier.tag_all();
        }
    }

    /// Returns active timed status spans if present and not yet expired.
    #[inline]
    #[must_use]
    pub fn status_spans(&self) -> Option<&[(String, Color)]> {
        self.status_message
            .as_ref()
            .filter(|msg| !msg.is_expired_at(Instant::now()))
            .map(TimedStatusMessage::spans)
    }

    /// Expires and removes an elapsed status notification before evaluating the frame lifecycle.
    pub(crate) fn expire_status_message_if_needed(&mut self) {
        if self
            .status_message
            .as_ref()
            .is_some_and(|msg| msg.is_expired_at(Instant::now()))
        {
            self.status_message = None;
            self.notifier.tag_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptors::WorkbenchDescriptor;
    use irisui_dock::{DockState, DockTree};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Tab {
        Main,
    }

    #[test]
    fn test_timed_status_message_sets_returns_and_expires() {
        let mut tree = DockTree::new();
        let root = tree.create_leaf(vec![Tab::Main]);
        tree.set_root(root);
        let mut wb: Workbench<Tab, (), ()> =
            Workbench::new(WorkbenchDescriptor::new("Test", DockState::new(tree)));

        wb.notifier.clear_all();
        wb.set_status_message(
            vec![("Saved".to_string(), Color::GREEN)],
            Duration::from_secs(5),
        );
        assert!(wb.notifier.is_shell_dirty());
        assert_eq!(wb.status_spans().map(<[_]>::len), Some(1));

        wb.notifier.clear_all();
        wb.status_message = Some(TimedStatusMessage {
            spans: vec![("Old".to_string(), Color::RED)],
            expires_at: Instant::now() - Duration::from_millis(50),
        });
        assert!(wb.status_spans().is_none());
        wb.expire_status_message_if_needed();
        assert!(wb.status_message.is_none());
        assert!(wb.notifier.is_shell_dirty());
    }
}