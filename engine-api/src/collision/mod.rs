//! Collision domain API for Rhai scripting.

pub mod api;
pub use api::{ScriptCollisionApi, filter_hits_by_kind, filter_hits_of_kind, register_collision_api};
