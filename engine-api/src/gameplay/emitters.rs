//! Emitter and FX-related gameplay API surface.

use std::collections::BTreeMap;

use engine_game::components::{FollowAnchor2D, LifecyclePolicy};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub struct EmitResolved {
    pub speed: f64,
    pub base_dir_x: f64,
    pub base_dir_y: f64,
    pub spread: f64,
    pub ttl_ms: i32,
    pub radius: i64,
    pub template: String,
    pub kind: String,
    pub fg: String,
    pub lifecycle: LifecyclePolicy,
    pub follow_anchor: Option<FollowAnchor2D>,
    pub extra_data: BTreeMap<String, JsonValue>,
}
