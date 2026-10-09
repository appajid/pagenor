//! Where MCP tool calls end up: a control-channel method call.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::{Value, json};

const MIN_CONTROL_TOKEN_BYTES: usize = 32;
const MAX_CONTROL_TOKEN_BYTES: usize = 1_024;
const MAX_CONTROL_REQUEST_BYTES: usize = 4 * 1024 * 1024;
const MAX_CONTROL_REPLY_BYTES: usize = 64 * 1024 * 1024;

/// Something that answers control-channel methods (`engine.execute`, `document.inspect`,
/// `ui.pointer`, `ui.render`, …). See `pagenor_ui_egui::control` for the full list.
pub trait Backend {
    /// Call one method. `Ok` carries the `result`, `Err` the error message.
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String>;
    /// True when a real UI is attached (`ui.screenshot`, `ui.click`, dialogs… work).
    fn has_ui(&self) -> bool;
    /// Short human description ("headless", "connected to 127.0.0.1:7979").
    fn describe(&self) -> String;
}

/// A running Pagenor app, reached through its loopback control port.
pub struct Remote {
    addr: String,
    token: String,
    conn: Option<(BufReader<TcpStream>, TcpStream)>,
    next_id: u64,
}

impl Remote {
    /// Connect with the capability from `PAGENOR_CONTROL_TOKEN`.
    pub fn connect(addr: &str) -> std::io::Result<Self> {
        let token = std::env::var("PAGENOR_CONTROL_TOKEN").map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "PAGENOR_CONTROL_TOKEN must be set and valid UTF-8 for a control connection")
        })?;
        Self::connect_with_token(addr, token)
    }

    /// Connect to `addr` with an explicit capability token.
    pub fn connect_with_token(addr: &str, token: impl Into<String>) -> std::io::Result<Self> {
        let token = token.into();
        if !(MIN_CONTROL_TOKEN_BYTES..=MAX_CONTROL_TOKEN_BYTES).contains(&token.len()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("control token must contain between {MIN_CONTROL_TOKEN_BYTES} and {MAX_CONTROL_TOKEN_BYTES} bytes"),
            ));
        }
        if addr.is_empty() || addr.len() > 4_096 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid control address length"));
        }
        let mut r = Self { addr: addr.to_string(), token, conn: None, next_id: 1 };
        r.reconnect()?;
        Ok(r)
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    fn reconnect(&mut self) -> std::io::Result<()> {
        self.conn = None;
        let mut last = std::io::Error::new(std::io::ErrorKind::NotFound, format!("cannot resolve {}", self.addr));
        for sa in self.addr.to_socket_addrs()? {
            if !sa.ip().is_loopback() {
                last =
                    std::io::Error::new(std::io::ErrorKind::PermissionDenied, format!("control connections must use a loopback address, not {sa}"));
                continue;
            }
            match TcpStream::connect_timeout(&sa, Duration::from_millis(800)) {
                Ok(s) => {
                    s.set_nodelay(true)?;
                    // The app answers within 60 s (its own timeout); leave headroom.
                    s.set_read_timeout(Some(Duration::from_secs(90)))?;
                    s.set_write_timeout(Some(Duration::from_secs(30)))?;
                    let read = s.try_clone()?;
                    self.conn = Some((BufReader::new(read), s));
                    return Ok(());
                }
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    fn roundtrip(&mut self, line: &[u8]) -> std::io::Result<String> {
        if self.conn.is_none() {
            self.reconnect()?;
        }
        let Some((reader, writer)) = self.conn.as_mut() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "not connected"));
        };
        writer.write_all(line)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        read_control_reply(reader, MAX_CONTROL_REPLY_BYTES)
    }
}

impl Backend for Remote {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        if method.is_empty() || method.len() > 256 {
            return Err("control method must contain 1–256 bytes".into());
        }
        let id = self.next_id;
        self.next_id = id.checked_add(1).ok_or("control request IDs exhausted")?;
        // Move the payload into the envelope; serialize through a bounded writer rather than
        // allocating an arbitrarily large JSON string before checking its length.
        let envelope = Value::Object(serde_json::Map::from_iter([
            ("id".into(), json!(id)),
            ("token".into(), Value::String(self.token.clone())),
            ("method".into(), Value::String(method.into())),
            ("params".into(), params),
        ]));
        let mut line = RequestBuffer(Vec::new());
        serde_json::to_writer(&mut line, &envelope)
            .map_err(|_| format!("control request is larger than {MAX_CONTROL_REQUEST_BYTES} bytes or cannot be serialized"))?;
        // A lost reply does not mean a command failed to run. Drop this connection, but never
        // replay the request; a later explicit call may reconnect with a new ID.
        let reply = self.roundtrip(&line.0).map_err(|e| {
            self.conn = None;
            format!("Pagenor control transport at {} failed: {e}; request may have executed and was not retried", self.addr)
        })?;
        let v: Value = serde_json::from_str(reply.trim()).map_err(|e| {
            self.conn = None;
            format!("bad reply from app: {e}; request may have executed and was not retried")
        })?;
        if v.get("id") != Some(&json!(id)) || v.get("ok").and_then(Value::as_bool).is_none() {
            self.conn = None;
            return Err("invalid control reply ID or status; request may have executed and was not retried".into());
        }
        if v.get("ok").and_then(Value::as_bool) == Some(true) {
            Ok(v.get("result").cloned().unwrap_or(Value::Null))
        } else {
            Err(v.get("error").and_then(Value::as_str).unwrap_or("unknown error").to_string())
        }
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn describe(&self) -> String {
        format!("connected to the Pagenor app at {}", self.addr)
    }
}

fn read_control_reply(reader: &mut impl BufRead, max: usize) -> std::io::Result<String> {
    let mut reply = String::new();
    let limit = u64::try_from(max).unwrap_or(u64::MAX).saturating_add(1);
    let bytes_read = reader.take(limit).read_line(&mut reply)?;
    if bytes_read == 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "control channel closed"));
    }
    if bytes_read > max || !reply.ends_with('\n') {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid control reply framing"));
    }
    Ok(reply)
}

struct RequestBuffer(Vec<u8>);
impl Write for RequestBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_CONTROL_REQUEST_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "control request too large"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod bounds_tests {
    use super::*;
    #[test]
    fn reply_framing_bounds_bytes_and_requires_a_complete_utf8_line() {
        assert_eq!(read_control_reply(&mut &b"1234\n"[..], 5).unwrap(), "1234\n");
        for bytes in [&b"12345\n"[..], &b"1234"[..], &b"\xff\n"[..]] {
            assert!(read_control_reply(&mut &bytes[..], 5).is_err());
        }
        assert_eq!(read_control_reply(&mut &b""[..], 5).unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);
    }
    #[test]
    fn exhausted_ids_fail_before_sending() {
        let mut remote = Remote { addr: "127.0.0.1:1".into(), token: "x".repeat(32), conn: None, next_id: u64::MAX };
        assert!(remote.call("engine.execute", json!({})).unwrap_err().contains("IDs exhausted"));
        assert!(remote.conn.is_none());
    }
}
