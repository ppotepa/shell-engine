//! Parallel physics system using worker pool.
//!
//! Integrates with engine-game's GameplayWorld using batch operations.

use rayon::prelude::*;

use crate::channels::{PhysicsWorkItem, PhysicsResultItem};

/// Parallel physics integration using rayon (simpler than worker pool for most cases).
/// This is the recommended approach for synchronous physics.
pub struct ParallelPhysics {
    pub world_gravity: f32,
    /// Minimum batch size to use parallel processing (below this, use sequential).
    pub parallel_threshold: usize,
}

impl Default for ParallelPhysics {
    fn default() -> Self {
        Self {
            world_gravity: 100.0,
            parallel_threshold: 64,
        }
    }
}

impl ParallelPhysics {
    pub fn new(world_gravity: f32) -> Self {
        Self {
            world_gravity,
            parallel_threshold: 64,
        }
    }

    /// Process physics for all items. Returns results in same order as input.
    pub fn process(&self, dt_ms: u64, items: &[PhysicsWorkItem]) -> Vec<PhysicsResultItem> {
        if items.is_empty() {
            return Vec::new();
        }

        let dt_sec = dt_ms as f32 / 1000.0;
        let gravity = self.world_gravity;

        if items.len() < self.parallel_threshold {
            // Sequential for small batches
            items.iter().map(|item| integrate_single(item, dt_sec, gravity)).collect()
        } else {
            // Parallel for large batches
            items.par_iter().map(|item| integrate_single(item, dt_sec, gravity)).collect()
        }
    }

    /// Process with custom gravity per-item (for particles with gravity_scale).
    pub fn process_with_gravity(&self, dt_ms: u64, items: &[PhysicsWorkItem]) -> Vec<PhysicsResultItem> {
        self.process(dt_ms, items)
    }
}

/// Single entity physics integration (pure function).
#[inline]
fn integrate_single(item: &PhysicsWorkItem, dt_sec: f32, world_gravity: f32) -> PhysicsResultItem {
    let mut vx = item.vx;
    let mut vy = item.vy;
    let mut ay = item.ay;

    // Apply per-item gravity scale
    if item.gravity_scale > 0.0 {
        ay += world_gravity * item.gravity_scale;
    }

    // Apply acceleration
    vx += item.ax * dt_sec;
    vy += ay * dt_sec;

    // Apply drag
    if item.drag > 0.0 {
        let drag = item.drag.clamp(0.0, 1.0);
        vx *= 1.0 - drag * dt_sec;
        vy *= 1.0 - drag * dt_sec;
    }

    // Clamp to max speed
    if item.max_speed > 0.0 {
        let speed_sq = vx * vx + vy * vy;
        let max_sq = item.max_speed * item.max_speed;
        if speed_sq > max_sq && speed_sq > 0.0 {
            let scale = (max_sq / speed_sq).sqrt();
            vx *= scale;
            vy *= scale;
        }
    }

    // Update position
    let x = item.x + vx * dt_sec;
    let y = item.y + vy * dt_sec;

    PhysicsResultItem { id: item.id, x, y, vx, vy }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_physics() {
        let physics = ParallelPhysics::default();
        
        let items: Vec<PhysicsWorkItem> = (0..100)
            .map(|i| PhysicsWorkItem {
                id: i,
                x: 0.0,
                y: 0.0,
                heading: 0.0,
                vx: 10.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.0,
                drag: 0.0,
                max_speed: 0.0,
                gravity_scale: 0.0,
            })
            .collect();

        let results = physics.process(100, &items);
        assert_eq!(results.len(), 100);
        
        // After 100ms at 10 px/s, should move 1 px
        assert!((results[0].x - 1.0).abs() < 0.01);
    }
}
