//! Engine event types and the per-frame [`EventQueue`] that shuttles them between systems.
//!
//! This module re-exports types from engine-events for backward compatibility.

pub use engine_events::{EngineEvent, EventQueue};
