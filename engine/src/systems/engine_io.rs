//! Engine-side integration for `engine-io`.
//!
//! This system is intentionally thin: it forwards terminal input events to an external sidecar
//! process and applies sidecar events back onto the scene runtime (terminal transcript + prompt).

use crate::debug_log::DebugLogBuffer;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_core::logging;
use engine_core::scene::TerminalShellMode;
use engine_io::{IoEvent, IoRequest, TcpSidecar};
use std::net::TcpListener;

#[derive(Default)]
pub struct EngineIoRuntime {
    sidecar: Option<TcpSidecar>,
    last_submit_seq: u64,
    last_change_seq: u64,
    last_key_sent: Option<String>,
    last_size: Option<(u16, u16)>,
    scene_id: Option<String>,
    /// Queue of lines waiting to be emitted with their scheduled engine-io time.
    delayed_lines: Vec<(u64, String)>,
    /// Monotonic time used to schedule delayed sidecar output.
    accumulated_ms: u64,
}

pub fn engine_io_system(world: &mut World, dt_ms: u64) {
    // Ensure runtime exists.
    if world.get::<EngineIoRuntime>().is_none() {
        world.register(EngineIoRuntime::default());
    }

    // Snapshot data we need without holding long-lived borrows.
    let (
        scene_id,
        controls,
        submit_snapshot,
        change_snapshot,
        key_snapshot,
        is_boot_scene,
        difficulty_label,
    ) = {
        let Some(scene_runtime) = world.scene_runtime() else {
            return;
        };
        let scene_id = scene_runtime.scene().id.clone();
        let Some(controls) = scene_runtime.terminal_shell_controls_snapshot() else {
            return;
        };
        let submit_snapshot = scene_runtime.ui_last_submit_snapshot();
        let change_snapshot = scene_runtime.ui_last_change_snapshot();
        let key_snapshot = scene_runtime.last_raw_key_snapshot();
        let is_boot_scene = scene_runtime
            .terminal_shell_controls_snapshot()
            .map(|c| c.boot_scene)
            .unwrap_or(false);
        let difficulty_label = world
            .get::<crate::game_state::GameState>()
            .and_then(|gs| gs.get("/game/difficulty"))
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        (
            scene_id,
            controls,
            submit_snapshot,
            change_snapshot,
            key_snapshot,
            is_boot_scene,
            difficulty_label,
        )
    };

    if controls.mode != TerminalShellMode::Sidecar {
        return;
    }

    let cwd = world
        .asset_root()
        .map(|root| root.mod_source().to_path_buf());

    let buf_size = world.buffer().map(|buf| {
        let rows = (controls.max_lines.max(1) as u16).min(buf.height);
        (buf.width, rows)
    });

    // Drive sidecar and collect events to apply.
    let mut pending_events: Vec<IoEvent> = Vec::new();
    let mut pending_lines: Vec<String> = Vec::new();
    let mut ipc_errors: Vec<String> = Vec::new();

    {
        let Some(runtime) = world.get_mut::<EngineIoRuntime>() else {
            return;
        };
        runtime.accumulated_ms = runtime.accumulated_ms.saturating_add(dt_ms);

        if runtime.scene_id.as_deref() != Some(scene_id.as_str()) {
            if let Some(sidecar) = runtime.sidecar.take() {
                sidecar.kill();
            }
            runtime.last_submit_seq = 0;
            runtime.last_change_seq = 0;
            runtime.last_key_sent = None;
            runtime.last_size = None;
            runtime.scene_id = Some(scene_id.clone());
            runtime.delayed_lines.clear();
            runtime.accumulated_ms = 0;
        }

        if runtime.sidecar.as_ref().is_some_and(|p| !p.is_alive()) {
            runtime.sidecar = None;
            runtime.last_submit_seq = 0;
            runtime.last_change_seq = 0;
            runtime.last_key_sent = None;
            runtime.last_size = None;
            runtime.delayed_lines.clear();
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
                    let mut sidecar_args = sidecar_cfg.args.clone();
                    let port = reserve_tcp_port();
                    if sidecar_cfg.command == "dotnet" && !sidecar_args.iter().any(|a| a == "--") {
                        sidecar_args.push("--".to_string());
                    }
                    sidecar_args.push("--game-port".to_string());
                    sidecar_args.push(port.to_string());

                    match TcpSidecar::spawn(
                        &sidecar_cfg.command,
                        &sidecar_args,
                        cwd.as_deref(),
                        port,
                    ) {
                        Ok(proc) => {
                            runtime.sidecar = Some(proc);
                            runtime.last_submit_seq = 0;
                            runtime.last_change_seq = 0;
                            runtime.last_key_sent = None;
                            runtime.last_size = None;
                            if let Some(sidecar) = runtime.sidecar.as_ref() {
                                if let Some((cols, rows)) = buf_size {
                                    if let Err(e) = sidecar.send(IoRequest::Hello {
                                        cols,
                                        rows,
                                        boot_scene: is_boot_scene,
                                        difficulty: difficulty_label.clone(),
                                    }) {
                                        ipc_errors
                                            .push(format!("[engine-io] hello send failed: {e}"));
                                    }
                                    runtime.last_size = Some((cols, rows));
                                }
                            }
                        }
                        Err(err) => {
                            pending_lines.push(format!("[engine-io] sidecar spawn failed: {err}"));
                        }
                    }
                }
            }
        }

        if let Some(sidecar) = runtime.sidecar.as_ref() {
            if let Some((cols, rows)) = buf_size {
                if runtime.last_size != Some((cols, rows)) {
                    if let Err(e) = sidecar.send(IoRequest::Resize { cols, rows }) {
                        ipc_errors.push(format!("[engine-io] resize send failed: {e}"));
                    }
                    runtime.last_size = Some((cols, rows));
                }
            }

            if let Err(e) = sidecar.send(IoRequest::Tick { dt_ms }) {
                ipc_errors.push(format!("[engine-io] tick send failed: {e}"));
            }

            if let Some((seq, _target, text)) = submit_snapshot {
                if seq != 0 && seq != runtime.last_submit_seq {
                    runtime.last_submit_seq = seq;
                    if let Err(e) = sidecar.send(IoRequest::Submit { line: text }) {
                        ipc_errors.push(format!("[engine-io] submit send failed: {e}"));
                    }
                }
            }
            if let Some((seq, _target, text)) = change_snapshot {
                if seq != 0 && seq != runtime.last_change_seq {
                    runtime.last_change_seq = seq;
                    if let Err(e) = sidecar.send(IoRequest::SetInput { text }) {
                        ipc_errors.push(format!("[engine-io] set-input send failed: {e}"));
                    }
                }
            }

            if let Some(key) = key_snapshot {
                let key_id = format!("{}:{}:{}:{}", key.code, key.ctrl, key.alt, key.shift);
                if runtime.last_key_sent.as_deref() != Some(&key_id) {
                    runtime.last_key_sent = Some(key_id);
                    if let Err(e) = sidecar.send(IoRequest::Key {
                        code: key.code,
                        ctrl: key.ctrl,
                        alt: key.alt,
                        shift: key.shift,
                    }) {
                        ipc_errors.push(format!("[engine-io] key send failed: {e}"));
                    }
                }
            } else {
                runtime.last_key_sent = None;
            }

            for ev in sidecar.try_drain_events(64) {
                match ev {
                    IoEvent::EmitLine { text, delay_ms } => {
                        let due_at = runtime
                            .accumulated_ms
                            .saturating_add(delay_ms.unwrap_or(0));
                        runtime.delayed_lines.push((due_at, text));
                    }
                    other => pending_events.push(other),
                }
            }
        }

        if !runtime.delayed_lines.is_empty() {
            runtime.delayed_lines.sort_by_key(|(due_at, _)| *due_at);
            let ready_count = runtime
                .delayed_lines
                .iter()
                .take_while(|(due_at, _)| *due_at <= runtime.accumulated_ms)
                .count();
            if ready_count > 0 {
                pending_lines.extend(
                    runtime
                        .delayed_lines
                        .drain(..ready_count)
                        .map(|(_, line)| line),
                );
            }
        }
    }

    // Log IPC errors to both file log and debug overlay buffer.
    if !ipc_errors.is_empty() {
        for msg in &ipc_errors {
            logging::warn("engine.io", msg);
        }
        if let Some(log) = world.get_mut::<DebugLogBuffer>() {
            for msg in ipc_errors {
                log.push_warn("io", None, None, msg);
            }
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

fn reserve_tcp_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("reserve sidecar TCP port")
        .local_addr()
        .expect("resolve sidecar TCP port")
        .port()
}

fn apply_event(scene_runtime: &mut crate::scene_runtime::SceneRuntime, ev: IoEvent) {
    match ev {
        IoEvent::Out { lines } => {
            for line in lines {
                scene_runtime.terminal_push_output(line);
            }
        }
        IoEvent::EmitLine { text, .. } => {
            scene_runtime.terminal_push_output(text);
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
            scene_runtime.sidecar_mark_screen_full(lines);
        }
        IoEvent::Custom { payload } => {
            scene_runtime.sidecar_push_custom_event(payload.to_string());
            scene_runtime.terminal_push_output(format!("[sidecar-event] {payload}"));
        }
    }
}
