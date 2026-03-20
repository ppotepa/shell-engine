//! Audio command queue and backend abstraction used by the engine's audio system.

use engine_core::logging;
use serde::Serialize;
use std::env;
use std::io::{self, BufWriter, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// A single audio playback request, bundling the cue name and optional volume.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioCommand {
    pub cue: String,
    pub volume: Option<f32>,
}

/// Abstracts an audio playback sink; implement this to integrate a real audio library.
pub trait AudioBackend: Send + Sync {
    fn play(&mut self, command: &AudioCommand);
}

/// A no-op [`AudioBackend`] that silently discards all commands, used in tests and headless runs.
#[derive(Default)]
pub struct NullAudioBackend;

impl AudioBackend for NullAudioBackend {
    fn play(&mut self, _command: &AudioCommand) {}
}

/// [`AudioBackend`] implementation that forwards audio commands to an external process over stdin.
///
/// The process is expected to accept one JSON object per line (JSONL).
pub struct StdIoSoundBackend {
    client: Mutex<SoundServerClient>,
    failed: AtomicBool,
}

impl StdIoSoundBackend {
    /// Spawns a sound-server process using `shell_command` and prepares a JSONL command stream.
    pub fn spawn(shell_command: impl Into<String>) -> io::Result<Self> {
        Ok(Self {
            client: Mutex::new(SoundServerClient::spawn(shell_command.into())?),
            failed: AtomicBool::new(false),
        })
    }

    fn log_once(&self, message: &str) {
        if !self.failed.swap(true, Ordering::Relaxed) {
            logging::warn("engine.audio", message);
            if !logging::is_enabled() {
                eprintln!("[audio] {message}");
            }
        }
    }
}

impl AudioBackend for StdIoSoundBackend {
    fn play(&mut self, command: &AudioCommand) {
        let mut client = match self.client.lock() {
            Ok(client) => client,
            Err(_) => {
                self.log_once("sound backend lock poisoned; disabling external audio backend");
                return;
            }
        };

        if let Err(error) = client.send(SoundServerCommand::Play {
            cue: &command.cue,
            volume: command.volume,
        }) {
            self.log_once(&format!(
                "failed to send audio command to sound-server: {error}; backend disabled"
            ));
        }
    }
}

impl Drop for StdIoSoundBackend {
    fn drop(&mut self) {
        if let Ok(mut client) = self.client.lock() {
            let _ = client.send(SoundServerCommand::Shutdown);
            client.terminate();
        }
    }
}

#[derive(Debug)]
struct SoundServerClient {
    child: Child,
    stdin: BufWriter<ChildStdin>,
}

impl SoundServerClient {
    fn spawn(shell_command: String) -> io::Result<Self> {
        let mut child = Command::new("sh")
            .arg("-lc")
            .arg(&shell_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "missing child stdin"))?;

