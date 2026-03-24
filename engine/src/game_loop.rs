//! Fixed-timestep game loop: polls input, ticks systems, and paces frames to the target FPS.

use crate::bench::{self, BenchmarkState, FrameSample};
use crate::error::EngineError;
use crate::events::EngineEvent;
use crate::services::EngineWorldAccess;
use crate::systems;
use crate::world::World;

/// Runs the engine game loop for `world` at `target_fps` until the player quits.
pub fn game_loop(world: &mut World, target_fps: u16) -> Result<(), EngineError> {
    use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
    use std::time::{Duration, Instant};
    use systems::scene_lifecycle::SceneLifecycleManager;

    const FAST_FORWARD_TICKS: u8 = 8;
    let mut debug_fast_forward = false;

    let _mouse_capture_guard = MouseCaptureGuard::new(should_capture_mouse(world))?;

    loop {
        // ── Benchmark: check if results phase ───────────────────────
        let bench_time_up = world
            .get::<BenchmarkState>()
            .map(|bs| bs.time_up())
            .unwrap_or(false);

        if bench_time_up {
            let should_quit = world
                .get::<BenchmarkState>()
                .map(|bs| bs.should_quit())
                .unwrap_or(false);
            if should_quit {
                break;
            }
            let already_shown = world
                .get::<BenchmarkState>()
                .map(|bs| bs.results_shown)
                .unwrap_or(true);
            if !already_shown {
                // Compute results and render the score screen.
                let results = world.get::<BenchmarkState>().unwrap().results();
                if let Some(buf) = world.buffer_mut() {
                    bench::render_bench_results(buf, &results);
                }
                // Flush the results screen to terminal.
                systems::renderer::renderer_system(world);
                if let Some(bs) = world.get_mut::<BenchmarkState>() {
                    bs.mark_results_shown();
                }
            }
            // During results display, just sleep and poll for quit keys.
            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Release { continue; }
                    if is_quit_key(key.code, key.modifiers) { break; }
                }
            }
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }

        // ── Normal frame ────────────────────────────────────────────
        let frame_start = Instant::now();
        let scene_target_fps = world
            .scene_runtime()
            .and_then(|runtime| runtime.scene().target_fps);
        let fps = resolve_target_fps(target_fps, scene_target_fps) as u64;
        let tick_ms = (1000 / fps).max(1);
        let frame_budget = Duration::from_millis(tick_ms);

        // --- INPUT ---
        let t_input_start = Instant::now();
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    if is_debug_fast_forward_toggle(key.code, key.modifiers) {
                        debug_fast_forward = !debug_fast_forward;
                        continue;
                    }

                    let ev = match key.code {
                        code if is_quit_key(code, key.modifiers) => EngineEvent::Quit,
                        _ => EngineEvent::KeyPressed(key),
                    };
                    world.events_mut().unwrap().push(ev);
                }
                Event::Mouse(mouse) => {
                    if matches!(mouse.kind, MouseEventKind::Moved) {
                        world.events_mut().unwrap().push(EngineEvent::MouseMoved {
                            column: mouse.column,
                            row: mouse.row,
                        });
                    }
                }
                Event::Resize(w, h) => {
                    world
                        .events_mut()
                        .unwrap()
                        .push(EngineEvent::TerminalResized {
                            width: w,
                            height: h,
                        });
                }
                _ => {}
            }
        }

        world.events_mut().unwrap().push(EngineEvent::Tick);
        let t_input = t_input_start.elapsed();

        // --- DRAIN EVENTS ---
        let t_lifecycle_start = Instant::now();
        let events = world.events_mut().unwrap().drain();

        let quit = SceneLifecycleManager::process_events(world, events);
        if quit {
            break;
        }
        let t_lifecycle = t_lifecycle_start.elapsed();

        // --- SYSTEMS ---
        let ticks_this_frame = if debug_fast_forward {
            FAST_FORWARD_TICKS
        } else {
            1
        };

        let t_anim_start = Instant::now();
        for _ in 0..ticks_this_frame {
            systems::animator::animator_system(world, tick_ms);
        }
        let t_anim = t_anim_start.elapsed();

        let t_hotreload_start = Instant::now();
        systems::hot_reload::debug_scene_hot_reload_system(world);
        let t_hotreload = t_hotreload_start.elapsed();

        // Bridge external sidecar IO before behaviors run (behaviors clear UI submit/change each frame).
        let t_io_start = Instant::now();
        systems::engine_io::engine_io_system(world, tick_ms);
        let t_io = t_io_start.elapsed();

        // Process transitions emitted by animator in the same frame to avoid
        // rendering one extra "done" frame that can briefly re-show sprites.
        let post_animator_events = world.events_mut().unwrap().drain();
        let quit_after_animator =
            SceneLifecycleManager::process_events(world, post_animator_events);
        if quit_after_animator {
            break;
        }

        let t0 = Instant::now();
        systems::behavior::behavior_system(world);
        let t1 = Instant::now();
        systems::audio::audio_system(world);
        let t1b = Instant::now();
        systems::compositor::compositor_system(world);
        let t2 = Instant::now();
        systems::postfx::postfx_system(world);
        let t3 = Instant::now();
        systems::renderer::renderer_system(world);
        let t4 = Instant::now();

        // Sample CPU/MEM stats (~1 Hz internally).
        if let Some(ps) = world.get_mut::<crate::debug_features::ProcessStats>() {
            ps.tick();
        }

        // Update EMA-smoothed per-system timings (same α as FPS counter).
        if let Some(st) = world.get_mut::<crate::debug_features::SystemTimings>() {
            const A: f32 = 0.15;
            st.behavior_us   = st.behavior_us   * (1.0 - A) + (t1 - t0).as_micros() as f32 * A;
            st.compositor_us = st.compositor_us * (1.0 - A) + (t2 - t1b).as_micros() as f32 * A;
            st.postfx_us     = st.postfx_us     * (1.0 - A) + (t3 - t2).as_micros() as f32 * A;
            st.renderer_us   = st.renderer_us   * (1.0 - A) + (t4 - t3).as_micros() as f32 * A;
        }

        let elapsed = frame_start.elapsed();

        // Update smoothed FPS counter (EMA, α=0.15).
        let actual_fps = if elapsed.as_micros() > 0 {
            1_000_000.0 / elapsed.as_micros() as f32
        } else {
            fps as f32
        };
        if let Some(counter) = world.get_mut::<crate::debug_features::FpsCounter>() {
            if counter.fps < 0.5 {
                counter.fps = actual_fps;
            } else {
                counter.fps = counter.fps * 0.85 + actual_fps * 0.15;
            }
        }

        let t_sleep_start = Instant::now();
        if elapsed < frame_budget {
            std::thread::sleep(frame_budget - elapsed);
        }
        let t_sleep = t_sleep_start.elapsed();

        // ── Benchmark: record frame sample ──────────────────────────
        if world.get::<BenchmarkState>().is_some() {
            let (diff_cells, dirty_cells, total_cells, write_ops) =
                if let Some(buf) = world.output_buffer() {
                    (
                        buf.last_diff_count,
                        buf.dirty_cell_count() as u32,
                        buf.total_cells() as u32,
                        buf.write_count,
                    )
                } else {
                    (0, 0, 0, 0)
                };
            if let Some(bs) = world.get_mut::<BenchmarkState>() {
                bs.push(FrameSample {
                    frame_us:      frame_start.elapsed().as_micros() as f32,
                    input_us:      t_input.as_micros() as f32,
                    lifecycle_us:  t_lifecycle.as_micros() as f32,
                    animator_us:   t_anim.as_micros() as f32,
                    hot_reload_us: t_hotreload.as_micros() as f32,
                    engine_io_us:  t_io.as_micros() as f32,
                    behavior_us:   (t1 - t0).as_micros() as f32,
                    audio_us:      (t1b - t1).as_micros() as f32,
                    compositor_us: (t2 - t1b).as_micros() as f32,
                    postfx_us:     (t3 - t2).as_micros() as f32,
                    renderer_us:   (t4 - t3).as_micros() as f32,
                    sleep_us:      t_sleep.as_micros() as f32,
                    diff_cells,
                    dirty_cells,
                    total_cells,
                    write_ops,
                });
            }
        }
    }
    Ok(())
}

