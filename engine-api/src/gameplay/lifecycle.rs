//! Lifecycle and ownership-related gameplay API surface.

use engine_game::components::{FollowAnchor2D, LifecyclePolicy};
use rhai::Map as RhaiMap;

use crate::rhai::conversion::{map_bool, map_number};

pub fn parse_lifecycle_policy(name: &str, fallback: LifecyclePolicy) -> LifecyclePolicy {
    match name {
        "Persistent" => LifecyclePolicy::Persistent,
        "Manual" => LifecyclePolicy::Manual,
        "Ttl" => LifecyclePolicy::Ttl,
        "OwnerBound" => LifecyclePolicy::OwnerBound,
        "TtlOwnerBound" => LifecyclePolicy::TtlOwnerBound,
        "FollowOwner" => LifecyclePolicy::FollowOwner,
        "TtlFollowOwner" => LifecyclePolicy::TtlFollowOwner,
        _ => fallback,
    }
}

pub fn is_ephemeral_lifecycle(name: &str) -> bool {
    matches!(
        name,
        "Ttl" | "OwnerBound" | "TtlOwnerBound" | "FollowOwner" | "TtlFollowOwner"
    )
}

pub fn follow_anchor_from_args(
    args: &RhaiMap,
    default_local_x: f64,
    default_local_y: f64,
    default_inherit_heading: bool,
) -> FollowAnchor2D {
    FollowAnchor2D {
        local_x: map_number(args, "follow_local_x", default_local_x) as f32,
        local_y: map_number(args, "follow_local_y", default_local_y) as f32,
        inherit_heading: map_bool(args, "follow_inherit_heading", default_inherit_heading),
    }
}
