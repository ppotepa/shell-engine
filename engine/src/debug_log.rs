//! Engine-owned diagnostic ring buffer for script and runtime errors.
//!
//! In debug/dev mode, entries are surfaced in the debug overlay so script
//! failures are visible inside the running game instead of producing a silent
//! black screen.

/// Severity level for a diagnostic entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSeverity {
    Info,
    Warn,
    Error,
}

impl DebugSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            DebugSeverity::Info => "INFO",
            DebugSeverity::Warn => "WARN",
            DebugSeverity::Error => "ERR ",
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, DebugSeverity::Error)
    }
}

/// A single diagnostic entry.
#[derive(Debug, Clone)]
pub struct DebugLogEntry {
    pub severity: DebugSeverity,
    /// Subsystem tag, e.g. "rhai", "scene", "renderer".
    pub subsystem: &'static str,
    /// Scene id that was active when the entry was produced, if known.
    pub scene_id: Option<String>,
    /// Script source path (e.g. `./scene.rhai`) or `None` for non-script entries.
    pub source: Option<String>,
    /// Human-readable message, including line/col when available.
    pub message: String,
}

impl DebugLogEntry {
    /// Format as a single display line suitable for the debug overlay.
    pub fn display_line(&self) -> String {
        let scene = self.scene_id.as_deref().unwrap_or("-");
        let src = self.source.as_deref().unwrap_or("");
        if src.is_empty() {
            format!("[{}] {} | {}", self.severity.label(), scene, self.message)
        } else {
            format!(
                "[{}] {} | {} | {}",
                self.severity.label(),
                scene,
                src,
                self.message
            )
        }
    }
}

/// Ring buffer holding the most recent diagnostic entries.
///
/// Thread-safety is not required — the engine runs on a single game thread.
#[derive(Debug, Default)]
pub struct DebugLogBuffer {
    entries: Vec<DebugLogEntry>,
    cap: usize,
    /// Set to `true` whenever at least one error-severity entry has been pushed.
    pub has_errors: bool,
    /// The message of the latest error-severity entry, if any.
    pub last_error: Option<String>,
}

impl DebugLogBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            entries: Vec::with_capacity(cap.min(256)),
            cap,
            has_errors: false,
            last_error: None,
        }
    }

    pub fn push(&mut self, entry: DebugLogEntry) {
        if entry.severity.is_error() {
            self.has_errors = true;
            self.last_error = Some(entry.display_line());
        }
        if self.entries.len() >= self.cap {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Push an error entry with the given fields.
    pub fn push_error(
        &mut self,
        subsystem: &'static str,
        scene_id: Option<String>,
        source: Option<String>,
        message: String,
    ) {
        self.push(DebugLogEntry {
            severity: DebugSeverity::Error,
            subsystem,
            scene_id,
            source,
            message,
        });
    }

    /// Push an info entry.
    pub fn push_info(
        &mut self,
        subsystem: &'static str,
        scene_id: Option<String>,
        source: Option<String>,
        message: String,
    ) {
        self.push(DebugLogEntry {
            severity: DebugSeverity::Info,
            subsystem,
            scene_id,
            source,
            message,
        });
    }

    /// Push a warning entry.
    pub fn push_warn(
        &mut self,
        subsystem: &'static str,
        scene_id: Option<String>,
        source: Option<String>,
        message: String,
    ) {
        self.push(DebugLogEntry {
            severity: DebugSeverity::Warn,
            subsystem,
            scene_id,
            source,
            message,
        });
    }

    /// Returns the N most recent entries, newest last.
    pub fn recent(&self, n: usize) -> &[DebugLogEntry] {
        let len = self.entries.len();
        if len <= n {
            &self.entries
        } else {
            &self.entries[len - n..]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.has_errors = false;
        self.last_error = None;
    }
}
