//! `engine-io` is a transport-agnostic bridge between the game engine and external sidecar apps.
//!
//! This crate intentionally contains no game logic and no knowledge of Shell Quest scenes.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Mutex;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum IoRequest {
    Hello { cols: u16, rows: u16 },
    Submit { line: String },
    Key { code: String, ctrl: bool, alt: bool, shift: bool },
    Resize { cols: u16, rows: u16 },
    Tick { dt_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum IoEvent {
    Out { lines: Vec<String> },
    Clear,
    SetPromptPrefix { text: String },
    SetPromptMasked { masked: bool },
    // Incremental buffer update for sidecars that prefer diff-style terminal output.
    ScreenDiff { clear: bool, lines: Vec<String> },
    // Reserved for fullscreen apps (vi-like) — engine can map this to a sprite or compositor layer.
    ScreenFull { lines: Vec<String>, cursor_x: u16, cursor_y: u16 },
    Custom { payload: JsonValue },
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
}

/// Handle to a running sidecar process.
///
/// Writes requests as JSON lines to stdin and parses JSON lines from stdout.
pub struct SidecarProcess {
    child: Mutex<Child>,
    stdin_tx: Sender<String>,
    event_rx: Mutex<Receiver<IoEvent>>,
}

impl SidecarProcess {
    pub fn spawn(command: &str, args: &[String], cwd: Option<&std::path::Path>) -> Result<Self, EngineIoError> {
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
        for line in reader.lines().flatten() {
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

#[cfg(test)]
mod tests {
    use super::{IoEvent, IoRequest};

    #[test]
    fn serializes_hello_request_tag() {
        let json = serde_json::to_string(&IoRequest::Hello { cols: 120, rows: 42 }).unwrap();
        assert!(json.contains(r#""type":"hello""#));
        assert!(json.contains(r#""cols":120"#));
        assert!(json.contains(r#""rows":42"#));
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
