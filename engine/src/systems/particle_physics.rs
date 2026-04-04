//! Async particle physics system.
//!
//! Particles with `thread_mode = Physics | Gravity` are extracted from the world,
//! computed on rayon's global threadpool, and written back — CONCURRENTLY with the
//! behavior system running on the main thread.
//!
//! Timeline per frame:
//! ```text
//! main:  [extract] → [collision] → [behavior system ............] → [collect+write] → [visual_sync]
//! rayon:              [particle physics compute ................]
//! ```

use std::sync::mpsc::{self, Receiver};

use engine_game::components::{ParticlePhysics, ParticleThreadMode, PhysicsBody2D, Transform2D};
use engine_game::GameplayWorld;
use rayon::prelude::*;

/// Opaque handle returned by `start_async`. Pass to `collect_async` to write results.
pub struct ParticlePhysicsHandle {
    rx: Receiver<Vec<(u64, Transform2D, PhysicsBody2D)>>,
}

/// Extract worker-thread particle data and spawn computation on rayon.
/// Returns None if there are no worker particles this frame.
pub fn start_async(
    world: &engine_core::world::World,
    dt_ms: u64,
) -> Option<ParticlePhysicsHandle> {
    if dt_ms == 0 {
        return None;
    }
    let gameplay = world.get::<GameplayWorld>()?;
    let work_items = gameplay.batch_read_worker_physics();
    if work_items.is_empty() {
        return None;
    }

    let (tx, rx) = mpsc::channel();
    let dt_sec = dt_ms as f32 / 1000.0;

    // Spawn on rayon's global threadpool — returns immediately, runs concurrently.
    rayon::spawn(move || {
        let results: Vec<(u64, Transform2D, PhysicsBody2D)> = if work_items.len() > 32 {
            work_items
                .into_par_iter()
                .map(|(id, xf, body, pp)| {
                    let (new_body, new_xf) = integrate_particle(dt_sec, body, xf, &pp);
                    (id, new_xf, new_body)
                })
                .collect()
        } else {
            work_items
                .into_iter()
                .map(|(id, xf, body, pp)| {
                    let (new_body, new_xf) = integrate_particle(dt_sec, body, xf, &pp);
                    (id, new_xf, new_body)
                })
                .collect()
        };
        let _ = tx.send(results);
    });

    Some(ParticlePhysicsHandle { rx })
}

/// Collect async particle results and write back to world. Call before visual_sync.
/// If handle is None (no worker particles this frame), this is a no-op.
pub fn collect_async(world: &engine_core::world::World, handle: Option<ParticlePhysicsHandle>) {
    let Some(h) = handle else { return };
    let Some(gameplay) = world.get::<GameplayWorld>() else { return };

    // recv() blocks until rayon finishes — by this point behavior system has run
    // so the overlap window has already been used productively.
    if let Ok(results) = h.rx.recv() {
        gameplay.batch_write_physics(&results);
    }
}

/// Physics integration for a single worker particle.
#[inline]
fn integrate_particle(
    dt_sec: f32,
    mut body: PhysicsBody2D,
    mut xf: Transform2D,
    pp: &ParticlePhysics,
) -> (PhysicsBody2D, Transform2D) {
    const WORLD_GRAVITY_Y: f32 = 100.0;

    // Light mode never reaches here (filtered at extract stage), but guard anyway.
    let extra_ay = if pp.thread_mode != ParticleThreadMode::Light && pp.gravity_scale > 0.0 {
        WORLD_GRAVITY_Y * pp.gravity_scale
    } else {
        0.0
    };

    body.vx += body.ax * dt_sec;
    body.vy += (body.ay + extra_ay) * dt_sec;

    if body.drag > 0.0 {
        let drag = body.drag.clamp(0.0, 1.0);
        body.vx *= 1.0 - drag * dt_sec;
        body.vy *= 1.0 - drag * dt_sec;
    }

    if body.max_speed > 0.0 {
        let speed_sq = body.vx * body.vx + body.vy * body.vy;
        let max_sq = body.max_speed * body.max_speed;
        if speed_sq > max_sq && speed_sq > 0.0 {
            let scale = (max_sq / speed_sq).sqrt();
            body.vx *= scale;
            body.vy *= scale;
        }
    }

    xf.x += body.vx * dt_sec;
    xf.y += body.vy * dt_sec;

    (body, xf)
}
