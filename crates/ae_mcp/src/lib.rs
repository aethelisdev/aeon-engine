// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Engine-agnostic live UI inspection, visual agent overlay, and MCP server crate.

pub mod controller;
pub mod drag;
pub mod host;
pub mod input;
pub mod overlay;
pub mod protocol;
pub mod query;
pub mod screenshot;
pub mod sequence;
pub mod server;
pub mod stdio_rpc;

pub use controller::McpSession;
pub use host::{
    McpEntityInfo, McpFloatingWindow, McpGpuHandles, McpHealthSnapshot, McpHost, McpLogEntry,
};
pub use overlay::{InspectActionKind, InspectVisualOverlay};
pub use protocol::{InspectRequest, InspectResponse};
pub use server::{InspectBridge, PendingInspectCall};