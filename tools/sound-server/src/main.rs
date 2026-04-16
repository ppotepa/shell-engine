use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "sound-server", about = "Shell Engine audio playback server")]
struct Cli {
    /// Emit structured ack lines for each accepted command.
    #[arg(long)]
    ack: bool,
    /// Print diagnostics to stderr.
    #[arg(long)]
    verbose: bool,
    /// Root directory to scan for audio files (WAV/MP3). Cue names are derived from filenames.
    #[arg(long, default_value = "assets")]
    assets_root: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum SoundServerRequest {
    Play { cue: String, volume: Option<f32> },
    Stop { cue: Option<String> },
    SetMaster { volume: f32 },
    Ping,
    Shutdown,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum SoundServerResponse<'a> {
    Ack { event: &'a str },
    Error { message: String },
}

/// Scan a directory (non-recursively) for audio files and build cue_name → path map.
fn scan_assets(root: &Path) -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!(
                "[sound-server] cannot read assets dir {}: {e}",
                root.display()
            );
            return map;
        }
    };

    // Also scan subdirectories one level deep (e.g. assets/audio/)
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

struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<String, Sink>,
    master_volume: f32,
}

impl AudioPlayer {
    fn new() -> Result<Self, String> {
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| format!("failed to open audio output: {e}"))?;
        Ok(Self {
            _stream: stream,
            stream_handle: handle,
            sinks: HashMap::new(),
            master_volume: 1.0,
        })
    }

    fn play(&mut self, cue: &str, path: &Path, volume: Option<f32>) -> Result<(), String> {
        // Stop existing playback of same cue
        self.sinks.remove(cue);

        let file =
            fs::File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
        let reader = io::BufReader::new(file);
        let source =
            Decoder::new(reader).map_err(|e| format!("cannot decode {}: {e}", path.display()))?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("cannot create audio sink: {e}"))?;

        let vol = volume.unwrap_or(1.0) * self.master_volume;
        sink.set_volume(vol);
        sink.append(source);

        self.sinks.insert(cue.to_string(), sink);
        Ok(())
    }

    fn stop(&mut self, cue: Option<&str>) {
        match cue {
            Some(name) => {
                self.sinks.remove(name);
            }
            None => {
                self.sinks.clear();
            }
        }
    }

    fn set_master(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        for sink in self.sinks.values() {
            sink.set_volume(self.master_volume);
        }
    }

    /// Remove sinks that have finished playing.
    fn gc(&mut self) {
        self.sinks.retain(|_, sink| !sink.empty());
    }
}

fn main() {
    let cli = Cli::parse();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    let assets = scan_assets(Path::new(&cli.assets_root));

    if cli.verbose {
        eprintln!(
            "[sound-server] started — {} audio cues indexed from '{}'",
            assets.len(),
            cli.assets_root
        );
        for (cue, path) in &assets {
            eprintln!("  {cue} -> {}", path.display());
        }
    }

    let mut player = match AudioPlayer::new() {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("[sound-server] WARNING: no audio output — {e}");
            eprintln!("[sound-server] continuing in log-only mode");
            None
        }
    };

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                if cli.verbose {
                    eprintln!("[sound-server] stdin read error: {error}");
                }
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<SoundServerRequest>(&line) {
            Ok(request) => request,
            Err(error) => {
                if cli.verbose {
                    eprintln!("[sound-server] invalid json request: {error}");
                }
                if cli.ack {
                    let _ = emit_ack(
                        &mut stdout,
                        &SoundServerResponse::Error {
                            message: format!("invalid request: {error}"),
                        },
                    );
                }
                continue;
            }
        };

        // GC finished sinks periodically
        if let Some(ref mut p) = player {
            p.gc();
        }

        match request {
            SoundServerRequest::Play { cue, volume } => {
                if cli.verbose {
                    eprintln!("[sound-server] play cue={cue} volume={volume:?}");
                }

                if let Some(path) = assets.get(&cue) {
                    if let Some(ref mut p) = player {
                        match p.play(&cue, path, volume) {
                            Ok(()) => {
                                if cli.verbose {
                                    eprintln!("[sound-server] playing {}", path.display());
                                }
                            }
                            Err(e) => {
                                eprintln!("[sound-server] playback error: {e}");
                                if cli.ack {
                                    let _ = emit_ack(
                                        &mut stdout,
                                        &SoundServerResponse::Error { message: e },
                                    );
                                }
                            }
                        }
                    }
                } else {
                    let msg = format!("unknown cue '{cue}'");
                    if cli.verbose {
                        eprintln!("[sound-server] {msg}");
                    }
                    if cli.ack {
                        let _ = emit_ack(&mut stdout, &SoundServerResponse::Error { message: msg });
                    }
                }

                if cli.ack {
                    let _ = emit_ack(&mut stdout, &SoundServerResponse::Ack { event: "play" });
                }
            }
            SoundServerRequest::Stop { cue } => {
                if let Some(ref mut p) = player {
                    p.stop(cue.as_deref());
                }
                if cli.verbose {
                    eprintln!("[sound-server] stop {:?}", cue.as_deref().unwrap_or("all"));
                }
                if cli.ack {
                    let _ = emit_ack(&mut stdout, &SoundServerResponse::Ack { event: "stop" });
                }
            }
            SoundServerRequest::SetMaster { volume } => {
                if let Some(ref mut p) = player {
                    p.set_master(volume);
                }
                if cli.verbose {
                    eprintln!("[sound-server] master volume={volume}");
                }
                if cli.ack {
                    let _ = emit_ack(
                        &mut stdout,
                        &SoundServerResponse::Ack {
                            event: "set-master",
                        },
                    );
                }
            }
            SoundServerRequest::Ping => {
                if cli.ack {
                    let _ = emit_ack(&mut stdout, &SoundServerResponse::Ack { event: "pong" });
                }
            }
            SoundServerRequest::Shutdown => {
                if cli.verbose {
                    eprintln!("[sound-server] shutdown requested");
                }
                if cli.ack {
                    let _ = emit_ack(&mut stdout, &SoundServerResponse::Ack { event: "shutdown" });
                }
                break;
            }
        }
    }

    // Drop player to stop all audio
    drop(player);

    if cli.verbose {
        eprintln!("[sound-server] stopped");
    }
}

fn emit_ack(mut out: impl Write, response: &SoundServerResponse<'_>) -> io::Result<()> {
    serde_json::to_writer(&mut out, response)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    out.write_all(b"\n")?;
    out.flush()
}
