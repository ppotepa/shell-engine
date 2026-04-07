//! `engine-io` is a transport-agnostic bridge between the game engine and external sidecar apps.
//!
//! This crate intentionally contains no game logic and no knowledge of Shell Quest scenes.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum IoRequest {
    Hello {
        cols: u16,
        rows: u16,
        boot_scene: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        difficulty: Option<String>,
    },
    SetInput {
        text: String,
    },
    Submit {
        line: String,
    },
    Key {
        code: String,
        ctrl: bool,
        alt: bool,
        shift: bool,
    },
    Resize {
        cols: u16,
        rows: u16,
    },
    Tick {
        dt_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum IoEvent {
    Out {
        lines: Vec<String>,
    },
    /// Single line output with optional delay in milliseconds.
    /// Engine queues this and displays after delay for realistic timing simulation.
    EmitLine {
        text: String,
        #[serde(rename = "delay_ms", skip_serializing_if = "Option::is_none")]
        delay_ms: Option<u64>,
    },
    Clear,
    SetPromptPrefix {
        text: String,
    },
    SetPromptMasked {
        masked: bool,
    },
    // Incremental buffer update for sidecars that prefer diff-style terminal output.
    ScreenDiff {
        clear: bool,
        lines: Vec<String>,
    },
    // Reserved for fullscreen apps (vi-like) — engine can map this to a sprite or compositor layer.
    ScreenFull {
        lines: Vec<String>,
        cursor_x: u16,
        cursor_y: u16,
    },
    Custom {
        payload: JsonValue,
    },
}

/// Transport-agnostic interface for communicating with a running sidecar.
///
/// Both `TcpSidecar` (localhost TCP) and `SidecarProcess` (stdio JSON-lines)
/// implement this trait, making the transport selection a startup-time concern
/// rather than a hard-coded call site.
pub trait SidecarTransport: Send + Sync {
    /// Send a request to the sidecar. Non-blocking; queued on an internal channel.
    fn send(&self, req: IoRequest) -> Result<(), EngineIoError>;
    /// Drain up to `max` pending inbound events without blocking.
    fn try_drain_events(&self, max: usize) -> Vec<IoEvent>;
    /// `true` if the child process is still running.
    fn is_alive(&self) -> bool;
    /// Kill the child process.
    fn kill(&self);
}

#[derive(Debug, thiserror::Error)]
pub enum EngineIoError {
    #[error("failed to spawn sidecar: {0}")]
    Spawn(String),
    #[error("sidecar stdin unavailable")]
    NoStdin,
    #[error("sidecar stdout unavailable")]
    NoStdout,
    #[error("sidecar write failed: {0}")]
    Write(String),
    #[error("sidecar connect failed: {0}")]
    Connect(String),
}

/// Handle to a running sidecar process.
///
/// Writes requests as JSON lines to stdin and parses JSON lines from stdout.
pub struct SidecarProcess {
    child: Mutex<Child>,
    stdin_tx: Sender<String>,
    event_rx: Mutex<Receiver<IoEvent>>,
}

/// Handle to a running TCP sidecar process.
///
/// Spawns the child process, connects to its localhost TCP server, then sends
/// and receives the same JSON-line protocol used by the stdio transport.
pub struct TcpSidecar {
    child: Mutex<Child>,
    write_tx: Sender<String>,
    event_rx: Mutex<Receiver<IoEvent>>,
}

impl TcpSidecar {
    pub fn spawn(
        command: &str,
        args: &[String],
        cwd: Option<&std::path::Path>,
        port: u16,
    ) -> Result<Self, EngineIoError> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        let child = cmd
            .spawn()
            .map_err(|e| EngineIoError::Spawn(format!("{command}: {e}")))?;

        let stream = connect_with_retry(port)?;
        let reader_stream = stream
            .try_clone()
            .map_err(|e| EngineIoError::Connect(e.to_string()))?;

        let (write_tx, write_rx) = mpsc::channel::<String>();
        let (event_tx, event_rx) = mpsc::channel::<IoEvent>();

        spawn_tcp_writer(stream, write_rx);
        spawn_stdout_reader(reader_stream, event_tx);

        Ok(Self {
            child: Mutex::new(child),
            write_tx,
            event_rx: Mutex::new(event_rx),
        })
    }

    pub fn send(&self, req: IoRequest) -> Result<(), EngineIoError> {
        let line = serde_json::to_string(&req).map_err(|e| EngineIoError::Write(e.to_string()))?;
        self.write_tx
            .send(line)
            .map_err(|e| EngineIoError::Write(e.to_string()))
    }

    pub fn try_drain_events(&self, max: usize) -> Vec<IoEvent> {
        let mut out = Vec::new();
        let Ok(rx) = self.event_rx.lock() else {
            return out;
        };
        for _ in 0..max {
            match rx.try_recv() {
                Ok(ev) => out.push(ev),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        out
    }

    pub fn is_alive(&self) -> bool {
        let Ok(mut child) = self.child.lock() else {
            return false;
        };
        child.try_wait().ok().flatten().is_none()
    }

    pub fn kill(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

impl SidecarProcess {
    pub fn spawn(
        command: &str,
        args: &[String],
        cwd: Option<&std::path::Path>,
    ) -> Result<Self, EngineIoError> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| EngineIoError::Spawn(format!("{command}: {e}")))?;

        let stdin = child.stdin.take().ok_or(EngineIoError::NoStdin)?;
        let stdout = child.stdout.take().ok_or(EngineIoError::NoStdout)?;

        let (stdin_tx, stdin_rx) = mpsc::channel::<String>();
        let (event_tx, event_rx) = mpsc::channel::<IoEvent>();

        spawn_stdin_writer(stdin, stdin_rx);
        spawn_stdout_reader(stdout, event_tx);

        Ok(Self {
            child: Mutex::new(child),
            stdin_tx,
            event_rx: Mutex::new(event_rx),
        })
    }

    pub fn send(&self, req: IoRequest) -> Result<(), EngineIoError> {
        let line = serde_json::to_string(&req).map_err(|e| EngineIoError::Write(e.to_string()))?;
        self.stdin_tx
            .send(line)
            .map_err(|e| EngineIoError::Write(e.to_string()))
    }

    pub fn try_drain_events(&self, max: usize) -> Vec<IoEvent> {
        let mut out = Vec::new();
        let Ok(rx) = self.event_rx.lock() else {
            return out;
        };
        for _ in 0..max {
            match rx.try_recv() {
                Ok(ev) => out.push(ev),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        out
    }

    pub fn is_alive(&self) -> bool {
        let Ok(mut child) = self.child.lock() else {
            return false;
        };
        child.try_wait().ok().flatten().is_none()
    }

    pub fn kill(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

impl SidecarTransport for TcpSidecar {
    fn send(&self, req: IoRequest) -> Result<(), EngineIoError> {
        TcpSidecar::send(self, req)
    }
    fn try_drain_events(&self, max: usize) -> Vec<IoEvent> {
        TcpSidecar::try_drain_events(self, max)
    }
    fn is_alive(&self) -> bool {
        TcpSidecar::is_alive(self)
    }
    fn kill(&self) {
        TcpSidecar::kill(self)
    }
}

impl SidecarTransport for SidecarProcess {
    fn send(&self, req: IoRequest) -> Result<(), EngineIoError> {
        SidecarProcess::send(self, req)
    }
    fn try_drain_events(&self, max: usize) -> Vec<IoEvent> {
        SidecarProcess::try_drain_events(self, max)
    }
    fn is_alive(&self) -> bool {
        SidecarProcess::is_alive(self)
    }
    fn kill(&self) {
        SidecarProcess::kill(self)
    }
}

/// A no-op transport that discards all requests and produces no events.
/// Useful for testing and for scenes that do not use a sidecar.
pub struct NullTransport;

impl SidecarTransport for NullTransport {
    fn send(&self, _req: IoRequest) -> Result<(), EngineIoError> {
        Ok(())
    }
    fn try_drain_events(&self, _max: usize) -> Vec<IoEvent> {
        Vec::new()
    }
    fn is_alive(&self) -> bool {
        false
    }
    fn kill(&self) {}
}

fn spawn_stdin_writer(mut stdin: ChildStdin, rx: Receiver<String>) {
    thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
            if stdin.write_all(b"\n").is_err() {
                break;
            }
            if stdin.flush().is_err() {
                break;
            }
        }
    });
}

fn spawn_stdout_reader(stdout: impl std::io::Read + Send + 'static, tx: Sender<IoEvent>) {
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(ev) = serde_json::from_str::<IoEvent>(trimmed) {
                let _ = tx.send(ev);
            } else {
                // If parsing fails, still surface the raw line to keep debugging easy.
                let _ = tx.send(IoEvent::Out {
                    lines: vec![format!("[sidecar] {trimmed}")],
                });
            }
        }
    });
}

