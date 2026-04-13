//! Fixed-timestep game loop: polls input, ticks systems, and paces frames to the target FPS.

use crate::bench::{self, BenchmarkState, FrameSample};
use crate::error::EngineError;
use crate::events::EngineEvent;
use crate::services::EngineWorldAccess;
use crate::systems;
use crate::world::World;
use engine_events::{EngineEvent::KeyPressed, InputBackend};
use engine_mod::display_config;

/// Runs the engine game loop for `world` at `target_fps` until the player quits.
/// If `frame_capture` is provided, captures each frame after rendering.
pub fn game_loop(
    world: &mut World,
    target_fps: u16,
    input: &mut dyn InputBackend,
    frame_capture: &mut Option<crate::frame_capture::FrameCapture>,
) -> Result<(), EngineError> {
    use std::time::{Duration, Instant};
    use systems::scene_lifecycle::SceneLifecycleManager;

    const FAST_FORWARD_TICKS: u8 = 8;
    let mut debug_fast_forward = false;

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
                // Flush the results screen immediately.
                systems::renderer::renderer_system(world);
                if let Some(bs) = world.get_mut::<BenchmarkState>() {
                    bs.mark_results_shown();
                }
            }
            // During results display, just sleep and poll for quit keys.
            for event in input.poll_events() {
                if matches!(event, EngineEvent::Quit) {
                    break;
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
        if let Some(runtime) = world.scene_runtime_mut() {
            // Key scope is single-frame: clear previous key before ingesting fresh events.
            runtime.clear_last_raw_key();
        }
        for event in input.poll_events() {
            match event {
                KeyPressed(key) if is_debug_fast_forward_toggle(key.code, key.modifiers) => {
                    debug_fast_forward = !debug_fast_forward;
                }
                other => {
                    world.events_mut().unwrap().push(other);
                }
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
            if let Some(to_scene_id) = engine_animation::animator_system(world, tick_ms) {
                if let Some(queue) = world.events_mut() {
                    queue.push(EngineEvent::SceneTransition { to_scene_id });
                }
            }
        }
        let t_anim = t_anim_start.elapsed();

        let t_hotreload_start = Instant::now();
        systems::hot_reload::debug_scene_hot_reload_system(world);
        let t_hotreload = t_hotreload_start.elapsed();

        let t_io = Duration::ZERO;

        // Process transitions emitted by animator in the same frame to avoid
        // rendering one extra "done" frame that can briefly re-show sprites.
        let post_animator_events = world.events_mut().unwrap().drain();
        let quit_after_animator =
            SceneLifecycleManager::process_events(world, post_animator_events);
        if quit_after_animator {
            break;
        }

        // Gameplay systems (physics/lifetime) run before script behaviors.
        // Inline physics (thread_mode=Light + non-particle entities) runs synchronously.
        // Worker particles (Physics/Gravity) are kicked off async on rayon — they will
        // compute concurrently with the behavior system below.
        let t_phys_start = Instant::now();
        systems::gameplay::gameplay_system(world, tick_ms);
        let particle_handle = systems::particle_physics::start_async(world, tick_ms);
        let t_phys = t_phys_start.elapsed();

        // Collision detection runs after integration so positions are current.
        // Response (impulse) is applied immediately so corrected velocities persist.
        let collision_hits = systems::collision::collision_system(world);
        systems::collision::apply_collision_response(world, &collision_hits);

        let particle_hits = systems::collision::particle_collision_system(world);
        systems::collision::apply_particle_bounce(world, &particle_hits);

        systems::gameplay_events::push_collisions(world, collision_hits);
        systems::gameplay_events::push_collisions(world, particle_hits);

        let t0 = Instant::now();
        systems::behavior::behavior_system(world); // ← runs while particle_handle computes on rayon
        let t1 = Instant::now();
        systems::free_look_camera::free_look_camera_system(world, tick_ms);
        // Apply lifecycle-driven visual despawns before visual sync/compositing so
        // expired FX layers do not survive one extra frame with reset state.
        systems::visual_binding::cleanup_visuals(world);

        // Collect async particle results and write back before visual sync.
        systems::particle_physics::collect_async(world, particle_handle);
        systems::particle_ramp::particle_ramp_system(world);
        systems::audio_sequencer::audio_sequencer_system(world, tick_ms);
        engine_audio::audio_system(world);
        let t1b = Instant::now();
        // Sync Transform2D → scene positions before compositing
        systems::visual_sync::visual_sync_system(world);
        systems::compositor::compositor_system(world);
        let t2 = Instant::now();
        systems::postfx::postfx_system(world);
        let t3 = Instant::now();
        systems::renderer::renderer_system(world);
        let t4 = Instant::now();
        systems::gameplay_events::clear(world);

        // Notify frame-skip oracle that frame has advanced
        if let Some(oracle) =
            world.get::<std::sync::Mutex<Box<dyn crate::strategy::FrameSkipOracle>>>()
        {
            if let Ok(mut o) = oracle.lock() {
                let frame_id = world.animator().map(|a| a.step_idx).unwrap_or(0) as u64;
                o.frame_advanced(frame_id, false);
            }
        }

        // Capture frame if capture mode is active
        if let Some(capture) = frame_capture {
            if let Some(buf) = world.output_buffer() {
                capture.capture(buf)?;
            }
        }

        // Sample CPU/MEM stats (~1 Hz internally).
        if let Some(ps) = world.get_mut::<crate::debug_features::ProcessStats>() {
            ps.tick();
        }

        let elapsed = frame_start.elapsed();
        let t_sleep_start = Instant::now();
        if elapsed < frame_budget {
            std::thread::sleep(frame_budget - elapsed);
        }
        let t_sleep = t_sleep_start.elapsed();
        let frame_elapsed = frame_start.elapsed();

        // Update EMA-smoothed per-system timings (α=0.15).
        if let Some(st) = world.get_mut::<crate::debug_features::SystemTimings>() {
            const A: f32 = 0.15;
            st.physics_us = st.physics_us * (1.0 - A) + t_phys.as_micros() as f32 * A;
            st.behavior_us = st.behavior_us * (1.0 - A) + (t1 - t0).as_micros() as f32 * A;
            st.compositor_us = st.compositor_us * (1.0 - A) + (t2 - t1b).as_micros() as f32 * A;
            st.postfx_us = st.postfx_us * (1.0 - A) + (t3 - t2).as_micros() as f32 * A;
            st.renderer_us = st.renderer_us * (1.0 - A) + (t4 - t3).as_micros() as f32 * A;
            st.sleep_us = st.sleep_us * (1.0 - A) + t_sleep.as_micros() as f32 * A;
            st.frame_us = st.frame_us * (1.0 - A) + frame_elapsed.as_micros() as f32 * A;
        }

        // Update smoothed FPS counter (EMA, α=0.15) using full frame time
        // including pacing sleep so the visible FPS matches what players see.
        let actual_fps = if frame_elapsed.as_micros() > 0 {
            1_000_000.0 / frame_elapsed.as_micros() as f32
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

        // ── Benchmark: record frame sample ──────────────────────────
        if world.get::<BenchmarkState>().is_some() {
            let scene_id = world
                .scene_runtime()
                .map(|r| r.scene().id.clone())
                .unwrap_or_default();
            let (diff_cells, dirty_cells, total_cells, write_ops) =
                if let Some(buf) = world.output_buffer() {
                    (
                        buf.last_diff_count,
                        buf.dirty_cell_count(),
                        buf.total_cells(),
                        buf.write_count,
                    )
                } else {
                    (0, 0, 0, 0)
                };
            if let Some(bs) = world.get_mut::<BenchmarkState>() {
                bs.push(FrameSample {
                    scene_id,
                    frame_us: frame_start.elapsed().as_micros() as f32,
                    input_us: t_input.as_micros() as f32,
                    lifecycle_us: t_lifecycle.as_micros() as f32,
                    animator_us: t_anim.as_micros() as f32,
                    hot_reload_us: t_hotreload.as_micros() as f32,
                    engine_io_us: t_io.as_micros() as f32,
                    behavior_us: (t1 - t0).as_micros() as f32,
                    audio_us: (t1b - t1).as_micros() as f32,
                    compositor_us: (t2 - t1b).as_micros() as f32,
                    postfx_us: (t3 - t2).as_micros() as f32,
                    renderer_us: (t4 - t3).as_micros() as f32,
                    sleep_us: t_sleep.as_micros() as f32,
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

fn is_debug_fast_forward_toggle(
    code: engine_events::KeyCode,
    modifiers: engine_events::KeyModifiers,
) -> bool {
    modifiers.contains(engine_events::KeyModifiers::CONTROL) && code == engine_events::KeyCode::F(5)
}

#[inline]
fn resolve_target_fps(default_target_fps: u16, scene_target_fps: Option<u16>) -> u16 {
    scene_target_fps
        .unwrap_or(default_target_fps)
        .clamp(1, display_config::MAX_TARGET_FPS)
}

#[cfg(test)]
mod tests {
    use super::resolve_target_fps;
    use engine_mod::display_config;

    #[test]
    fn scene_fps_override_has_priority() {
        assert_eq!(resolve_target_fps(60, Some(30)), 30);
    }

    #[test]
    fn fps_is_clamped_to_supported_range() {
        assert_eq!(resolve_target_fps(0, None), 1);
        assert_eq!(
            resolve_target_fps(display_config::MAX_TARGET_FPS + 100, None),
            display_config::MAX_TARGET_FPS
        );
        assert_eq!(
            resolve_target_fps(60, Some(display_config::MAX_TARGET_FPS + 1)),
            display_config::MAX_TARGET_FPS
        );
    }
}
