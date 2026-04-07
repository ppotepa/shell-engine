//! Run-scoped file logging shared by launcher, runtime, and editor.
//!
//! Directory layout (default):
//! `logs/<dd-mm-yy>/run-XXX/run.log`

use chrono::Local;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Log verbosity level written to the run log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO ",
            Self::Warn => "WARN ",
            Self::Error => "ERROR",
        }
    }
}

/// A single line in the log overlay buffer.
#[derive(Debug, Clone)]
pub struct LogOverlayLine {
    pub level: &'static str,
    pub target: String,
    pub message: String,
}

/// Initialization options for the process-global run logger.
#[derive(Debug, Clone)]
pub struct RunLoggerConfig {
    pub app_name: String,
    pub enabled: bool,
    pub root_dir: Option<PathBuf>,
    /// Also write log lines to stderr in real time (for console diagnostics).
    /// Automatically true when `SHELL_QUEST_CONSOLE_LOG=1` is set.
    pub also_stderr: bool,
}

/// Resolved file-system location for the current run logs.
#[derive(Debug, Clone)]
pub struct RunLogInfo {
    pub root_dir: PathBuf,
    pub day_dir: PathBuf,
    pub run_dir: PathBuf,
    pub run_number: u32,
    pub file_path: PathBuf,
}

#[derive(Debug)]
struct RunLogger {
    info: RunLogInfo,
    file: Mutex<File>,
    cached_pid: u32,
    also_stderr: bool,
}

impl RunLogger {
    fn write_line(&self, level: LogLevel, target: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!(
            "{timestamp} [{}] [{}] [{}] {message}\n",
            self.cached_pid,
            level.as_str(),
            target
        );
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            // Only flush on Warn/Error, or let OS buffer smaller Infos.
            if matches!(level, LogLevel::Warn | LogLevel::Error) {
                let _ = file.flush();
            }
        }

        if self.also_stderr {
            let stderr_line = format!("[{}] [{}] {message}\n", level.as_str(), target);
            let _ = std::io::stderr().write_all(stderr_line.as_bytes());
        }

        // Append to in-memory ring buffer for overlay
        append_to_log_ring(level, target, message);
    }
}

static LOGGER: OnceLock<RunLogger> = OnceLock::new();
static PANIC_HOOK: OnceLock<()> = OnceLock::new();
static LOG_RING: Mutex<VecDeque<LogOverlayLine>> = Mutex::new(VecDeque::new());
const LOG_RING_CAPACITY: usize = 500;

/// Appends a log line to the in-memory ring buffer.
fn append_to_log_ring(level: LogLevel, target: &str, message: &str) {
    if let Ok(mut ring) = LOG_RING.lock() {
        // Pre-check capacity and drop before allocating if needed.
        if ring.len() >= LOG_RING_CAPACITY {
            ring.pop_front();
        }
        ring.push_back(LogOverlayLine {
            level: level.as_str(),
            target: target.to_string(),
            message: message.to_string(),
        });
    }
}

/// Resolves default `enabled` state from explicit flags and environment.
///
/// Priority:
/// 1. `--no-logs` (`force_disabled`)
/// 2. `--logs` (`force_enabled`)
/// 3. debug build (`cfg!(debug_assertions)`)
/// 4. `SHELL_QUEST_LOGS=1|true|yes|on`
pub fn resolve_enabled(force_enabled: bool, force_disabled: bool) -> bool {
    if force_disabled {
        return false;
    }
    if force_enabled {
        return true;
    }
    if cfg!(debug_assertions) {
        return true;
    }
    env_flag_enabled("SHELL_QUEST_LOGS")
}

/// Initializes the process-global run logger.
///
/// Returns `Ok(None)` when logging is disabled.
/// If logger is already initialized, returns the existing [`RunLogInfo`].
pub fn init_run_logger(config: RunLoggerConfig) -> io::Result<Option<RunLogInfo>> {
    if !config.enabled {
        return Ok(None);
    }

    if let Some(existing) = LOGGER.get() {
        return Ok(Some(existing.info.clone()));
    }

    let also_stderr = config.also_stderr || env_flag_enabled("SHELL_QUEST_CONSOLE_LOG");

    let root_dir = config.root_dir.unwrap_or_else(|| PathBuf::from("logs"));
    let run_info = create_run_layout(&root_dir)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&run_info.file_path)?;

    let logger = RunLogger {
        info: run_info.clone(),
        file: Mutex::new(file),
        cached_pid: std::process::id(),
        also_stderr,
    };

    let _ = LOGGER.set(logger);
    info(
        "logging.init",
        format!(
            "run logger initialized: file={} run={} day={}",
            run_info.file_path.display(),
            run_info.run_number,
            run_info.day_dir.display()
        ),
    );
    Ok(Some(run_info))
}

