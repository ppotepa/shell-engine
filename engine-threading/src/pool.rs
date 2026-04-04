//! Worker thread pool for dedicated physics/particle processing.
//!
//! Provides persistent worker threads that avoid thread spawn overhead.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::channels::{
    command_channel, bounded_channel, CommandReceiver, CommandSender,
    PhysicsCommand, PhysicsResultItem, PhysicsWorkItem,
};

/// Configuration for worker pool.
#[derive(Clone, Debug)]
pub struct WorkerConfig {
    /// Number of physics worker threads.
    pub physics_workers: usize,
    /// Gravity constant (pixels/sec²).
    pub world_gravity: f32,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            physics_workers: num_cpus().saturating_sub(1).max(1),
            world_gravity: 100.0,
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
}

/// Handle to a running worker thread.
pub struct WorkerHandle {
    handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl WorkerHandle {
    /// Signal shutdown and wait for thread to finish.
    pub fn shutdown(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        // Don't join in drop - could block. Thread will exit on next check.
    }
}

/// Worker pool managing physics computation threads.
pub struct WorkerPool {
    config: WorkerConfig,
    cmd_tx: CommandSender<PhysicsCommand>,
    result_rx: CommandReceiver<Vec<PhysicsResultItem>>,
    workers: Vec<WorkerHandle>,
    running: Arc<AtomicBool>,
}

impl WorkerPool {
    /// Create a new worker pool with given configuration.
    pub fn new(config: WorkerConfig) -> Self {
        let (cmd_tx, cmd_rx) = command_channel::<PhysicsCommand>();
        let (result_tx, result_rx) = command_channel::<Vec<PhysicsResultItem>>();
        let running = Arc::new(AtomicBool::new(true));

        let mut workers = Vec::with_capacity(config.physics_workers);

        // Spawn physics workers
        for i in 0..config.physics_workers {
            let cmd_rx_clone = cmd_rx.inner.clone();
            let result_tx_clone = result_tx.clone();
            let running_clone = running.clone();
            let gravity = config.world_gravity;

            let shutdown = Arc::new(AtomicBool::new(false));
            let shutdown_clone = shutdown.clone();

            let handle = thread::Builder::new()
                .name(format!("physics-worker-{}", i))
                .spawn(move || {
                    physics_worker_loop(cmd_rx_clone, result_tx_clone, running_clone, shutdown_clone, gravity);
                })
                .expect("Failed to spawn physics worker");

            workers.push(WorkerHandle {
                handle: Some(handle),
                shutdown,
            });
        }

        Self {
            config,
            cmd_tx,
            result_rx,
            workers,
            running,
        }
    }

    /// Submit physics work and get results synchronously.
    /// This is the main API for frame processing.
    pub fn process_physics(&self, dt_ms: u64, items: Vec<PhysicsWorkItem>) -> Vec<PhysicsResultItem> {
        if items.is_empty() {
            return Vec::new();
        }

        // For small batches, just compute inline (thread overhead not worth it)
        if items.len() < 32 {
            return compute_physics_batch(dt_ms, &items, self.config.world_gravity);
        }

        // Split work across workers
        let chunk_size = (items.len() / self.config.physics_workers).max(1);
        let chunks: Vec<Vec<PhysicsWorkItem>> = items
            .chunks(chunk_size)
            .map(|c| c.to_vec())
            .collect();

        // Send to workers
        for chunk in chunks {
            let _ = self.cmd_tx.send(PhysicsCommand::Step {
                dt_ms,
                items: chunk,
            });
        }

        // Collect results (blocking wait for all chunks)
        let mut all_results = Vec::with_capacity(items.len());
        let expected_chunks = (items.len() + chunk_size - 1) / chunk_size;
        
        for _ in 0..expected_chunks {
            if let Some(results) = self.result_rx.recv() {
                all_results.extend(results);
            }
        }

        all_results
    }

    /// Shutdown all workers gracefully.
    pub fn shutdown(self) {
        self.running.store(false, Ordering::SeqCst);
        
        // Send shutdown commands
        for _ in 0..self.workers.len() {
            let _ = self.cmd_tx.send(PhysicsCommand::Shutdown);
        }

        // Wait for workers
        for worker in self.workers {
            worker.shutdown();
        }
    }
}

fn physics_worker_loop(
    cmd_rx: crossbeam_channel::Receiver<PhysicsCommand>,
    result_tx: CommandSender<Vec<PhysicsResultItem>>,
    running: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    world_gravity: f32,
) {
    while running.load(Ordering::SeqCst) && !shutdown.load(Ordering::SeqCst) {
        match cmd_rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(PhysicsCommand::Step { dt_ms, items }) => {
                let results = compute_physics_batch(dt_ms, &items, world_gravity);
                let _ = result_tx.send(results);
            }
            Ok(PhysicsCommand::Shutdown) => break,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
        }
    }
}

/// Compute physics for a batch of items (pure function, no locks).
fn compute_physics_batch(
    dt_ms: u64,
    items: &[PhysicsWorkItem],
    world_gravity: f32,
) -> Vec<PhysicsResultItem> {
    let dt_sec = dt_ms as f32 / 1000.0;

    items
        .iter()
        .map(|item| {
            let mut vx = item.vx;
            let mut vy = item.vy;
            let mut ax = item.ax;
            let mut ay = item.ay;

            // Apply gravity
            if item.gravity_scale > 0.0 {
                ay += world_gravity * item.gravity_scale;
            }

            // Apply acceleration
            vx += ax * dt_sec;
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
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_physics_batch() {
        let items = vec![
            PhysicsWorkItem {
                id: 1,
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
            },
        ];

        let results = compute_physics_batch(1000, &items, 100.0);
        assert_eq!(results.len(), 1);
        assert!((results[0].x - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_gravity() {
        let items = vec![
            PhysicsWorkItem {
                id: 1,
                x: 0.0,
                y: 0.0,
                heading: 0.0,
                vx: 0.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.0,
                drag: 0.0,
                max_speed: 0.0,
                gravity_scale: 1.0,
            },
        ];

        let results = compute_physics_batch(1000, &items, 100.0);
        assert!((results[0].vy - 100.0).abs() < 0.01); // 1 sec of gravity
    }
}
