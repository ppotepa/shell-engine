//! World-level gameplay API surface.

use std::collections::BTreeMap;

use engine_game::components::{FollowAnchor2D, LifecyclePolicy};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub struct EphemeralPrefabResolved {
    pub ttl_ms: i32,
    pub vx: f32,
    pub vy: f32,
    pub drag: f32,
    pub max_speed: f32,
    pub owner_id: Option<u64>,
    pub lifecycle: LifecyclePolicy,
    pub follow_anchor: Option<FollowAnchor2D>,
    pub extra_data: BTreeMap<String, JsonValue>,
}
