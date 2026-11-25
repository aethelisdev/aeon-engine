// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Pure-Rust MCP stdio JSON-RPC 2.0 protocol router, HTTP inspect client, and process launcher.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const DEFAULT_PORT: u16 = 7878;

/// Stateful MCP stdio bridge owning any launched child engine process.
pub struct McpServerRuntime {
    workspace_root: PathBuf,
    default_port: u16,
    child: Option<Child>,
}

impl McpServerRuntime {
    /// Initializes a stdio MCP runtime resolving `AEON_WORKSPACE` and `AEON_INSPECT_PORT`.
    #[must_use]
    pub fn from_env() -> Self {
        let workspace_root = std::env::var_os("AEON_WORKSPACE")
            .map(PathBuf::from)
            .unwrap_or_else(infer_workspace_root);
        let default_port = std::env::var("AEON_INSPECT_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        Self {
            workspace_root,
            default_port,
            child: None,
        }
    }

    /// Runs the blocking line-delimited JSON-RPC 2.0 MCP loop on `reader` and `writer`.
    ///
    /// # Errors
    /// Returns an I/O error if reading stdin or writing stdout fails.
    pub fn serve_stdio<R: BufRead, W: Write>(
        &mut self,
        reader: R,
        mut writer: W,
    ) -> std::io::Result<()> {
        for line_res in reader.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(msg) = serde_json::from_str::<Value>(trimmed) else {
                continue;
            };
            if let Some(reply) = self.handle_mcp_message(&msg) {
                let serialized = serde_json::to_string(&reply).map_err(std::io::Error::other)?;
                writer.write_all(serialized.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
        }
        Ok(())
    }

    /// Processes a single MCP JSON-RPC request object and returns an optional response payload.
    pub fn handle_mcp_message(&mut self, msg: &Value) -> Option<Value> {
        let req_id = msg.get("id").cloned();
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        match method {
            "initialize" => {
                let proto = params
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .unwrap_or("2024-11-05");
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "protocolVersion": proto,
                        "capabilities": { "tools": {} },
                        "serverInfo": { "name": "ae-mcp-server", "version": env!("CARGO_PKG_VERSION") },
                    }
                }))
            }
            "notifications/initialized" => None,
            "ping" => Some(json!({ "jsonrpc": "2.0", "id": req_id, "result": {} })),
            "tools/list" => Some(json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": { "tools": tool_definitions(self.default_port) },
            })),
            "tools/call" => {
                let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let tool_args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let out = self.execute_tool(tool_name, &tool_args);
                let is_err = !out.get("ok").and_then(Value::as_bool).unwrap_or(true);
                let content = format_mcp_content(tool_name, &tool_args, &out);
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "content": content,
                        "isError": is_err,
                    }
                }))
            }
            other => {
                let id_val = req_id?;
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id_val,
                    "error": { "code": -32601, "message": format!("Method not found: {other}") },
                }))
            }
        }
    }

    /// Executes a named MCP tool locally (`aeon_launch`/`aeon_stop`) or over localhost HTTP RPC.
    pub fn execute_tool(&mut self, name: &str, args: &Value) -> Value {
        let port = args
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|p| u16::try_from(p).ok())
            .unwrap_or(self.default_port);

        match name {
            "aeon_launch" => self.launch_engine(args, port),
            "aeon_stop" => self.stop_engine(),
            "aeon_health" => http_get_health(port),
            _ => {
                let Some(rpc_method) = map_tool_to_rpc(name) else {
                    return json!({ "ok": false, "error": format!("Unknown tool: {name}") });
                };
                let mut params = args.as_object().cloned().unwrap_or_default();
                params.remove("port");
                params.remove("inline");
                if name == "aeon_screenshot" && !params.contains_key("path") {
                    let default_shot = self.workspace_root.join("target/aeon_inspect_shot.png");
                    params.insert(
                        "path".to_string(),
                        Value::String(default_shot.to_string_lossy().into_owned()),
                    );
                }
                http_post_rpc(port, rpc_method, Value::Object(params))
            }
        }
    }

    fn pid_file(&self) -> PathBuf {
        self.workspace_root.join("target/aeon_inspect.pid")
    }

    fn launch_engine(&mut self, args: &Value, port: u16) -> Value {
        let mode = args.get("mode").and_then(Value::as_str).unwrap_or("3D");
        let build = args.get("build").and_then(Value::as_bool).unwrap_or(true);
        let project_path = args.get("project_path").and_then(Value::as_str);
        let timeout_secs = args
            .get("timeout_secs")
            .and_then(Value::as_f64)
            .unwrap_or(25.0)
            .max(1.0);

        if build {
            let output = Command::new("cargo")
                .args(["build", "-p", "ae_editor_app"])
                .current_dir(&self.workspace_root)
                .output();
            match output {
                Ok(res) if !res.status.success() => {
                    let err_txt = String::from_utf8_lossy(&res.stderr);
                    return json!({ "ok": false, "error": format!("cargo build failed:\n{err_txt}") });
                }
                Err(e) => {
                    return json!({ "ok": false, "error": format!("Failed running cargo: {e}") });
                }
                Ok(_) => {}
            }
        }

        let exe = self.workspace_root.join("target/debug/ae_editor_app");
        if !exe.is_file() {
            return json!({
                "ok": false,
                "error": format!("Engine binary not found at {}", exe.display()),
            });
        }

        let log_path = self.workspace_root.join("target/aeon_inspect.log");
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let Ok(log_file) = std::fs::File::create(&log_path) else {
            return json!({ "ok": false, "error": "Failed creating engine log file" });
        };
        let Ok(err_file) = log_file.try_clone() else {
            return json!({ "ok": false, "error": "Failed cloning log handle" });
        };

        let mut cmd = Command::new(&exe);
        cmd.current_dir(&self.workspace_root)
            .args(["--mode", mode, "--inspect", &format!("127.0.0.1:{port}")])
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file));
        if let Some(proj) = project_path {
            cmd.args(["--project", proj]);
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return json!({ "ok": false, "error": format!("Failed spawning engine: {e}") });
            }
        };
        let pid = child.id();
        let _ = std::fs::write(self.pid_file(), pid.to_string());

        let deadline = Instant::now() + Duration::from_secs_f64(timeout_secs);
        while Instant::now() < deadline {
            if let Ok(Some(status)) = child.try_wait() {
                return json!({
                    "ok": false,
                    "error": format!("Engine exited early ({status}). See {}", log_path.display()),
                });
            }
            let health = http_get_health(port);
            if health.get("ok").and_then(Value::as_bool).unwrap_or(false) {
                self.child = Some(child);
                return json!({ "ok": true, "pid": pid, "health": health });
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        self.child = Some(child);
        json!({ "ok": false, "error": format!("Timed out waiting for inspect server on port {port}") })
    }

    fn stop_engine(&mut self) -> Value {
        let mut stopped = Vec::new();
        if let Some(mut child) = self.child.take() {
            let pid = child.id();
            let _ = child.kill();
            let _ = child.wait();
            stopped.push(pid);
        }
        let pid_path = self.pid_file();
        if let Ok(raw) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = raw.trim().parse::<u32>()
                && !stopped.contains(&pid)
            {
                let _ = terminate_pid(pid);
                stopped.push(pid);
            }
            let _ = std::fs::remove_file(&pid_path);
        }
        json!({ "ok": true, "stopped_pids": stopped })
    }
}