/// Installs a panic hook that forwards panic diagnostics into the run log.
pub fn install_panic_hook(target: &'static str) {
    if PANIC_HOOK.set(()).is_err() {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error(target, format!("panic: {panic_info}"));
        previous(panic_info);
    }));
}

/// Returns `true` when a run logger is active for this process.
pub fn is_enabled() -> bool {
    LOGGER.get().is_some()
}

/// Returns filesystem info for the active run logger, if initialized.
pub fn run_log_info() -> Option<RunLogInfo> {
    LOGGER.get().map(|logger| logger.info.clone())
}

/// Returns the most recent N log entries from the in-memory ring buffer.
pub fn tail_recent(limit: usize) -> Vec<LogOverlayLine> {
    if let Ok(ring) = LOG_RING.lock() {
        let len = ring.len();
        let skip = len.saturating_sub(limit);
        ring.iter().skip(skip).cloned().collect()
    } else {
        Vec::new()
    }
}

/// Writes a message at the given [`LogLevel`] and `target`.
pub fn log(level: LogLevel, target: &str, message: impl AsRef<str>) {
    if let Some(logger) = LOGGER.get() {
        logger.write_line(level, target, message.as_ref());
    }
}

/// Convenience logger at `DEBUG` level.
pub fn debug(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Debug, target, message);
}

/// Convenience logger at `INFO` level.
pub fn info(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Info, target, message);
}

/// Convenience logger at `WARN` level.
pub fn warn(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Warn, target, message);
}

/// Convenience logger at `ERROR` level.
pub fn error(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Error, target, message);
}

fn create_run_layout(root_dir: &Path) -> io::Result<RunLogInfo> {
    fs::create_dir_all(root_dir)?;
    let day_dir = root_dir.join(Local::now().format("%d-%m-%y").to_string());
    fs::create_dir_all(&day_dir)?;

    let mut run_number = discover_max_run_number(&day_dir).saturating_add(1);
    let run_dir = loop {
        let candidate = day_dir.join(format!("run-{run_number:03}"));
        match fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                run_number = run_number.saturating_add(1);
            }
            Err(error) => return Err(error),
        }
    };

    let file_path = run_dir.join("run.log");
    Ok(RunLogInfo {
        root_dir: root_dir.to_path_buf(),
        day_dir,
        run_dir,
        run_number,
        file_path,
    })
}

fn discover_max_run_number(day_dir: &Path) -> u32 {
    let Ok(entries) = fs::read_dir(day_dir) else {
        return 0;
    };
    let mut max_run = 0_u32;
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if let Some(num) = parse_run_dir_name(&name) {
            max_run = max_run.max(num);
        }
    }
    max_run
}

fn parse_run_dir_name(name: &str) -> Option<u32> {
    let stripped = name.strip_prefix("run-")?;
    if stripped.is_empty() || !stripped.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    stripped.parse::<u32>().ok()
}

fn env_flag_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[allow(dead_code)]
fn _is_run_log(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(|name| name == "run.log")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{parse_run_dir_name, resolve_enabled};

    #[test]
    fn parse_run_number_from_directory_name() {
        assert_eq!(parse_run_dir_name("run-001"), Some(1));
        assert_eq!(parse_run_dir_name("run-42"), Some(42));
        assert_eq!(parse_run_dir_name("run-"), None);
        assert_eq!(parse_run_dir_name("run-x"), None);
        assert_eq!(parse_run_dir_name("x-001"), None);
    }

    #[test]
    fn resolve_enabled_prioritizes_flags() {
        assert!(resolve_enabled(true, false));
        assert!(!resolve_enabled(true, true));
        assert!(!resolve_enabled(false, true));
    }
}
