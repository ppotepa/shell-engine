//! Engine-side integration for `engine-io`.
//!
//! This system is intentionally thin: it forwards terminal input events to an external sidecar
//! process and applies sidecar events back onto the scene runtime (terminal transcript + prompt).

use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_core::scene::TerminalShellMode;
use engine_io::{IoEvent, IoRequest, SidecarProcess};

#[derive(Default)]
pub struct EngineIoRuntime {
    sidecar: Option<SidecarProcess>,
    last_submit_seq: u64,
    last_key_sent: Option<String>,
    last_size: Option<(u16, u16)>,
}

pub fn engine_io_system(world: &mut World, dt_ms: u64) {
    // Ensure runtime exists.
    if world.get::<EngineIoRuntime>().is_none() {
        world.register(EngineIoRuntime::default());
    }

    // Snapshot data we need without holding long-lived borrows.
    let (controls, submit_snapshot, key_snapshot) = {
        let Some(scene_runtime) = world.scene_runtime() else {
            return;
        };
        let Some(controls) = scene_runtime.terminal_shell_controls_snapshot() else {
            return;
        };
        let submit_snapshot = scene_runtime.ui_last_submit_snapshot();
        let key_snapshot = scene_runtime.last_raw_key_snapshot();
        (controls, submit_snapshot, key_snapshot)
    };

    if controls.mode != TerminalShellMode::Sidecar {
        return;
    }

    let cwd = world
        .asset_root()
        .map(|root| root.mod_source().to_path_buf());

    let buf_size = world.buffer().map(|buf| (buf.width, buf.height));

    // Drive sidecar and collect events to apply.
    let mut pending_events: Vec<IoEvent> = Vec::new();
    let mut pending_lines: Vec<String> = Vec::new();

    {
        let Some(runtime) = world.get_mut::<EngineIoRuntime>() else {
            return;
        };

        if runtime.sidecar.as_ref().is_some_and(|p| !p.is_alive()) {
            runtime.sidecar = None;
            runtime.last_submit_seq = 0;
            runtime.last_key_sent = None;
            runtime.last_size = None;
            pending_lines.push("[engine-io] sidecar exited".to_string());
        }

        if runtime.sidecar.is_none() {
            match controls.sidecar.clone() {
                None => {
                    pending_lines.push(
                        "[engine-io] terminal-shell mode=sidecar but no sidecar config".to_string(),
                    );
                }
                Some(sidecar_cfg) => {
                    match SidecarProcess::spawn(
                        &sidecar_cfg.command,
                        &sidecar_cfg.args,
                        cwd.as_deref(),
                    ) {
                        Ok(proc) => {
                            runtime.sidecar = Some(proc);
                            runtime.last_submit_seq = 0;
                            runtime.last_key_sent = None;
                            runtime.last_size = None;
                            if let Some(sidecar) = runtime.sidecar.as_ref() {
                                if let Some((cols, rows)) = buf_size {
                                    let _ = sidecar.send(IoRequest::Hello { cols, rows });
                                    runtime.last_size = Some((cols, rows));
                                }
                            }
                        }
                        Err(err) => {
                            pending_lines
                                .push(format!("[engine-io] sidecar spawn failed: {err}"));
                        }
                    }
                }
            }
        }

        if let Some(sidecar) = runtime.sidecar.as_ref() {
            if let Some((cols, rows)) = buf_size {
                if runtime.last_size != Some((cols, rows)) {
                    let _ = sidecar.send(IoRequest::Resize { cols, rows });
                    runtime.last_size = Some((cols, rows));
                }
            }

            let _ = sidecar.send(IoRequest::Tick { dt_ms });

            if let Some((seq, _target, text)) = submit_snapshot {
                if seq != 0 && seq != runtime.last_submit_seq {
                    runtime.last_submit_seq = seq;
                    let _ = sidecar.send(IoRequest::Submit { line: text });
                }
            }

            if let Some(key) = key_snapshot {
                let key_id = format!("{}:{}:{}:{}", key.code, key.ctrl, key.alt, key.shift);
                if runtime.last_key_sent.as_deref() != Some(&key_id) {
                    runtime.last_key_sent = Some(key_id);
                    let _ = sidecar.send(IoRequest::Key {
                        code: key.code,
                        ctrl: key.ctrl,
                        alt: key.alt,
                        shift: key.shift,
                    });
                }
            } else {
                runtime.last_key_sent = None;
            }

            pending_events.extend(sidecar.try_drain_events(64));
        }
    }

    // Apply events/lines onto runtime.
    if pending_events.is_empty() && pending_lines.is_empty() {
        return;
    }

    let Some(scene_runtime) = world.scene_runtime_mut() else {
        return;
    };

    for line in pending_lines {
        scene_runtime.terminal_push_output(line);
    }
    for ev in pending_events {
        apply_event(scene_runtime, ev);
    }
}

fn apply_event(scene_runtime: &mut crate::scene_runtime::SceneRuntime, ev: IoEvent) {
    match ev {
        IoEvent::Out { lines } => {
            for line in lines {
                scene_runtime.terminal_push_output(line);
            }
        }
        IoEvent::Clear => {
            scene_runtime.terminal_clear_output();
        }
        IoEvent::SetPromptPrefix { text } => {
            scene_runtime.terminal_set_prompt_prefix(text);
        }
        IoEvent::SetPromptMasked { masked } => {
            scene_runtime.terminal_set_prompt_masked(masked);
        }
        IoEvent::ScreenDiff { clear, lines } => {
            if clear {
                scene_runtime.terminal_clear_output();
            }
            for line in lines {
                scene_runtime.terminal_push_output(line);
            }
        }
        IoEvent::ScreenFull { lines, .. } => {
            // MVP: just dump to transcript so it’s visible even before a proper fullscreen compositor.
            scene_runtime.sidecar_mark_screen_full(lines.clone());
            scene_runtime.terminal_push_output("[fullscreen]".to_string());
            for line in lines {
                scene_runtime.terminal_push_output(line);
            }
        }
        IoEvent::Custom { payload } => {
            scene_runtime.sidecar_push_custom_event(payload.to_string());
            scene_runtime.terminal_push_output(format!("[sidecar-event] {payload}"));
        }
    }
}
