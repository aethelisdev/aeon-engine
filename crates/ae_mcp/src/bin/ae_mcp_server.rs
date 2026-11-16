// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone stdio Model Context Protocol (MCP) server binary for Aeon Engine live inspection.

use ae_mcp::stdio_rpc::McpServerRuntime;
use serde_json::Value;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut runtime = McpServerRuntime::from_env();
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() >= 2 && args[0] == "--call" {
        let tool = &args[1];
        let raw_params = args.get(2).map_or("{}", String::as_str);
        let params: Value =
            serde_json::from_str(raw_params).unwrap_or_else(|_| Value::Object(Default::default()));
        let result = runtime.execute_tool(tool, &params);
        let formatted = serde_json::to_string_pretty(&result).map_err(io::Error::other)?;
        let mut out = io::stdout().lock();
        out.write_all(formatted.as_bytes())?;
        out.write_all(b"\n")?;
        return out.flush();
    }

    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();
    runtime.serve_stdio(stdin, stdout)
}