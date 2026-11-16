// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Non-blocking local TCP/HTTP JSON-RPC server forwarding live inspection requests to the host loop.

use crate::protocol::{InspectRequest, InspectResponse};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

/// Pending request queued from the background HTTP listener for execution on the host main thread.
pub struct PendingInspectCall {
    /// Deserialized JSON-RPC request payload.
    pub request: InspectRequest,
    /// One-shot response channel consumed by the HTTP worker handling the client connection.
    pub responder: Sender<InspectResponse>,
}

/// Localhost inspection bridge listening for HTTP JSON-RPC commands without blocking the render loop.
pub struct InspectBridge {
    addr: String,
    rx: Receiver<PendingInspectCall>,
}

impl InspectBridge {
    /// Binds a localhost TCP listener at `addr` and starts accepting HTTP inspection requests.
    ///
    /// # Errors
    /// Returns a descriptive error string if `TcpListener::bind` fails.
    pub fn start(addr: &str) -> Result<Self, String> {
        let listener = TcpListener::bind(addr)
            .map_err(|e| format!("Failed to bind inspect server {addr}: {e}"))?;
        let bound_addr = listener
            .local_addr()
            .map_or_else(|_| addr.to_string(), |a| a.to_string());
        let (tx, rx) = mpsc::channel::<PendingInspectCall>();

        std::thread::Builder::new()
            .name("ae-mcp-inspect-server".to_string())
            .spawn(move || accept_loop(listener, tx))
            .map_err(|e| format!("Failed to spawn inspect server thread: {e}"))?;

        Ok(Self {
            addr: bound_addr,
            rx,
        })
    }

    /// Returns the actual bound socket address string.
    #[must_use]
    pub fn bound_addr(&self) -> &str {
        &self.addr
    }

    /// Drains all queued inspection requests waiting for main-thread execution.
    pub fn drain_requests(&self) -> Vec<PendingInspectCall> {
        let mut out = Vec::new();
        while let Ok(call) = self.rx.try_recv() {
            out.push(call);
        }
        out
    }
}

fn accept_loop(listener: TcpListener, tx: Sender<PendingInspectCall>) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };
        let tx_clone = tx.clone();
        std::thread::spawn(move || handle_client(stream, tx_clone));
    }
}

fn handle_client(mut stream: TcpStream, tx: Sender<PendingInspectCall>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(20)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(20)));

    let Ok(raw_req) = read_http_request(&mut stream) else {
        write_http_json(
            &mut stream,
            400,
            &InspectResponse::err("Malformed HTTP request"),
        );
        return;
    };

    let req = if raw_req.method == "GET" && raw_req.path == "/health" {
        InspectRequest {
            method: "health".to_string(),
            params: serde_json::Value::Null,
        }
    } else if raw_req.method == "POST" && raw_req.path == "/rpc" {
        match serde_json::from_slice::<InspectRequest>(&raw_req.body) {
            Ok(parsed) => parsed,
            Err(e) => {
                write_http_json(
                    &mut stream,
                    400,
                    &InspectResponse::err(format!("Invalid JSON payload: {e}")),
                );
                return;
            }
        }
    } else {
        write_http_json(
            &mut stream,
            404,
            &InspectResponse::err(format!(
                "Unsupported route: {} {}",
                raw_req.method, raw_req.path
            )),
        );
        return;
    };

    let (resp_tx, resp_rx) = mpsc::channel();
    if tx
        .send(PendingInspectCall {
            request: req,
            responder: resp_tx,
        })
        .is_err()
    {
        write_http_json(
            &mut stream,
            503,
            &InspectResponse::err("Host loop is shutting down"),
        );
        return;
    }

    match resp_rx.recv_timeout(Duration::from_secs(18)) {
        Ok(resp) => {
            let status = if resp.ok { 200 } else { 422 };
            write_http_json(&mut stream, status, &resp);
        }
        Err(_) => {
            write_http_json(
                &mut stream,
                504,
                &InspectResponse::err("Timed out waiting for host frame execution"),
            );
        }
    }
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    let bytes = reader
        .read_line(&mut request_line)
        .map_err(|e| e.to_string())?;
    if bytes == 0 {
        return Err("Empty request".to_string());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or("Missing method")?.to_string();
    let path = parts.next().ok_or("Missing path")?.to_string();

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':')
            && k.trim().eq_ignore_ascii_case("content-length")
        {
            content_length = v.trim().parse::<usize>().unwrap_or(0);
        }
    }

    const MAX_BODY: usize = 4 * 1024 * 1024;
    if content_length > MAX_BODY {
        return Err("Body exceeds 4 MiB limit".to_string());
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).map_err(|e| e.to_string())?;
    }

    Ok(HttpRequest { method, path, body })
}

fn write_http_json(stream: &mut TcpStream, status: u16, payload: &InspectResponse) {
    let body = serde_json::to_vec(payload)
        .unwrap_or_else(|_| b"{\"ok\":false,\"error\":\"Serialization error\"}".to_vec());
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        503 => "Service Unavailable",
        _ => "Gateway Timeout",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
    let _ = stream.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_inspect_bridge_roundtrip_rpc() {
        let bridge = InspectBridge::start("127.0.0.1:0").expect("bind ephemeral port");
        let addr = bridge.bound_addr().to_string();

        let client_handle = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(&addr).expect("connect to inspect bridge");
            let body = r#"{"method":"health","params":{}}"#;
            let req = format!(
                "POST /rpc HTTP/1.1\r\nHost: {addr}\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(req.as_bytes()).expect("write request");
            let mut raw_resp = String::new();
            stream
                .read_to_string(&mut raw_resp)
                .expect("read HTTP response");
            raw_resp
        });

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let calls = loop {
            let drained = bridge.drain_requests();
            if !drained.is_empty() {
                break drained;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Timed out waiting for bridge request"
            );
            std::thread::sleep(Duration::from_millis(5));
        };

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].request.method, "health");
        calls[0]
            .responder
            .send(InspectResponse::ok(json!({ "status": "ok" })))
            .expect("send response");

        let http_response = client_handle.join().expect("client thread");
        assert!(http_response.starts_with("HTTP/1.1 200 OK"));
        assert!(http_response.contains(r#""status":"ok""#));
    }
}