fn terminate_pid(pid: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()?;
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()?;
    }
    Ok(())
}

fn infer_workspace_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        // target/{debug|release}/ae_mcp_server -> workspace root
        if let Some(target_dir) = dir.parent()
            && target_dir.file_name().is_some_and(|n| n == "target")
            && let Some(root) = target_dir.parent()
        {
            return root.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn map_tool_to_rpc(name: &str) -> Option<&'static str> {
    match name {
        "aeon_ui_dump" => Some("ui.tree_dump"),
        "aeon_ui_click" => Some("ui.click"),
        "aeon_ui_drag" => Some("ui.drag"),
        "aeon_ui_move" => Some("ui.mouse_move"),
        "aeon_ui_scroll" => Some("ui.scroll"),
        "aeon_ui_key" => Some("ui.key"),
        "aeon_ui_type" => Some("ui.type_text"),
        "aeon_list_entities" => Some("scene.list_entities"),
        "aeon_select_entity" => Some("scene.select_entity"),
        "aeon_inspect_entity" => Some("scene.inspect_entity"),
        "aeon_engine_logs" => Some("engine.logs"),
        "aeon_screenshot" => Some("render.screenshot"),
        _ => None,
    }
}

fn format_mcp_content(tool_name: &str, tool_args: &Value, out: &Value) -> Vec<Value> {
    let mut content = vec![json!({
        "type": "text",
        "text": serde_json::to_string_pretty(out).unwrap_or_else(|_| out.to_string()),
    })];
    let inline = tool_args
        .get("inline")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let ok = out.get("ok").and_then(Value::as_bool).unwrap_or(false);
    if tool_name == "aeon_screenshot"
        && ok
        && inline
        && let Some(path_str) = out
            .get("result")
            .and_then(|r| r.get("path"))
            .and_then(Value::as_str)
        && let Ok(bytes) = std::fs::read(Path::new(path_str))
    {
        content.push(json!({
            "type": "image",
            "data": encode_base64(&bytes),
            "mimeType": "image/png",
        }));
    }
    content
}

