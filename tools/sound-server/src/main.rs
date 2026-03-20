use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(
    name = "sound-server",
    about = "Shell Quest prototype audio command server"
)]
struct Cli {
    /// Emit structured ack lines for each accepted command.
    #[arg(long)]
    ack: bool,
    /// Print diagnostics to stderr.
    #[arg(long)]
    verbose: bool,
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

fn main() {
    let cli = Cli::parse();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    let mut active_cues: HashMap<String, Option<f32>> = HashMap::new();
    let mut master_volume: f32 = 1.0;

    if cli.verbose {
        eprintln!("[sound-server] started (prototype)");
    }

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

        match request {
            SoundServerRequest::Play { cue, volume } => {
                active_cues.insert(cue.clone(), volume);
                if cli.verbose {
                    eprintln!(
                        "[sound-server] play cue={cue} volume={:?} master={master_volume}",
                        volume
                    );
                }
                if cli.ack {
                    let _ = emit_ack(&mut stdout, &SoundServerResponse::Ack { event: "play" });
                }
            }
            SoundServerRequest::Stop { cue } => {
                match cue {
                    Some(cue) => {
                        active_cues.remove(&cue);
                        if cli.verbose {
                            eprintln!("[sound-server] stop cue={cue}");
                        }
                    }
                    None => {
                        active_cues.clear();
                        if cli.verbose {
                            eprintln!("[sound-server] stop all cues");
                        }
                    }
                }
                if cli.ack {
                    let _ = emit_ack(&mut stdout, &SoundServerResponse::Ack { event: "stop" });
                }
            }
            SoundServerRequest::SetMaster { volume } => {
                master_volume = volume.clamp(0.0, 1.0);
                if cli.verbose {
                    eprintln!("[sound-server] set master volume={master_volume}");
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
