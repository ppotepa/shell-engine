//! High-performance threading utilities for shell-quest engine.
//!
//! Provides:
//! - `BatchAccessor`: Single-lock batch read/write for component stores
//! - `WorkerPool`: Dedicated worker threads for physics/particles
//! - `CommandChannel`: Lock-free channels for result collection
//!
//! # Architecture
//!
//! The key insight is that per-element mutex locks are slow. Instead:
//! 1. Acquire lock ONCE, extract ALL data needed
//! 2. Process in parallel (no locks)
//! 3. Acquire lock ONCE, write ALL results back
//!
//! This module provides abstractions to make this pattern easy.

pub mod batch;
pub mod channels;
pub mod pool;
pub mod systems;

pub use batch::{BatchRead, BatchWrite, BatchAccessor};
pub use channels::{CommandSender, CommandReceiver, command_channel};
pub use pool::{WorkerPool, WorkerHandle, WorkerConfig};
