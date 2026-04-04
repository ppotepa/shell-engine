//! Gameplay strategy traits and defaults for interchangeable simulation pieces.

use crate::components::{ParticlePhysics, ParticleThreadMode, PhysicsBody2D, Transform2D};
use crate::GameplayWorld;
use rayon::prelude::*;

/// Strategies controlling gameplay simulation behavior.
pub struct GameplayStrategies {
    pub physics: Box<dyn PhysicsIntegrationStrategy + Send + Sync>,
}

impl Default for GameplayStrategies {
    fn default() -> Self {
        Self {
            physics: Box::new(ParallelEulerIntegration::default()),
        }
    }
}

pub trait PhysicsIntegrationStrategy: Send + Sync {
    fn step(&self, world: &GameplayWorld, dt_ms: u64);
}

/// Simple semi-implicit Euler integrator with optional drag and max speed.
/// Sequential version - use ParallelEulerIntegration for multi-core.
#[derive(Default)]
pub struct SimpleEulerIntegration;

impl PhysicsIntegrationStrategy for SimpleEulerIntegration {
    fn step(&self, world: &GameplayWorld, dt_ms: u64) {
        if dt_ms == 0 {
            return;
        }
        let dt_sec = dt_ms as f32 / 1000.0;
        let ids = world.ids_with_physics();
        for id in ids {
            let Some(mut body) = world.physics(id) else {
                continue;
            };
            let Some(mut xf) = world.transform(id) else {
                continue;
            };

            body.vx += body.ax * dt_sec;
            body.vy += body.ay * dt_sec;

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

            // Persist updates back into the world
            let _ = world.set_transform(id, xf);
            let _ = world.set_physics(id, body);
        }
    }
}

/// Parallel Euler integrator using rayon for multi-core physics.
/// Batches read, computes in parallel, then batches write.
#[derive(Default)]
pub struct ParallelEulerIntegration;

impl ParallelEulerIntegration {
    /// Compute physics step for a single entity (pure function, no locks).
    #[inline]
    fn integrate_single(
        dt_sec: f32,
        mut body: PhysicsBody2D,
        mut xf: Transform2D,
        particle_physics: Option<&ParticlePhysics>,
    ) -> (PhysicsBody2D, Transform2D) {
        // Apply acceleration.
        // gravity_scale is a transient per-frame impulse — NOT stored back into body.ay.
        // thread_mode=Light means "no gravity" even if gravity_scale is set.
        const WORLD_GRAVITY_Y: f32 = 100.0;
        let extra_ay = particle_physics
            .filter(|pp| pp.gravity_scale > 0.0 && pp.thread_mode != ParticleThreadMode::Light)
            .map(|pp| WORLD_GRAVITY_Y * pp.gravity_scale)
            .unwrap_or(0.0);

        body.vx += body.ax * dt_sec;
        body.vy += (body.ay + extra_ay) * dt_sec;

        // Apply drag
        if body.drag > 0.0 {
            let drag = body.drag.clamp(0.0, 1.0);
            body.vx *= 1.0 - drag * dt_sec;
            body.vy *= 1.0 - drag * dt_sec;
        }

        // Clamp to max speed
        if body.max_speed > 0.0 {
            let speed_sq = body.vx * body.vx + body.vy * body.vy;
            let max_sq = body.max_speed * body.max_speed;
            if speed_sq > max_sq && speed_sq > 0.0 {
                let scale = (max_sq / speed_sq).sqrt();
                body.vx *= scale;
                body.vy *= scale;
            }
        }

        // Update position
        xf.x += body.vx * dt_sec;
        xf.y += body.vy * dt_sec;

        (body, xf)
    }
}

/// Minimum entity count to use parallel processing.
/// Below this, serial is faster due to rayon spawn overhead.
const PARALLEL_THRESHOLD: usize = 64;

impl PhysicsIntegrationStrategy for ParallelEulerIntegration {
    fn step(&self, world: &GameplayWorld, dt_ms: u64) {
        if dt_ms == 0 {
            return;
        }
        let dt_sec = dt_ms as f32 / 1000.0;

        // PHASE 1: Single-lock batch read (ONE lock for ALL entities)
        let work_items = world.batch_read_all_physics();
        if work_items.is_empty() {
            return;
        }

        // PHASE 2: Compute physics (parallel only above threshold)
        let results: Vec<(u64, Transform2D, PhysicsBody2D)> = if work_items.len() > PARALLEL_THRESHOLD {
            work_items
                .into_par_iter()
                .map(|(id, xf, body, pp)| {
                    let (new_body, new_xf) = Self::integrate_single(dt_sec, body, xf, pp.as_ref());
                    (id, new_xf, new_body)
                })
                .collect()
        } else {
            work_items
                .into_iter()
                .map(|(id, xf, body, pp)| {
                    let (new_body, new_xf) = Self::integrate_single(dt_sec, body, xf, pp.as_ref());
                    (id, new_xf, new_body)
                })
                .collect()
        };

        // PHASE 3: Single-lock batch write (ONE lock for ALL updates)
        world.batch_write_physics(&results);
    }
}