        Ok(Self {
            child,
            stdin: BufWriter::new(stdin),
        })
    }

    fn send(&mut self, command: SoundServerCommand<'_>) -> io::Result<()> {
        serde_json::to_writer(&mut self.stdin, &command).map_err(serde_json_to_io)?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()
    }

    fn terminate(&mut self) {
        match self.child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
            Err(_) => {
                let _ = self.child.kill();
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum SoundServerCommand<'a> {
    Play { cue: &'a str, volume: Option<f32> },
    Shutdown,
}

fn serde_json_to_io(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

/// Buffers [`AudioCommand`]s each frame and flushes them to the active [`AudioBackend`].
pub struct AudioRuntime {
    backend: Box<dyn AudioBackend + Send + Sync>,
    pending: Vec<AudioCommand>,
    played: Vec<AudioCommand>,
}

impl AudioRuntime {
    /// Creates an [`AudioRuntime`] backed by [`NullAudioBackend`].
    pub fn null() -> Self {
        Self::with_backend(Box::new(NullAudioBackend))
    }

    /// Creates an [`AudioRuntime`] with the provided backend implementation.
    pub fn with_backend(backend: Box<dyn AudioBackend + Send + Sync>) -> Self {
        Self {
            backend,
            pending: Vec::new(),
            played: Vec::new(),
        }
    }

    /// Creates an audio runtime from env toggles.
    ///
    /// - `SHELL_QUEST_SOUND_SERVER=1|true|yes|on` enables external sound backend.
    /// - `SHELL_QUEST_SOUND_SERVER_CMD` overrides the shell command used to spawn the backend process.
    ///   If command is present, backend is considered enabled even when `SHELL_QUEST_SOUND_SERVER`
    ///   is missing.
    pub fn from_env() -> Self {
        let enabled = env_flag("SHELL_QUEST_SOUND_SERVER")
            || env::var("SHELL_QUEST_SOUND_SERVER_CMD").is_ok();
        let command = env::var("SHELL_QUEST_SOUND_SERVER_CMD").ok();
        Self::from_options(enabled, command)
    }

    /// Creates an audio runtime from explicit launch options.
    pub fn from_options(enabled: bool, command: Option<String>) -> Self {
        if !enabled && command.is_none() {
            return Self::null();
        }
        let command = command.unwrap_or_else(|| "cargo run -p sound-server --quiet --".to_string());
        match StdIoSoundBackend::spawn(command.clone()) {
            Ok(backend) => {
                logging::info(
                    "engine.audio",
                    format!("external sound-server backend enabled: command='{command}'"),
                );
                Self::with_backend(Box::new(backend))
            }
            Err(error) => {
                logging::warn(
                    "engine.audio",
                    format!(
                        "failed to spawn sound-server with command '{command}': {error}; using null audio backend"
                    ),
                );
                if !logging::is_enabled() {
                    eprintln!(
                        "[audio] failed to spawn sound-server with command '{command}': {error}; continuing with null audio"
                    );
                }
                Self::null()
            }
        }
    }

    /// Enqueues `command` for playback on the next [`flush`](Self::flush) call.
    pub fn queue(&mut self, command: AudioCommand) {
        self.pending.push(command);
    }

    /// Sends all pending commands to the backend and moves them to the played history.
    pub fn flush(&mut self) {
        for command in std::mem::take(&mut self.pending) {
            self.backend.play(&command);
            self.played.push(command);
        }
    }

    /// Returns the slice of commands that have already been flushed to the backend.
    pub fn played(&self) -> &[AudioCommand] {
        &self.played
    }

    /// Returns the number of commands waiting to be flushed.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

fn env_flag(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{env_flag, AudioBackend, AudioCommand, AudioRuntime};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingBackend {
        sink: Arc<Mutex<Vec<AudioCommand>>>,
    }

    impl AudioBackend for RecordingBackend {
        fn play(&mut self, command: &AudioCommand) {
            self.sink.lock().expect("sink lock").push(command.clone());
        }
    }

    #[test]
    fn null_audio_runtime_tracks_flushed_commands() {
        let mut runtime = AudioRuntime::null();
        runtime.queue(AudioCommand {
            cue: "beep".to_string(),
            volume: Some(1.0),
        });

        assert_eq!(runtime.pending_len(), 1);
        runtime.flush();

        assert_eq!(runtime.pending_len(), 0);
        assert_eq!(
            runtime.played(),
            &[AudioCommand {
                cue: "beep".to_string(),
                volume: Some(1.0)
            }]
        );
    }

    #[test]
    fn runtime_forwards_commands_to_custom_backend() {
        let sink = Arc::new(Mutex::new(Vec::<AudioCommand>::new()));
        let backend = RecordingBackend { sink: sink.clone() };
        let mut runtime = AudioRuntime::with_backend(Box::new(backend));
        runtime.queue(AudioCommand {
            cue: "alarm".to_string(),
            volume: Some(0.42),
        });

        runtime.flush();

        let recorded = sink.lock().expect("sink lock");
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].cue, "alarm");
        assert_eq!(recorded[0].volume, Some(0.42));
    }

    #[test]
    fn env_flag_parses_truthy_values() {
        std::env::set_var("SHELL_QUEST_AUDIO_TEST_FLAG", "true");
        assert!(env_flag("SHELL_QUEST_AUDIO_TEST_FLAG"));

        std::env::set_var("SHELL_QUEST_AUDIO_TEST_FLAG", "0");
        assert!(!env_flag("SHELL_QUEST_AUDIO_TEST_FLAG"));

        std::env::remove_var("SHELL_QUEST_AUDIO_TEST_FLAG");
        assert!(!env_flag("SHELL_QUEST_AUDIO_TEST_FLAG"));
    }
}
