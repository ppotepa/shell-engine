//! Audio command queue and backend abstraction used by the engine's audio system.
//!
//! When audio is enabled, the engine uses an embedded [`RodioAudioBackend`] that plays
//! WAV/MP3 files directly via the system audio device — no external process needed.

use engine_core::logging;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

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

/// In-process audio backend using rodio for direct WAV/MP3 playback.
pub struct RodioAudioBackend {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<String, Sink>,
    assets: HashMap<String, PathBuf>,
    master_volume: f32,
}

impl RodioAudioBackend {
    /// Opens the default audio output device and indexes audio files under `assets_root`.
    pub fn new(assets_root: &Path) -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|e| format!("failed to open audio output: {e}"))?;

        let assets = scan_audio_assets(assets_root);
        logging::info(
            "engine.audio",
            format!(
                "rodio backend ready — {} cues indexed from '{}'",
                assets.len(),
                assets_root.display()
            ),
        );
        for (cue, path) in &assets {
            logging::debug(
                "engine.audio",
                format!("  cue '{cue}' -> {}", path.display()),
            );
        }

        Ok(Self {
            _stream: stream,
            stream_handle: handle,
            sinks: HashMap::new(),
            assets,
            master_volume: 1.0,
        })
    }
}

impl AudioBackend for RodioAudioBackend {
    fn play(&mut self, command: &AudioCommand) {
        // GC finished sinks.
        self.sinks.retain(|_, sink| !sink.empty());

        let Some(path) = self.assets.get(&command.cue) else {
            logging::warn(
                "engine.audio",
                format!("unknown audio cue '{}'", command.cue),
            );
            return;
        };

        // Stop existing playback of same cue.
        self.sinks.remove(&command.cue);

        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                logging::warn(
                    "engine.audio",
                    format!("cannot open {}: {e}", path.display()),
                );
                return;
            }
        };
        let reader = io::BufReader::new(file);
        let source = match Decoder::new(reader) {
            Ok(s) => s,
            Err(e) => {
                logging::warn(
                    "engine.audio",
                    format!("cannot decode {}: {e}", path.display()),
                );
                return;
            }
        };

        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(e) => {
                logging::warn(
                    "engine.audio",
                    format!("cannot create audio sink: {e}"),
                );
                return;
            }
        };

        let vol = command.volume.unwrap_or(1.0) * self.master_volume;
        sink.set_volume(vol);
        sink.append(source);

        logging::info(
            "engine.audio",
            format!(
                "playing cue='{}' file={} volume={:.2}",
                command.cue,
                path.display(),
                vol,
            ),
        );

        self.sinks.insert(command.cue.clone(), sink);
    }
}

/// Scan a directory (+ one level of subdirs) for WAV/MP3/OGG files.
/// Returns a map of filename stem → path.
fn scan_audio_assets(root: &Path) -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) => {
            logging::debug(
                "engine.audio",
                format!("cannot read assets dir {}: {e}", root.display()),
            );
            return map;
        }
    };

    let mut dirs_to_scan = vec![root.to_path_buf()];
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            dirs_to_scan.push(entry.path());
        }
    }

    for dir in dirs_to_scan {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "wav" | "mp3" | "ogg") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                map.insert(stem.to_string(), path);
            }
        }
    }
    map
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
    /// - `SHELL_QUEST_AUDIO=1|true|yes|on` enables embedded audio backend.
    pub fn from_env() -> Self {
        let enabled = env_flag("SHELL_QUEST_AUDIO");
        let mod_source =
            env::var("SHELL_QUEST_MOD_SOURCE").unwrap_or_else(|_| "mods/shell-quest".to_string());
        Self::from_options(enabled, &mod_source)
    }

    /// Creates an audio runtime from explicit launch options.
    ///
    /// When `enabled` is true, opens the system audio device via rodio and indexes
    /// audio assets from `<mod_source>/assets/`. Falls back to [`NullAudioBackend`]
    /// if the audio device cannot be opened.
    pub fn from_options(enabled: bool, mod_source: &str) -> Self {
        if !enabled {
            logging::info(
                "engine.audio",
                "audio disabled (pass --audio to enable)",
            );
            return Self::null();
        }

        let assets_root = PathBuf::from(format!(
            "{}/assets",
            mod_source.trim_end_matches('/')
        ));

        match RodioAudioBackend::new(&assets_root) {
            Ok(backend) => Self::with_backend(Box::new(backend)),
            Err(error) => {
                logging::warn(
                    "engine.audio",
                    format!("cannot initialize audio: {error}; using null backend"),
                );
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
