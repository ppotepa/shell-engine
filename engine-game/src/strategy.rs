//! Gameplay strategy traits and defaults for interchangeable simulation pieces.

use crate::components::{ParticlePhysics, PhysicsBody2D, Transform2D};
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
        // Apply world gravity if particle has gravity_scale > 0
        if let Some(pp) = particle_physics {
            if pp.gravity_scale > 0.0 {
                // Standard gravity: 9.81 m/s² → scaled to ~100 pixels/s² for terminal
                const WORLD_GRAVITY_Y: f32 = 100.0;
                body.ay += WORLD_GRAVITY_Y * pp.gravity_scale;
            }
        }

        // Apply acceleration
        body.vx += body.ax * dt_sec;
        body.vy += body.ay * dt_sec;

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

impl PhysicsIntegrationStrategy for ParallelEulerIntegration {
    fn step(&self, world: &GameplayWorld, dt_ms: u64) {
        if dt_ms == 0 {
            return;
        }
        let dt_sec = dt_ms as f32 / 1000.0;

        // PHASE 1: Batch read (single lock acquisition)
        let ids = world.ids_with_physics();
        if ids.is_empty() {
            return;
        }

        // Collect all physics data we need to process (including optional ParticlePhysics)
        let work_items: Vec<(u64, PhysicsBody2D, Transform2D, Option<ParticlePhysics>)> = ids
            .iter()
            .filter_map(|&id| {
                let body = world.physics(id)?;
                let xf = world.transform(id)?;
                let pp = world.particle_physics(id);
                Some((id, body, xf, pp))
            })
            .collect();

        // PHASE 2: Parallel compute (no locks, pure computation)
        // Use rayon parallel iterator for multi-core processing
        let results: Vec<(u64, PhysicsBody2D, Transform2D)> = work_items
            .into_par_iter()
            .map(|(id, body, xf, pp)| {
                let (new_body, new_xf) = Self::integrate_single(dt_sec, body, xf, pp.as_ref());
                (id, new_body, new_xf)
            })
            .collect();

        // PHASE 3: Batch write (sequential, but fast)
        for (id, body, xf) in results {
            let _ = world.set_transform(id, xf);
            let _ = world.set_physics(id, body);
        }
    }
}