struct MouseCaptureGuard {
    enabled: bool,
}

impl MouseCaptureGuard {
    fn new(enabled: bool) -> std::io::Result<Self> {
        if enabled {
            crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
        }
        Ok(Self { enabled })
    }
}

impl Drop for MouseCaptureGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        }
    }
}

#[inline]
fn should_capture_mouse(world: &World) -> bool {
    world
        .get::<crate::debug_features::DebugFeatures>()
        .map(|debug| !debug.enabled)
        .unwrap_or(true)
}

#[inline]
fn is_quit_key(code: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> bool {
    modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        && matches!(
            code,
            crossterm::event::KeyCode::Char('c')
                | crossterm::event::KeyCode::Char('C')
                | crossterm::event::KeyCode::Char('q')
                | crossterm::event::KeyCode::Char('Q')
        )
}

#[inline]
fn is_debug_fast_forward_toggle(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> bool {
    cfg!(debug_assertions)
        && modifiers.contains(crossterm::event::KeyModifiers::ALT)
        && matches!(
            code,
            crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F')
        )
}

#[inline]
fn resolve_target_fps(default_target_fps: u16, scene_target_fps: Option<u16>) -> u16 {
    scene_target_fps
        .unwrap_or(default_target_fps)
        .clamp(1, crate::terminal_caps::MAX_TARGET_FPS)
}

#[cfg(test)]
mod tests {
    use super::{
        is_debug_fast_forward_toggle, is_quit_key, resolve_target_fps, should_capture_mouse,
    };
    use crate::debug_features::DebugFeatures;
    use crate::world::World;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn detects_alt_f_combo_for_debug_fast_forward() {
        let expected = cfg!(debug_assertions);
        assert_eq!(
            is_debug_fast_forward_toggle(KeyCode::Char('f'), KeyModifiers::ALT),
            expected
        );
        assert_eq!(
            is_debug_fast_forward_toggle(KeyCode::Char('F'), KeyModifiers::ALT),
            expected
        );
    }

    #[test]
    fn ignores_non_alt_or_non_f_keys() {
        assert!(!is_debug_fast_forward_toggle(
            KeyCode::Char('f'),
            KeyModifiers::NONE
        ));
        assert!(!is_debug_fast_forward_toggle(
            KeyCode::Char('g'),
            KeyModifiers::ALT
        ));
        assert!(!is_debug_fast_forward_toggle(
            KeyCode::Esc,
            KeyModifiers::ALT
        ));
    }

    #[test]
    fn quit_key_includes_ctrl_c_and_ctrl_q() {
        assert!(is_quit_key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(is_quit_key(
            KeyCode::Char('C'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        ));
        assert!(is_quit_key(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert!(is_quit_key(
            KeyCode::Char('Q'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        ));
        assert!(!is_quit_key(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!is_quit_key(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(!is_quit_key(KeyCode::Char('c'), KeyModifiers::NONE));
    }

    #[test]
    fn scene_fps_override_has_priority() {
        assert_eq!(resolve_target_fps(60, Some(30)), 30);
    }

    #[test]
    fn fps_is_clamped_to_supported_range() {
        assert_eq!(resolve_target_fps(0, None), 1);
        assert_eq!(
            resolve_target_fps(crate::terminal_caps::MAX_TARGET_FPS + 100, None),
            crate::terminal_caps::MAX_TARGET_FPS
        );
        assert_eq!(
            resolve_target_fps(60, Some(crate::terminal_caps::MAX_TARGET_FPS + 1)),
            crate::terminal_caps::MAX_TARGET_FPS
        );
    }

    #[test]
    fn mouse_capture_is_disabled_when_debug_features_are_enabled() {
        let mut world = World::new();
        assert!(should_capture_mouse(&world));

        world.register(DebugFeatures {
            enabled: false,
            overlay_visible: false,
            overlay_mode: Default::default(),
        });
        assert!(should_capture_mouse(&world));

        let mut world_debug = World::new();
        world_debug.register(DebugFeatures {
            enabled: true,
            overlay_visible: true,
            overlay_mode: Default::default(),
        });
        assert!(!should_capture_mouse(&world_debug));
    }
}