fn http_get_health(port: u16) -> Value {
    let raw_req =
        format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    http_exchange(port, raw_req.as_bytes(), Duration::from_secs(2))
}

fn http_post_rpc(port: u16, method: &str, params: Value) -> Value {
    let body = json!({ "method": method, "params": params }).to_string();
    let raw_req = format!(
        "POST /rpc HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    http_exchange(port, raw_req.as_bytes(), Duration::from_secs(20))
}

fn http_exchange(port: u16, raw_request: &[u8], timeout: Duration) -> Value {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => return json!({ "ok": false, "error": e.to_string() }),
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    if let Err(e) = stream.write_all(raw_request) {
        return json!({ "ok": false, "error": e.to_string() });
    }
    let mut response = Vec::new();
    if let Err(e) = BufReader::new(stream).read_to_end(&mut response) {
        return json!({ "ok": false, "error": e.to_string() });
    }
    let Some(split) = response.windows(4).position(|w| w == b"\r\n\r\n") else {
        return json!({ "ok": false, "error": "Invalid HTTP response from inspect server" });
    };
    let body = &response[split + 4..];
    serde_json::from_slice(body).unwrap_or_else(|e| {
        json!({
            "ok": false,
            "error": format!("Invalid JSON response ({e}): {}", String::from_utf8_lossy(body)),
        })
    })
}

/// Encodes arbitrary binary slices using standard RFC 4648 base64 (`=` padded).
#[must_use]
pub fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let combined = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((combined >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((combined >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((combined >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(combined & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn tool_definitions(default_port: u16) -> Value {
    let target_schema = json!({
        "description": "Target as [x, y] array, widget name string, or {point, tag, name} object."
    });
    json!([
        {
            "name": "aeon_launch",
            "description": "Builds (optional) and launches Aeon Engine with live MCP inspection enabled.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "mode": { "type": "string", "enum": ["3D", "2D"], "default": "3D" },
                    "project_path": { "type": "string" },
                    "build": { "type": "boolean", "default": true },
                    "timeout_secs": { "type": "number", "default": 25.0 }
                }
            }
        },
        {
            "name": "aeon_stop",
            "description": "Terminates the running Aeon Engine inspection instance.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "aeon_health",
            "description": "Returns live window size, DPI scale factor, active mode, selected entity, and popup state.",
            "inputSchema": {
                "type": "object",
                "properties": { "port": { "type": "integer", "default": default_port } }
            }
        },
        {
            "name": "aeon_ui_dump",
            "description": "Dumps live UiTree nodes, computed_rect, style insets, active popups, and tag_rects.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "name_filter": { "type": "string" },
                    "only_popups": { "type": "boolean", "default": false },
                    "tag": { "type": "string", "description": "Decimal or 0x hex tag filter" }
                }
            }
        },
        {
            "name": "aeon_ui_click",
            "description": "Injects real MouseMove + MouseDown + MouseUp UI events (single or double click) with visual ripple.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "point": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 },
                    "tag": { "type": "string" },
                    "name": { "type": "string" },
                    "index": { "type": "integer", "default": 0 },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" },
                    "double_click": { "type": "boolean", "default": false }
                }
            }
        },
        {
            "name": "aeon_ui_drag",
            "description": "Performs a smooth multi-frame interpolated pointer drag from 'from' to 'to' visible live on screen.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "from": target_schema,
                    "to": target_schema,
                    "steps": { "type": "integer", "default": 12, "minimum": 1, "maximum": 120 },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" }
                },
                "required": ["from", "to"]
            }
        },
        {
            "name": "aeon_ui_move",
            "description": "Injects a live MouseMove event to trigger hover states and updates the visual cursor marker.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "x": { "type": "number" },
                    "y": { "type": "number" }
                },
                "required": ["x", "y"]
            }
        },
        {
            "name": "aeon_ui_scroll",
            "description": "Injects a live MouseScroll event at (x, y) with a visual scroll ripple.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "x": { "type": "number" },
                    "y": { "type": "number" },
                    "delta_y": { "type": "number" }
                },
                "required": ["x", "y", "delta_y"]
            }
        },
        {
            "name": "aeon_ui_key",
            "description": "Injects a keyboard key press/release (Escape, Enter, Tab, Ctrl+Z, Delete, F5, etc.).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "key": { "type": "string" },
                    "ctrl": { "type": "boolean", "default": false },
                    "shift": { "type": "boolean", "default": false }
                },
                "required": ["key"]
            }
        },
        {
            "name": "aeon_ui_type",
            "description": "Types UTF-8 text into the currently focused UI input widget, optionally submitting with Enter.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "text": { "type": "string" },
                    "submit": { "type": "boolean", "default": false }
                },
                "required": ["text"]
            }
        },
        {
            "name": "aeon_list_entities",
            "description": "Lists entities in the active ECS world and shows the current selection.",
            "inputSchema": {
                "type": "object",
                "properties": { "port": { "type": "integer", "default": default_port } }
            }
        },
        {
            "name": "aeon_select_entity",
            "description": "Selects an entity by name or id_bits and marks Inspector/Hierarchy dirty.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "name": { "type": "string" },
                    "id_bits": { "type": "integer" }
                }
            }
        },
        {
            "name": "aeon_inspect_entity",
            "description": "Reads all dynamic ECS components on the target or currently selected entity as JSON.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "name": { "type": "string" },
                    "id_bits": { "type": "integer" }
                }
            }
        },
        {
            "name": "aeon_engine_logs",
            "description": "Fetches recent entries from the live editor logger ring buffer.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "limit": { "type": "integer", "default": 50 },
                    "level": { "type": "string", "enum": ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"] }
                }
            }
        },
        {
            "name": "aeon_screenshot",
            "description": "Captures the live WGPU UI + 3D viewport (+ agent visual marker) into PNG and optionally returns it inline.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": default_port },
                    "path": { "type": "string" },
                    "inline": { "type": "boolean", "default": true }
                }
            }
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base64_rfc4648_vectors() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_mcp_initialize_and_tools_list_protocol() {
        let mut rt = McpServerRuntime::from_env();
        let init = rt
            .handle_mcp_message(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": "2024-11-05" }
            }))
            .expect("initialize reply");
        assert_eq!(init["result"]["serverInfo"]["name"], "ae-mcp-server");

        let list = rt
            .handle_mcp_message(&json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }))
            .expect("tools/list reply");
        let tools = list["result"]["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 15);
    }
}