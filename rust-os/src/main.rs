use engine_io::{IoEvent, IoRequest};
use rust_os::difficulty::{Difficulty, MachineSpec};
use rust_os::AppHost;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    let mut host: Option<AppHost> = None;

    let reader = BufReader::new(stdin.lock());
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let req: IoRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                let err = serde_json::to_string(&IoEvent::Out {
                    lines: vec![format!("[rust-os] parse error: {e}")],
                })
                .unwrap_or_default();
                let _ = writeln!(out, "{err}");
                let _ = out.flush();
                continue;
            }
        };

        // Initialize host on Hello
        if let IoRequest::Hello {
            ref difficulty,
            cols: _,
            rows: _,
            ..
        } = req
        {
            let diff = difficulty
                .as_deref()
                .map(Difficulty::from_label)
                .unwrap_or(Difficulty::ICanExitVim);
            let spec = MachineSpec::from_difficulty(diff);
            host = Some(AppHost::new(spec));
        }

        if let Some(h) = &mut host {
            let events = h.handle(req);
            for ev in events {
                if let Ok(json) = serde_json::to_string(&ev) {
                    let _ = writeln!(out, "{json}");
                }
            }
            let _ = out.flush();
        }
    }
}
