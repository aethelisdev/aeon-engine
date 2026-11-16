// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! JSON-RPC inspection protocol types and value parsing helpers.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Incoming JSON-RPC inspection command dispatched from the MCP server.
#[derive(Debug, Clone, Deserialize)]
pub struct InspectRequest {
    /// Command identifier (e.g. `"health"`, `"ui.tree_dump"`, `"ui.drag"`, `"render.screenshot"`).
    pub method: String,
    /// Optional JSON parameters object for the command.
    #[serde(default)]
    pub params: Value,
}

/// Outgoing JSON-RPC inspection response returned to the MCP client.
#[derive(Debug, Clone, Serialize)]
pub struct InspectResponse {
    /// Whether the command completed without error.
    pub ok: bool,
    /// Result JSON payload when `ok` is `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Human-readable error message when `ok` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl InspectResponse {
    /// Builds a successful response wrapping `result`.
    #[must_use]
    pub fn ok(result: Value) -> Self {
        Self {
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    /// Builds an error response with `message`.
    #[must_use]
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            result: None,
            error: Some(message.into()),
        }
    }
}

/// Parses a `u64` integer or hex/decimal string from a JSON [`Value`].
#[must_use]
pub fn parse_u64_json_value(val: &Value) -> Option<u64> {
    if let Some(n) = val.as_u64() {
        return Some(n);
    }
    let s = val.as_str()?.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_u64_json_value_handles_numbers_decimal_and_hex() {
        assert_eq!(parse_u64_json_value(&json!(42)), Some(42));
        assert_eq!(parse_u64_json_value(&json!("0xDA00_0001")), None);
        assert_eq!(
            parse_u64_json_value(&json!("0xDA000001")),
            Some(0xDA00_0001)
        );
        assert_eq!(parse_u64_json_value(&json!("0X10")), Some(16));
        assert_eq!(parse_u64_json_value(&json!("12345")), Some(12345));
        assert_eq!(parse_u64_json_value(&json!(null)), None);
    }

    #[test]
    fn test_inspect_response_constructors() {
        let ok = InspectResponse::ok(json!({ "ready": true }));
        assert!(ok.ok);
        assert!(ok.error.is_none());
        let err = InspectResponse::err("failure");
        assert!(!err.ok);
        assert_eq!(err.error.as_deref(), Some("failure"));
    }
}