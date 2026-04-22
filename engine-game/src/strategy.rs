//! Gameplay strategy traits and defaults for interchangeable simulation pieces.

use crate::components::{
    ParticleGravityMode, ParticlePhysics, ParticleThreadMode, PhysicsBody2D, Transform2D,
};
use crate::GameplayWorld;
use rayon::prelude::*;

/// Strategies controlling gameplay simulation behavior.
pub struct GameplayStrategies {
    pub physics: Box<dyn PhysicsIntegrationStrategy + Send + Sync>,
    pub physics_3d: Box<dyn PhysicsIntegrationStrategy3D + Send + Sync>,
    pub reference_frames_3d: Box<dyn ReferenceFrameResolutionStrategy3D + Send + Sync>,
    pub motors_3d: Box<dyn MotorApplyStrategy3D + Send + Sync>,
}

impl Default for GameplayStrategies {
    fn default() -> Self {
        Self {
            physics: Box::new(ParallelEulerIntegration),
            physics_3d: Box::new(NoopPhysicsIntegration3D),
            reference_frames_3d: Box::new(NoopReferenceFrameResolution3D),
            motors_3d: Box::new(NoopMotorApply3D),
        }
    }
}

pub trait PhysicsIntegrationStrategy: Send + Sync {
    fn step(&self, world: &GameplayWorld, dt_ms: u64);
}

pub trait PhysicsIntegrationStrategy3D: Send + Sync {
    fn step(&self, world: &GameplayWorld, dt_ms: u64);
}

pub trait ReferenceFrameResolutionStrategy3D: Send + Sync {
    fn resolve(&self, world: &GameplayWorld, dt_ms: u64);
}

pub trait MotorApplyStrategy3D: Send + Sync {
    fn apply(&self, world: &GameplayWorld, dt_ms: u64);
}

#[derive(Default)]
pub struct NoopPhysicsIntegration3D;

impl PhysicsIntegrationStrategy3D for NoopPhysicsIntegration3D {
    fn step(&self, _world: &GameplayWorld, _dt_ms: u64) {}
}

#[derive(Default)]
pub struct NoopReferenceFrameResolution3D;

impl ReferenceFrameResolutionStrategy3D for NoopReferenceFrameResolution3D {
    fn resolve(&self, _world: &GameplayWorld, _dt_ms: u64) {}
}

#[derive(Default)]
pub struct NoopMotorApply3D;

impl MotorApplyStrategy3D for NoopMotorApply3D {
    fn apply(&self, _world: &GameplayWorld, _dt_ms: u64) {}
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
            body.vz += body.az * dt_sec;

            if body.drag > 0.0 {
                let drag = body.drag.clamp(0.0, 1.0);
                body.vx *= 1.0 - drag * dt_sec;
                body.vy *= 1.0 - drag * dt_sec;
                body.vz *= 1.0 - drag * dt_sec;
            }

            if body.max_speed > 0.0 {
                let speed_sq = body.vx * body.vx + body.vy * body.vy + body.vz * body.vz;
                let max_sq = body.max_speed * body.max_speed;
                if speed_sq > max_sq && speed_sq > 0.0 {
                    let scale = (max_sq / speed_sq).sqrt();
                    body.vx *= scale;
                    body.vy *= scale;
                    body.vz *= scale;
                }
            }

            xf.x += body.vx * dt_sec;
            xf.y += body.vy * dt_sec;
            xf.z += body.vz * dt_sec;

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
        // Extra transient accelerations are computed from ParticlePhysics each frame and NOT
        // stored back into the body, keeping particle physics stateless between frames.
        // Orbital gravity applies to ALL thread modes; flat gravity still skips Light particles.
        const WORLD_GRAVITY_Y: f32 = 100.0;
        let (extra_ax, extra_ay, extra_az) = match particle_physics {
            Some(pp) => match pp.gravity_mode {
                ParticleGravityMode::Orbital if pp.gravity_constant > 0.0 => {
                    let dx = pp.gravity_center_x - xf.x;
                    let dy = pp.gravity_center_y - xf.y;
                    let dz = pp.gravity_center_z - xf.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq > 1.0 {
                        let dist = dist_sq.sqrt();
                        let g = pp.gravity_constant / dist_sq;
                        (dx / dist * g, dy / dist * g, dz / dist * g)
                    } else {
                        (0.0, 0.0, 0.0)
                    }
                }
                ParticleGravityMode::Flat
                    if pp.gravity_scale > 0.0 && pp.thread_mode != ParticleThreadMode::Light =>
                {
                    (0.0, WORLD_GRAVITY_Y * pp.gravity_scale, 0.0)
                }
                _ => (0.0, 0.0, 0.0),
            },
            None => (0.0, 0.0, 0.0),
        };

        body.vx += (body.ax + extra_ax) * dt_sec;
        body.vy += (body.ay + extra_ay) * dt_sec;
        body.vz += (body.az + extra_az) * dt_sec;

        // Apply drag
        if body.drag > 0.0 {
            let drag = body.drag.clamp(0.0, 1.0);
            body.vx *= 1.0 - drag * dt_sec;
            body.vy *= 1.0 - drag * dt_sec;
            body.vz *= 1.0 - drag * dt_sec;
        }

        // Clamp to max speed (3D magnitude)
        if body.max_speed > 0.0 {
            let speed_sq = body.vx * body.vx + body.vy * body.vy + body.vz * body.vz;
            let max_sq = body.max_speed * body.max_speed;
            if speed_sq > max_sq && speed_sq > 0.0 {
                let scale = (max_sq / speed_sq).sqrt();
                body.vx *= scale;
                body.vy *= scale;
                body.vz *= scale;
            }
        }

        // Update position
        xf.x += body.vx * dt_sec;
        xf.y += body.vy * dt_sec;
        xf.z += body.vz * dt_sec;

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

        // PHASE 1: Single-lock batch read — only inline (non-worker) entities.
        // Worker-thread particles (Physics/Gravity) are handled by async particle system.
        let work_items = world.batch_read_inline_physics();
        if work_items.is_empty() {
            return;
        }

        // PHASE 2: Compute physics (parallel only above threshold)
        let results: Vec<(u64, Transform2D, PhysicsBody2D)> = if work_items.len()
            > PARALLEL_THRESHOLD
        {
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