fn connect_with_retry(port: u16) -> Result<TcpStream, EngineIoError> {
    let addr = ("127.0.0.1", port);
    for _ in 0..50 {
        match TcpStream::connect(addr) {
            Ok(stream) => return Ok(stream),
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }

    Err(EngineIoError::Connect(format!(
        "timeout connecting to 127.0.0.1:{port}"
    )))
}

fn spawn_tcp_writer(mut stream: TcpStream, rx: Receiver<String>) {
    thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            if stream.write_all(line.as_bytes()).is_err() {
                break;
            }
            if stream.write_all(b"\n").is_err() {
                break;
            }
            if stream.flush().is_err() {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{IoEvent, IoRequest};

    #[test]
    fn serializes_hello_request_tag() {
        let json = serde_json::to_string(&IoRequest::Hello {
            cols: 120,
            rows: 42,
            boot_scene: true,
            difficulty: Some("I CAN EXIT VIM".to_string()),
        })
        .unwrap();
        assert!(json.contains(r#""type":"hello""#));
        assert!(json.contains(r#""cols":120"#));
        assert!(json.contains(r#""rows":42"#));
        assert!(json.contains(r#""boot_scene":true"#));
        assert!(json.contains(r#""difficulty":"I CAN EXIT VIM""#));
    }

    #[test]
    fn deserializes_screen_diff_event() {
        let raw = r#"{"type":"screen-diff","clear":true,"lines":["a","b"]}"#;
        let event: IoEvent = serde_json::from_str(raw).unwrap();
        match event {
            IoEvent::ScreenDiff { clear, lines } => {
                assert!(clear);
                assert_eq!(lines, vec!["a".to_string(), "b".to_string()]);
            }
            _ => panic!("expected screen-diff"),
        }
    }
}
