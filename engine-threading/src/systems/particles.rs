//! Dedicated particle system with optimized batch processing.
//!
//! Particles are lightweight entities that can be processed more efficiently
//! than full gameplay entities because they have fewer components.

use rayon::prelude::*;

/// Packed particle data for batch processing.
#[derive(Clone, Debug)]
pub struct ParticleData {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ttl_ms: i32,
    pub gravity_scale: f32,
}

/// Result of particle update.
#[derive(Clone, Debug)]
pub struct ParticleResult {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ttl_ms: i32,
    pub expired: bool,
}

/// High-performance particle processor.
pub struct ParticleProcessor {
    pub world_gravity: f32,
    pub parallel_threshold: usize,
}

impl Default for ParticleProcessor {
    fn default() -> Self {
        Self {
            world_gravity: 100.0,
            parallel_threshold: 128,
        }
    }
}

impl ParticleProcessor {
    pub fn new(world_gravity: f32) -> Self {
        Self {
            world_gravity,
            parallel_threshold: 128,
        }
    }

    /// Process all particles in parallel.
    /// Returns updated particles and list of expired IDs for cleanup.
    pub fn process(&self, dt_ms: u64, particles: &[ParticleData]) -> (Vec<ParticleResult>, Vec<u64>) {
        if particles.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let dt_sec = dt_ms as f32 / 1000.0;
        let dt_ms_i32 = dt_ms as i32;
        let gravity = self.world_gravity;

        let results: Vec<ParticleResult> = if particles.len() < self.parallel_threshold {
            particles.iter().map(|p| update_particle(p, dt_sec, dt_ms_i32, gravity)).collect()
        } else {
            particles.par_iter().map(|p| update_particle(p, dt_sec, dt_ms_i32, gravity)).collect()
        };

        let expired: Vec<u64> = results.iter().filter(|r| r.expired).map(|r| r.id).collect();
        
        (results, expired)
    }

    /// Process particles and return only alive ones (filters out expired).
    pub fn process_filter_expired(&self, dt_ms: u64, particles: &[ParticleData]) -> Vec<ParticleResult> {
        let (results, _) = self.process(dt_ms, particles);
        results.into_iter().filter(|r| !r.expired).collect()
    }
}

#[inline]
fn update_particle(p: &ParticleData, dt_sec: f32, dt_ms: i32, world_gravity: f32) -> ParticleResult {
    let mut vx = p.vx;
    let mut vy = p.vy;

    // Apply gravity
    if p.gravity_scale > 0.0 {
        vy += world_gravity * p.gravity_scale * dt_sec;
    }

    // Update position
    let x = p.x + vx * dt_sec;
    let y = p.y + vy * dt_sec;

    // Update TTL
    let ttl_ms = p.ttl_ms - dt_ms;
    let expired = ttl_ms <= 0;

    ParticleResult { id: p.id, x, y, vx, vy, ttl_ms, expired }
}

/// Batch lifecycle check (parallel TTL decrement).
pub fn batch_lifecycle_check(
    particles: &[(u64, i32)],  // (id, ttl_ms)
    dt_ms: i32,
    parallel_threshold: usize,
) -> (Vec<(u64, i32)>, Vec<u64>) {
    if particles.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let compute = |&(id, ttl): &(u64, i32)| -> (u64, i32, bool) {
        let new_ttl = ttl - dt_ms;
        (id, new_ttl, new_ttl <= 0)
    };

    let results: Vec<(u64, i32, bool)> = if particles.len() < parallel_threshold {
        particles.iter().map(compute).collect()
    } else {
        particles.par_iter().map(compute).collect()
    };

    let alive: Vec<(u64, i32)> = results.iter()
        .filter(|(_, _, expired)| !expired)
        .map(|(id, ttl, _)| (*id, *ttl))
        .collect();

    let expired: Vec<u64> = results.iter()
        .filter(|(_, _, expired)| *expired)
        .map(|(id, _, _)| *id)
        .collect();

    (alive, expired)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_processor() {
        let processor = ParticleProcessor::default();
        
        let particles = vec![
            ParticleData { id: 1, x: 0.0, y: 0.0, vx: 10.0, vy: 0.0, ttl_ms: 500, gravity_scale: 0.0 },
            ParticleData { id: 2, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, ttl_ms: 50, gravity_scale: 1.0 },
        ];

        let (results, expired) = processor.process(100, &particles);
        
        assert_eq!(results.len(), 2);
        assert_eq!(expired.len(), 1); // particle 2 expired (50 - 100 = -50)
        assert_eq!(expired[0], 2);
        
        // Particle 1 moved
        assert!((results[0].x - 1.0).abs() < 0.01);
        assert_eq!(results[0].ttl_ms, 400);
        
        // Particle 2 got gravity
        assert!(results[1].vy > 0.0);
    }

    #[test]
    fn test_batch_lifecycle() {
        let particles = vec![(1, 500), (2, 50), (3, 100)];
        let (alive, expired) = batch_lifecycle_check(&particles, 100, 10);
        
        assert_eq!(alive.len(), 1);
        assert_eq!(alive[0], (1, 400));
        assert_eq!(expired.len(), 2);
    }
}
