//! Gameplay strategy traits and defaults for interchangeable simulation pieces.

use crate::GameplayWorld;

/// Strategies controlling gameplay simulation behavior.
pub struct GameplayStrategies {
    pub physics: Box<dyn PhysicsIntegrationStrategy + Send + Sync>,
}

impl Default for GameplayStrategies {
    fn default() -> Self {
        Self {
            physics: Box::new(SimpleEulerIntegration::default()),
        }
    }
}

pub trait PhysicsIntegrationStrategy: Send + Sync {
    fn step(&self, world: &GameplayWorld, dt_ms: u64);
}

/// Simple semi-implicit Euler integrator with optional drag and max speed.
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
            let Some(mut body) = world.physics(id) else { continue };
            let Some(mut xf) = world.transform(id) else { continue };

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
